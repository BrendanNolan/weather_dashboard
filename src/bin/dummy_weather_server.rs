use rand::Rng;
use server::{request_processing::RequestProcessor, server_runner, shutdown::ShutdownListener};
use tokio::{signal, sync::watch};
use weather_dashboard::{county::County, weather_report::WeatherReport};

struct CountyWeatherProvider {}

impl CountyWeatherProvider {
    fn new() -> Self {
        CountyWeatherProvider {}
    }
}

impl RequestProcessor<County, WeatherReport> for CountyWeatherProvider {
    fn process(&self, _request: &County) -> WeatherReport {
        let mut random_number_generator = rand::thread_rng();
        WeatherReport {
            sunshine: random_number_generator.gen::<f32>(),
            wind_speed: random_number_generator.gen::<f32>(),
            rainfall: random_number_generator.gen::<f32>(),
        }
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