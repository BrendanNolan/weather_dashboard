use connection_utils::ServerError;
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use std::{
    collections::HashMap,
    sync::mpsc::{Receiver as StdReceiver, Sender as StdSender},
    thread,
    time::{Duration, Instant},
};
use tokio::sync::mpsc::{Receiver as TokioReceiver, Sender as TokioSender};
use weather_dashboard::{
    county::County,
    weather_report::{WeatherReport, WeatherType},
};

mod drawing;
pub mod logging;
mod networking;
mod terminal_utils;

pub fn spawn_client() -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error>> {
    let (tx_results, rx_results) = tokio::sync::mpsc::channel(100);
    let (tx_county, rx_county) = tokio::sync::mpsc::channel(100);
    let client = tokio::spawn(networking::run_client(
        "127.0.0.1:6379",
        rx_county,
        tx_results,
    ));

    let (tx_user_input, rx_user_input) = std::sync::mpsc::channel();

    thread::spawn(|| run_user_event_loop(Duration::from_millis(200), tx_user_input));
    run_app_loop(rx_user_input, tx_county, rx_results)?;

    Ok(client)
}

pub struct AppState {
    pub active_weather_type: WeatherType,
    pub counties_list: ListState,
    pub counties: Vec<County>,
    pub weather_requested: bool,
}

impl AppState {
    pub fn select_next_index(&mut self) {
        let next_index = self.counties_list.selected().map_or(0, |i| i + 1);
        if next_index < self.counties.len() {
            self.counties_list.select(Some(next_index));
        }
    }

    pub fn select_previous_index(&mut self) {
        if self.counties.is_empty() {
            return;
        }
        let previous_index = self
            .counties_list
            .selected()
            .map_or(0, |i| i.saturating_sub(1));
        self.counties_list.select(Some(previous_index));
    }

    pub fn get_selected_county(&self) -> Option<County> {
        let county_index = self.counties_list.selected()?;
        Some(self.counties[county_index].clone())
    }
}

impl Default for AppState {
    fn default() -> Self {
        let mut weather_list = ListState::default();
        weather_list.select(Some(0));
        Self {
            active_weather_type: WeatherType::Rain,
            counties_list: weather_list,
            counties: vec![County("Wexford".to_string()), County("Cork".to_string())],
            weather_requested: false,
        }
    }
}

pub enum TickedUserInput {
    Input(KeyEvent),
    Tick,
}

pub fn run_app_loop(
    rx_user_input: StdReceiver<TickedUserInput>,
    tx_county: TokioSender<County>,
    mut rx_server_results: TokioReceiver<Result<(County, WeatherReport), ServerError>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut app_state = AppState::default();
    let mut terminal = terminal_utils::create_terminal()?;
    let mut response_to_input = ResponseToUserInput::Continue;
    let mut county_weather = HashMap::<County, WeatherReport>::new();

    while response_to_input == ResponseToUserInput::Continue {
        terminal.draw(|rect| {
            drawing::draw(
                rect,
                app_state.active_weather_type,
                &mut app_state.counties_list,
                &app_state.counties,
                &county_weather,
            );
        })?;

        if app_state.weather_requested {
            send_weather_request(&app_state, &tx_county);
            tracing::info!("Sending weather request.");
        }

        response_to_input = handle_user_input(&rx_user_input.recv()?, &mut app_state)?;
        receive_weather(&mut rx_server_results, &mut county_weather);
    }

    terminal_utils::prepare_terminal_for_app_exit(&mut terminal)?;
    Ok(())
}

pub fn handle_user_input(
    user_input: &TickedUserInput,
    app_state: &mut AppState,
) -> Result<ResponseToUserInput, Box<dyn std::error::Error>> {
    let TickedUserInput::Input(event) = user_input else {
        app_state.weather_requested = false;
        return Ok(ResponseToUserInput::Continue);
    };
    if event.code == KeyCode::Char('q') {
        return Ok(ResponseToUserInput::Stop);
    }
    match event.code {
        KeyCode::Char('w') => app_state.active_weather_type = WeatherType::Wind,
        KeyCode::Char('r') => app_state.active_weather_type = WeatherType::Rain,
        KeyCode::Char('s') => app_state.active_weather_type = WeatherType::Sun,
        KeyCode::Char('k') => app_state.select_previous_index(),
        KeyCode::Char('j') => app_state.select_next_index(),
        KeyCode::Char('g') => app_state.weather_requested = true,
        _ => {}
    }
    if event.code != KeyCode::Char('g') {
        app_state.weather_requested = false;
    }
    Ok(ResponseToUserInput::Continue)
}

#[derive(PartialEq)]
pub enum ResponseToUserInput {
    Continue,
    Stop,
}

pub fn run_user_event_loop(tick_rate: Duration, tx: StdSender<TickedUserInput>) {
    let mut last_tick = Instant::now();
    loop {
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout).expect("poll works") {
            if let CEvent::Key(key) = event::read().expect("can read events") {
                tx.send(TickedUserInput::Input(key))
                    .expect("can send events");
            }
        }

        if last_tick.elapsed() >= tick_rate && tx.send(TickedUserInput::Tick).is_ok() {
            last_tick = Instant::now();
        }
    }
}

fn receive_weather(
    rx_server_results: &mut TokioReceiver<Result<(County, WeatherReport), ServerError>>,
    county_weather: &mut HashMap<County, WeatherReport>,
) {
    let Ok(weather_report) = rx_server_results.try_recv() else { return; };
    match weather_report {
        Ok((county, report)) => {
            county_weather.insert(county, report);
        }
        Err(error) => {
            tracing::info!("Error: {error}");
        }
    }
}

fn send_weather_request(app_state: &AppState, tx_county: &TokioSender<County>) {
    let Some(county) = app_state.get_selected_county() else {
         return;
     };
    let tx_county_clone = tx_county.clone();
    tokio::spawn(async move {
        match tx_county_clone.send(county).await {
            Ok(()) => tracing::info!("Successfully Sent County"),
            Err(error) => tracing::info!("Failed To Send Conunty: {error}"),
        }
    });
}
