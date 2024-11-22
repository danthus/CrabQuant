use crate::event_manager::{Event, ModulePublish, ModuleReceive};
use crate::events::{EventType, OrderCompleteEvent, OrderPlaceEvent, EventContent};
use crossbeam::channel::{Sender, Receiver, bounded};
use std::collections::HashMap;
use rand::Rng;
#[cfg(feature= "order_test")]
use crate::util::Counter;
use std::time::Duration;
use std::thread;
pub struct PortfolioManager {
    subscribe_sender: Sender<Event>,
    subscribe_receiver: Receiver<Event>,
    portfolio: Portfolio,
}

struct Portfolio {
    asset: f64,
    cash: f64,
    available_cash: f64,
    positions: HashMap<String, i32>,
}

impl PortfolioManager {
    /// Creates a new PortfolioManager with an initial cash balance
    pub fn new(initial_cash: f64) -> Self {
        let (subscribe_sender, subscribe_receiver) = bounded(0);
        let portfolio = Portfolio {
            asset: initial_cash,
            cash: initial_cash,
            available_cash: initial_cash,
            positions: HashMap::new(),
        };

        PortfolioManager {
            subscribe_sender,
            subscribe_receiver,
            portfolio,
        }
    }

/// Continuously process events
pub fn run(&mut self) {
    #[cfg(feature= "order_test")]
    let mut counter_a = Counter::new();
    #[cfg(feature= "order_test")]
    let mut counter_b = Counter::new();
    #[cfg(feature= "random_sleep_test")]
    let mut rng = rand::thread_rng();

    loop {
        let event = self.subscribe_receiver.recv().unwrap();
        // println!("PortfolioManager: received event: {:?}", event);

        match event.contents {
            EventContent::OrderPlace(order_place_event) => {
                // println!(
                //     "PortfolioManager: Received OrderPlaceEvent: {:?}",
                //     order_place_event
                // );
                #[cfg(feature= "order_test")]
                {
                    println!("PM: Received OrderPlaceEvent{}", counter_a.next());
                    // println!("PM Timestamp: {:?}", std::time::SystemTime::now());
                }
                // Update portfolio with the order details
                self.update_position(
                    "DummySymbol".to_string(),
                    order_place_event.quantity,
                );
                self.update_cash(-order_place_event.quantity as f64 * 100.0);
            },
            EventContent::OrderComplete(order_complete_event) =>{
                // DO something
                // println!(
                //     "PortfolioManager: Received OrderCompleteEvent: {:?}",
                //     order_complete_event
                // );
                let _ = order_complete_event;
                #[cfg(feature = "order_test")]
                {
                    println!("PM: Received OrderCompleteEvent{}", counter_b.next());
                }
                #[cfg(feature = "timestamp_test")]
                {
                    println!("PM Timestamp: {:?}", std::time::SystemTime::now());
                }
            },
            _ => {
                println!("Unsupported event type: {:?}", event.event_type);
            }
        }
    }
}

    /// Updates the available cash in the portfolio
    pub fn update_cash(&mut self, amount: f64) {
        self.portfolio.cash += amount;
        self.portfolio.available_cash += amount;
    }

    /// Updates the position for a specific symbol
    pub fn update_position(&mut self, symbol: String, quantity: i32) {
        self.portfolio
            .positions
            .entry(symbol.clone())
            .and_modify(|q| *q += quantity)
            .or_insert(quantity);
    }

    /// Gets the available cash in the portfolio
    pub fn get_available_cash(&self) -> f64 {
        self.portfolio.available_cash
    }
}

impl ModuleReceive for PortfolioManager {
    fn get_sender(&self) -> Sender<Event> {
        self.subscribe_sender.clone()
    }
}