use crate::event_manager::ModulePublish;
use crate::events::*;

use crossbeam::channel::Sender;
use csv::ReaderBuilder;
use simplelog::*;
use std::fs::File;
use std::thread;

pub struct MarketDataFeederLocal {
    publish_sender: Option<Sender<Event>>,
    csv_path: String,
    symbol: String,
}

impl ModulePublish for MarketDataFeederLocal {
    fn use_sender(&mut self, sender: Sender<Event>) {
        self.publish_sender = Some(sender.clone());
    }
}

impl MarketDataFeederLocal {
    pub fn new(symbol: String, csv_path: String) -> Self {
        MarketDataFeederLocal {
            publish_sender: None,
            csv_path,
            symbol,
        }
    }

    fn publish(&self, event: Event) {
        if let Some(publish_sender) = &self.publish_sender {
            publish_sender.send(event).unwrap();
        } else {
            panic!("publish_sender is not initialized!");
        }
    }

    pub fn start_feeding(&self) {
        let file = File::open(&self.csv_path).expect("Failed to open CSV file");
        let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);

        #[cfg(feature = "random_sleep_test")]
        let mut rng = rand::thread_rng();

        for result in reader.records() {
            let record = result.expect("Failed to read record");
            // Parse the timestamp (Date)
            let timestamp = record[0].to_string();

            // Parse the Close/Last price (removing the '$' sign)
            let close: f64 = record[1]
                .trim_start_matches('$')
                .parse()
                .expect("Invalid close value");

            // Parse the Volume
            let volume: i32 = record[2].parse().expect("Invalid volume value");

            // Parse the Open price (removing the '$' sign)
            let open: f64 = record[3]
                .trim_start_matches('$')
                .parse()
                .expect("Invalid open value");

            // Parse the High price (removing the '$' sign)
            let high: f64 = record[4]
                .trim_start_matches('$')
                .parse()
                .expect("Invalid high value");

            // Parse the Low price (removing the '$' sign)
            let low: f64 = record[5]
                .trim_start_matches('$')
                .parse()
                .expect("Invalid low value");

            // Create a MarketDataEvent
            let market_data_event = Event::new_market_data(
                timestamp,
                self.symbol.clone(),
                open,
                close,
                high,
                low,
                volume,
            );

            // Send data through the channel
            #[cfg(feature = "random_sleep_test")]
            {
                let sleep_duration = rng.gen_range(10..500);
                thread::sleep(Duration::from_millis(sleep_duration));
            }

            debug!("Market data event: {:?}", market_data_event);
            self.publish(market_data_event);

            thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}
