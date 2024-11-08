use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::thread;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum Event {
    MarketData,
    OrderPlace,
    OrderComplete,
}

pub trait EventHandler: Send + Sync {
    fn handle_event(&self, event: &Event, event_sender: &Sender<Event>);
}

pub struct EventManager {
    subscriber_book: HashMap<Event, Vec<Arc<dyn EventHandler>>>,
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

    pub fn subscribe(&mut self, event_type: Event, handler: Arc<dyn EventHandler>) {
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
            let event = {
                let em = event_manager.lock().unwrap();
                em.event_receiver.recv().unwrap()
            };

            let handlers = {
                let em = event_manager.lock().unwrap();
                em.subscriber_book.get(&event).cloned()
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
