mod app_state;
mod networking;
mod tui_utils;
mod weather_report;
mod widgets;

use app_state::{AppState, County, WeatherType};
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent},
    terminal,
};
use networking::run_client;
use std::{
    collections::HashMap,
    io::{self, Stdout},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Line},
    widgets::{Block, Borders, ListState, Tabs},
    Terminal,
};
use tui_utils::{get_next_index, get_previous_index};
use weather_report::WeatherReport;
use widgets::{create_county_list_widget, create_county_table_widget};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    thread::spawn(|| tokio::spawn(run_client()));

    let (tx_user_input, rx_user_input) = mpsc::channel();

    thread::spawn(move || run_user_event_loop(Duration::from_millis(200), tx_user_input));
    let (tx_results, rx_results) = std::sync::mpsc::channel();  // TODO: Attach tx to weather
                                                                // client.
    run_drawing_loop(rx_user_input, rx_results)?;

    Ok(())
}

fn run_drawing_loop(
    rx_user_input: Receiver<TickedUserInput>,
    rx_server_results: std::sync::mpsc::Receiver<(County, WeatherReport)>,
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
                &county_weather,
            );
        })?;

        response_to_input = handle_user_input(&rx_user_input.recv()?, &mut app_state)?;
        if let Ok((county, report)) = rx_server_results.try_recv() {
            county_weather.insert(county, report);
        };
    }

    prepare_terminal_for_app_exit(&mut terminal)?;
    Ok(())
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
        return Ok(ResponseToUserInput::Continue);
    };
    if event.code == KeyCode::Char('q') {
        return Ok(ResponseToUserInput::Stop);
    }
    match event.code {
        KeyCode::Char('w') => app_state.active_weather_type = WeatherType::Wind,
        KeyCode::Char('r') => app_state.active_weather_type = WeatherType::Rain,
        KeyCode::Char('s') => app_state.active_weather_type = WeatherType::Sun,
        KeyCode::Char('k') => app_state
            .counties_list
            .select(get_previous_index(&app_state.counties_list)),
        KeyCode::Char('j') => app_state
            .counties_list
            .select(get_next_index(&app_state.counties_list)),
        _ => {}
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

fn run_user_event_loop(tick_rate: Duration, tx: mpsc::Sender<TickedUserInput>) {
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
    counties: &mut ListState,
    county_weather: &HashMap<County, WeatherReport>,
) {
    let app_rects = create_app_rects(total_drawing_rect.size());
    let tabs = create_weather_type_tabs(create_menu()).select(weather_type.into());
    total_drawing_rect.render_widget(tabs, app_rects.selection_hints);
    let _ = draw_weather(
        weather_type,
        total_drawing_rect,
        &app_rects,
        counties,
        &[County("Wexford".to_string()), County("Cork".to_string())],
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
