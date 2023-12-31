use std::{collections::HashMap, io::Error};

use rusqlite::{Connection, Rows, ToSql};

use crate::met::{Location, Observation, Station, ToSqlParams};

const STMT_GET_CLOSEST_STATIONS: &str = "SELECT id, name, lat, lon, (ABS(lat)-:my_lat) * (ABS(lat)-:my_lat) + (ABS(lon)-:my_lon) * (ABS(lon)-:my_lon) as diff FROM stations GROUP BY lat, lon ORDER BY diff ASC LIMIT 3";
const STMT_GET_LATEST_OBSERVATIONS: &str =  "SELECT * FROM observations WHERE station_id IN (:s1, :s2, :s3) ORDER BY observation_time DESC, station_id ASC LIMIT 12";
const STMT_SET_STATION: &str =
    "INSERT INTO stations (id, name, lat, lon) VALUES (:id, :name, :lat, :lon) ON CONFLICT (id) DO NOTHING";
const STMT_SET_OBSERVATION: &str = "INSERT INTO observations (station_id, observation_time, air_temperature, rel_humidity) VALUES (:station_id, :observation_time, :air_temperature, :rel_humidity) ON CONFLICT (station_id, observation_time) DO NOTHING";

pub fn get_connection() -> Result<Connection, rusqlite::Error> {
    let connection = Connection::open(".met.sqlite")?;
    connection.execute_batch(
        "BEGIN;
        CREATE TABLE IF NOT EXISTS stations (
            id TEXT NOT NULL UNIQUE,
            name TEXT,
            lat	REAL NOT NULL,
            lon	REAL NOT NULL,
            PRIMARY KEY(id)
        );
        CREATE TABLE IF NOT EXISTS observations (
            station_id TEXT NOT NULL,
            observation_time TEXT NOT NULL,
            air_temperature REAL NOT NULL,
            rel_humidity REAL NOT NULL,
            PRIMARY KEY(observation_time,station_id),
            FOREIGN KEY(station_id) REFERENCES stations(id) ON DELETE CASCADE
        );
        COMMIT;",
    )?;
    Ok(connection)
}

fn run_get_stmt<T>(
    query: &str,
    params: &[(&str, &dyn ToSql)],
    rows_mapper: &dyn Fn(Rows) -> Result<T, rusqlite::Error>,
) -> Result<T, rusqlite::Error> {
    let connection = get_connection()?;
    let mut stmt = (&connection).prepare(query)?;
    let rows = stmt.query(params)?;
    rows_mapper(rows)
}

pub fn get_closest_stations_from_db(loc: &Location) -> Result<[Station; 3], Error> {
    fn extract_closest_stations(mut rows: Rows) -> Result<[Station; 3], rusqlite::Error> {
        let mut closest_stations: Vec<Station> = Vec::new();
        let mut i = true;
        while i {
            match rows.next() {
                Ok(None) => i = false,
                Ok(Some(r)) => {
                    let read_station = Station {
                        id: r.get_unwrap("id"),
                        name: r.get_unwrap("name"),
                        lat: r.get_unwrap("lat"),
                        lon: r.get_unwrap("lon"),
                    };
                    closest_stations.push(read_station);
                }
                Err(_) => {}
            };
        }
        Ok(closest_stations.try_into().unwrap())
    }

    match run_get_stmt::<[Station; 3]>(
        STMT_GET_CLOSEST_STATIONS,
        &[(":my_lat", &loc.lat.abs()), (":my_lon", &loc.lon.abs())],
        &extract_closest_stations,
    ) {
        Ok(result) => return Ok(result),
        Err(err) => {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!("Data loading failed: {}", err),
            ))
        }
    }
}

pub fn get_latest_observations(stations: &[Station; 3]) -> Result<[Observation; 3], Error> {
    fn extract_latest_observations(mut rows: Rows) -> Result<[Observation; 3], rusqlite::Error> {
        let mut latest_observations_map: HashMap<String, Observation> = HashMap::new();
        let mut latest_observations: [Observation; 3] = Default::default();
        let mut i = true;
        while i {
            match rows.next() {
                Ok(None) => {
                    i = false;
                }
                Ok(Some(row)) => {
                    let new_obs = Observation {
                        station_id: row.get_unwrap("station_id"),
                        observation_time: row.get_unwrap("observation_time"),
                        aerial_temperature: row.get_unwrap("air_temperature"),
                        relative_humidity: row.get_unwrap("rel_humidity"),
                    };
                    let prev_obs = latest_observations_map.get(&new_obs.station_id);
                    if prev_obs.is_none()
                        || prev_obs.unwrap().observation_time < new_obs.observation_time
                    {
                        latest_observations_map.insert(new_obs.station_id.clone(), new_obs);
                    }
                }
                Err(_) => {}
            }
        }
        let mut j = 0;
        latest_observations_map.values().for_each(|o| {
            latest_observations[j] = o.clone();
            j = j + 1;
        });
        Ok(latest_observations.try_into().unwrap())
    }

    match run_get_stmt(
        STMT_GET_LATEST_OBSERVATIONS,
        &[
            (":s1", &stations[0].id),
            (":s2", &stations[1].id),
            (":s3", &stations[2].id),
        ],
        &extract_latest_observations,
    ) {
        Ok(result) => return Ok(result),
        Err(err) => {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!("Data loading failed: {}", err),
            ))
        }
    }
}

fn write_items_to_db<T: ToSqlParams>(
    items: &Vec<T>,
    write_statement: &str,
) -> Result<(), rusqlite::Error> {
    let mut connection = get_connection()?;
    let transaction = connection.transaction()?;
    for item in items {
        let params = item.to_sql_params();
        transaction.execute(write_statement, params)?;
    }
    transaction.commit()?;
    Ok(())
}

pub fn write_stations_to_db(stations: &Vec<Station>) -> Result<(), Error> {
    match write_items_to_db::<Station>(stations, STMT_SET_STATION) {
        Ok(_) => return Ok(()),
        Err(err) => {
            println!("Error with connection: {}", err);
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!("Data saving failed: {}", err),
            ));
        }
    };
}

pub fn write_observations_to_db(observations: &Vec<Observation>) -> Result<(), Error> {
    match write_items_to_db::<Observation>(observations, STMT_SET_OBSERVATION) {
        Ok(_) => return Ok(()),
        Err(err) => {
            println!("Error with connection: {}", err);
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!("Data saving failed: {}", err),
            ));
        }
    };
}
