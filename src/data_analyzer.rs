use crate::event_manager::ModuleReceive;
use crate::shared_structures::*;
use crossbeam::channel::{unbounded, Receiver, Sender};
use num_traits::cast::ToPrimitive;
use plotters::prelude::*;
use simplelog::*;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct DataAnalyzer {
    subscribe_sender: Sender<Event>,
    subscribe_receiver: Receiver<Event>,
    market_data_history: Arc<Mutex<Vec<(String, f64)>>>,
    asset_history: Arc<Mutex<Vec<(String, f64)>>>,
    cash_history: Arc<Mutex<Vec<(String, f64)>>>,
    local_portfolio: Portfolio,
}

// Metrics struct
struct Metrics {
    total_return: f64,
    annualized_return: f64,
    volatility: f64,
    sharpe_ratio: f64,
    max_drawdown: f64,
    alpha: f64,
    beta: f64,
    sortino_ratio: f64,
    information_ratio: f64,
    tracking_error: f64,
    longest_drawdown: usize,
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
        let data_analyzer_clone = self.clone();

        // Spawn a new thread for plotting
        // let market_data_history_clone = Arc::clone(&market_data_history);
        // let asset_history_clone = Arc::clone(&asset_history);
        // let cash_history_clone = Arc::clone(&cash_history);

