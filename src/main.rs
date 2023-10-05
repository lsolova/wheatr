use clokwerk::{Scheduler, TimeUnits};
use std::{io::Error, thread, time::{Instant, Duration}, str::FromStr};
use tide::{prelude::*, Request, Response, http::Mime};

use connectors::sqlite_connector::get_closest_stations_from_db;

use crate::{met::{Location, WheatrApiResponseData}, connectors::sqlite_connector::get_latest_observations};

mod calculators;
mod connectors;
mod met;

fn update_meteo_db() {
    println!("Meteo data downloading started");
    let start = Instant::now();
    let meteo_data = match connectors::aemet_connector::load_data() {
        Ok(md) => md,
        Err(e) => { println!("Meteo data downloading failed. {}", e); return; }
    };
    println!("Meteo data downloading finished in {:?}", start.elapsed());

    println!("Meteo data persisting started");
    let start = Instant::now();
    match connectors::db_writer::write_to_database(&meteo_data) {
        Ok(_) => (),
        Err(e) => { println!("Meteo data persisting failed. {}", e); return; }
    };
    println!("Meteo data persisting finished in {:?}", start.elapsed());
}

fn read_query_params(req: Request<()>) -> Result<Location, Error> {
    let mut query_pairs = req.url().query_pairs();
    if query_pairs.count() < 2 {
        return Err(Error::new(std::io::ErrorKind::InvalidData, "Bad Request: missing query params"));
    }
    let lat = match query_pairs.find(|item| { item.0 == "lat" }) {
        None => return Err(Error::new(std::io::ErrorKind::InvalidData, "Bad Request: lat param is missing")),
        Some(p) => p.1
    };
    let lon = match query_pairs.find(|item| { item.0 == "lon" }) {
        None => return Err(Error::new(std::io::ErrorKind::InvalidData, "Bad Request: lon param is missing")),
        Some(p) => p.1
    };
    let lat = match f32::from_str(&lat) {
        Ok(l) => l,
        Err(_e) => return Err(Error::new(std::io::ErrorKind::InvalidData, "Bad Request: lat is not a number")),
    };
    let lon = match f32::from_str(&lon) {
        Ok(l) => l,
        Err(_e) => return Err(Error::new(std::io::ErrorKind::InvalidData, "Bad Request: lon is not a number")),
    };
    println!("Request: {}, {}", lat, lon);
    let loc = Location {
        lat,
        lon,
    };

    Ok(loc)
}

fn get_local_data(loc: Location) -> Result<WheatrApiResponseData, Error> {

    let start = Instant::now();

    let closest_stations = get_closest_stations_from_db(&loc)?;
    let latest_observations = get_latest_observations(&closest_stations)?;
    let located_temperature_values = calculators::location_data_calculations::get_located_values(&closest_stations, &latest_observations, &|o| { o.aerial_temperature});
    let located_humidity_values = calculators::location_data_calculations::get_located_values(&closest_stations, &latest_observations, &|o| { o.relative_humidity});
    let local_temperature_data = calculators::location_data_calculations::calculate_local_data(&loc,&located_temperature_values);
    let local_humidity_data = calculators::location_data_calculations::calculate_local_data(&loc,&located_humidity_values);
    let local_hi = calculators::location_data_calculations::calculate_heat_index(local_temperature_data, local_humidity_data);

    let api_response = WheatrApiResponseData {
        used_stations: closest_stations.to_vec(),
        local_air_temperature: local_temperature_data,
        local_hi: local_hi,
        local_lat: loc.lat,
        local_lon: loc.lon,
        local_rel_humidity: local_humidity_data,
    };

    println!("Data: {}", api_response);
    println!("Time elapsed to serve request is: {:?}", start.elapsed());
    Ok(api_response)
}

#[async_std::main]
async fn main() -> tide::Result<()> {

    thread::spawn(|| {
        update_meteo_db();
        let mut scheduler = Scheduler::new();
        scheduler.every(1.hours()).run(update_meteo_db);
        loop {
            scheduler.run_pending();
            thread::sleep(Duration::from_secs(1));
        }
    });

    let mut app = tide::new();
    app.with(tide::log::LogMiddleware::new());

    app.at("/").serve_dir("public")?;
    app.at("/api/hi").get(|request: Request<()>| async move {
        let loc = match read_query_params(request) {
            Ok(l) => l,
            Err(e) => {
                let mut response = Response::new(400);
                response.set_error(e);
                return Ok(response)
            }
        };
        match get_local_data(loc) {
            Ok(local_data) => {
                let mut response = Response::new(200);
                // response.append_header("Access-Control-Allow-Origin", "*");
                response.set_content_type(Mime::from_str("application/json;charset=utf-8").unwrap());
                response.set_body(json!(local_data));
                return Ok(response)
            },
            Err(e) => {
                let mut response = Response::new(500);
                response.set_error(e);
                return Ok(response)
            }
        }
    });
    app.listen("127.0.0.1:8088").await?;
    Ok(())
}