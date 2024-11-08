use crate::event_manager::{Event, EventHandler};
use crossbeam::channel::Sender;

pub struct MockExchange;

impl EventHandler for MockExchange {
    fn handle_event(&self, event: &Event, event_sender: &Sender<Event>) {
        println!("MockExchange handling event: {:?}, sending OrderComplete", event);
        event_sender.send(Event::OrderComplete).unwrap();
    }
}
