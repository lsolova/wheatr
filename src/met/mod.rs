use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub lat: f32,
    pub lon: f32,
}
impl Display for Station {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Station {} ({}) on {}, {}", self.name, self.id, self.lat, self.lon)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Observation {
    pub station_id: String,
    pub observation_time: String,
    pub aerial_temperature: f32,
    pub relative_humidity: f32,
}

impl Display for Observation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Observation of {} at {}: temp/hum {}/{} ", self.station_id, self.observation_time, self.aerial_temperature, self.relative_humidity)
    }
}

pub struct MeteoData {
    pub stations: Vec<Station>,
    pub observations: Vec<Observation>,
}

pub struct Location {
    pub lat: f32,
    pub lon: f32,
}

#[derive(Serialize)]
pub struct WheatrApiResponseData {
    pub used_stations: Vec<Station>,
    pub local_lat: f32,
    pub local_lon: f32,
    pub local_air_temperature: f32,
    pub local_rel_humidity: f32,
    pub local_hi: f32,
}
impl Display for WheatrApiResponseData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Weather data on {}, {} place:\n temp/hum {}/{} with HI {}\n\n by the following stations\n{:?}", self.local_lat, self.local_lon, self.local_air_temperature, self.local_rel_humidity, self.local_hi, self.used_stations)
    }
}