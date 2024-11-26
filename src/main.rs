mod event_manager;
mod events;
mod market_data_feeder;
mod mock_exchange;
mod strategy;
mod util;
mod data_analyzer;
mod strategy_helper;

use crate::event_manager::EventManager;

use market_data_feeder::MarketDataFeeder;
use mock_exchange::MockExchange;
// use portfolio_manager::PortfolioManager;
use strategy::Strategy;
use data_analyzer::DataAnalyzer;
use std::thread;
use events::*;
fn main() {
    // Initialize event manager
    let mut event_manager = EventManager::new();

    // Initialize Modules
    let mut strategy = Strategy::new();
    // Subscribe to specific event types for the strategy module
    event_manager.subscribe::<MarketDataEvent, Strategy>(&strategy);
    event_manager.subscribe::<PortfolioInfoEvent, Strategy>(&strategy);
    event_manager.allow_publish("high".to_string(), &mut strategy);

    let mut mock_exchange: MockExchange = MockExchange::new();
    // Subscribe to specific event types for the mock exchange module
    event_manager.subscribe::<MarketDataEvent, MockExchange>(&mock_exchange);
    event_manager.subscribe::<OrderPlaceEvent, MockExchange>(&mock_exchange);
    event_manager.allow_publish("high".to_string(), &mut mock_exchange);

    let mut market_data_feeder = MarketDataFeeder::new();
    // Allow the market data feeder to publish low-priority events
    event_manager.allow_publish("low".to_string(), &mut market_data_feeder);

    let mut data_analyzer = DataAnalyzer::new();
    // Subscribe the data analyzer to all event types it needs
    event_manager.subscribe::<MarketDataEvent, DataAnalyzer>(&data_analyzer);
    event_manager.subscribe::<PortfolioInfoEvent, DataAnalyzer>(&data_analyzer);
    

    // Run modules
    let _mock_exchange_thread = thread::spawn(move || {
        mock_exchange.run();
    });

    let _strategy_thread = thread::spawn(move || {
        strategy.run();
    });

    let _data_analyzer_thread = thread::spawn(move || {
        data_analyzer.run();
    });

    // Start feeding data
    let _market_data_feeder_thread = thread::spawn(move || {
        market_data_feeder.start_feeding("./data/SPY_1day_sample.csv");
    });

    event_manager.proceed();
    
    return
}