mod event_manager;
mod market_data_feeder;
mod strategy;
mod mock_exchange;
mod portfolio_manager;

use std::sync::{Arc, Mutex};
use std::thread;
use event_manager::{Event, EventManager};
use market_data_feeder::MarketDataFeeder;
use strategy::Strategy;
use mock_exchange::MockExchange;
use portfolio_manager::PortfolioManager;

fn main() {
    let event_manager = Arc::new(Mutex::new(EventManager::new()));

    {
        let mut em = event_manager.lock().unwrap();
        em.subscribe(Event::MarketData, Arc::new(Strategy));
        em.subscribe(Event::MarketData, Arc::new(MockExchange));
        em.subscribe(Event::OrderPlace, Arc::new(MockExchange));
        em.subscribe(Event::OrderComplete, Arc::new(PortfolioManager));
    }

    let event_sender = event_manager.lock().unwrap().get_event_sender();
    
    let event_manager_clone = Arc::clone(&event_manager);
    thread::spawn(move || {
        EventManager::process_events(event_manager_clone);
    });

    event_sender.send(Event::MarketData).unwrap();

    let market_data_feeder = MarketDataFeeder::new(event_sender.clone());
    market_data_feeder.start();

    thread::sleep(std::time::Duration::from_secs(5));
}
