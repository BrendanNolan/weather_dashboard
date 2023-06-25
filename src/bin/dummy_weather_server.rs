use crate::{county::County, weather_report::WeatherReport};
use server::{request_processing::RequestProcessor, server_runner, shutdown::ShutdownListener};
use tokio::{signal, sync::watch};

fn put_int_in_string(i: &u32) -> String {
    std::thread::sleep(std::time::Duration::from_secs(1));
    format!("The number is: {}", i)
}

struct CountyWeatherProvider {}

impl CountyWeatherProvider {
    fn new() -> Self {
        CountyWeatherProvider {}
    }
}

impl RequestProcessor<County, String> for CountyWeatherProvider {
    fn process(&self, request: &u32) -> String {
        put_int_in_string(request)
    }
}

#[tokio::main]
async fn main() {
    let (tx_shutdown, rx_shutdown) = watch::channel(());
    let server_task = tokio::spawn(server_runner::run_server(
        "127.0.0.1:6379",
        CountyWeatherProvider::new(),
        ShutdownListener::new(rx_shutdown),
        10,
        10,
    ));
    handle_shutdown(tx_shutdown).await;
    let _ = server_task.await;
}

async fn handle_shutdown(tx_shutdown: watch::Sender<()>) {
    let _ = signal::ctrl_c().await;
    println!("Server shutting down ...");
    let _ = tx_shutdown.send(());
}

