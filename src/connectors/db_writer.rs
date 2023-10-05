use std::io::Error;

use super::sqlite_connector::{write_observations_to_db, write_stations_to_db};
use crate::met::MeteoData;

pub fn write_to_database(meteo_data: &MeteoData) -> Result<(), Error> {
    write_stations_to_db(&meteo_data.stations)?;
    write_observations_to_db(&meteo_data.observations)
}
