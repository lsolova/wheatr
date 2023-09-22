use super::sqlite_connector::{write_observation_to_db, write_station_to_db};
use crate::met::MeteoData;

pub fn write_to_database(meteo_data: &MeteoData) {
    for station in &meteo_data.stations {
        match write_station_to_db(&station) {
            Ok(_) => {
                //println!("Station successfully saved: {}", station)
            }
            Err(err) => println!("Error with station: {} -> {}", station.id, err),
        };
    }
    for observation in &meteo_data.observations {
        match write_observation_to_db(&observation) {
            Ok(_) => {
                //println!("Observation successfully saved: {}", observation)
            }
            Err(err) => println!(
                "Error with observation: {}/{} -> {}",
                observation.station_id, observation.observation_time, err
            ),
        }
    }
}
