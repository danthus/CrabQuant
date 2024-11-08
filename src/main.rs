use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::thread;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum Event {
    MarketData,
    OrderPlace,
    OrderComplete,
}

trait EventHandler: Send + Sync {
    fn handle_event(&self, event: &Event, event_sender: &Sender<Event>);
}

struct EventManager {
    subscriber_book: HashMap<Event, Vec<Arc<dyn EventHandler>>>,
    event_sender: Sender<Event>,
    event_receiver: Receiver<Event>,
}

impl EventManager {
    fn new() -> Self {
        let (event_sender, event_receiver) = unbounded();
        EventManager {
            subscriber_book: HashMap::new(),
            event_sender,
            event_receiver,
        }
    }

    fn subscribe(&mut self, event_type: Event, handler: Arc<dyn EventHandler>) {
        self.subscriber_book
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    fn push_event(&self, event: Event) {
        self.event_sender.send(event).unwrap();
    }

    // Now `process_events` takes self by Arc<Mutex<Self>> for independent thread handling
    fn process_events(event_manager: Arc<Mutex<Self>>) {
        loop {
            let event = {
                // Temporarily unlock `event_receiver` to receive an event
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

                    // Spawn a new thread for each handler to process the event concurrently
                    thread::spawn(move || {
                        handler.handle_event(&event, &event_sender);
                    });
                }
            }
        }
    }

    fn get_event_sender(&self) -> Sender<Event> {
        self.event_sender.clone()
    }
}

struct Strategy;

impl EventHandler for Strategy {
    fn handle_event(&self, event: &Event, event_sender: &Sender<Event>) {
        println!("Strategy handling event: {:?}, sending OrderPlace", event);
        event_sender.send(Event::OrderPlace).unwrap();
    }
}

struct MockExchange;

impl EventHandler for MockExchange {
    fn handle_event(&self, event: &Event, event_sender: &Sender<Event>) {
        println!("MockExchange handling event: {:?}, sending OrderComplete", event);
        event_sender.send(Event::OrderComplete).unwrap();
    }
}

struct ProtfolioManager;

impl EventHandler for ProtfolioManager {
    fn handle_event(&self, event: &Event, _event_sender: &Sender<Event>) {
        println!("PortfolioManager handling event: {:?}", event);
    }
}

// New MarketDataFeeder struct
struct MarketDataFeeder {
    event_sender: Sender<Event>,
}

impl MarketDataFeeder {
    fn new(event_sender: Sender<Event>) -> Self {
        MarketDataFeeder { event_sender }
    }

    fn start(self) {
        thread::spawn(move || loop {
            println!("MarketDataFeeder sending MarketData event");
            self.event_sender.send(Event::MarketData).unwrap();
            thread::sleep(std::time::Duration::from_millis(500)); // Adjust frequency as needed
        });
    }
}

fn main() {
    let event_manager = Arc::new(Mutex::new(EventManager::new()));

    // Acquire a lock to mutate and add subscribers
    {
        let mut em = event_manager.lock().unwrap();
        em.subscribe(Event::MarketData, Arc::new(Strategy));
        em.subscribe(Event::MarketData, Arc::new(MockExchange));
        em.subscribe(Event::OrderPlace, Arc::new(MockExchange));
        em.subscribe(Event::OrderComplete, Arc::new(ProtfolioManager));
    }

    let event_sender = event_manager.lock().unwrap().get_event_sender();
    
    // Start processing events in a separate thread
    let event_manager_clone = Arc::clone(&event_manager);
    thread::spawn(move || {
        EventManager::process_events(event_manager_clone);
    });

    // Initial event
    event_sender.send(Event::MarketData).unwrap();

    let market_data_feeder = MarketDataFeeder::new(event_sender.clone());
    market_data_feeder.start();

    thread::sleep(std::time::Duration::from_secs(5));
}
