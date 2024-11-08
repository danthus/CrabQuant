use crate::event_manager::{Event, EventHandler};
use crossbeam::channel::Sender;

pub struct PortfolioManager;

impl EventHandler for PortfolioManager {
    fn handle_event(&self, event: &Event, _event_sender: &Sender<Event>) {
        println!("PortfolioManager handling event: {:?}", event);
    }
}
