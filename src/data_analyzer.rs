use crate::event_manager::{ModulePublish, ModuleReceive};
use crate::{events::*, market_data_feeder};
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use num_traits::cast::ToPrimitive;
use plotters::prelude::*;
use std::error::Error;
#[cfg(feature= "order_test")]
use crate::util::Counter;
use std::collections::HashMap;
use std::thread;
use rand::Rng;
use std::time::Duration;


pub struct DataAnalyzer {
    // subscribe_sender is for event_manager to use only.
    // as DA is not supposed to block any other events,
    // it will use a unbounded channel.
    subscribe_sender: Sender<Event>,
    subscribe_receiver: Receiver<Event>,
    market_data: Vec<(String, f64)>,
    // local_portfolio: use this to parse PortfolioInfoEvents
    local_portfolio: Portfolio,
    asset_history: Vec<Order>,
}

impl ModuleReceive for DataAnalyzer {
    fn get_sender(&self) -> Sender<Event> {
        self.subscribe_sender.clone()
    }
}

impl DataAnalyzer {
    pub fn new() -> Self {
        let (subscribe_sender, subscribe_receiver) = unbounded();
        let market_data = Vec::new();
        let local_portfolio = Portfolio::new(0.0);
        let asset_history = Vec::new();
        DataAnalyzer {
            subscribe_sender,
            subscribe_receiver,
            market_data,
            local_portfolio,
            asset_history,
        }
    }

    pub fn run(&mut self) -> () {
        #[cfg(feature= "order_test")]
        let mut counter_a = Counter::new();
        #[cfg(feature= "order_test")]
        let mut counter_b = Counter::new();
        #[cfg(feature= "order_test")]
        let mut counter_c = Counter::new();
        #[cfg(feature= "random_sleep_test")]
        let mut rng = rand::thread_rng();

        // Control Loop
        loop {
            let event = self.subscribe_receiver.recv().unwrap();

            match event {
                Event::MarketData(market_data_event) => {
                    // self.process_marketevent(market_data_event);
                    self.process_marketevent(market_data_event).unwrap_or_else(|err| {
                        eprintln!("Error updating plot: {}", err);
                    });
                }
                Event::PortfolioInfo(portfolio_info_event) => {
                    self.process_portfolioinfo(portfolio_info_event);
                }
                _ => {
                    println!("DataAnalyzer: Unsupported event: {:?}", event);
                }
            }
        }
    }

    fn process_marketevent(&mut self, market_data_event: MarketDataEvent) -> Result<(), Box<dyn Error>> {
        // TODO: use marketdata event to plot baseline
        // Or maintain a vec to update asset (currently to be handled by mockexchange)
        println!("DA: Updating event: {:?}", market_data_event);
        self.market_data.push((market_data_event.timestamp.clone(), market_data_event.close));
        // if self.market_data.len()>10 {println!("market_data: {:?}", &self.market_data);}
        self.plot(&self.market_data, "baseline.png")?;
        Ok(())
    }

    fn plot(&self, data: &[(String, f64)], output_path: &str) -> Result<(), Box<dyn Error>> {
        if data.is_empty() {
            eprintln!("No data points available for plotting.");
            return Ok(());
        }
        // Calculate y-axis range dynamically
        let y_min = data.iter().map(|&(_, close)| close).fold(f64::INFINITY, f64::min);
        let y_max = data.iter().map(|&(_, close)| close).fold(f64::NEG_INFINITY, f64::max);
        // if data.len()>10 {
        //     println!("ymin: {}", y_min);
        //     println!("ymax: {}", y_max);
        // }

        // Set up the drawing area
        let root_area = BitMapBackend::new(output_path, (1280, 720)).into_drawing_area();
        root_area.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root_area)
            .caption("Baseline Performance", ("sans-serif", 18))
            .x_label_area_size(60)
            .y_label_area_size(60)
            .build_cartesian_2d(0..data.len(), y_min..y_max)?;

        // Configure the mesh
        chart
            .configure_mesh()
            // .x_labels(11)
            // .y_labels(11)
            .x_desc("Date")
            .y_desc("Price")
            .axis_desc_style(("sans-serif", 18))
            .x_label_formatter(&|x| {
                if let Some(index) = x.to_usize() {
                    // println!("index: {}", index);
                    // println!("datalen: {}", data.len());
                    if index < data.len() {
                        // Extract the date portion from the timestamp
                        // println!("index: {}", index);
                        // println!("x: {}", data[index].0.to_string());
                        return data[index].0.to_string();
                    }
                }
                "".to_string()
            })
            .draw()?;

        // Draw the data series
        chart
            .draw_series(LineSeries::new(
                data.iter().enumerate().map(|(i, &(_, close))| (i, close)),
                &BLUE,
            ))?
            .label("Close Price")
            .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], &BLUE));

        // Draw the legend
        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        // println!("Real-time plot updated: {}", output_path);
        // println!("Plot updated with {} data points.", data.len());
        // drop(root_area);
        Ok(())
    }

    fn process_portfolioinfo(&mut self, portfolio_info_event: PortfolioInfoEvent) {
        println!("DA: Updating event: {:?}", portfolio_info_event);
        self.local_portfolio = portfolio_info_event.portfolio.clone();            
        
        // TODO: update assets vector

    }
}