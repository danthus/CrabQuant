use crate::event_manager::{Event, EventHandler};
use crossbeam::channel::Sender;
// use std::collections::VecDeque;
pub struct PortfolioManager{
    portfolio: Portfolio,
}

struct Position{
    symbol_name: String,
    quantity: u32,
}
struct Portfolio{
    asset: f64,
    cash: f64,
    available_cash: f64,
    positions:Vec<Position>,
}

impl EventHandler for PortfolioManager {
    fn handle_event(&self, event: &Event, _event_sender: &Sender<Event>) {
        println!("PortfolioManager receiving event: {:?}", event);
    }
}

impl PortfolioManager{
    pub fn new(initial_cash: f64) -> Self {
        let portfolio = Portfolio {
            asset: initial_cash,
            cash: initial_cash,
            available_cash: initial_cash,
            positions: Vec::new(),
        };
        PortfolioManager { portfolio }
    }
}
