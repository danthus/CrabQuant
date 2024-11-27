// src/models/mod.rs
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use crate::util::Counter;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref EVENT_ID_COUNTER_MDE: Mutex<Counter> = Mutex::new(Counter::new());
    static ref EVENT_ID_COUNTER_OPE: Mutex<Counter> = Mutex::new(Counter::new());
    static ref EVENT_ID_COUNTER_PIE: Mutex<Counter> = Mutex::new(Counter::new());}
// Note that the events should only contain immutable simple data. NVM.


// Events
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum Event {
    MarketData(MarketDataEvent),
    OrderPlace(OrderPlaceEvent),
    PortfolioInfo(PortfolioInfoEvent),
}
impl Event {
    pub fn new_market_data(
        timestamp: String,
        symbol: String,
        open: f64,
        close: f64,
        high: f64,
        low: f64,
        volume: i32,
    ) -> Self {
        let id = EVENT_ID_COUNTER_MDE.lock().unwrap().next();
        Event::MarketData(MarketDataEvent {
            id,
            symbol,
            timestamp,
            open,
            close,
            high,
            low,
            volume,
        })
    }

    pub fn new_order_place(order: Order) -> Self {
        let id = EVENT_ID_COUNTER_OPE.lock().unwrap().next();
        Event::OrderPlace(OrderPlaceEvent { id, order })
    }

    pub fn new_portfolio_info(portfolio: Portfolio) -> Self {
        let id = EVENT_ID_COUNTER_PIE.lock().unwrap().next();
        Event::PortfolioInfo(PortfolioInfoEvent { id, portfolio })
    }
}
// MarketDataEvent
#[derive(Debug, Clone)]
pub struct MarketDataEvent {
    pub id: u64,
    pub symbol: String,
    pub timestamp: String,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: i32,
}

impl PartialEq for MarketDataEvent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id // Events are equal if their `id` is the same
    }
}

impl Eq for MarketDataEvent {}

impl Hash for MarketDataEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state); // Use the `id` field to compute the hash
    }
}

// OrderPlaceEvent
#[derive(Debug, Clone)]
pub struct OrderPlaceEvent {
    pub id: u64,
    pub order: Order,
}

#[derive(Debug, Clone)]
pub enum Order{
    FireAndDrop(FireAndDropOrder),
}

#[derive(Debug, Clone)]
pub enum OrderDirection{
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct FireAndDropOrder{
    pub symbol: String,
    pub amount: i32,
    pub direction: OrderDirection,
}

impl PartialEq for OrderPlaceEvent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id 
    }
}

impl Eq for OrderPlaceEvent {}

impl Hash for OrderPlaceEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state); 
    }
}

// PortfolioInfoEvent
#[derive(Debug, Clone)]
pub struct PortfolioInfoEvent {
    pub id: u64,
    pub portfolio: Portfolio,
    // TBD: other fields
}
impl PartialEq for PortfolioInfoEvent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id // Events are equal if their `id` is the same
    }
}

impl Eq for PortfolioInfoEvent {}

impl Hash for PortfolioInfoEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state); // Use the `id` field to compute the hash
    }
}

#[derive(Debug, Clone)]

pub struct Portfolio {
    pub asset: f64,
    pub cash: f64,
    pub available_cash: f64,
    pub positions: HashMap<String, i32>,
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
}

pub trait PortfolioUpdater {
    fn update_asset(&mut self, portfolio: &mut Portfolio, market_data: MarketDataEvent);
    // fn set_volume(&mut self, portfolio: &mut Portfolio);
}