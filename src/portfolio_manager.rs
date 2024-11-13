use crate::event_manager::{Event, EventHandler};
use crossbeam::channel::Sender;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// use std::collections::VecDeque;
pub struct PortfolioManager{
    portfolio: Portfolio,
}

// struct Position{
//     symbol_name: String,
//     quantity: u32,
// }
struct Portfolio{
    asset: f64,
    cash: f64,
    available_cash: f64,
    positions: HashMap<String, u32>,
}

impl EventHandler for PortfolioManager {
    fn handle_event(&self, event: &Event, _event_sender: &Sender<Event>) {
        println!("PortfolioManager receiving event: {:?}", event);
    }
}

pub struct PortfolioManagerWrapper {
    p_manager: Arc<Mutex<PortfolioManager>>,
}

impl PortfolioManagerWrapper {
    pub fn new(p_manager: Arc<Mutex<PortfolioManager>>) -> Self {
        PortfolioManagerWrapper { p_manager: p_manager }
    }
}

impl EventHandler for PortfolioManagerWrapper {
    fn handle_event(&self, event: &Event, event_sender: &Sender<Event>) {
        let pm = self.p_manager.lock().unwrap();
        pm.handle_event(event, event_sender);
    }
}

impl PortfolioManager{
    pub fn new(initial_cash: f64) -> Self {
        let portfolio = Portfolio {
            asset: initial_cash,
            cash: initial_cash,
            available_cash: initial_cash,
            positions: HashMap::new(),
        };
        PortfolioManager { portfolio }
    }

    pub fn get_available_cash(&self) -> f64 {
        self.portfolio.available_cash
    }

    pub fn update_cash(&mut self, amount: f64) {
        self.portfolio.cash += amount;
        self.portfolio.available_cash += amount;
    }

    pub fn update_position(&mut self, symbol: String, quantity: u32) {
        self.portfolio.positions.entry(symbol.clone())
            .and_modify(|q| *q += quantity)
            .or_insert(quantity); 
    }
}
