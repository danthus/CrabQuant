use crate::MarketDataEvent;

use crate::event_manager::{ModulePublish, ModuleReceive};
use crate::shared_structures::*;
use crossbeam::channel::{Sender, Receiver, bounded};
use simplelog::*;

pub trait Strategy {
    /// Called when market data is received.
    /// Returns an optional order event if the strategy decides to trade.
    fn process(&mut self, market_data_event: MarketDataEvent) -> Option<Event>;
    fn update(&mut self, portfolio: Portfolio);
}

pub struct StrategyManager {
    subscribe_sender: Sender<Event>,
    subscribe_receiver: Receiver<Event>,
    publish_sender: Option<Sender<Event>>,
    portfolio_local: Portfolio,
    strategies: Vec<Box<dyn Strategy + Send>>,
    weights: Vec<f64>,
}

impl StrategyManager {
    pub fn new(weights: Vec<f64>) -> Self {
        let (subscribe_sender, subscribe_receiver) = bounded(0);
        let portfolio_local = Portfolio::new(0.0);

        StrategyManager {
            subscribe_sender,
            subscribe_receiver,
            publish_sender: None,
            portfolio_local,
            strategies: Vec::new(),
            weights,
        }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy + Send>) {
        self.strategies.push(strategy);
    }

    pub fn run(&mut self) {
        if self.publish_sender.is_none() {
            panic!("Publish sender is not initialized!");
        }
        if self.strategies.len() != self.weights.len() {
            panic!("Number of weights should equal number of strategies");
        }
        let mut events_to_publish:Vec<Event> = Vec::new();

        loop {
            let event = self.subscribe_receiver.recv().unwrap();

            match event {
                Event::MarketData(market_data_event) => {
                    // println!("Strategy: Received: {:?}", market_data_event);
                    self.process_marketevent(market_data_event, &mut events_to_publish);
                    // thread::sleep(time::Duration::from_secs(1));
                }
                Event::PortfolioInfo(portfolio_info_event) => {
                    // println!("Strategy: Received: {:?}", portfolio_info_event);
                    self.process_portfolioinfo(portfolio_info_event);
                }
                _ => {
                    // println!("Strategy: Unsupported event: {:?}", event);
                }
            }
        }
    }

    fn process_portfolioinfo(&mut self, portfolio_info_event: PortfolioInfoEvent) {
        // Ensure the event content is of type PortfolioInfo
        self.portfolio_local = portfolio_info_event.portfolio.clone();

        for (strategy, weight) in &mut self.strategies.iter_mut().zip(self.weights.iter()) {
            let mut sub_portfolio = self.portfolio_local.clone();
            sub_portfolio.available_cash *= weight;
            strategy.update(sub_portfolio);
        }
    }

    fn process_marketevent(&mut self, market_data_event: MarketDataEvent, events: &mut Vec<Event>){

        for strategy in &mut self.strategies {

            if let Some(order_place_event) = strategy.process(market_data_event.clone()) {
                // events.push(order_place_event);
                // self.publish(order_place_event);
                events.push(order_place_event);
            }

            // if let Some(order_event) = strategy.on_market_data(event.clone()) {
            //     self.event_sender.send(Event::OrderPlaceEvent(order_event)).unwrap();
            // }
        }
        for event in events.drain(..) {
            // println!("Strategy: Publishing event: {:?}, \n\tbased on {:?}", event, market_data_event);
            debug!("Publish order place (market id = {:?}): {:?}", market_data_event.id, event);
            self.publish(event);
        } 
    }

    fn publish(&self, event: Event) {
        if let Some(publish_sender) = &self.publish_sender {
            publish_sender.send(event).unwrap();
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