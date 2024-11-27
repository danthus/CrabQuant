use crate::event_manager::{ModulePublish, ModuleReceive};
use crate::events::{Order, Event, Portfolio, PortfolioInfoEvent, OrderPlaceEvent, MarketDataEvent};
use crossbeam::channel::{bounded, Receiver, Sender};
#[cfg(feature= "order_test")]
use crate::util::Counter;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::thread;
use rand::Rng;
use std::time::Duration;


pub struct MockExchange {
    // subscribe_sender is for event_manager to use only.
    // s_sender and s_receiver belongs to a rendezvous channel
    // use s_sender inside module will introduce deadlock
    subscribe_sender: Sender<Event>,
    subscribe_receiver: Receiver<Event>,
    // Use publish_sender to send events to event manager
    publish_sender: Option<Sender<Event>>,
    portfolio: Portfolio,
    pending_orders: Vec<Order>,
}



impl ModuleReceive for MockExchange {
    fn get_sender(&self) -> Sender<Event> {
        self.subscribe_sender.clone()
    }
}

impl ModulePublish for MockExchange {
    fn use_sender(&mut self, sender: Sender<Event>) {
        // MockModule receives the Sender from EventManager to publish events
        // You can store this sender if needed or use it directly
        self.publish_sender = Some(sender.clone());
    }
}

impl MockExchange {
    pub fn new() -> Self {
        let (subscribe_sender, subscribe_receiver) = bounded(0);
        let portfolio = Portfolio::new(1000000.);
        let pending_orders = Vec::new();
        MockExchange {
            subscribe_sender,
            subscribe_receiver,
            publish_sender: None,
            portfolio,
            pending_orders,
        }
    }

    fn publish(&mut self, event:Event) -> (){
        // To push an Event to EventManager.
        if let Some(publish_sender) = &self.publish_sender {
            publish_sender.send(event).unwrap();
        } else {
            panic!("publish_sender is not initialized!");
        }
    }

    pub fn run(&mut self) -> () {
        if self.publish_sender.is_none() {
            panic!("publish_sender is not initialized!");
        }
        #[cfg(feature= "order_test")]
        let mut counter_a = Counter::new();
        #[cfg(feature= "order_test")]
        let mut counter_b = Counter::new();
        #[cfg(feature= "order_test")]
        let mut counter_c = Counter::new();
        #[cfg(feature= "random_sleep_test")]
        let mut rng = rand::thread_rng();

        loop {
            let event = self.subscribe_receiver.recv().unwrap();
    
            match event {
                Event::MarketData(market_data_event) => {
                    self.process_marketevent(market_data_event);
                }
                Event::OrderPlace(order_place_event) => {
                    self.process_orderplace(order_place_event);
                }
                _ => {
                    println!("Strategy: Unsupported event: {:?}", event);
                }
            }
        }
    }

    fn process_marketevent(&mut self, market_data_event:MarketDataEvent){
        // TODO: Check if order is valid. If yes, modify portfolio and send. If not, drop it.
        // println!("MEX: Processing event: {:?}", market_data_event);
        println!("MEX: Handling: {:?}", market_data_event);
    }

    fn process_orderplace(&mut self, order_place_event:OrderPlaceEvent){
        // Check if order is valid. If yes, modify portfolio and send. If not, drop it.
        // Add order to to_do_list
        let order = order_place_event.order;

        // Add the parsed order to the pending_orders Vec
        self.pending_orders.push(order.clone());

        let portfolio_info_event = Event::new_portfolio_info(self.portfolio.clone());
        println!("MEX: Publishing event: {:?}", portfolio_info_event);
        self.publish(portfolio_info_event);
    }
}
