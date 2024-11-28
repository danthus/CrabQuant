use crate::event_manager::{ModulePublish, ModuleReceive};
use crate::events::{Order, OrderDirection, Event, Portfolio, PortfolioInfoEvent, OrderPlaceEvent, MarketDataEvent};
use crate::PortfolioUpdater;
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

    fn process_marketevent(&mut self, market_data_event: MarketDataEvent) {
        println!("MEX: Handling: {:?}", market_data_event);
    
        // Calculate the mean price from market data
        let mean_price = (market_data_event.high + market_data_event.low) / 2.0;
        println!("MEX: Calculated mean price: {}", mean_price);
    
        // Temporary vector to store filled orders
        let mut filled_orders = Vec::new();
    
        // Iterate through pending orders, mark filled ones
        self.pending_orders.retain(|order| {
            match order {
                Order::LimitPrice(limit_order) => {
                    let symbol = limit_order.symbol.clone();
                    let amount = limit_order.amount;
                    let limit_price = limit_order.limit_price;
                    let direction = limit_order.direction;
    
                    match direction {
                        OrderDirection::Buy => {
                            if mean_price <= limit_price {
                                filled_orders.push((symbol, amount, mean_price, direction));
                                return false; // Remove this order
                            }
                        }
                        OrderDirection::Sell => {
                            if mean_price >= limit_price {
                                filled_orders.push((symbol, amount, mean_price, direction));
                                return false; // Remove this order
                            }
                        }
                    }
                }
                _ => println!("MEX: Unsupported order type"),
            }
            // true // Keep order if not filled
            false // Drop the order anyway for now
        });
    
        // Process filled orders separately to avoid mutable borrow conflicts
        for (symbol, amount, price, direction) in filled_orders {
            self.update_fill(symbol, amount, price, direction);
        }

        self.update_asset(market_data_event);
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

impl PortfolioUpdater for MockExchange{
    fn update_asset(&mut self, market_data: MarketDataEvent) {
        // Calculate the value of the specific symbol based on its current close price
        if let Some(position) = self.portfolio.positions.get(&market_data.symbol) {
            let position_value = *position as f64 * market_data.close;
    
            // Update the total asset value
            self.portfolio.asset = self.portfolio.cash
                + self.portfolio.positions.iter().fold(0.0, |total, (symbol, &_amount)| {
                    if symbol == &market_data.symbol {
                        total + position_value
                    } else {
                        // For other symbols, you could fetch their values dynamically if available
                        total
                    }
                });
    
            println!(
                "Updated Portfolio Asset Value: {}, Cash: {}, Symbol: {}, Position Value: {}",
                self.portfolio.asset, self.portfolio.cash, market_data.symbol, position_value
            );
        } else {
            // If the symbol is not in the positions, just recalculate the total asset
            self.portfolio.asset = self.portfolio.cash;
            println!(
                "Symbol {} not found in portfolio. Asset Value set to cash: {}",
                market_data.symbol, self.portfolio.cash
            );
        }
    }
    fn update_fill(&mut self, symbol: String, amount: i32, price: f64, direction: OrderDirection) {
        match direction {
            OrderDirection::Buy => {
                // Calculate the total cost of the buy
                let total_cost = price * amount.abs() as f64;

                // Deduct cash and update the position
                self.portfolio.cash -= total_cost;

                let position_entry = self.portfolio.positions.entry(symbol.clone()).or_insert(0);
                *position_entry += amount;

                println!(
                    "Filled Buy Order: Symbol: {}, Amount: {}, Price: {}, Total Cost: {}",
                    symbol, amount, price, total_cost
                );
            }
            OrderDirection::Sell => {
                // Calculate the total value of the sell
                let total_value = price * amount.abs() as f64;

                // Add cash and update the position
                self.portfolio.cash += total_value;

                if let Some(position_entry) = self.portfolio.positions.get_mut(&symbol) {
                    *position_entry -= amount;

                    // Ensure no negative holdings
                    if *position_entry < 0 {
                        println!(
                            "Warning: Selling more than owned for symbol {}. Setting position to 0.",
                            symbol
                        );
                        *position_entry = 0;
                    }
                } else {
                    println!(
                        "Warning: Attempted to sell {} of {}, but no position exists.",
                        amount, symbol
                    );
                }

                println!(
                    "Filled Sell Order: Symbol: {}, Amount: {}, Price: {}, Total Value: {}",
                    symbol, amount, price, total_value
                );
            }
        }
    }
}