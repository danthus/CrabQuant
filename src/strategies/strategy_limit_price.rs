use crate::events::*;
use crate::strategies::strategy_helper::*;
use crate::strategy_manager::*;
#[cfg(feature= "order_test")]
use crate::util::Counter;

pub struct StrategyLimitPrice {
    portfolio_local: Portfolio,
    moving_window: MovingWindow,
    factor: f64,
}

impl StrategyLimitPrice{
    /// Creates a new Strategy module
    pub fn new() -> Self {
        let portfolio_local = Portfolio::new(0.0);
        let moving_window = MovingWindow::new(20);
        let factor: f64 = 1.2;
        StrategyLimitPrice {
            portfolio_local,
            moving_window,
            factor,
        }
    }
}

impl Strategy for StrategyLimitPrice {
    fn process(&mut self, market_data_event: MarketDataEvent) -> Option<Event> {
        self.moving_window.update(market_data_event.close as f32);
        
        let ma_short = self.moving_window.average(2);
        let ma_long = self.moving_window.average(3);
        // println!("ma5: {}, ma10: {}", ma5, ma10);

        // ma5 > ma10 buy
        if ma_short > ma_long {
            let quantity = (self.portfolio_local.available_cash / (market_data_event.close * self.factor)).round() as i32;
            if quantity > 0 {
                let fire_and_drop = LimitPriceOrder{ symbol: market_data_event.symbol, amount: quantity, limit_price:market_data_event.low, direction: OrderDirection::Buy };
                let order_place_event = Event::new_order_place(Order::LimitPrice(fire_and_drop));
                Some(order_place_event)
            }
            else {
                None
            }
        }
        // ma5 < ma10 sell
        else if ma_short < ma_long {
            let quantity = (self.portfolio_local.available_cash / (market_data_event.close * self.factor)).round() as i32;
            if quantity > 0 {
                let fire_and_drop = LimitPriceOrder{ symbol: market_data_event.symbol, amount: quantity, limit_price:market_data_event.high, direction: OrderDirection::Sell };
                let order_place_event = Event::new_order_place(Order::LimitPrice(fire_and_drop));
                Some(order_place_event)
            }
            else {
                None
            }
            
        }
        else {
            None
        }
    }
}