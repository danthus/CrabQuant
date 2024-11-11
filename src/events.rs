// src/models/mod.rs

#[derive(Debug, Clone)]
pub struct MarketData {
    pub timestamp: String,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: u64,
}

#[derive(Debug, Clone)]
pub struct OrderPlace {
    pub order_id: u64,
    pub quantity: f64,
    pub price: f64,
    // TBD: other fields
}

#[derive(Debug, Clone)]
pub struct OrderComplete {
    pub order_id: u64,
    pub filled_quantity: f64,
    // TBD: other fields
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum EventType {
    MarketData,
    OrderPlace,
    OrderComplete,
}
