use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug)]
pub enum WeatherType {
    Wind,
    Rain,
    Sun,
}

impl From<WeatherType> for usize {
    fn from(weather_type: WeatherType) -> Self {
        match weather_type {
            WeatherType::Wind => 0,
            WeatherType::Rain => 1,
            WeatherType::Sun => 2,
        }
    }
}

impl From<usize> for WeatherType {
    fn from(number: usize) -> Self {
        match number {
            0 => WeatherType::Wind,
            1 => WeatherType::Rain,
            2 => WeatherType::Sun,
            _ => panic!("Cannot Create Weathertype From Integer"),
        }
    }
}

impl From<WeatherType> for &'static str {
    fn from(weather_type: WeatherType) -> Self {
        match weather_type {
            WeatherType::Wind => "Wind",
            WeatherType::Rain => "Rain",
            WeatherType::Sun => "Sun",
        }
    }
}

pub fn get_weather_type_strings() -> Vec<&'static str> {
    vec![0, 1, 2]
        .into_iter()
        .map(|i| {
            let weather_type: WeatherType = i.into();
            let weather_type: &'static str = weather_type.into();
            weather_type
        })
        .collect()
}

pub struct AppState {
    pub active_weather_type: WeatherType,
    pub counties_list: ListState,
    pub counties: Vec<County>,
}

impl AppState {
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
        }
    }
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug, Hash)]
pub struct County(pub String);
