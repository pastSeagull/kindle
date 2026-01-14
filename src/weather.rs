use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WeatherCombined {
    pub now: Option<NowWeather>,
    pub hourly: Vec<HourlyForecast>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NowWeather {
    pub temp: String,
    pub icon: String,
    pub text: String,
    pub humidity: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HourlyForecast {
    #[serde(rename = "fxTime")]
    pub fx_time: String,
    pub temp: String,
    pub icon: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponseHourly {
    code: String,
    hourly: Option<Vec<HourlyForecast>>,
}

#[derive(Debug, Deserialize)]
struct ApiResponseNow {
    code: String,
    now: Option<NowWeather>,
}

pub async fn fetch_weather() -> Result<WeatherCombined, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    // Fetch 24h
    let url_24h = format!(
        "https://{}/v7/weather/24h?location={}&key={}",
        crate::config::WEATHER_API_HOST,
        crate::config::WEATHER_LOCATION,
        crate::config::WEATHER_API_KEY
    );
    let resp_24h = client.get(&url_24h).send().await?;
    let status_24h = resp_24h.status();
    if !status_24h.is_success() {
        return Err(format!("Weather API 24h error: {}", status_24h).into());
    }
    let api_resp_24h: ApiResponseHourly = resp_24h.json().await?;
    if api_resp_24h.code != "200" {
        return Err(format!("Weather API 24h returned code: {}", api_resp_24h.code).into());
    }
    let hourly = api_resp_24h.hourly.unwrap_or_default();

    // Fetch Now
    let url_now = format!(
        "https://{}/v7/weather/now?location={}&key={}",
        crate::config::WEATHER_API_HOST,
        crate::config::WEATHER_LOCATION,
        crate::config::WEATHER_API_KEY
    );
    let resp_now = client.get(&url_now).send().await?;
    let status_now = resp_now.status();
    if !status_now.is_success() {
        return Err(format!("Weather API Now error: {}", status_now).into());
    }
    let api_resp_now: ApiResponseNow = resp_now.json().await?;
    if api_resp_now.code != "200" {
        return Err(format!("Weather API Now returned code: {}", api_resp_now.code).into());
    }
    let now = api_resp_now.now;

    Ok(WeatherCombined { now, hourly })
}
