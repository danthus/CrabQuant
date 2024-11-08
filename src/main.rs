use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::cell::RefCell;

// Define an enum for different event types
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum Event {
    MarketData,
    OrderPlace,
    OrderComplete
}

// Define a trait for handling events
trait EventHandler {
    fn handle_event(&self, event: &Event);
}


// Define an EventManager struct that manages events and handlers
struct EventManager {
    subscriber_book: HashMap<Event, Vec<Rc<RefCell<dyn EventHandler>>>>,
    event_queue: VecDeque<Event>,
}

impl EventManager {
    fn new() -> Self {
        EventManager {
            subscriber_book: HashMap::new(),
            event_queue: VecDeque::new(),
        }
    }

    // Subscribe a handler for a specific event type
    fn subscribe(&mut self, event_type: Event, handler: Rc<RefCell<dyn EventHandler>>) {
        self.subscriber_book.entry(event_type).or_insert(Vec::new()).push(handler);
    }

    // Add an event to the queue
    fn push_event(&mut self, event: Event) {
        self.event_queue.push_back(event);
    }

    // Process events by dispatching them to the registered handlers
    fn process_events(&mut self) {
        while let Some(event) = self.event_queue.pop_front() {
            if let Some(handlers) = self.subscriber_book.get(&event) {
                for handler in handlers {
                    handler.borrow().handle_event(&event);
                }
            }
        }
    }
}

// Implementing example modules that implement EventHandler trait

struct MarketDataFeeder {
    event_manager: Rc<RefCell<EventManager>>,
}

impl MarketDataFeeder {
    fn new(event_manager: Rc<RefCell<EventManager>>) -> Self {
        MarketDataFeeder { event_manager }
    }

    // Method to push a MarketData event
    fn push_event(&self) {
        let event = Event::MarketData ;
        self.event_manager.borrow_mut().push_event(event);
    }
}

struct Strategy{
    event_manager: Rc<RefCell<EventManager>>,
}

impl Strategy {
    fn new(event_manager: Rc<RefCell<EventManager>>) -> Self {
        Strategy { event_manager }
    }

    // Method to push a OrderPlace event
    fn push_event(&self) {
        let event = Event::OrderPlace ;
        self.event_manager.borrow_mut().push_event(event);
    }
}

impl EventHandler for Strategy {
    fn handle_event(&self, event: &Event) {
        println!("Strategy handling event: {:?}, publish a new OrderPlace", event);
        // Implement specific logic for handling strategy events
        // self.push_event();
    }
}

struct MockExchange {
    event_manager: Rc<RefCell<EventManager>>,
}

impl MockExchange {
    fn new(event_manager: Rc<RefCell<EventManager>>) -> Self {
        MockExchange { event_manager }
    }

    // Push an OrderComplete event
    fn push_event(&self) {
        let event = Event::OrderComplete;
        self.event_manager.borrow_mut().push_event(event);
    }
}

impl EventHandler for MockExchange {
    fn handle_event(&self, event: &Event) {
        println!("OrderHandler handling event: {:?}, publishing OrderComplete", event);
        // Implement specific logic for handling order events
        // self.push_event();
    }
}

struct ProtfolioManager;

impl EventHandler for ProtfolioManager {
    fn handle_event(&self, event: &Event) {
        println!("PortfolioHandler handling event: {:?}", event);
        // Implement specific logic for handling portfolio events
    }
}

fn main() {
    let event_manager = Rc::new(RefCell::new(EventManager::new()));
    let market_data_feeder = MarketDataFeeder::new(Rc::clone(&event_manager));
    let mock_exchange = MockExchange::new(Rc::clone(&event_manager));
    let my_strategy = Strategy::new(Rc::clone(&event_manager));

    // Register different handlers for different event types
    event_manager.borrow_mut().subscribe(Event::MarketData, Rc::new(RefCell::new(Strategy::new(Rc::clone(&event_manager)))));
    event_manager.borrow_mut().subscribe(Event::MarketData, Rc::new(RefCell::new(MockExchange::new(Rc::clone(&event_manager)))));
    event_manager.borrow_mut().subscribe(Event::OrderPlace, Rc::new(RefCell::new(ProtfolioManager)));
    event_manager.borrow_mut().subscribe(Event::OrderPlace, Rc::new(RefCell::new(MockExchange::new(Rc::clone(&event_manager)))));
    event_manager.borrow_mut().subscribe(Event::OrderComplete, Rc::new(RefCell::new(MockExchange::new(Rc::clone(&event_manager)))));


    // Push events to the event manager
    market_data_feeder.push_event();
    // market_data_feeder.push_event();
    // market_data_feeder.push_event();
    my_strategy.push_event();
    mock_exchange.push_event();

    // Process events
    event_manager.borrow_mut().process_events();
}
