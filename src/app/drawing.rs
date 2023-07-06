use super::widgets::{create_county_list_widget, create_county_table_widget};
use crate::{County, WeatherReport, WeatherType};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, ListState, Tabs},
};
use std::collections::HashMap;

pub fn draw(
    total_drawing_rect: &mut ratatui::Frame<CrosstermBackend<std::io::Stdout>>,
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
    let menu_items = super::get_weather_type_strings();
    menu_items
        .iter()
        .map(|t| Line::from(vec![Span::styled(*t, Style::default())]))
        .collect()
}

fn draw_weather(
    weather_type: WeatherType,
    rect: &mut ratatui::Frame<CrosstermBackend<std::io::Stdout>>,
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
