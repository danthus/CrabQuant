use crate::event_manager::{ModulePublish, ModuleReceive};
use crate::{shared_structures::*, market_data_feeder};
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use num_traits::cast::ToPrimitive;
use plotters::prelude::*;
use simplelog::*;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct DataAnalyzer {
    subscribe_sender: Sender<Event>,
    subscribe_receiver: Receiver<Event>,
    market_data_history: Arc<Mutex<Vec<(String, f64)>>>,
    asset_history: Arc<Mutex<Vec<(String, f64)>>>,
    cash_history: Arc<Mutex<Vec<(String, f64)>>>,
    local_portfolio: Portfolio,
}

impl ModuleReceive for DataAnalyzer {
    fn get_sender(&self) -> Sender<Event> {
        self.subscribe_sender.clone()
    }
}

impl DataAnalyzer {
    pub fn new() -> Self {
        let (subscribe_sender, subscribe_receiver) = unbounded();
        let market_data_history = Arc::new(Mutex::new(Vec::new()));
        let asset_history = Arc::new(Mutex::new(Vec::new()));
        let cash_history = Arc::new(Mutex::new(Vec::new()));
        let local_portfolio = Portfolio::new(0.0);

        // Spawn a new thread for plotting
        let market_data_history_clone = Arc::clone(&market_data_history);
        let asset_history_clone = Arc::clone(&asset_history);
        let cash_history_clone = Arc::clone(&cash_history);

        thread::spawn(move || {
            let mut last_lengths = (0, 0); // Track lengths of histories
            loop {
                // Take a snapshot of data
                let market_data_snapshot = {
                    let data = market_data_history_clone.lock().unwrap();
                    data.clone()
                };
                let asset_history_snapshot = {
                    let data = asset_history_clone.lock().unwrap();
                    data.clone()
                };
                let cash_history_snapshot = {
                    let data = cash_history_clone.lock().unwrap();
                    data.clone()
                };

                // Plot and save the data if there are updates
                if let Err(err) = DataAnalyzer::plot(
                    &market_data_snapshot,
                    &asset_history_snapshot,
                    &cash_history_snapshot,
                    &mut last_lengths,
                    "performance.png",
                ) {
                    eprintln!("Error during plotting: {}", err);
                }

                thread::sleep(Duration::from_secs(1)); // Reduce resource usage
            }
        });

        DataAnalyzer {
            subscribe_sender,
            subscribe_receiver,
            market_data_history,
            asset_history,
            cash_history,
            local_portfolio,
        }
    }

