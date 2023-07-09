use tokio::net::ToSocketAddrs;

use connection_utils::ServerError;
use tokio::{
    net::TcpStream,
    sync::mpsc::{Receiver as TokioReceiver, Sender as TokioSender},
};

use weather_dashboard::{county::County, weather_report::WeatherReport};

pub async fn run_client<Addr: ToSocketAddrs>(
    address: Addr,
    rx_county: TokioReceiver<County>,
    tx_results: TokioSender<Result<(County, WeatherReport), ServerError>>,
) {
    let stream = TcpStream::connect(address).await.unwrap();
    let manager = tokio::spawn(client::tasks::create_connection_manager(
        stream, rx_county, tx_results,
    ));
    manager.await.unwrap();
}
