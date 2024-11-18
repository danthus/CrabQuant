use crate::events::{EventType, EventContent};
use crossbeam::channel::{unbounded, Receiver, Sender, bounded};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::util::Counter;

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
    #[cfg(feature= "custom_test")]
    event_counters: HashMap<EventType, Counter>,
}

impl EventManager {
    pub fn new() -> Self {
        let (hp_sender, hp_receiver) = unbounded();
        let (lp_sender, lp_receiver) = bounded(10);

        #[cfg(feature = "custom_test")]
        let mut event_counters = HashMap::new();

        #[cfg(feature = "custom_test")]
        {
            // Initialize counters for each EventType you are using
            event_counters.insert(EventType::TypeMarketData, Counter::new());
            event_counters.insert(EventType::TypeOrderPlace, Counter::new());
            event_counters.insert(EventType::TypeOrderComplete, Counter::new());
            // Add other EventTypes as needed
        }

        EventManager {
            subscriber_book: HashMap::new(),
            lp_sender,
            lp_receiver,
            hp_sender,
            hp_receiver,
            #[cfg(feature = "custom_test")]
            event_counters,
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

    fn dispatch_event(&mut self, event: Event) {
        #[cfg(feature = "custom_test")]
        {
            if let Some(counter) = self.event_counters.get_mut(&event.event_type) {
                let count = counter.next();
                println!("Dispatching {:?}{}", event.event_type, count);
            } else {
                println!("Dispatching {:?}", event.event_type);
            }
        }
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

    // pub fn proceed(&mut self) {
    //     loop {
    //         // If some events in hp channel, handle them
    //         while !self.hp_receiver.is_empty() {
    //             match self.hp_receiver.recv(){
    //                 Ok(event) => self.dispatch_event(event),
    //                 Err(e) => panic!("{}", e),
    //             }
    //         }
    //         // If no HP event is available, process a low-priority event
    //         if !self.lp_receiver.is_empty(){
    //             match self.lp_receiver.recv(){
    //                 Ok(event) => self.dispatch_event(event),
    //                 Err(e) => panic!("{}", e),
    //             }
    //         } else{
    //             // If backtesting, return.
    //             // println!("All lp events are handled. Backtesting process completed.");
    //             // return;
    //             // TODO: If live trade, keep looping.
    //         }
    //     }
    // }
    pub fn proceed(&mut self) {
        loop {
            // Continuously process all high-priority events
            while let Ok(event) = self.hp_receiver.try_recv() {
                self.dispatch_event(event);
            }

            // Process low-priority events
            if let Ok(event) = self.lp_receiver.try_recv() {
                self.dispatch_event(event);

                // After dispatching a low-priority event, check for new high-priority events
                while let Ok(event) = self.hp_receiver.try_recv() {
                    self.dispatch_event(event);
                }
            } else {
                // If no events are available, you may want to sleep or handle idle state
                // For example:
                // std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    }
}
