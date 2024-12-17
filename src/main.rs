mod data_analyzer;
mod event_manager;
mod market_data_feeder;
mod mock_exchange;
mod shared_structures;
mod strategies;
mod strategy_manager;
mod util;

use crate::event_manager::EventManager;

use market_data_feeder::MarketDataFeederLocal;
use mock_exchange::MockExchange;
use data_analyzer::DataAnalyzer;
use strategy_manager::StrategyManager;
use shared_structures::*;
use std::thread;
use strategies::moving_average_crossover::MAcross;

use simplelog::*;
use std::fs::File;

fn main() {
    let log_config = ConfigBuilder::new()
        .set_time_level(LevelFilter::Off) // Turn off timestamps
        .build();

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            log_config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            log_config,
            File::create("Trading.log").unwrap(),
        ),
    ])
    .unwrap();

    info!("CrabQuant Starting ...");

    // Initialize event manager
    let mut event_manager = EventManager::new();

    // Initilize the strategy and add to strategy_manager
    let strategy_ma_cross = MAcross::new(5, 10);
    let mut strategy_manager = StrategyManager::new();
    strategy_manager.add_strategy(Box::new(strategy_ma_cross));

    // Let strategy_manager subscribe to MarketDataEvent and PortfolioInfoEvent.
    event_manager.subscribe::<MarketDataEvent, StrategyManager>(&strategy_manager);
    event_manager.subscribe::<PortfolioInfoEvent, StrategyManager>(&strategy_manager);
    // Allow strategy_manager to publish events.
    event_manager.allow_publish("high".to_string(), &mut strategy_manager);

    // Initialize the mock_exchange for stock.
    /*
    Custom fee in the fee_function.
    It will be applied to every transactions (buy and sell).
    */
    fn fee_function(trade_cost: f64) -> f64 {
        // 0.1% percentage and 0 fixed fee
        trade_cost * 0.001+ 0.0
    }
    let mut mock_exchange: MockExchange = MockExchange::new(fee_function);
    // Let event_manager subscribe to MarketDataEvent and OrderPlaceEvent.
    event_manager.subscribe::<MarketDataEvent, MockExchange>(&mock_exchange);
    event_manager.subscribe::<OrderPlaceEvent, MockExchange>(&mock_exchange);
    // Allow event_manager to publish events.
    event_manager.allow_publish("high".to_string(), &mut mock_exchange);

    // Initialize market_data_feeder.
    let mut market_data_feeder =
        MarketDataFeederLocal::new("TSLA".to_string(), "./data/TSLA_1min_2W.csv".to_string());
    // Allow the market data feeder to publish low-priority events
    event_manager.allow_publish("low".to_string(), &mut market_data_feeder);

    // Initialize a data_analyzer
    let mut data_analyzer = DataAnalyzer::new();
    // Let the data analyzer subscribe to all event types it needs
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

    info!(
        "Mock Exchange, Strategy, Data Analyzer, Data Feeder initialized, start data feeding ..."
    );
    event_manager.proceed();

    return;
}
