use serde::Deserialize;

use crate::metrics::apply_weather;
use crate::snapshot::SharedSnapshot;

#[derive(Debug, Deserialize)]
struct IpWho {
    success: Option<bool>,
    city: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct OpenMeteo {
    current: Option<OpenMeteoCurrent>,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoCurrent {
    temperature_2m: Option<f32>,
    weather_code: Option<i32>,
    wind_speed_10m: Option<f32>,
    relative_humidity_2m: Option<f32>,
}

pub async fn refresh(snapshot: &SharedSnapshot) {
    match load_weather().await {
        Ok(data) => apply_weather(
            snapshot,
            data.0,
            data.1,
            data.2,
            data.3,
            data.4,
        ),
        Err(_) => apply_weather(
            snapshot,
            "Nearest city unavailable".to_string(),
            None,
            String::new(),
            None,
            None,
        ),
    }
}

async fn load_weather() -> Result<(String, Option<f32>, String, Option<f32>, Option<f32>), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let geo: IpWho = client
        .get("https://ipwho.is/")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if geo.success == Some(false) {
        return Err("geolocation failed".to_string());
    }
    let lat = geo.latitude.ok_or_else(|| "missing latitude".to_string())?;
    let lon = geo.longitude.ok_or_else(|| "missing longitude".to_string())?;
    let city = geo
        .city
        .filter(|c| !c.trim().is_empty())
        .unwrap_or_else(|| "Nearest city".to_string());

    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current=temperature_2m,weather_code,wind_speed_10m,relative_humidity_2m"
    );
    let forecast: OpenMeteo = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let current = forecast.current.unwrap_or(OpenMeteoCurrent {
        temperature_2m: None,
        weather_code: None,
        wind_speed_10m: None,
        relative_humidity_2m: None,
    });
    Ok((
        city,
        current.temperature_2m,
        weather_text(current.weather_code.unwrap_or(0)),
        current.relative_humidity_2m,
        current.wind_speed_10m,
    ))
}

fn weather_text(code: i32) -> String {
    let label = match code {
        0 => "Clear sky",
        1 => "Mainly clear",
        2 => "Partly cloudy",
        3 => "Overcast",
        45 | 48 => "Fog",
        51 | 53 | 55 => "Drizzle",
        56 | 57 => "Freezing drizzle",
        61 | 63 | 65 => "Rain",
        66 | 67 => "Freezing rain",
        71 | 73 | 75 => "Snow",
        77 => "Snow grains",
        80 | 81 | 82 => "Rain showers",
        85 | 86 => "Snow showers",
        95 => "Thunderstorm",
        96 | 99 => "Thunderstorm with hail",
        _ => "Weather",
    };
    label.to_string()
}
