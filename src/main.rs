mod events;
mod event_manager;
mod market_data_feeder;
mod strategy;
mod mock_exchange;
mod portfolio_manager;

use crate::event_manager::EventManager;
use crate::events::EventType;

use std::sync::{Arc, Mutex};
use std::thread;
use market_data_feeder::MarketDataFeeder;
use strategy::Strategy;
use mock_exchange::MockExchange;
use portfolio_manager::PortfolioManager;

fn main() {
    let event_manager = Arc::new(Mutex::new(EventManager::new()));
    let initial_cash = 1000000.;
    let portfolio_manager = PortfolioManager::new(initial_cash);

    {
        let mut em = event_manager.lock().unwrap();
        em.subscribe(EventType::MarketData, Arc::new(Strategy));
        em.subscribe(EventType::MarketData, Arc::new(MockExchange));
        em.subscribe(EventType::OrderPlace, Arc::new(MockExchange));
        em.subscribe(EventType::OrderComplete, Arc::new(portfolio_manager));
    }

    let event_sender = event_manager.lock().unwrap().get_event_sender();
    
    let event_manager_clone = Arc::clone(&event_manager);
    thread::spawn(move || {
        EventManager::process_events(event_manager_clone);
    });

    let market_data_feeder = MarketDataFeeder::new(event_sender.clone());
    market_data_feeder.start();

    thread::sleep(std::time::Duration::from_secs(5));
}
