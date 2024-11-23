mod event_manager;
mod events;
mod market_data_feeder;
mod mock_exchange;
mod strategy;
mod util;

use crate::event_manager::EventManager;
use crate::events::EventType;

use market_data_feeder::MarketDataFeeder;
use mock_exchange::MockExchange;
// use portfolio_manager::PortfolioManager;
use strategy::Strategy;
use std::thread;

fn main() {
    // Initialize event manager
    let mut event_manager = EventManager::new();

    // Initialize Modules
    let mut strategy = Strategy::new();
    event_manager.subscribe(EventType::TypeMarketData, &strategy);
    event_manager.subscribe(EventType::TypePortfolioInfo, &strategy);
    event_manager.allow_publish("high".to_string(), &mut strategy);

    let mut mock_exchange: MockExchange = MockExchange::new();
    event_manager.subscribe(EventType::TypeMarketData, &mock_exchange);
    event_manager.subscribe(EventType::TypeOrderPlace, &mock_exchange);
    event_manager.allow_publish("high".to_string(), &mut mock_exchange);

    let mut market_data_feeder = MarketDataFeeder::new();
    event_manager.allow_publish("low".to_string(), &mut market_data_feeder);

    // let mut portfolio_manager = PortfolioManager::new(1000000000.0);
    // event_manager.subscribe(EventType::TypeOrderPlace, &portfolio_manager);
    // event_manager.subscribe(EventType::TypePortfolioInfo, &portfolio_manager);

    // Run modules
    let _mock_exchange_thread = thread::spawn(move || {
        mock_exchange.run();
    });

    let _strategy_thread = thread::spawn(move || {
        strategy.run();
    });


    // Start feeding data
    let _market_data_feeder_thread = thread::spawn(move || {
        market_data_feeder.start_feeding("./data/SPY_1min_sample.csv");
    });

    event_manager.proceed();
    
    return
}