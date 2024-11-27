use crate::event_manager::ModulePublish;
use crate::events::*;

use crossbeam::channel::{Sender, Receiver};
use csv::ReaderBuilder;
use std::fs::File;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use rand::Rng;

#[cfg(feature= "order_test")]
use crate::util::Counter;

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

        #[cfg(feature= "order_test")]
        let mut counter = Counter::new();
        #[cfg(feature= "random_sleep_test")]
        let mut rng = rand::thread_rng();

        // let mut first_data = true;

        for result in reader.records() {
            let record = result.expect("Failed to read record");
            let timestamp= record[0].to_string();
            let open= record[1].parse().expect("Invalid open value");
            let high= record[2].parse().expect("Invalid high value");
            let low= record[3].parse().expect("Invalid low value");
            let close= record[4].parse().expect("Invalid close value");
            let volume= record[5].parse().expect("Invalid volume value");
            let symbol = "SNP".to_string() ;
            let market_data_event = Event::new_market_data(timestamp, symbol, open, close, high, low, volume);

            // Send data through the channel
            // println!("MarketDataFeeder: Sending: {:?}", market_data);
            #[cfg(feature= "random_sleep_event")]
            {
                let sleep_duration = rng.gen_range(10..500);
                thread::sleep(Duration::from_millis(sleep_duration));
            }
            #[cfg(feature= "order_test")]
            {
                println!("MarketDataFeeder: Sending MarketDataEvent{}", counter.next());
                println!("MDF Timestamp: {:?}", std::time::SystemTime::now());
            }
            println!("MDF: Publishing : {:?}", market_data_event);
            self.publish(market_data_event);

            // if first_data{
            //     // A tiny little sprinkle of magic
            //     // To fix the case when others hp modules are not ready
            //     // But multiple lp events have been dispatched
            //     println!("MDF: Warming up...");
            //     thread::sleep(std::time::Duration::from_millis(1));
            //     first_data = false;
            // }
            thread::sleep(std::time::Duration::from_millis(1));
            // break;
        }
    }
}
