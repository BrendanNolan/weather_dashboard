mod app;

use crossterm::terminal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    app::logging::setup_logger();

    terminal::enable_raw_mode()?;

    let socket_address = app::get_socket_address_from_user();

    app::run_client(socket_address)?.await?;

    Ok(())
}
