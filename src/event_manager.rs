use crate::events::{Event, MarketDataEvent, PortfolioInfoEvent, OrderPlaceEvent};
use crossbeam::channel::{unbounded, Receiver, Sender, bounded};
use std::any::TypeId;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use simplelog::*;


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
    // sb: 
    subscriber_book: HashMap<TypeId, Vec<Sender<Event>>>,
    lp_sender: Sender<Event>,
    lp_receiver: Receiver<Event>,
    hp_sender: Sender<Event>,
    hp_receiver: Receiver<Event>,
}

impl EventManager {
    pub fn new() -> Self {
        let (hp_sender, hp_receiver) = unbounded();
        let (lp_sender, lp_receiver) = bounded(20);

        #[cfg(feature= "random_sleep_test")]
        let mut rng = rand::thread_rng();

        EventManager {
            subscriber_book: HashMap::new(),
            lp_sender,
            lp_receiver,
            hp_sender,
            hp_receiver,
            #[cfg(feature = "order_test")]
            event_counters,
        }
    }

    pub fn subscribe<E: 'static, T: ModuleReceive>(&mut self, module: &T) {
        let type_id = TypeId::of::<E>();
        let sender = module.get_sender();
        self.subscriber_book
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(sender);
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
        // Use TypeId of the event for dynamic dispatch
        let type_id = match &event {
            Event::MarketData(_) => TypeId::of::<MarketDataEvent>(),
            Event::OrderPlace(_) => TypeId::of::<OrderPlaceEvent>(),
            Event::PortfolioInfo(_) => TypeId::of::<PortfolioInfoEvent>(),
        };

        // Dispatch to subscribers
        if let Some(subscribers) = self.subscriber_book.get(&type_id) {
            for dispatch_sender in subscribers {
                if let Err(e) = dispatch_sender.send(event.clone()) {
                    eprintln!("Failed to send event to subscriber: {:?}", e);
                }
            }
        } else {
            eprintln!("No subscribers found for event type: {:?}", type_id);
        }
    }

    pub fn proceed(&mut self) {

        let event = self.lp_receiver.recv().unwrap();
        self.dispatch_event(event);

        let timeout = Duration::from_secs(10);
        let mut start = Instant::now();

        loop {
            // Continuously process all high-priority events
            while let Ok(event) = self.hp_receiver.try_recv() {
                self.dispatch_event(event);
                start = Instant::now();
            }

            // Process low-priority events
            if let Ok(event) = self.lp_receiver.try_recv() {
                self.dispatch_event(event);
                start = Instant::now();
            } else {
                // If no events are available, you may want to sleep or handle idle state
                // For example:
                if start.elapsed() >= timeout {
                    // wait until no upcoming event for timeout period
                    // note that the returning of the event_manager.proceed will terminate the main thread.
                    info!("All data feeded, backtesting completed.");                   
                    break;
                }
                // break;
            }
        }
    }
}
