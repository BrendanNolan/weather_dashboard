mod app;

use crossterm::terminal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    app::logging::setup_logger();

    terminal::enable_raw_mode()?;

    app::run_client()?.await?;

    Ok(())
}
