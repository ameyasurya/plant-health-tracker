//! Location lookup and weather, via Open-Meteo.
//!
//! Open-Meteo is used because it needs no API key, no signup and no
//! payment details for non-commercial use, which keeps this app free to
//! run for anyone who installs it.
//!
//! Everything here is best-effort. The app is offline-first: if any of
//! these calls fail the caller falls back to the cached forecast, and
//! failing that to plain month-based seasons. A plant reminder widget must
//! not stop working because the network is down.

use serde::{Deserialize, Serialize};

use crate::models::{DailyWeather, Location};

const GEOCODE_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";
const FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";
/// Returns coordinates directly, over HTTPS, without a key.
const IP_LOOKUP_URL: &str = "https://ipwho.is/";

const TIMEOUT_SECS: u64 = 12;

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .user_agent("plant-health-tracker")
        .build()
        .map_err(|e| e.to_string())
}

// ---- City search ----------------------------------------------------

#[derive(Debug, Deserialize)]
struct GeoResponse {
    #[serde(default)]
    results: Vec<GeoHit>,
}

#[derive(Debug, Deserialize)]
struct GeoHit {
    name: String,
    latitude: f64,
    longitude: f64,
    #[serde(default)]
    timezone: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    country_code: Option<String>,
    #[serde(default)]
    admin1: Option<String>,
}

impl GeoHit {
    fn into_location(self) -> Location {
        // "Bengaluru, Karnataka, India" reads better than a bare city name
        // when several places share one.
        let mut parts = vec![self.name];
        if let Some(a) = self.admin1.filter(|s| !s.is_empty()) {
            parts.push(a);
        }
        if let Some(c) = self.country.filter(|s| !s.is_empty()) {
            parts.push(c);
        }
        Location {
            label: parts.join(", "),
            latitude: self.latitude,
            longitude: self.longitude,
            timezone: self.timezone.unwrap_or_else(|| "UTC".to_string()),
            country_code: self.country_code.unwrap_or_default(),
        }
    }
}

pub async fn search_places(query: &str) -> Result<Vec<Location>, String> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let resp: GeoResponse = client()?
        .get(GEOCODE_URL)
        .query(&[("name", q), ("count", "8"), ("language", "en")])
        .send()
        .await
        .map_err(|e| format!("couldn't reach the location service: {e}"))?
        .json()
        .await
        .map_err(|e| format!("unexpected response from the location service: {e}"))?;

    Ok(resp.results.into_iter().map(|h| h.into_location()).collect())
}

// ---- IP-based detection ---------------------------------------------

#[derive(Debug, Deserialize)]
struct IpResponse {
    #[serde(default)]
    success: Option<bool>,
    #[serde(default)]
    city: Option<String>,
    #[serde(default)]
    region: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    country_code: Option<String>,
    #[serde(default)]
    latitude: Option<f64>,
    #[serde(default)]
    longitude: Option<f64>,
    #[serde(default)]
    timezone: Option<IpTimezone>,
}

#[derive(Debug, Deserialize)]
struct IpTimezone {
    #[serde(default)]
    id: Option<String>,
}

/// Approximate location from the caller's IP address.
///
/// Deliberately only called when the user presses "Detect" -- the app is
/// otherwise fully offline, and quietly sending someone's IP to a third
/// party on every launch is not a reasonable default.
pub async fn detect_location() -> Result<Location, String> {
    let resp: IpResponse = client()?
        .get(IP_LOOKUP_URL)
        .send()
        .await
        .map_err(|e| format!("couldn't reach the location service: {e}"))?
        .json()
        .await
        .map_err(|e| format!("unexpected response from the location service: {e}"))?;

    if resp.success == Some(false) {
        return Err("the location service couldn't place this connection".to_string());
    }
    let (Some(latitude), Some(longitude)) = (resp.latitude, resp.longitude) else {
        return Err("the location service didn't return coordinates".to_string());
    };

    let mut parts: Vec<String> = Vec::new();
    for p in [resp.city, resp.region, resp.country] {
        if let Some(v) = p.filter(|s| !s.is_empty()) {
            parts.push(v);
        }
    }
    let label = if parts.is_empty() {
        format!("{latitude:.2}, {longitude:.2}")
    } else {
        parts.join(", ")
    };

    Ok(Location {
        label,
        latitude,
        longitude,
        timezone: resp.timezone.and_then(|t| t.id).unwrap_or_else(|| "UTC".to_string()),
        country_code: resp.country_code.unwrap_or_default(),
    })
}

// ---- Forecast -------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ForecastResponse {
    daily: DailyBlock,
}

#[derive(Debug, Deserialize)]
struct DailyBlock {
    time: Vec<String>,
    #[serde(default)]
    precipitation_sum: Vec<Option<f64>>,
    #[serde(default)]
    temperature_2m_max: Vec<Option<f64>>,
    #[serde(default)]
    temperature_2m_min: Vec<Option<f64>>,
}

/// Recent past plus near future. The past days are what let the schedule
/// notice "it already rained, skip the watering".
pub const PAST_DAYS: u8 = 5;
pub const FORECAST_DAYS: u8 = 3;

pub async fn fetch_forecast(lat: f64, lon: f64) -> Result<Vec<DailyWeather>, String> {
    let resp: ForecastResponse = client()?
        .get(FORECAST_URL)
        .query(&[
            ("latitude", lat.to_string()),
            ("longitude", lon.to_string()),
            (
                "daily",
                "precipitation_sum,temperature_2m_max,temperature_2m_min".to_string(),
            ),
            ("past_days", PAST_DAYS.to_string()),
            ("forecast_days", FORECAST_DAYS.to_string()),
            ("timezone", "auto".to_string()),
        ])
        .send()
        .await
        .map_err(|e| format!("couldn't reach the weather service: {e}"))?
        .json()
        .await
        .map_err(|e| format!("unexpected response from the weather service: {e}"))?;

    let d = resp.daily;
    let mut out = Vec::with_capacity(d.time.len());
    for (i, day) in d.time.iter().enumerate() {
        let Ok(date) = chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d") else {
            continue;
        };
        out.push(DailyWeather {
            date,
            precipitation_mm: d.precipitation_sum.get(i).copied().flatten().unwrap_or(0.0),
            temp_max_c: d.temperature_2m_max.get(i).copied().flatten().unwrap_or(f64::NAN),
            temp_min_c: d.temperature_2m_min.get(i).copied().flatten().unwrap_or(f64::NAN),
        });
    }
    Ok(out)
}

/// Shape returned to the UI for the weather strip.
#[derive(Debug, Clone, Serialize)]
pub struct WeatherSummary {
    pub location_label: String,
    pub today_max_c: Option<f64>,
    pub today_min_c: Option<f64>,
    /// Total rain over the recent past window, in mm.
    pub recent_rain_mm: f64,
    pub rained_recently: bool,
    pub fetched_at: Option<String>,
    /// True when showing cached data because the last refresh failed.
    pub stale: bool,
}
