use crate::event_manager::{Event, EventHandler};
use crate::events::{EventType, OrderComplete};
use crossbeam::channel::Sender;

pub struct MockExchange;

impl EventHandler for MockExchange {
    fn handle_event(&self, event: &Event, event_sender: &Sender<Event>) {
        let order_complete_event = OrderComplete {
            order_id:0,
            filled_quantity:0.0,
        };

        let event = Event::new(EventType::OrderComplete, order_complete_event);

        event_sender.send(event).unwrap();
    }
}

impl MockExchange {
    pub fn new() -> Self{
        MockExchange{}
    }
}