        thread::spawn(move || {
            let mut last_lengths = (0, 0); // Track lengths of histories
                                           // thread::sleep(Duration::from_millis(100));
            loop {
                // Take a snapshot of data
                let market_data_snapshot = {
                    let data = data_analyzer_clone.market_data_history.lock().unwrap();
                    data.clone()
                };
                let asset_history_snapshot = {
                    let data = data_analyzer_clone.asset_history.lock().unwrap();
                    data.clone()
                };
                let cash_history_snapshot = {
                    let data = data_analyzer_clone.cash_history.lock().unwrap();
                    data.clone()
                };

                // Plot and save the data if there are updates
                if let Err(err) = data_analyzer_clone.plot(
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

    fn calculate_metrics(&self) -> Result<Metrics, Box<dyn Error>> {
        let market_data = self.market_data_history.lock().unwrap();
        let asset_history = self.asset_history.lock().unwrap();
        // println!(
        //     "Market data size: {}, Asset history size: {}",
        //     market_data.len(),
        //     asset_history.len()
        // );

        if market_data.is_empty() || asset_history.is_empty() {
            return Err("Insufficient data for metrics calculation".into());
        }

        // Extract returns
        let returns: Vec<f64> = asset_history
            .windows(2)
            .map(|window| (window[1].1 - window[0].1) / window[0].1)
            .collect();

        let benchmark_returns: Vec<f64> = market_data
            .windows(2)
            .map(|window| (window[1].1 - window[0].1) / window[0].1)
            .collect();

        // Total Return
        let total_return = (asset_history.last().unwrap().1 / asset_history[0].1) - 1.0;

        // Annualized Return
        let n = returns.len() as f64;
        let annualized_return = (1.0 + total_return).powf(252.0 / n) - 1.0;

        // Volatility
        let volatility = returns.iter().copied().fold(0.0, |acc, x| acc + x.powi(2)) / (n - 1.0);

        // Sharpe Ratio
        let mean_return = returns.iter().sum::<f64>() / n;
        let sharpe_ratio = mean_return / volatility;

        // Max Drawdown
        if asset_history
            .iter()
            .any(|&(_, value)| value.is_nan() || value.is_infinite() || value <= 0.0)
        {
            return Err("Asset history contains invalid or zero values".into());
        }
        let mut peak = asset_history
            .first()
            .map(|&(_, value)| value)
            .unwrap_or(0.0);
        let mut max_drawdown: f64 = 0.0;
        for &(_, value) in asset_history.iter() {
            if value > peak {
                peak = value;
            }
            if peak > 0.0 {
                let drawdown = (value / peak) - 1.0;
                max_drawdown = max_drawdown.min(drawdown);
            }
        }

        // Sortino Ratio
        let downside_deviation: f64 = returns
            .iter()
            .filter(|&&r| r < 0.0)
            .map(|r| r.powi(2))
            .sum::<f64>()
            / n;
        let sortino_ratio = mean_return / downside_deviation.sqrt();

        // Alpha and Beta
        let covariance: f64 = returns
            .iter()
            .zip(&benchmark_returns)
            .map(|(&r_p, &r_b)| r_p * r_b)
            .sum::<f64>();
        let variance: f64 = benchmark_returns.iter().map(|&r| r.powi(2)).sum::<f64>();
        let beta = covariance / variance;
        let alpha = mean_return - beta * (benchmark_returns.iter().sum::<f64>() / n);

        // Information Ratio
        let excess_returns: Vec<f64> = returns
            .iter()
            .zip(&benchmark_returns)
            .map(|(&r_p, &r_b)| r_p - r_b)
            .collect();
        let tracking_error = excess_returns
            .iter()
            .map(|&x| x.powi(2))
            .sum::<f64>()
            .sqrt();
        let information_ratio = mean_return / tracking_error;

        // Longest Drawdown Period
        let mut peak: f64 = asset_history
            .first()
            .map(|&(_, value)| value)
            .unwrap_or(0.0);
        let mut drawdown_start = None;
        let mut longest_drawdown = 0;
        for (i, &(_, value)) in asset_history.iter().enumerate() {
            // If drawdown ended or at the last value, calculate the drawdown length
            if drawdown_start.is_some() && (value >= peak || i == asset_history.len() - 1) {
                println!("i: {}, drawdown_start: {}", i, drawdown_start.unwrap());
                println!("peak: {}", peak);
                let length = i - drawdown_start.unwrap();
                longest_drawdown = longest_drawdown.max(length);
                println!("longest_drawdown: {}, length: {}", longest_drawdown, length);
                drawdown_start = None; // Reset drawdown_start after calculating length
            }
            if value > peak {
                // Update peak and reset drawdown tracking
                peak = value;
                drawdown_start = None;
            } else if value < peak {
                // Enter drawdown
                if drawdown_start.is_none() {
                    drawdown_start = Some(i);
                }
            }
        }

        Ok(Metrics {
            total_return,
            annualized_return,
            volatility,
            sharpe_ratio,
            max_drawdown,
            alpha,
            beta,
            sortino_ratio,
            information_ratio,
            tracking_error,
            longest_drawdown,
        })
    }

    fn plot(
        &self,
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
        // println!(
        //     "Plotting triggered: market_data.len()={}, asset_history.len()={}",
        //     market_data.len(),
        //     asset_history.len()
        // );

        // Calculate metrics
        // let metrics = self.calculate_metrics()?;
        // println!("metrics calculated: {}", metrics.total_return);
        let metrics = match self.calculate_metrics() {
            Ok(metrics) => metrics,
            Err(err) => {
                eprintln!("Failed to calculate metrics: {}", err);
                return Err(err); // Or handle the error as appropriate
            }
        };
        // println!("metrics calculated: {}", metrics.total_return);

        let first_market_value = market_data.get(0).map_or(1.0, |(_, value)| *value);
        let first_asset_value = asset_history.get(0).map_or(1.0, |(_, value)| *value);
        // let first_cash_value = cash_history.get(0).map_or(1.0, |(_, value)| *value);

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

        let res_x = 3840;
        let res_y = 2160;
        let root_area = BitMapBackend::new(output_path, (res_x, res_y)).into_drawing_area();
        root_area.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root_area)
            .caption("Market Data and Asset History", ("sans-serif", res_x / 52))
            .x_label_area_size(res_y / 16)
            .y_label_area_size(res_x / 22)
            .build_cartesian_2d(
                0..market_data.len().max(asset_history.len()) + market_data.len() / 25,
                y_min..y_max + 1.0,
            )?;

        // Configure the mesh
        chart
            .configure_mesh()
            .x_desc("Date")
            .y_desc("Value")
            .axis_desc_style(("sans-serif", res_x / 58))
            .label_style(("sans-serif", res_x / 71)) // x & y label
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
            .label(" Market Data")
            .legend(|(x, y)| PathElement::new([(x, y), (x + 30, y)], &BLUE));

        // Plot the asset history in red
        chart
            .draw_series(LineSeries::new(
                standardized_asset_history
                    .iter()
                    .enumerate()
                    .map(|(i, &(_, asset))| (i, asset)),
                &RED,
            ))?
            .label(" Total Asset Value")
            .legend(|(x, y)| PathElement::new([(x, y), (x + 30, y)], &RED));

        // Plot the asset-cash difference as a color block (area chart)
        chart
            .draw_series(AreaSeries::new(
                standardized_difference
                    .iter()
                    .enumerate()
                    .map(|(i, &(_, diff))| (i, diff)),
                0.0,             // Baseline for the area chart
                &GREEN.mix(0.4), // Semi-transparent green for the color block
            ))?
            .label(" Position Value")
            .legend(|(x, y)| Rectangle::new([(x, y - 6), (x + 30, y + 6)], &GREEN));

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

        // chart.draw_series(std::iter::once(Text::new(
        //     format!(
        //         "Total Return: {:.2}% ,\n,Annualized Return: {:.2}%
        //         Volatility: {:.4}
        //         Sharpe Ratio: {:.2}
        //         Max Drawdown: {:.2}%
        //         Alpha: {:.4}
        //         Beta: {:.4}
        //         Sortino Ratio: {:.4}
        //         Information Ratio: {:.4}
        //         Tracking Error: {:.4}
        //         Longest Drawdown Period: {} days",
        //         metrics.total_return * 100.0,
        //         metrics.annualized_return * 100.0,
        //         metrics.volatility,
        //         metrics.sharpe_ratio,
        //         metrics.max_drawdown * 100.0,
        //         metrics.alpha,
        //         metrics.beta,
        //         metrics.sortino_ratio,
        //         metrics.information_ratio,
        //         metrics.tracking_error,
        //         metrics.longest_drawdown,
        //     ),
        //     (10, 0.8), // Corrected: x-coordinate is `usize`, y-coordinate is `f64`
        //     ("sans-serif", 50).into_font(),
        // )))?;
        let metrics_text = vec![
            format!("Total Return: {:.2}%", metrics.total_return * 100.0),
            format!(
                "Annualized Return: {:.2}%",
                metrics.annualized_return * 100.0
            ),
            format!("Volatility: {:.4}", metrics.volatility),
            format!("Sharpe Ratio: {:.2}", metrics.sharpe_ratio),
            format!("Max Drawdown: {:.2}%", metrics.max_drawdown * 100.0),
            format!("Alpha: {:.4}", metrics.alpha),
            format!("Beta: {:.4}", metrics.beta),
            format!("Sortino Ratio: {:.4}", metrics.sortino_ratio),
            format!("Information Ratio: {:.4}", metrics.information_ratio),
            format!("Tracking Error: {:.4}", metrics.tracking_error),
            format!("Longest Drawdown Period: {} days", metrics.longest_drawdown),
        ];
        let start_x = standardized_market_data.len() / 50; // X-coordinate
        let mut start_y = y_max + 1.0 - (y_max - y_min) / 30.0; // Initial Y-coordinate
        for line in metrics_text {
            chart.draw_series(std::iter::once(Text::new(
                line,
                (start_x, start_y),
                ("sans-serif", res_x / 77).into_font(),
            )))?;
            start_y -= (y_max + 1.0 - y_min) / 42.0; // Increment Y-coordinate for the next line
        }

        // Draw the legend
        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::Coordinate(
                (res_x / 2 - res_x / 14).to_i32().unwrap(),
                50,
            ))
            .background_style(&WHITE.mix(0.2))
            .border_style(&BLACK)
            .label_font(("sans-serif", res_x / 77)) // legend label
            .draw()?;

        println!("Plot updated: {}", output_path);

        Ok(())
    }
}
