use crate::events::{MarketData, EventType};
use crate::event_manager::Event;
use crossbeam::channel::Sender;
use std::thread;

pub struct MarketDataFeeder {
    event_sender: Sender<Event>,
}

impl MarketDataFeeder {
    pub fn new(event_sender: Sender<Event>) -> Self {
        MarketDataFeeder { event_sender }
    }

    pub fn start(self) {
        thread::spawn(move || loop {
            let market_data = MarketData {
                timestamp: "0".to_string(),
                open: 0.0,
                close: 0.0,
                high: 0.0,
                low: 0.0,
                volume: 0,
            };

            let event = Event::new(EventType::MarketData, market_data);

            self.event_sender.send(event).unwrap();

            println!("MarketDataFeeder sending MarketData event.");

            thread::sleep(std::time::Duration::from_millis(500));
        });
    }
}
