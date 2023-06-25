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
