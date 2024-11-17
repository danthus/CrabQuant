mod event_manager;
mod events;
mod market_data_feeder;
mod mock_exchange;
mod portfolio_manager;
mod strategy;

use crate::event_manager::EventManager;
use crate::events::EventType;

use market_data_feeder::MarketDataFeeder;
use mock_exchange::MockExchange;
use portfolio_manager::PortfolioManager;
use strategy::Strategy;
use std::thread;

fn main() {
    // Initialize event manager
    let mut event_manager = EventManager::new();

    // Initialize Modules
    let mut mock_exchange = MockExchange::new();
    event_manager.subscribe(EventType::TypeMarketData, &mock_exchange);
    event_manager.subscribe(EventType::TypeOrderPlace, &mock_exchange);
    event_manager.allow_publish("high".to_string(), &mut mock_exchange);

    let mut market_data_feeder = MarketDataFeeder::new();
    event_manager.allow_publish("low".to_string(), &mut market_data_feeder);

    let mut strategy = Strategy::new();
    event_manager.subscribe(EventType::TypeMarketData, &strategy);
    event_manager.allow_publish("high".to_string(), &mut strategy);

    let mut portfolio_manager = PortfolioManager::new(1000000000.0);
    event_manager.subscribe(EventType::TypeOrderPlace, &portfolio_manager);
    event_manager.subscribe(EventType::TypeOrderComplete, &portfolio_manager);

    // Run modules
    let mock_exchange_thread = thread::spawn(move || {
        mock_exchange.run();
    });

    let strategy_thread = thread::spawn(move || {
        strategy.run();
    });

    let portfolio_manager_thread = thread::spawn(move || {
        portfolio_manager.run();
    });

    // Start feeding data
    let market_data_feeder_thread = thread::spawn(move || {
        market_data_feeder.start_feeding("./data/SPY_1min_sample.csv");
    });

    // thread::sleep(std::time::Duration::from_millis(1));

    // Process events in the main thread
    let event_manager_thread = thread::spawn(move || {
        event_manager.process_events();
    });

    // Wait for threads to complete
    mock_exchange_thread.join().unwrap();
    market_data_feeder_thread.join().unwrap();
    strategy_thread.join().unwrap();
    portfolio_manager_thread.join().unwrap();
    event_manager_thread.join().unwrap();
}