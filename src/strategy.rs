use crate::event_manager::{Event, ModulePublish, ModuleReceive};
use crate::events::{EventType, MarketDataEvent, OrderPlaceEvent, OrderCompleteEvent, EventContent};
use crossbeam::channel::{Sender, Receiver, bounded};
#[cfg(feature= "custom_test")]
use crate::util::Counter;
pub struct Strategy {
    subscribe_sender: Sender<Event>,
    subscribe_receiver: Receiver<Event>,
    publish_sender: Option<Sender<Event>>,
}

impl Strategy {
    /// Creates a new Strategy module
    pub fn new() -> Self {
        let (subscribe_sender, subscribe_receiver) = bounded(0);
        Strategy {
            subscribe_sender,
            subscribe_receiver,
            publish_sender: None,
        }
    }

    /// Runs the strategy logic, processing MarketDataEvent and sending OrderPlaceEvent
    pub fn run(&self) {
        if self.publish_sender.is_none() {
            panic!("Publish sender is not initialized!");
        }

        #[cfg(feature= "custom_test")]
        let mut counter_a = Counter::new();
        #[cfg(feature= "custom_test")]
        let mut counter_b = Counter::new();

        loop {
            // Receive an event from the subscribe_receiver
            let event = self.subscribe_receiver.recv().unwrap();
            // println!("Strategy: received event: {:?}", event);

            if let EventType::TypeMarketData = event.event_type {
                if let EventContent::MarketData(market_data) = event.contents {
                    // println!("Strategy: Received MarketDataEvent: {:?}", market_data);
                    #[cfg(feature= "custom_test")]
                    {
                        println!("Strategy: Received MarketDataEvent{}", counter_a.next());
                    }
                    // Sample OrderPlaceEvent based on MarketDataEvent
                    let order_event = OrderPlaceEvent {
                        order_id: 1, 
                        quantity: 100, 
                        price: market_data.close,
                    };

                    let order_event = Event::new(EventType::TypeOrderPlace, EventContent::OrderPlace(order_event));
                    // println!("Strategy: Sending OrderPlaceEvent: {:?}", order_event);
                    #[cfg(feature= "custom_test")]
                    {
                        println!("Strategy: Sending OrderPlaceEvent{}", counter_b.next());
                    }
                    // Publish the OrderPlaceEvent
                    self.publish(order_event);
                } else {
                    eprintln!("Failed to pattern match event to MarketDataEvent.");
                }
            } else {
                println!("Unsupported event type: {:?}", event.event_type);
            }
        }
    }


    /// Helper method to publish an event
    fn publish(&self, event: Event) {
        if let Some(publish_sender) = &self.publish_sender {
            publish_sender.send(event).unwrap();
        } else {
            panic!("Publish sender is not initialized!");
        }
    }
}

impl ModuleReceive for Strategy {
    fn get_sender(&self) -> Sender<Event> {
        self.subscribe_sender.clone()
    }
}

impl ModulePublish for Strategy {
    fn use_sender(&mut self, sender: Sender<Event>) {
        self.publish_sender = Some(sender);
    }
}