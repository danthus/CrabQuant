use crate::event_manager::{ModulePublish, ModuleReceive};
use crate::{events::*, market_data_feeder};
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
#[cfg(feature= "order_test")]
use crate::util::Counter;
use std::collections::HashMap;
use std::thread;
use rand::Rng;
use std::time::Duration;


pub struct DataAnalyzer {
    // subscribe_sender is for event_manager to use only.
    // as DA is not supposed to block any other events,
    // it will use a unbounded channel.
    subscribe_sender: Sender<Event>,
    subscribe_receiver: Receiver<Event>,
    // local_portfolio: use this to parse PortfolioInfoEvents
    local_portfolio: Portfolio, 
    asset_history: Vec<Order>,
}

impl ModuleReceive for DataAnalyzer {
    fn get_sender(&self) -> Sender<Event> {
        self.subscribe_sender.clone()
    }
}

impl DataAnalyzer {
    pub fn new() -> Self {
        let (subscribe_sender, subscribe_receiver) = unbounded();
        let local_portfolio = Portfolio::new(0.0);
        let asset_history = Vec::new();
        DataAnalyzer {
            subscribe_sender,
            subscribe_receiver,
            local_portfolio,
            asset_history,
        }
    }

    pub fn run(&mut self) -> () {
        #[cfg(feature= "order_test")]
        let mut counter_a = Counter::new();
        #[cfg(feature= "order_test")]
        let mut counter_b = Counter::new();
        #[cfg(feature= "order_test")]
        let mut counter_c = Counter::new();
        #[cfg(feature= "random_sleep_test")]
        let mut rng = rand::thread_rng();

        // Control Loop
        loop {
            let event = self.subscribe_receiver.recv().unwrap();

            match event {
                Event::MarketData(market_data_event) => {
                    self.process_marketevent(market_data_event);
                }
                Event::PortfolioInfo(portfolio_info_event) => {
                    self.process_portfolioinfo(portfolio_info_event);
                }
                _ => {
                    println!("MockExchange: Unsupported event: {:?}", event);
                }
            }
        }
    }

    fn process_marketevent(&mut self, market_data_event: MarketDataEvent){
        // TODO: use marketdata event to plot baseline
        // Or maintain a vec to update asset (currently to be handled by mockexchange)

    }
    fn process_portfolioinfo(&mut self, portfolio_info_event: PortfolioInfoEvent) {
        println!("DA: Updating event: {:?}", portfolio_info_event);
        self.local_portfolio = portfolio_info_event.portfolio.clone();            
        
        // TODO: update assets vector
        
    }
}