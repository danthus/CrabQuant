use crate::MarketDataEvent;

use crate::event_manager::{ModulePublish, ModuleReceive};
use crate::events::*;
use crossbeam::channel::{Sender, Receiver, bounded};

pub trait Strategy {
    /// Called when market data is received.
    /// Returns an optional order event if the strategy decides to trade.
    fn process(&mut self, market_data_event: MarketDataEvent) -> Option<Event>;
}

pub struct StrategyManager {
    subscribe_sender: Sender<Event>,
    subscribe_receiver: Receiver<Event>,
    publish_sender: Option<Sender<Event>>,
    portfolio_local: Portfolio,
    strategies: Vec<Box<dyn Strategy + Send>>,
    events: Vec<Event>,
}

impl StrategyManager {
    pub fn new() -> Self {
        let (subscribe_sender, subscribe_receiver) = bounded(0);
        let portfolio_local = Portfolio::new(0.0);

        StrategyManager {
            subscribe_sender,
            subscribe_receiver,
            publish_sender: None,
            portfolio_local,
            strategies: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy + Send>) {
        self.strategies.push(strategy);
    }

    pub fn run(&mut self) {
        if self.publish_sender.is_none() {
            panic!("Publish sender is not initialized!");
        }

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
                    println!("Strategy: Unsupported event: {:?}", event);
                }
            }
        }
    }

    fn process_portfolioinfo(&mut self, portfolio_info_event: PortfolioInfoEvent) {
        // Ensure the event content is of type PortfolioInfo
        self.portfolio_local = portfolio_info_event.portfolio.clone();
    }

    fn process_marketevent(&mut self, market_data_event: MarketDataEvent){

        for strategy in &mut self.strategies {

            if let Some(order_place_event) = strategy.process(market_data_event.clone()) {
                self.events.push(order_place_event);
            }

            // if let Some(order_event) = strategy.on_market_data(event.clone()) {
            //     self.event_sender.send(Event::OrderPlaceEvent(order_event)).unwrap();
            // }
        }
        self.publish(&mut self.events.clone());
    }

    fn publish(&self, events: &mut Vec<Event>) {
        if let Some(publish_sender) = &self.publish_sender {
            for event in events.drain(..){
                publish_sender.send(event).unwrap();
            }
        } else {
            panic!("Publish sender is not initialized!");
        }
    }

}

impl ModuleReceive for StrategyManager {
    fn get_sender(&self) -> Sender<Event> {
        self.subscribe_sender.clone()
    }
}

impl ModulePublish for StrategyManager {
    fn use_sender(&mut self, sender: Sender<Event>) {
        self.publish_sender = Some(sender);
    }
}