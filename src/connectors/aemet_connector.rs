use encoding_rs::ISO_8859_15;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::io::{Cursor, Read, Result};
use std::{env, process};

use crate::met::{MeteoData, Observation, Station};

use super::downloader;

const ENV_API_KEY: &str = "AEMET_API_KEY";
const ENV_URL: &str = "AEMET_URL";

#[derive(Debug, Deserialize, Serialize)]
pub struct AemetData {
    pub fint: String,    // Observation time
    pub idema: String,   // Station ID
    pub hr: Option<f32>, // Relative humidity
    pub lat: f32,        // Station latitude
    pub lon: f32,        // Station longitude
    pub ta: Option<f32>, // Aerial temperature
    pub ubi: String,     // Station name
}
impl Display for AemetData {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let hr = match self.hr {
            Some(hr) => hr,
            None => -1.0,
        };
        let ta = match self.ta {
            Some(ta) => ta,
            None => -1.0,
        };
        write!(
            f,
            "{} {} {} {} {} {} {}",
            self.fint, self.idema, self.ubi, hr, self.lat, self.lon, ta
        )
    }
}

#[derive(Deserialize)]
pub struct AemetFirstResponse {
    pub estado: i32,
    pub datos: String,
    pub metadatos: String,
}

fn get_env_var(key: &str) -> String {
    match env::var(key) {
        Ok(val) => {
            return val.to_string();
        },
        Err(e) => {
            println!("{} is undefined: {}", key, e);
            process::exit(0x001)
        }
    };
}

fn vec_to_string(content: Vec<u8>) -> Result<String> {
    let reader = Cursor::new(content);
    let mut rdr = encoding_rs_io::DecodeReaderBytesBuilder::new()
        .encoding(Some(ISO_8859_15))
        .build(reader);
    let mut content = String::new();
    rdr.read_to_string(&mut content)?;
    Ok(content)
}

fn read_main_download_json(content: &str) -> Result<String> {
    let download_url: AemetFirstResponse = match serde_json::from_str(content) {
        Ok(download_content) => download_content,
        Err(e) => Err(e)?,
    };
    Ok(download_url.datos)
}
fn read_data_set(content: &str) -> Result<Vec<AemetData>> {
    let aemet_data_set = match serde_json::from_str(content) {
        Ok(aemet_data_set) => aemet_data_set,
        Err(e) if e.is_eof() => Vec::new(),
        Err(e) => Err(e)?,
    };
    Ok(aemet_data_set)
}

fn convert_to_data_objects(data_set: &Vec<AemetData>) -> MeteoData {
    let mut stations: Vec<Station> = vec![];
    let mut observations: Vec<Observation> = vec![];
    for data_entry in data_set {
        let station = Station {
            id: data_entry.idema.clone(),
            name: data_entry.ubi.clone(),
            lat: data_entry.lat,
            lon: data_entry.lon,
        };
        stations.push(station);
        if data_entry.ta.is_some() && data_entry.hr.is_some() {
            let observation = Observation {
                station_id: data_entry.idema.clone(),
                aerial_temperature: data_entry.ta.unwrap(),
                observation_time: data_entry.fint.clone(),
                relative_humidity: data_entry.hr.unwrap(),
            };
            observations.push(observation);
        }
    }
    MeteoData {
        stations: stations,
        observations: observations,
    }
}

pub fn load_data() -> Result<MeteoData> {
    let api_key = get_env_var(ENV_API_KEY);
    let url = get_env_var(ENV_URL);

    let main_download_reader = downloader::download_content(&url, &api_key)?;
    let main_download_content = vec_to_string(main_download_reader)?;
    let main_download_url = read_main_download_json(&main_download_content)?;
    let data_reader = downloader::download_content(&main_download_url, &api_key)?;
    let data_content = vec_to_string(data_reader)?;
    let data_set = read_data_set(&data_content.as_str())?;
    let meteo_data = convert_to_data_objects(&data_set);
    Ok(meteo_data)
}
