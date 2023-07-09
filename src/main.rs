mod app;

use crossterm::terminal;
use weather_dashboard::{
    county::County,
    weather_report::{WeatherReport, WeatherType},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    app::logging::setup_logger();

    terminal::enable_raw_mode()?;

    app::spawn_client()?.await?;

    Ok(())
}
