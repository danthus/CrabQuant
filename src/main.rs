use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::cell::RefCell;

// Define an enum for different event types
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum EventType {
    MarketData,
    Signal,
    Strategy,
    Portfolio,
}

// Define a trait for handling events
trait EventHandler {
    fn handle_event(&self, event: &Event);
}

// Event structure containing the event type and any additional data
struct Event {
    event_type: EventType,
    // You can add more fields as needed, e.g., market data, signal info, etc.
}

// Define an EventManager struct that manages events and handlers
struct EventManager {
    subscriber_book: HashMap<EventType, Vec<Rc<RefCell<dyn EventHandler>>>>,
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
    fn subscribe(&mut self, event_type: EventType, handler: Rc<RefCell<dyn EventHandler>>) {
        self.subscriber_book.entry(event_type).or_insert(Vec::new()).push(handler);
    }

    // Add an event to the queue
    fn push_event(&mut self, event: Event) {
        self.event_queue.push_back(event);
    }

    // Process events by dispatching them to the registered handlers
    fn process_events(&mut self) {
        while let Some(event) = self.event_queue.pop_front() {
            if let Some(handlers) = self.subscriber_book.get(&event.event_type) {
                for handler in handlers {
                    handler.borrow().handle_event(&event);
                }
            }
        }
    }
}

// Implementing example modules that implement EventHandler trait

struct MarketDataFeeder;

struct Strategy;

impl EventHandler for Strategy {
    fn handle_event(&self, event: &Event) {
        println!("Strategy handling event: {:?}", event.event_type);
        // Implement specific logic for handling strategy events
    }
}

struct OrderHandler;

impl EventHandler for OrderHandler {
    fn handle_event(&self, event: &Event) {
        println!("OrderHandler handling event: {:?}", event.event_type);
        // Implement specific logic for handling order events
    }
}

struct PortfolioHandler;

impl EventHandler for PortfolioHandler {
    fn handle_event(&self, event: &Event) {
        println!("PortfolioHandler handling event: {:?}", event.event_type);
        // Implement specific logic for handling portfolio events
    }
}

fn main() {
    let mut event_manager = EventManager::new();

    // Register different handlers for different event types
    event_manager.subscribe(EventType::Strategy, Rc::new(RefCell::new(Strategy)));
    event_manager.subscribe(EventType::Portfolio, Rc::new(RefCell::new(PortfolioHandler)));
    event_manager.subscribe(EventType::Signal, Rc::new(RefCell::new(OrderHandler)));

    // Push events to the event manager
    event_manager.push_event(Event { event_type: EventType::MarketData });

    // Process events
    event_manager.process_events();
}
