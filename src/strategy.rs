use crate::event_manager::{ModulePublish, ModuleReceive};
use crate::events::*;
use crate::strategy_helper::*;
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
    moving_window: MovingWindow,
}

impl Strategy {
    /// Creates a new Strategy module
    pub fn new() -> Self {
        let (subscribe_sender, subscribe_receiver) = bounded(0);
        let portfolio_local = Portfolio::new(0.0);
        let moving_window = MovingWindow::new(20);
        Strategy {
            subscribe_sender,
            subscribe_receiver,
            publish_sender: None,
            portfolio_local,
            moving_window,
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
        self.moving_window.update(market_data_event.close as f32);
        
        let ma5 = self.moving_window.average(2);
        let ma10 = self.moving_window.average(3);
        // println!("ma5: {}, ma10: {}", ma5, ma10);

        // ma5 > ma10 buy
        if ma5 > ma10 {
            let fire_and_drop = FireAndDropOrder{ symbol: String::from("None"), amount: 100, direction: OrderDirection::Buy };
            let order_place_event = Event::new_order_place(Order::FireAndDrop(fire_and_drop));
            println!("Strategy: Publishing event: {:?}", order_place_event);
            self.publish(order_place_event);
        }
        // ma5 < ma10 sell
        else if ma5 < ma10 {
            let fire_and_drop = FireAndDropOrder{ symbol: String::from("None"), amount: 100, direction: OrderDirection::Sell };
            let order_place_event = Event::new_order_place(Order::FireAndDrop(fire_and_drop));
            println!("Strategy: Publishing event: {:?}", order_place_event);
            self.publish(order_place_event);
        }
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