mod event_manager;
mod shared_structures;
mod market_data_feeder;
mod mock_exchange;
mod util;
mod data_analyzer;
mod strategy_manager;
mod strategies;

use crate::event_manager::EventManager;

use market_data_feeder::MarketDataFeederLocal;
use mock_exchange::MockExchange;
// use portfolio_manager::PortfolioManager;
use data_analyzer::DataAnalyzer;
use strategy_manager::StrategyManager;
// use strategies::strategy_fire_and_drop::StrategyFireAndDrop;
use strategies::moving_average_crossover::MAcross;
use std::thread;
use shared_structures::*;

use simplelog::*;
use std::fs::File;

fn main() {
    let log_config = ConfigBuilder::new()
        .set_time_level(LevelFilter::Off) // Turn off timestamps
        .build();

    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, log_config.clone(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Debug, log_config, File::create("Trading.log").unwrap()),
        ]
    ).unwrap();
    
    info!("CrabQuant Starting ...");

    // Initialize event manager
    let mut event_manager = EventManager::new();

    // let strategy_fire_and_drop = StrategyFireAndDrop::new();
    let strategy_limit_price = MAcross::new();
    let mut strategy_manager = StrategyManager::new(vec![1.]);
    strategy_manager.add_strategy(Box::new(strategy_limit_price));

    event_manager.subscribe::<MarketDataEvent, StrategyManager>(&strategy_manager);
    event_manager.subscribe::<PortfolioInfoEvent, StrategyManager>(&strategy_manager);
    event_manager.allow_publish("high".to_string(), &mut strategy_manager);

    // // Initialize Modules
    // let mut strategy = Strategy::new();
    // // Subscribe to specific event types for the strategy module
    // event_manager.subscribe::<MarketDataEvent, Strategy>(&strategy);
    // event_manager.subscribe::<PortfolioInfoEvent, Strategy>(&strategy);
    // event_manager.allow_publish("high".to_string(), &mut strategy);

    fn fee_function(trade_cost: f64) -> f64 {
        trade_cost * 0.001 // 1% fee
    }
    let mut mock_exchange: MockExchange = MockExchange::new(fee_function);
    // Subscribe to specific event types for the mock exchange module
    event_manager.subscribe::<MarketDataEvent, MockExchange>(&mock_exchange);
    event_manager.subscribe::<OrderPlaceEvent, MockExchange>(&mock_exchange);
    event_manager.allow_publish("high".to_string(), &mut mock_exchange);

    let mut market_data_feeder = MarketDataFeederLocal::new("TSLA".to_string(), "./data/TSLA_DAY_10Y.csv".to_string());
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
        strategy_manager.run();
    });

    let _data_analyzer_thread = thread::spawn(move || {
        data_analyzer.run();
    });

    // Start feeding data
    let _market_data_feeder_thread = thread::spawn(move || {
        market_data_feeder.start_feeding();
    });

    info!("Mock Exchange, Strategy, Data Analyzer, Data Feeder initialized, start data feeding ...");
    event_manager.proceed();
    
    return
}