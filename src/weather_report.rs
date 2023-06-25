use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug)]
pub enum WeatherType {
    Wind,
    Rain,
    Sun,
}

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

impl Default for WeatherReport {
    fn default() -> Self {
        WeatherReport {
            sunshine: 0.0,
            wind_speed: 0.0,
            rainfall: 0.0,
        }
    }
}
