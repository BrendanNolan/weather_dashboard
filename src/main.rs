mod app_state;
mod networking;
mod widgets;

use app_state::AppState;
use connection_utils::ServerError;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent},
    terminal,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, ListState, Tabs},
    Terminal,
};
use std::{
    collections::HashMap,
    io::{self, Stdout},
    sync::mpsc::{self, Receiver as StdReceiver, Sender as StdSender},
    thread,
    time::{Duration, Instant},
};
use tokio::sync::mpsc::{Receiver as TokioReceiver, Sender as TokioSender};
use tracing::info;
use weather_dashboard::{
    county::County,
    weather_report::{WeatherReport, WeatherType},
};
use widgets::{create_county_list_widget, create_county_table_widget};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    terminal::enable_raw_mode()?;

    let (tx_results, rx_results) = tokio::sync::mpsc::channel(100);
    let (tx_county, rx_county) = tokio::sync::mpsc::channel(100);
    let client = tokio::spawn(networking::run_client(rx_county, tx_results));

    let (tx_user_input, rx_user_input) = mpsc::channel();

    thread::spawn(move || run_user_event_loop(Duration::from_millis(200), tx_user_input));
    run_app_loop(rx_user_input, tx_county, rx_results)?;

    client.await.unwrap();
    Ok(())
}

fn setup_logger() {
    let file_appender = tracing_appender::rolling::hourly(".", "log.txt");
    let subscriber = tracing_subscriber::fmt::fmt()
        .with_writer(file_appender)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

fn run_app_loop(
    rx_user_input: StdReceiver<TickedUserInput>,
    tx_county: TokioSender<County>,
    mut rx_server_results: TokioReceiver<Option<Result<(County, WeatherReport), ServerError>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut app_state = AppState::default();
    let mut terminal = create_terminal()?;
    let mut response_to_input = ResponseToUserInput::Continue;
    let mut county_weather = HashMap::<County, WeatherReport>::new();

    while response_to_input == ResponseToUserInput::Continue {
        terminal.draw(|rect| {
            draw(
                rect,
                app_state.active_weather_type,
                &mut app_state.counties_list,
                &app_state.counties,
                &county_weather,
            );
        })?;

        if app_state.weather_requested {
            send_weather_request(&app_state, &tx_county);
            info!("Sending weather request.");
        }

        response_to_input = handle_user_input(&rx_user_input.recv()?, &mut app_state)?;
        if let Ok(Some(Ok((county, report)))) = rx_server_results.try_recv() {
            county_weather.insert(county, report);
        };
    }

    prepare_terminal_for_app_exit(&mut terminal)?;
    Ok(())
}

fn send_weather_request(app_state: &AppState, tx_county: &TokioSender<County>) {
    let Some(county) = app_state.get_selected_county() else {
         return;
     };
    let tx_county_clone = tx_county.clone();
    tokio::spawn(async move {
        match tx_county_clone.send(county).await {
            Ok(()) => info!("Successfully Sent County"),
            Err(error) => info!("Failed To Send Conunty: {error}"),
        }
    });
}

enum TickedUserInput {
    Input(KeyEvent),
    Tick,
}

#[derive(PartialEq)]
enum ResponseToUserInput {
    Continue,
    Stop,
}

fn handle_user_input(
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

fn prepare_terminal_for_app_exit(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    terminal::disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}

fn create_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

fn run_user_event_loop(tick_rate: Duration, tx: StdSender<TickedUserInput>) {
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

fn draw(
    total_drawing_rect: &mut ratatui::Frame<CrosstermBackend<io::Stdout>>,
    weather_type: WeatherType,
    counties_list: &mut ListState,
    counties: &[County],
    county_weather: &HashMap<County, WeatherReport>,
) {
    let app_rects = create_app_rects(total_drawing_rect.size());
    let tabs = create_weather_type_tabs(create_menu()).select(weather_type.into());
    total_drawing_rect.render_widget(tabs, app_rects.selection_hints);
    let _ = draw_weather(
        weather_type,
        total_drawing_rect,
        &app_rects,
        counties_list,
        counties,
        county_weather,
    );
}

fn create_menu<'a>() -> Vec<Line<'a>> {
    let menu_items = app_state::get_weather_type_strings();
    menu_items
        .iter()
        .map(|t| Line::from(vec![Span::styled(*t, Style::default())]))
        .collect()
}

fn draw_weather(
    weather_type: WeatherType,
    rect: &mut ratatui::Frame<CrosstermBackend<io::Stdout>>,
    app_rects: &AppRects,
    county_list_state: &mut ListState,
    all_counties: &[County],
    county_weather: &HashMap<County, WeatherReport>,
) -> Option<()> {
    let weather_rects = create_weather_rects(&app_rects.weather_display);
    let list = create_county_list_widget(all_counties);
    let county_index = county_list_state.selected()?;
    let table = create_county_table_widget(
        all_counties.get(county_index)?,
        weather_type,
        county_weather,
    );
    rect.render_stateful_widget(list, weather_rects.names, county_list_state);
    rect.render_widget(table, weather_rects.details);
    Some(())
}

struct WeatherRects {
    names: Rect,
    details: Rect,
}

fn create_weather_rects(parent_rect: &Rect) -> WeatherRects {
    let pet_rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(*parent_rect);
    WeatherRects {
        names: pet_rects[0],
        details: pet_rects[1],
    }
}

fn create_weather_type_tabs(menu: Vec<Line<'_>>) -> Tabs<'_> {
    Tabs::new(menu)
        .block(Block::default().title("Weather Type").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider(Span::raw("|"))
}

struct AppRects {
    selection_hints: Rect,
    weather_display: Rect,
}

fn create_app_rects(total_drawing_rect: Rect) -> AppRects {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(vec![Constraint::Length(3), Constraint::Min(2)])
        .split(total_drawing_rect);
    AppRects {
        selection_hints: areas[0],
        weather_display: areas[1],
    }
}
