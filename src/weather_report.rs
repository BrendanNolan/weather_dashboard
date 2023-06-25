use serde::{Deserialize, Serialize};

use crate::app_state::WeatherType;

#[derive(Serialize, Deserialize, Debug)]
pub struct WeatherReport {
    sunshine: f32,
    wind_speed: f32,
    rainfall: f32,
}

impl WeatherReport {
    pub fn get(&self, weather_type: WeatherType) -> f32 {
        match weather_type {
            WeatherType::Rain => self.rainfall,
            WeatherType::Sun => self.sunshine,
            WeatherType::Wind => self.wind_speed,
        }
    }
}
