use ratatui::widgets::ListState;
use weather_dashboard::county::County;
use weather_dashboard::weather_report::WeatherType;

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
