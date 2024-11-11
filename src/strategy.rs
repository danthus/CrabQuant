use crate::event_manager::{Event, EventHandler};
use crate::events::{EventType, OrderPlace, MarketData};
use crossbeam::channel::Sender;

pub struct Strategy;

impl EventHandler for Strategy {
    fn handle_event(&self, event: &Event, event_sender: &Sender<Event>) {
        if let Some(market_data) = event.contents.downcast_ref::<MarketData>() {
            println!("Strategy Module receiving and handling market data event: {:?}.", market_data);
        }

        let market_data = OrderPlace {
            order_id:0,
            quantity:0.0,
            price: 0.0,
        };

        let event = Event::new(EventType::OrderPlace, market_data);

        event_sender.send(event).unwrap();
    }
}
