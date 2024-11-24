use crate::event_manager::{ModulePublish, ModuleReceive};
use crate::events::*;
use crossbeam::channel::{Sender, Receiver, bounded};
#[cfg(feature= "order_test")]
use crate::util::Counter;
use rand::Rng;
use std::time::Duration;
use std::thread;

pub struct Strategy {
    subscribe_sender: Sender<Event>,
    subscribe_receiver: Receiver<Event>,
    publish_sender: Option<Sender<Event>>,
    portfolio_local: Portfolio,
}

impl Strategy {
    /// Creates a new Strategy module
    pub fn new() -> Self {
        let (subscribe_sender, subscribe_receiver) = bounded(0);
        let portfolio_local = Portfolio::new(0.0);
        Strategy {
            subscribe_sender,
            subscribe_receiver,
            publish_sender: None,
            portfolio_local,
        }
    }

    /// Runs the strategy logic, processing MarketDataEvent and sending OrderPlaceEvent
    pub fn run(&mut self) {
        if self.publish_sender.is_none() {
            panic!("Publish sender is not initialized!");
        }
        
        loop {
            // Receive an event from the subscribe_receiver
            let event = self.subscribe_receiver.recv().unwrap();

            // Call the corresponding process function based on event type
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
        // let orders = 
        // self.publish(Event::new_order_place(orders));
    }

    /// Helper method to publish an event
    fn publish(&self, event: Event) {
        if let Some(publish_sender) = &self.publish_sender {
            publish_sender.send(event).unwrap();
        } else {
            panic!("Publish sender is not initialized!");
        }
    }
}

impl ModuleReceive for Strategy {
    fn get_sender(&self) -> Sender<Event> {
        self.subscribe_sender.clone()
    }
}

impl ModulePublish for Strategy {
    fn use_sender(&mut self, sender: Sender<Event>) {
        self.publish_sender = Some(sender);
    }
}