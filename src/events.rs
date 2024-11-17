// src/models/mod.rs


// Note that the events should only contain immutable simple data.
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
    pub order_id: u64,
    pub quantity: i32,
    pub price: f64,
    // TBD: other fields
}

#[derive(Debug, Clone)]
pub struct OrderCompleteEvent {
    pub order_id: u64,
    pub filled_quantity: i32,
    // TBD: other fields
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum EventType {
    TypeMarketData,
    TypeOrderPlace,
    TypeOrderComplete,
}

#[derive(Debug, Clone)]
pub enum EventContent {
    MarketData(MarketDataEvent),
    OrderPlace(OrderPlaceEvent),
    OrderComplete(OrderCompleteEvent),
}

// #[derive(Debug, Hash, Eq, PartialEq, Clone)]
// pub enum Modules {
//     MarketDataFeeder,
//     MockExchange,
//     Strategy,
//     PortfolioManager,
// }
