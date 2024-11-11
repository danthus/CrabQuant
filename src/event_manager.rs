use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crossbeam::channel::{unbounded, Receiver, Sender};
use crate::events::EventType;
use std::any::Any;
use std::thread;


#[derive(Debug, Clone)]
pub struct Event{
    pub event_type: EventType,
    pub contents: Arc<dyn Any + Send + Sync>,
}

impl Event {
    pub fn new<T: Any + Send + Sync>(event_type: EventType, contents: T) -> Self {
        Event {
            event_type,
            contents: Arc::new(contents),
        }
    }
}

pub trait EventHandler: Send + Sync {
    fn handle_event(&self, event: &Event, event_sender: &Sender<Event>);
}

pub struct EventManager {
    subscriber_book: HashMap<EventType, Vec<Arc<dyn EventHandler>>>,
    event_sender: Sender<Event>,
    event_receiver: Receiver<Event>,
}

impl EventManager {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = unbounded();
        EventManager {
            subscriber_book: HashMap::new(),
            event_sender,
            event_receiver,
        }
    }

    pub fn subscribe(&mut self, event_type: EventType, handler: Arc<dyn EventHandler>)
    {
        self.subscriber_book
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);    
    }

    pub fn push_event(&self, event: Event) {
        self.event_sender.send(event).unwrap();
    }

    pub fn process_events(event_manager: Arc<Mutex<Self>>) {
        loop {
            let (event, handlers) = {
                let em = event_manager.lock().unwrap();
                let event = em.event_receiver.recv().unwrap();
                let handlers = em.subscriber_book.get(&event.event_type).cloned();
                (event, handlers)
            };

            if let Some(handlers) = handlers {
                for handler in handlers {
                    let event_sender = event_manager.lock().unwrap().event_sender.clone();
                    let event = event.clone();

                    thread::spawn(move || {
                        handler.handle_event(&event, &event_sender);
                    });
                }
            }
        }
    }

    pub fn get_event_sender(&self) -> Sender<Event> {
        self.event_sender.clone()
    }
}
