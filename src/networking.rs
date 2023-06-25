use client::{command::Command, tasks};
use futures::stream::{FuturesUnordered, StreamExt};
use std::sync::mpsc::Sender as StdSender;
use tokio::{
    net::TcpStream,
    sync::{
        mpsc::{self, Receiver as TokioReceiver, Sender as TokioSender},
        oneshot,
    },
};
use tracing::info;

use weather_dashboard::{county::County, weather_report::WeatherReport};

type WeatherCommand = Command<County, (County, WeatherReport)>;

pub async fn run_client(
    rx_county: TokioReceiver<County>,
    tx_results: StdSender<(County, WeatherReport)>,
) {
    let (tx, rx) = mpsc::channel::<WeatherCommand>(32);
    let stream = TcpStream::connect("127.0.0.1:6379").await.unwrap();
    let manager = tokio::spawn(tasks::create_cyclic_connection_manager(stream, rx));
    let client = tokio::spawn(create_client_task(rx_county, tx, tx_results));
    client.await.unwrap();
    manager.await.unwrap();
}

async fn create_client_task(
    mut rx_county: TokioReceiver<County>,
    tx: TokioSender<WeatherCommand>,
    tx_results: StdSender<(County, WeatherReport)>,
) {
    let mut response_receivers = FuturesUnordered::new();
    loop {
        tokio::select! {
            Some(county) = rx_county.recv() => {
                let (response_tx, rx) = oneshot::channel();
                tx.send(WeatherCommand {
                    data: county,
                    responder: response_tx,
                })
                .await
                .unwrap();
                response_receivers.push(rx);
            },
            response = response_receivers.next() => {
                match response {
                    Some(Ok(Some(Ok(response)))) => tx_results.send(response).unwrap(),
                    None => info!("Got None response"),
                    _ => info!("Got bad response: {:?}", response),
                }
            },
        }
    }
}
