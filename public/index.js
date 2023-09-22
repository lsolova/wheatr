const OPTIMAL_TEMPERATURE = 25;
const TEMPERATURE_STEP = 12.5;

function success(result) {
  document.getElementById("main-info").innerText =
    "Retrieving local data. Stay tuned.";
  const loc = {
    lat: result.coords.latitude,
    lon: result.coords.longitude,
  };
  fetch(`http://localhost:8088/api/hi?lat=${loc.lat}&lon=${loc.lon}`)
    .then((response) => response.json())
    .then((data) => {
      document.getElementById("main-title").remove();
      document.getElementById("main-info").remove();
      const rootE = document.createElement("div");
      rootE.setAttribute("id", "app-root");
      const tempE = document.createElement("div");
      tempE.setAttribute("id", "temp");
      tempE.appendChild(
        document.createTextNode(`Temperature: ${data.local_air_temperature.toFixed(2)} °C`)
      );
      const humidE = document.createElement("div");
      humidE.setAttribute("id", "humid");
      humidE.appendChild(
        document.createTextNode(`Humidity: ${data.local_rel_humidity.toFixed(2)} %`)
      );
      const hiE = document.createElement("div");
      hiE.setAttribute("id", "hi");
      const hiColorE = document.createElement("div");
      const red = Math.min(
        255,
        Math.max(0, data.local_air_temperature - OPTIMAL_TEMPERATURE) *
          TEMPERATURE_STEP
      );
      const green = Math.max(
        0,
        255 -
          Math.abs(
            Math.max(0, data.local_air_temperature - OPTIMAL_TEMPERATURE)
          ) *
            TEMPERATURE_STEP
      );
      const blue = Math.max(
        0,
        255 - data.local_air_temperature * TEMPERATURE_STEP
      );
      hiColorE.setAttribute(
        "style",
        `background-color: rgb(${red}, ${green}, ${blue})`
      );
      hiE.appendChild(hiColorE);
      hiE.appendChild(document.createTextNode(`Heat index: ${data.local_hi.toFixed(2)} °C`));
      const stationsE = document.createElement("div");
      stationsE.setAttribute("id", "stations");
      stationsE.appendChild(
        document.createTextNode(
          `Based on ${data.used_stations.map((s) => s.name).join(", ")} stations`
        )
      );
      rootE.appendChild(tempE);
      rootE.appendChild(humidE);
      rootE.appendChild(hiE);
      rootE.appendChild(stationsE);
      document.body.appendChild(rootE);
    });
}

function denied(e) {
  console.error(e);
  document.getElementById("main-title").innerText = "Access denied";
  document.getElementById("main-info").innerText =
    "Sorry, we are unable to serve your request as access to your location is denied.";
}
window.navigator.geolocation.getCurrentPosition(success, denied);
