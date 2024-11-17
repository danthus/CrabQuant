use crate::events::{EventType, EventContent};
use crossbeam::channel::{unbounded, Receiver, Sender, bounded};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub contents: EventContent,
}

impl Event {
    pub fn new(event_type: EventType, contents: EventContent) -> Self {
        Event {
            event_type,
            contents,
        }
    }
}

pub trait ModuleReceive {
    // For modules to give the rendezvous channel sender to the event_manager.
    // The event_manager will clone the sender for subscription.
    fn get_sender(&self) -> Sender<Event>;
}

pub trait ModulePublish {
    // To allow modules to obtain senders
    fn use_sender(&mut self, sender: Sender<Event>);
}
pub struct EventManager {
    subscriber_book: HashMap<EventType, Vec<Sender<Event>>>,
    lp_sender: Sender<Event>,
    lp_receiver: Receiver<Event>,
    hp_sender: Sender<Event>,
    hp_receiver: Receiver<Event>,
}

impl EventManager {
    pub fn new() -> Self {
        let (hp_sender, hp_receiver) = unbounded();
        let (lp_sender, lp_receiver) = bounded(1);
        EventManager {
            subscriber_book: HashMap::new(),
            lp_sender,
            lp_receiver,
            hp_sender,
            hp_receiver,
        }
    }

    pub fn subscribe<T: ModuleReceive>(&mut self, event_type: EventType, module: &T) {
        let sender = module.get_sender();
        self.subscriber_book
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(sender.clone());
    }

    pub fn allow_publish<T: ModulePublish>(&mut self, priority: String, module: &mut T) {
        // Allow module to publish to one of the lp/hp channel.
        match priority.as_str() {
            "high" => module.use_sender(self.hp_sender.clone()),
            "low" => module.use_sender(self.lp_sender.clone()),
            _ => panic!("Invalid priority: expected 'high' or 'low', but got '{}'", priority),
        }
    }

    fn dispatch_event(&self, event: Event) {
        if let Some(subscribers) = self.subscriber_book.get(&event.event_type) {
            for sender in subscribers {
                if let Err(e) = sender.send(event.clone()) {
                    eprintln!("Failed to send event to subscriber: {:?}", e);
                }
            }
        } else {
            eprintln!("No subscribers found for event type: {:?}", event.event_type);
        }
    }

    pub fn process_events(&mut self) {
        loop {
            // If some events in hp channel, handle them
            if !self.hp_receiver.is_empty(){
                match self.hp_receiver.recv(){
                    Ok(event) => self.dispatch_event(event),
                    Err(e) => panic!("{}", e),
                }
            }
            // If no HP event is available, process a low-priority event
            if !self.lp_receiver.is_empty(){
                match self.lp_receiver.recv(){
                    Ok(event) => self.dispatch_event(event),
                    Err(e) => panic!("{}", e),
                }
            } else{
                // If backtesting, return.
                // println!("All lp events are handled. Backtesting process completed.");
                // return;
                // TODO: If live trade, keep looping.
            }
        }
    }
}
