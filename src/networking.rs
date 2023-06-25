use connection_utils::ServerError;
use tokio::{
    net::TcpStream,
    sync::mpsc::{Receiver as TokioReceiver, Sender as TokioSender},
};

use weather_dashboard::{county::County, weather_report::WeatherReport};

pub async fn run_client(
    rx_county: TokioReceiver<County>,
    tx_results: TokioSender<Option<Result<(County, WeatherReport), ServerError>>>,
) {
    let stream = TcpStream::connect("127.0.0.1:6379").await.unwrap();
    let manager = tokio::spawn(client::tasks::create_connection_manager(
        stream, rx_county, tx_results,
    ));
    manager.await.unwrap();
}
