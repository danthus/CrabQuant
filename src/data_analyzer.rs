use crate::event_manager::{Event, ModulePublish, ModuleReceive};
use crate::events::{Order, EventType, Portfolio, PortfolioInfoEvent, OrderPlaceEvent, MarketDataEvent, EventContent};
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

            match event.event_type{
                EventType::TypeMarketData => {
                    self.process_marketevent(event);
                }
                EventType::TypePortfolioInfo => {
                    self.process_portfolioinfo(event);
                }
                _ => {
                    println!("MockExchange: Unsupported event type: {:?}", event.event_type);
                }
            }
        }
    }

    fn process_marketevent(&mut self, event:Event){
        // TODO: use marketdata event to plot baseline
        // Or maintain a vec to update asset (currently to be handled by mockexchange)
    }
    fn process_portfolioinfo(&mut self, event: Event) {
        // Ensure the event content is of type PortfolioInfo
        if let EventContent::PortfolioInfo(portfolio_info_event) = event.contents {
            // Update the local portfolio with the received portfolio information
            self.local_portfolio = portfolio_info_event.portfolio.clone();
            // println!("Updated local portfolio: {:?}", self.portfolio_local);
            // TODO: update assets vector

        } else {
            // Handle invalid event content gracefully
            eprintln!("Received an invalid event for PortfolioInfo: {:?}", event.contents);
        }
    }
}