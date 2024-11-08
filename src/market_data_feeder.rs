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
            println!("MarketDataFeeder sending MarketData event");
            self.event_sender.send(Event::MarketData).unwrap();
            thread::sleep(std::time::Duration::from_millis(500));
        });
    }
}
