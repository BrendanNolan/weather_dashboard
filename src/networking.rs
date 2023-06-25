use client::{command::Command, tasks};
use connection_utils::ServerError;
use futures::stream::{FuturesUnordered, StreamExt};
use std::sync::mpsc::{Receiver as StdReceiver, Sender as StdSender};
use tokio::{
    net::TcpStream,
    sync::{
        mpsc::{self, Sender as TokioSender},
        oneshot,
    },
};

use crate::{app_state::County, weather_report::WeatherReport};

type WeatherCommand = Command<County, (County, WeatherReport)>;

pub async fn run_client(
    rx_county: StdReceiver<County>,
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
    rx_county: StdReceiver<County>,
    tx: TokioSender<WeatherCommand>,
    tx_results: StdSender<(County, WeatherReport)>,
) {
    let mut response_receivers = FuturesUnordered::new();
    loop {
        tokio::select! {
            Some(county) = try_recv(&rx_county) => {
                let (response_tx, rx) = oneshot::channel();
                tx.send(WeatherCommand {
                    data: county,
                    responder: response_tx,
                })
                .await
                .unwrap();
                response_receivers.push(rx);
            },
            Some(Ok(Some(Ok(response)))) = response_receivers.next() => {
                tx_results.send(response);
            },
        }
    }
}

async fn try_recv(rx: &StdReceiver<County>) -> Option<County> {
    rx.try_recv().ok()
}

enum ClientAction {
    Continue,
    Stop,
}

fn process_response_from_server(
    response: &Result<(County, WeatherReport), ServerError>,
) -> ClientAction {
    match response {
        Ok(_response) => ClientAction::Continue,
        Err(_error_message) => ClientAction::Stop,
    }
}
