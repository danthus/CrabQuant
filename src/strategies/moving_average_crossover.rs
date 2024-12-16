/*
A sample strategy, executes a simple single sided MA-cross.
The strategy would generate a buy signal when MA-short surpass the
MA-long, and a sell signal when MA-short falls under MA-long.
The Strategy will simply buy with all available cash to buy the 
instrument when a buy signal is generated, and sell all current
holdings when a sell signal is generated.
*/

/*
For a strategy, always use these crates.
*/
use crate::shared_structures::*;
use crate::strategies::strategy_helper::*;
use crate::strategy_manager::*;


/*
Define data structures that will be used in the strategy in
this section.

Note that in any strategy, a local_portfolio is needed to 
implement the trait bound:
update(&mut self, portfolio: Portfolio)
This trait will update the local portfolio whenever a 
PortfolioInfoEvent is received, which includes an updated
portfolio after some transactions.
You should not change the implementation of the update method.

The process trait will be executed each time a new MarketDataEvent
is received.

You can also define a structure in the strategy_helper.rs, if
such structure will be used by multiple strategies.
*/
#[derive(PartialEq)]
enum LastSignal{
    IsBuy,
    IsSell,
    IsNone,
}

pub struct MAcross {
    portfolio_local: Portfolio,
    moving_window: MovingWindow,
    price_factor: f64,
    volume_factor: f32,
    last_signal: LastSignal,
    short: usize,
    long: usize,
}

impl MAcross{
    /// Creates a new Strategy module
    pub fn new(short: usize, long: usize) -> Self {
        let portfolio_local = Portfolio::new(0.0);
        let moving_window = MovingWindow::new(long);
        let price_factor: f64 = 1.2;
        let volume_factor: f32 = 1.;
        let last_signal = LastSignal::IsNone;
        MAcross {
            portfolio_local,
            moving_window,
            price_factor,
            volume_factor,
            last_signal,
            short,
            long,
        }
    }
}

impl Strategy for MAcross {
    fn process(&mut self, market_data_event: MarketDataEvent) -> Option<Event> {
        /*
        Implement logic to process MarketDataEvent in this traint.
        To place an order, return a Option<Event::orderplace>. An order will 
        the be added to the mock_exchange and waits to be executed or dropped.
        You can also generate other types or self-defined event types to be 
        process by some other modules if needed.
         */

        // Update moving_window
        self.moving_window.update(market_data_event.close as f32);
        let ma_short = self.moving_window.average(self.short);
        let ma_long = self.moving_window.average(self.long);
        
        // ma_short > ma_long buy and last signal is not buy
        if ma_short > ma_long && self.last_signal != LastSignal::IsBuy {
            let quantity = (self.portfolio_local.available_cash / (market_data_event.close * self.price_factor)).floor() as i32;

            let max_volume = (market_data_event.volume as f32 *self.volume_factor).floor() as i32;
            let buy_volume = if quantity > max_volume {max_volume} else {quantity};
            self.last_signal = LastSignal::IsBuy;

            if quantity > 0 {
                let limit_price_order = LimitPriceOrder{ symbol: market_data_event.symbol, amount: buy_volume, limit_price:market_data_event.low*2., direction: OrderDirection::Buy };
                let order_place_event = Event::new_order_place(Order::LimitPrice(limit_price_order));
                self.portfolio_local.available_cash = self.portfolio_local.available_cash - quantity as f64*market_data_event.close;
                Some(order_place_event)
            }
            else {
                None
            }
            
        } 
        // ma_short < ma_long sell and last signal is not sell
        else if ma_short < ma_long && self.last_signal != LastSignal::IsSell {
            self.last_signal = LastSignal::IsSell;            
            if let Some(current_position) = self.portfolio_local.positions.get(&market_data_event.symbol) {

                if *current_position > 0 {
                    let limit_price_order = LimitPriceOrder{ symbol: market_data_event.symbol, amount: *current_position, limit_price:market_data_event.low*0., direction: OrderDirection::Sell };
                    let order_place_event = Event::new_order_place(Order::LimitPrice(limit_price_order));
                    Some(order_place_event)
                }
                else{
                    None
                }
            }
            else {
                None
            }
        }
        else {
            None
        }
    }

    fn update(&mut self, portfolio: Portfolio) {
        /*
        This function is to update the local_portfolio.
        It should not be changed.
         */
        self.portfolio_local = portfolio.clone();
    }
}