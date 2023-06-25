use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WeatherReport {
    temperature: f32,
    wind_speed: f32,
    rainfall: f32,
}
