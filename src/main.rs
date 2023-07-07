mod app;

use crossterm::terminal;
use std::{sync::mpsc, thread, time::Duration};
use weather_dashboard::{
    county::County,
    weather_report::{WeatherReport, WeatherType},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    app::logging::setup_logger();

    terminal::enable_raw_mode()?;

    let (tx_results, rx_results) = tokio::sync::mpsc::channel(100);
    let (tx_county, rx_county) = tokio::sync::mpsc::channel(100);
    let client = tokio::spawn(app::networking::run_client(rx_county, tx_results));

    let (tx_user_input, rx_user_input) = mpsc::channel();

    thread::spawn(|| app::run_user_event_loop(Duration::from_millis(200), tx_user_input));
    app::run_app_loop(rx_user_input, tx_county, rx_results)?;

    client.await.unwrap();
    Ok(())
}
