use crate::event_manager::{Event, ModulePublish};
use crate::events::{EventType, MarketDataEvent, EventContent};
use crossbeam::channel::{Sender, Receiver};
use csv::ReaderBuilder;
use std::fs::File;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
// use std::thread;
// use std::time::Duration;

pub struct MarketDataFeeder {
    publish_sender: Option<Sender<Event>>,
}

impl ModulePublish for MarketDataFeeder {
    fn use_sender(&mut self, sender: Sender<Event>) {
        self.publish_sender = Some(sender.clone());
    }
}
impl MarketDataFeeder {
    pub fn new() -> Self {
        MarketDataFeeder {
            publish_sender: None,
        }
    }

    fn publish(&self, event:Event) -> (){
        if let Some(publish_sender) = &self.publish_sender {
            publish_sender.send(event).unwrap();
        } else {
            panic!("publish_sender is not initialized!");
        }
    }

    pub fn start_feeding(&self, csv_path: &str)  {
        let file = File::open(csv_path).expect("Failed to open CSV file");
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        for result in reader.records() {
            let record = result.expect("Failed to read record");
            let market_data = MarketDataEvent {
                timestamp: record[0].to_string(),
                open: record[1].parse().expect("Invalid open value"),
                high: record[2].parse().expect("Invalid high value"),
                low: record[3].parse().expect("Invalid low value"),
                close: record[4].parse().expect("Invalid close value"),
                volume: record[5].parse().expect("Invalid volume value"),
            };

            // Send data through the channel
            println!("MarketDataFeeder: Read and Sending: {:?}", market_data);
            #[cfg(feature= "timeit")]
            println!("MDF Timestamp: {:?}", std::time::SystemTime::now());
            self.publish(Event::new(EventType::TypeMarketData, EventContent::MarketData(market_data)));
            thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}
