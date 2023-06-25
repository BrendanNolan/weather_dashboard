use std::collections::HashMap;

use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, Row, Table},
};

use crate::{
    app_state::{County, WeatherType},
    weather_report::WeatherReport,
};

fn create_block(title: &str) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title(title)
        .border_type(BorderType::Plain)
}

fn create_highlight_style() -> Style {
    Style::default()
        .bg(Color::Yellow)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD)
}

pub fn create_county_list_widget<'a>(all_counties: &[County]) -> List<'a> {
    let county_items: Vec<_> = all_counties
        .iter()
        .map(|county| {
            ListItem::new(Spans::from(vec![Span::styled(
                county.0.clone(),
                Style::default(),
            )]))
        })
        .collect();
    List::new(county_items)
        .block(create_block("Counties"))
        .highlight_style(create_highlight_style())
}

pub fn create_county_table_widget<'a>(
    county: &County,
    weather_type: WeatherType,
    county_weather: &HashMap<County, WeatherReport>
) -> Table<'a> {
    let weather_type_string: &str = weather_type.into();
    Table::new(vec![Row::new(vec![Cell::from(Span::raw(format!(
        "{} forecast for {}  is {}",
        weather_type_string,
        county.0,
        county_weather.get(county).unwrap().get(weather_type)
    )))])])
    .header(Row::new(vec![Cell::from(Span::styled(
        weather_type_string,
        Style::default().add_modifier(Modifier::BOLD),
    ))]))
    .block(create_block("Forecast"))
    .widths(&[Constraint::Percentage(100)])
}
