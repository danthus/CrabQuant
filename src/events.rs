// src/models/mod.rs
use std::collections::HashMap;

// Note that the events should only contain immutable simple data. NVM.
#[derive(Debug, Clone)]
pub struct MarketDataEvent {
    pub timestamp: String,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: i32,
}

#[derive(Debug, Clone)]
pub struct OrderPlaceEvent {
    pub event_id: u64,
    pub order: Order,
}

#[derive(Debug, Clone)]

pub struct Portfolio {
    pub asset: f64,
    pub cash: f64,
    pub available_cash: f64,
    pub positions: HashMap<String, i32>,
}

#[derive(Debug, Clone)]
pub struct PortfolioInfoEvent {
    pub id: u64,
    pub portfolio: Portfolio,
    // TBD: other fields
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum EventType {
    TypeMarketData,
    TypeOrderPlace,
    TypePortfolioInfo,
}

#[derive(Debug, Clone)]
pub enum EventContent {
    MarketData(MarketDataEvent),
    OrderPlace(OrderPlaceEvent),
    PortfolioInfo(PortfolioInfoEvent),
}

// #[derive(Debug, Hash, Eq, PartialEq, Clone)]
// pub enum Modules {
//     MarketDataFeeder,
//     MockExchange,
//     Strategy,
//     PortfolioManager,
// }
#[derive(Debug, Clone)]
pub enum Order{
    FireAndDrop(FireAndDropOrder),
}

#[derive(Debug, Clone)]

pub enum OrderDirection{
    Buy,
    Sell,
}#[derive(Debug, Clone)]

pub struct FireAndDropOrder{
    order_id: i32,
    amount: i32,
    direction: OrderDirection,
}

impl Portfolio{
    pub fn new(initial_cash:f64 ) -> Self{
        Portfolio{
            asset : 0.0,
            cash : initial_cash,
            available_cash : initial_cash,
            positions : HashMap::new(),
        }
    }

    fn try_update_asset(market_data: MarketDataEvent) -> () {
        // tries to update asset
    }
}