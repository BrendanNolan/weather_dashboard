use client::{command::Command, tasks};
use connection_utils::ServerError;
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};

use crate::{app_state::County, weather_report::WeatherReport};

type WeatherCommand = Command<County, WeatherReport>;

async fn run_client() {
    let (tx, rx) = mpsc::channel::<WeatherCommand>(32);
    let stream = TcpStream::connect("127.0.0.1:6379").await.unwrap();
    let manager = tokio::spawn(tasks::create_cyclic_connection_manager(stream, rx));
    let client = tokio::spawn(create_client_task(tx));
    client.await.unwrap();
    manager.await.unwrap();
}

async fn create_client_task(tx: mpsc::Sender<WeatherCommand>) {
    for i in 0..10 {
        let (response_tx, response_rx) = oneshot::channel();
        tx.send(WeatherCommand {
            data: County(format!("County_{i}")),
            responder: response_tx,
        })
        .await
        .unwrap();
        let response = response_rx.await;
        match response {
            Ok(Some(response)) => match process_response_from_server(&response) {
                ClientAction::Continue => {}
                ClientAction::Stop => return,
            },
            Ok(None) => println!("Failed to read response."),
            Err(_) => panic!("Client unexpectedly failed to receive a response"),
        }
    }
}

enum ClientAction {
    Continue,
    Stop,
}

fn process_response_from_server(
    response: &Result<String, ServerError>,
) -> ClientAction {
    match response {
        Ok(response) => {
            println!(
                "Received a response: {:?}.",
                response,
            );
            ClientAction::Continue
        }
        Err(error_message) => {
            println!(
                "Received a server error: {:?}.",
                error_message,
            );
            ClientAction::Stop
        }
    }
}
