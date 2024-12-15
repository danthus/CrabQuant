use crate::event_manager::{ModulePublish, ModuleReceive};
use crate::shared_structures::{
    Event, MarketDataEvent, Order, OrderDirection, OrderPlaceEvent, Portfolio,
};
use crate::PortfolioUpdater;
use crossbeam::channel::{bounded, Receiver, Sender};
use simplelog::debug;

pub struct MockExchange {
    // subscribe_sender is for event_manager to use only.
    // s_sender and s_receiver belongs to a rendezvous channel
    // use s_sender inside module will introduce potential deadlock
    subscribe_sender: Sender<Event>,
    subscribe_receiver: Receiver<Event>,
    // Use publish_sender to send events to event manager
    publish_sender: Option<Sender<Event>>,
    portfolio: Portfolio,
    pending_orders: Vec<Order>,
    fee_function: fn(f64) -> f64,
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
    pub fn new(fee_function: fn(f64) -> f64) -> Self {
        let (subscribe_sender, subscribe_receiver) = bounded(0);
        let portfolio = Portfolio::new(100000.);
        let pending_orders = Vec::new();
        MockExchange {
            subscribe_sender,
            subscribe_receiver,
            publish_sender: None,
            portfolio,
            pending_orders,
            fee_function,
        }
    }

    fn publish(&mut self, event: Event) -> () {
        // To push an Event to EventManager.
        if let Some(publish_sender) = &self.publish_sender {
            publish_sender.send(event).unwrap();
        } else {
            panic!("MEX: publish_sender is not initialized!");
        }
    }

    pub fn run(&mut self) -> () {
        if self.publish_sender.is_none() {
            panic!("MEX: publish_sender is not initialized!");
        }

        #[cfg(feature = "random_sleep_test")]
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
                    println!("MEX: Unsupported event: {:?}", event);
                }
            }
        }
    }

    fn process_marketevent(&mut self, market_data_event: MarketDataEvent) {
        // println!("MEX: Received: {:?}", market_data_event);
        debug!("Received market data: {:?}", market_data_event);

        // Calculate the mean price from market data
        let mean_price = (market_data_event.high + market_data_event.low) / 2.0;
        // println!("MEX: Calculated mean price: {}", mean_price);

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
                                debug!(
                                    "Filling order (market id = {:?}): {:?}",
                                    market_data_event.id, order
                                );
                                filled_orders.push((symbol, amount, mean_price, direction));
                                return false; // Remove this order
                            } else {
                                debug!(
                                    "Dropping order (market id = {:?}): {:?}",
                                    market_data_event.id, order
                                );
                            }
                        }
                        OrderDirection::Sell => {
                            if mean_price >= limit_price {
                                debug!(
                                    "Filling order (market id = {:?}): {:?}",
                                    market_data_event.id, order
                                );
                                filled_orders.push((symbol, amount, mean_price, direction));
                                return false; // Remove this order
                            } else {
                                debug!(
                                    "Dropping order (market id = {:?}): {:?}",
                                    market_data_event.id, order
                                );
                            }
                        }
                    }
                }
                _ => println!("MEX: Unsupported order type"),
            }
            // Set the return to to true if to keep the unprocessed order for the next market data feed
            // Currently as no cancle order mechanism is implemented we will just drop it anyway
            false
        });

        // Process filled orders separately to avoid mutable borrow conflicts
        for (symbol, amount, price, direction) in filled_orders {
            self.update_fill(symbol, amount, price, direction);
        }
        self.update_asset(market_data_event);

        let portfolio_info_event = Event::new_portfolio_info(self.portfolio.clone());
        // println!("MEX: Publishing: {:?}", portfolio_info_event);
        debug!("Publishing portfolio: {:?}", portfolio_info_event);
        self.publish(portfolio_info_event);
    }

    fn process_orderplace(&mut self, order_place_event: OrderPlaceEvent) {
        // Check if order is valid. If yes, modify portfolio and send. If not, drop it.
        // Add order to to_do_list
        debug!("Received order place: {:?}", order_place_event);
        let order = order_place_event.order;
        // Add the parsed order to the pending_orders Vec
        self.pending_orders.push(order.clone());
    }
}

impl PortfolioUpdater for MockExchange {
    fn update_asset(&mut self, market_data: MarketDataEvent) {
        // Calculate the value of the specific symbol based on its current close price
        if let Some(position) = self.portfolio.positions.get(&market_data.symbol) {
            let position_value = *position as f64 * market_data.close;

            // Update the total asset value
            self.portfolio.asset = self.portfolio.cash
                + self
                    .portfolio
                    .positions
                    .iter()
                    .fold(0.0, |total, (symbol, &_amount)| {
                        if symbol == &market_data.symbol {
                            total + position_value
                        } else {
                            total
                        }
                    });

            debug!(
                "Updated portfolio: Value: {}, Cash: {}, Symbol: {}, Position Value: {}",
                self.portfolio.asset, self.portfolio.cash, market_data.symbol, position_value
            );
        }

        // Update the available cash too.
        self.portfolio.available_cash = self.portfolio.cash;
    }

    fn update_fill(&mut self, symbol: String, amount: i32, price: f64, direction: OrderDirection) {
        match direction {
            OrderDirection::Buy => {
                // Calculate the total cost of the buy
                let trade_cost = price * amount.abs() as f64;
                let fee = (self.fee_function)(trade_cost); // Apply fee function
                let total_cost = trade_cost + fee; // Include fee in total cost

                // Deduct cash and update the position
                self.portfolio.cash -= total_cost;

                let position_entry = self.portfolio.positions.entry(symbol.clone()).or_insert(0);
                *position_entry += amount;

                debug!(
                    "Filled Buy Order: Symbol: {}, Amount: {}, Price: {}, Trade Cost: {}, Fee: {}, Total Cost: {}",
                    symbol, amount, price, trade_cost, fee, total_cost
                );
            }
            OrderDirection::Sell => {
                // Calculate the total value of the sell
                let trade_value = price * amount.abs() as f64;
                let fee = (self.fee_function)(trade_value); // Apply fee function
                let net_value = trade_value - fee; // Deduct fee from total value

                // Ensure the sell order is valid before proceeding
                match self.portfolio.positions.get(&symbol) {
                    Some(&position_entry) => {
                        if position_entry < amount.abs() {
                            debug!("Warning: Insufficient holdings to sell {} of {}. Available: {}. Selling the available amount instead.", amount.abs(), symbol, position_entry);
                            let partial_trade_value = price * position_entry as f64;
                            let partial_fee = (self.fee_function)(partial_trade_value);
                            self.portfolio.cash += partial_trade_value - partial_fee; // Update cash with partial value minus fee
                            self.portfolio.positions.insert(symbol.clone(), 0);
                        } else {
                            let new_pos = position_entry - amount;
                            self.portfolio.cash += net_value; // Update cash with net value after fee
                            self.portfolio.positions.insert(symbol.clone(), new_pos);
                        }
                    }
                    None => {
                        debug!(
                            "Warning: No holdings for symbol {} to sell {}.",
                            symbol,
                            amount.abs()
                        );
                        return; // Exit the function without processing the order
                    }
                }

                debug!(
                    "Filled Sell Order: Symbol: {}, Amount: {}, Price: {}, Trade Value: {}, Fee: {}, Net Value: {}",
                    symbol, amount, price, trade_value, fee, net_value
                );
            }
        }
    }
}
