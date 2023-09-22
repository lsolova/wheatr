use crate::met::{Observation, Station, Location};

// Heat index calculation constants
const C1: f32 = -8.78469475556;
const C2: f32 = 1.61139411;
const C3: f32 = 2.33854883889;
const C4: f32 = -0.14611605;
const C5: f32 = -1.2308094e-2;
const C6: f32 = -1.64248277778e-2;
const C7: f32 = 2.211732e-3;
const C8: f32 = 7.2546e-4;
const C9: f32 = -3.582e-6;


#[derive(Default)]
pub struct LocatedValue {
    lat: f32,
    lon: f32,
    val: f32,
}

pub fn get_located_values(stations: &[Station; 3], observations: &[Observation; 3], get_value: &dyn Fn(&Observation) -> f32) -> [LocatedValue; 3] {
    let mut located_values: [LocatedValue; 3] = Default::default();
    for i in 0..stations.len() {
        let station = &stations[i];
        let rel_observation_pos = observations.iter().position(|o| o.station_id == station.id).unwrap();
        let observation = &observations[rel_observation_pos];
        let l_value = LocatedValue {
            lat: station.lat,
            lon: station.lon,
            val: get_value(observation),
        };
        located_values[i] = l_value;
    }
    located_values
}

pub fn calculate_local_data(location: &Location, known_points: &[LocatedValue; 3]) -> f32 {
    let pa = &known_points[0]; //p
    let pb = &known_points[1]; //q
    let pc = &known_points[2]; //r
    let pd = location; //s
    let nv = LocatedValue {
        lat: ((pb.lon - pa.lon) * (pc.val - pa.val)) - ((pb.val - pa.val) * (pc.lon - pa.lon)),
        lon: ((pb.lat - pa.lat) * (pc.val - pa.val)) - ((pb.val - pa.val) * (pc.lat - pa.lat)),
        val: ((pb.lat - pa.lat) * (pc.lon - pa.lon)) - ((pb.lon - pa.lon) * (pc.lat - pa.lat)),
    };
    let pd_val = ((nv.lat * pa.lat) - (nv.lon * pa.lon) + (nv.val * pa.val)
        - (nv.lat * pd.lat)
        - (-nv.lon * pd.lon))
        / nv.val;
    pd_val
}

/**
 * Sources:
 * - https://en.wikipedia.org/wiki/Heat_index
 * - https://www.ncbi.nlm.nih.gov/pmc/articles/PMC3801457/
 *   algorithm by Blazejczyk et al. 2012
 */
pub fn calculate_heat_index(temperature: f32, humidity: f32) -> f32 {
    if temperature < 20.0 {
        return temperature;
    }
    let t_pow2 = temperature.powi(2);
    let h_pow2 = humidity.powi(2);
    let hi = C1
        + C2 * temperature
        + C3 * humidity
        + C4 * temperature * humidity
        + C5 * t_pow2
        + C6 * h_pow2
        + C7 * t_pow2 * humidity
        + C8 * temperature * h_pow2
        + C9 * t_pow2 * h_pow2;
    hi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_temperature() {
        let location = Location {
            lat: 36.6952842,
            lon: -4.4538607,
        };
        let point_1 = LocatedValue {
            lat: 36.66612,
            lon: -4.482307,
            val: 43.3,
        };
        let point_2 = LocatedValue {
            lat: 36.717785,
            lon: -4.48167,
            val: 41.2,
        };
        let point_3 = LocatedValue {
            lat: 36.716663,
            lon: -4.41972,
            val: 34.6,
        };

        let temp = calculate_local_data(&location, &[point_1, point_2, point_3]);

        assert_eq!(temp, 39.10227);
    }
    #[test]
    fn calculate_hi() {
        assert_eq!(calculate_heat_index(19.0, 40.0), 19.0);
        assert_eq!(calculate_heat_index(29.0, 40.0), 28.606316);
        assert_eq!(calculate_heat_index(35.0, 60.0), 45.050167);
    }
}
