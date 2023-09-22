# Wheatr

This is a sample Rust project, my first one using this language.

It is a small server, which gets observation data from Aemet (the Spanish meteorology service), put it into an SQLite database and provides a located temperature, humidity and calculated heat index data within Spain.

Local temperature and humidity is based on the three closest stations. There is a plane calculation to find the forth point on the plane, where x, y are latitude and longitude values, z is the temperature or humidity.

Heat Index is calculated following the algorithm described on [Wikipedia](https://en.wikipedia.org/wiki/Heat_index), formula for Celsius calculations

It requires two environment variables:

- API_KEY: It can be get from Aemet (<https://opendata.aemet.es>)
- AEMET_MAIN_URL: This is the data url for observations: <https://opendata.aemet.es/opendata/api/observacion/convencional/todas>

## Usage

1. Checkout
2. Set environment variables
3. Run
4. Wait for first database update (it is scheduled)
5. Open <http://localhost:8088/index.html>
