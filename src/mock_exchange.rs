use crate::event_manager::{Event, ModulePublish, ModuleReceive};
use crate::events::{EventType, OrderCompleteEvent, OrderPlaceEvent, MarketDataEvent, EventContent};
use crossbeam::channel::{bounded, Receiver, Sender};
#[cfg(feature= "custom_test")]
use crate::util::Counter;
use std::thread;
pub struct MockExchange {
    // subscribe_sender is for event_manager to use only.
    // s_sender and s_receiver belongs to a rendezvous channel
    // use s_sender inside module will introduce deadlock
    subscribe_sender: Sender<Event>,
    subscribe_receiver: Receiver<Event>,
    // Use publish_sender to send events to event manager
    publish_sender: Option<Sender<Event>>,
}

impl ModuleReceive for MockExchange {
    fn get_sender(&self) -> Sender<Event> {
        self.subscribe_sender.clone()
    }
}

impl ModulePublish for MockExchange {
    fn use_sender(&mut self, sender: Sender<Event>) {
        // MockModule receives the Sender from EventManager to publish events
        // You can store this sender if needed or use it directly
        self.publish_sender = Some(sender.clone());
    }
}

impl MockExchange {
    pub fn new() -> Self {
        let (subscribe_sender, subscribe_receiver) = bounded(0);
        MockExchange {
            subscribe_sender,
            subscribe_receiver,
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
    pub fn run(&self) -> () {
        if self.publish_sender.is_none() {
            panic!("publish_sender is not initialized!");
        }
        #[cfg(feature= "custom_test")]
        let mut counter_a = Counter::new();
        #[cfg(feature= "custom_test")]
        let mut counter_b = Counter::new();
        #[cfg(feature= "custom_test")]
        let mut counter_c = Counter::new();
        loop {
            let event = self.subscribe_receiver.recv().unwrap();
            // println!("MockExchange: Received event: {:?}", event);
    
            match event.contents {
                EventContent::MarketData(market_data) => {
                    // println!("MockExchange: Received MarketDataEvent: {:?}", market_data);
                    #[cfg(feature= "custom_test")]
                    println!("MockExchange: Received MarketDataEvent{}", counter_a.next());
                    // MockExchange doesn't generate new events for MarketDataEvent
                }
                EventContent::OrderPlace(order_place_event) => {
                    // println!("MockExchange: Received OrderPlaceEvent: {:?}", order_place_event);
                    #[cfg(feature= "custom_test")]
                    {
                        println!("MockExchange: Received OrderPlaceEvent{}", counter_b.next());
                    }
                    // Generate an OrderCompleteEvent in response
                    let order_complete_event = OrderCompleteEvent {
                        order_id: order_place_event.order_id, // Use the same order ID
                        filled_quantity: order_place_event.quantity, // Example filled quantity
                    };
    
                    let complete_event = Event::new(EventType::TypeOrderComplete, EventContent::OrderComplete(order_complete_event));
                    // println!("MockExchange: sending OrderCompleteEvent: {:?}", complete_event);
                    #[cfg(feature= "custom_test")]
                    {
                        thread::sleep(std::time::Duration::from_millis(10));
                        println!("MockExchange: Sending OrderCompleteEvent{}", counter_c.next());
                    }
                    // Publish the OrderCompleteEvent
                    self.publish(complete_event);
                }
                _ => {
                    println!("MockExchange: Unsupported event type: {:?}", event.event_type);
                }
            }
        }
    }
}
