use crate::event_manager::{Event, EventHandler};
use crossbeam::channel::Sender;

pub struct Strategy;

impl EventHandler for Strategy {
    fn handle_event(&self, event: &Event, event_sender: &Sender<Event>) {
        println!("Strategy handling event: {:?}, sending OrderPlace", event);
        event_sender.send(Event::OrderPlace).unwrap();
    }
}