    pub fn run(&mut self) {
        loop {
            let event = self.subscribe_receiver.recv().unwrap();
            match event {
                Event::MarketData(market_data_event) => {
                    self.process_marketevent(market_data_event);
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

    fn process_marketevent(&mut self, market_data_event: MarketDataEvent) {
        let mut market_data_history = self.market_data_history.lock().unwrap();
        market_data_history.push((market_data_event.timestamp.clone(), market_data_event.close));
        // println!("DA: Updated market data history: {:?}", market_data_event);
        debug!("Updated market data history: {:?}", market_data_event);
    }

    fn process_portfolioinfo(&mut self, portfolio_info_event: PortfolioInfoEvent) {
        self.local_portfolio = portfolio_info_event.portfolio.clone();
        let mut asset_history = self.asset_history.lock().unwrap();
        let mut cash_history = self.cash_history.lock().unwrap();

        if let Some((latest_timestamp, _)) = self.market_data_history.lock().unwrap().last() {
            asset_history.push((latest_timestamp.clone(), self.local_portfolio.asset));
            cash_history.push((latest_timestamp.clone(), self.local_portfolio.cash));
        }
        // println!("DA: Updated asset history: {:?}", self.local_portfolio);
        debug!("Updated asset history: {:?}", self.local_portfolio);
    }

    fn plot(
        market_data: &[(String, f64)],
        asset_history: &[(String, f64)],
        cash_history: &[(String, f64)],
        last_lengths: &mut (usize, usize), // Track the lengths of the histories
        output_path: &str,
    ) -> Result<(), Box<dyn Error>> {
        // Check if there are updates
        if market_data.len() == last_lengths.0 && asset_history.len() == last_lengths.1 {
            // No updates, skip plotting
            println!("No updates in data, skipping plot.");
            return Ok(());
        }

        // Update last_lengths to the current lengths
        last_lengths.0 = market_data.len();
        last_lengths.1 = asset_history.len();

        if market_data.is_empty() && asset_history.is_empty() {
            eprintln!("No data points available for plotting.");
            return Ok(());
        }

        let first_market_value = market_data.get(0).map_or(1.0, |(_, value)| *value);
        let first_asset_value = asset_history.get(0).map_or(1.0, |(_, value)| *value);
        let first_cash_value = cash_history.get(0).map_or(1.0, |(_, value)| *value);

        let standardized_market_data: Vec<(String, f64)> = market_data
            .iter()
            .map(|(timestamp, value)| (timestamp.clone(), value / first_market_value))
            .collect();

        let standardized_asset_history: Vec<(String, f64)> = asset_history
            .iter()
            .map(|(timestamp, value)| (timestamp.clone(), value / first_asset_value))
            .collect();

        // Calculate (asset - cash) and standardize
        let standardized_difference: Vec<(String, f64)> = asset_history
            .iter()
            .map(|(timestamp, asset_value)| {
                // Find corresponding cash value by timestamp
                let cash_value = cash_history
                    .iter()
                    .find(|(cash_timestamp, _)| cash_timestamp == timestamp)
                    .map(|(_, value)| *value)
                    .unwrap_or(0.0); // Default to 0.0 if no matching timestamp

                let difference = asset_value - cash_value; // Calculate asset - cash
                (timestamp.clone(), difference / first_asset_value) // Standardize by first asset value
            })
            .collect();

        let all_y_values = standardized_market_data
            .iter()
            .map(|&(_, value)| value)
            .chain(standardized_asset_history.iter().map(|&(_, value)| value))
            .chain(standardized_difference.iter().map(|&(_, value)| value));
        let y_min = all_y_values.clone().fold(f64::INFINITY, f64::min);
        let y_max = all_y_values.fold(f64::NEG_INFINITY, f64::max);

        let root_area = BitMapBackend::new(output_path, (3840, 2160)).into_drawing_area();
        root_area.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root_area)
            .caption("Market Data and Asset History", ("sans-serif", 60))
            .x_label_area_size(120)
            .y_label_area_size(150)
            .build_cartesian_2d(0..market_data.len().max(asset_history.len()), y_min..y_max)?;

        // Configure the mesh
        chart
            .configure_mesh()
            .x_desc("Date")
            .y_desc("Value")
            .axis_desc_style(("sans-serif", 56))
            .label_style(("sans-serif", 50))
            .x_label_formatter(&|x| {
                if let Some(index) = x.to_usize() {
                    if index < standardized_market_data.len() {
                        return standardized_market_data[index].0.clone();
                    }
                }
                "".to_string()
            })
            .draw()?;

        // Plot the market data (close prices) in blue
        chart
            .draw_series(LineSeries::new(
                standardized_market_data
                    .iter()
                    .enumerate()
                    .map(|(i, &(_, close))| (i, close)),
                &BLUE,
            ))?
            .label("Market Data")
            .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], &BLUE));

        // Plot the asset history in red
        chart
            .draw_series(LineSeries::new(
                standardized_asset_history
                    .iter()
                    .enumerate()
                    .map(|(i, &(_, asset))| (i, asset)),
                &RED,
            ))?
            .label("Total Asset Value")
            .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], &RED));

        // Plot the asset-cash difference as a color block (area chart)
        chart
            .draw_series(AreaSeries::new(
                standardized_difference
                    .iter()
                    .enumerate()
                    .map(|(i, &(_, diff))| (i, diff)),
                0.0,             // Baseline for the area chart
                &GREEN.mix(0.2), // Semi-transparent green for the color block
            ))?
            .label("Position Value")
            .legend(|(x, y)| Rectangle::new([(x, y - 5), (x + 20, y + 5)], &GREEN.mix(0.5)));

        // chart
        //     .draw_series(LineSeries::new(
        //         standardized_difference
        //             .iter()
        //             .enumerate()
        //             .map(|(i, &(_, diff))| (i, diff)),
        //         &GREEN,
        //     ))?
        //     .label("Position Value")
        //     .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], &GREEN));

        // Draw the legend
        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::UpperMiddle)
            .background_style(&WHITE.mix(0.2))
            .border_style(&BLACK)
            .label_font(("sans-serif", 50))
            .draw()?;

        println!("Plot updated: {}", output_path);

        Ok(())
    }
}
