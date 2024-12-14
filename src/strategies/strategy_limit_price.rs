use crate::events::*;
use crate::strategies::strategy_helper::*;
use crate::strategy_manager::*;
#[cfg(feature= "order_test")]
use crate::util::Counter;

#[derive(PartialEq)]
enum LastSignal{
    IsBuy,
    IsSell,
    IsNone,
}

pub struct StrategyLimitPrice {
    portfolio_local: Portfolio,
    moving_window: MovingWindow,
    price_factor: f64,
    volume_factor: f32,
    last_signal: LastSignal,
}

impl StrategyLimitPrice{
    /// Creates a new Strategy module
    pub fn new() -> Self {
        let portfolio_local = Portfolio::new(0.0);
        let moving_window = MovingWindow::new(20);
        let price_factor: f64 = 1.2;
        let volume_factor: f32 = 1.;
        let last_signal = LastSignal::IsNone;
        StrategyLimitPrice {
            portfolio_local,
            moving_window,
            price_factor,
            volume_factor,
            last_signal,
        }
    }
}

impl Strategy for StrategyLimitPrice {
    fn process(&mut self, market_data_event: MarketDataEvent) -> Option<Event> {
        self.moving_window.update(market_data_event.close as f32);
        
        let ma_short = self.moving_window.average(5);
        let ma_long = self.moving_window.average(10);
        // let quantity = (self.portfolio_local.available_cash / (market_data_event.close * self.price_factor)).floor() as i32;
        
        // ma_short > ma_long buy and last signal is not buy
        if ma_short > ma_long && self.last_signal != LastSignal::IsBuy {
            let quantity = (self.portfolio_local.available_cash / (market_data_event.close * self.price_factor)).floor() as i32;

            let max_volume = (market_data_event.volume as f32 *self.volume_factor).floor() as i32;
            let buy_volume = if quantity > max_volume {max_volume} else {quantity};
            self.last_signal = LastSignal::IsBuy;

            if quantity > 0 {
                let fire_and_drop = LimitPriceOrder{ symbol: market_data_event.symbol, amount: buy_volume, limit_price:market_data_event.low*2., direction: OrderDirection::Buy };
                let order_place_event = Event::new_order_place(Order::LimitPrice(fire_and_drop));
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
                // if quantity > 0 && quantity < *current_position {
                //     let fire_and_drop = LimitPriceOrder{ symbol: market_data_event.symbol, amount: quantity, limit_price:market_data_event.low, direction: OrderDirection::Sell };
                //     let order_place_event = Event::new_order_place(Order::LimitPrice(fire_and_drop));
                //     Some(order_place_event)
                // }
                // else if quantity > *current_position {
                //     let fire_and_drop = LimitPriceOrder{ symbol: market_data_event.symbol, amount: *current_position, limit_price:market_data_event.low, direction: OrderDirection::Sell };
                //     let order_place_event = Event::new_order_place(Order::LimitPrice(fire_and_drop));
                //     Some(order_place_event)
                // }
                // else {
                //     None
                // }
                if *current_position > 0 {
                    let fire_and_drop = LimitPriceOrder{ symbol: market_data_event.symbol, amount: *current_position, limit_price:market_data_event.low*0., direction: OrderDirection::Sell };
                    let order_place_event = Event::new_order_place(Order::LimitPrice(fire_and_drop));
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
        self.portfolio_local = portfolio.clone();
        // println!("");
    }
}