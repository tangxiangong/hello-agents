use crate::Error;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
struct WeatherResponse {
    #[serde(default)]
    current_condition: Vec<CurrentCondition>,
    #[serde(default)]
    nearest_area: Vec<NearestArea>,
    #[serde(default)]
    weather: Vec<ForecastDay>,
}

impl WeatherResponse {
    fn summary(&self, fallback_city: &str) -> String {
        let area = self
            .nearest_area
            .first()
            .and_then(NearestArea::display_name)
            .unwrap_or(fallback_city);

        let current = self.current_condition.first();
        let description = current
            .and_then(CurrentCondition::description)
            .unwrap_or("unknown");
        let temp_c = current.map(|item| item.temp_c.as_str()).unwrap_or("?");
        let feels_like_c = current
            .map(|item| item.feels_like_c.as_str())
            .unwrap_or("?");
        let humidity = current.map(|item| item.humidity.as_str()).unwrap_or("?");
        let wind_kmph = current
            .map(|item| item.windspeed_kmph.as_str())
            .unwrap_or("?");

        let forecast = self
            .weather
            .iter()
            .filter(|day| !day.date.is_empty())
            .take(3)
            .map(ForecastDay::summary)
            .collect::<Vec<_>>()
            .join("; ");

        if forecast.is_empty() {
            format!(
                "{area}: {description}, {temp_c}°C, feels like {feels_like_c}°C, humidity {humidity}%, wind {wind_kmph} km/h"
            )
        } else {
            format!(
                "{area}: {description}, {temp_c}°C, feels like {feels_like_c}°C, humidity {humidity}%, wind {wind_kmph} km/h. Forecast: {forecast}"
            )
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct CurrentCondition {
    #[serde(rename = "temp_C", default)]
    temp_c: String,
    #[serde(rename = "FeelsLikeC", default)]
    feels_like_c: String,
    #[serde(default)]
    humidity: String,
    #[serde(rename = "windspeedKmph", default)]
    windspeed_kmph: String,
    #[serde(rename = "weatherDesc", default)]
    weather_desc: Vec<TextValue>,
}

impl CurrentCondition {
    fn description(&self) -> Option<&str> {
        first_text_value(&self.weather_desc)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct NearestArea {
    #[serde(rename = "areaName", default)]
    area_name: Vec<TextValue>,
    #[serde(default)]
    country: Vec<TextValue>,
    #[serde(default)]
    region: Vec<TextValue>,
}

impl NearestArea {
    fn display_name(&self) -> Option<&str> {
        first_text_value(&self.area_name)
            .or_else(|| first_text_value(&self.region))
            .or_else(|| first_text_value(&self.country))
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ForecastDay {
    #[serde(default)]
    date: String,
    #[serde(rename = "avgtempC", default)]
    avg_temp_c: String,
    #[serde(rename = "maxtempC", default)]
    max_temp_c: String,
    #[serde(rename = "mintempC", default)]
    min_temp_c: String,
    #[serde(default)]
    hourly: Vec<HourlyForecast>,
}

impl ForecastDay {
    fn summary(&self) -> String {
        let hourly = self
            .hourly
            .iter()
            .find_map(HourlyForecast::summary)
            .unwrap_or_else(|| "unknown".to_owned());

        format!(
            "{}: {}-{}°C, avg {}°C, {}",
            self.date, self.min_temp_c, self.max_temp_c, self.avg_temp_c, hourly
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
struct HourlyForecast {
    #[serde(default)]
    time: String,
    #[serde(rename = "tempC", default)]
    temp_c: String,
    #[serde(rename = "chanceofrain", default)]
    chance_of_rain: String,
    #[serde(rename = "weatherDesc", default)]
    weather_desc: Vec<TextValue>,
}

impl HourlyForecast {
    fn summary(&self) -> Option<String> {
        let description = first_text_value(&self.weather_desc)?;
        Some(format!(
            "{} at {}, {}°C, rain {}%",
            description, self.time, self.temp_c, self.chance_of_rain
        ))
    }
}

#[derive(Debug, Clone, Deserialize)]
struct TextValue {
    value: String,
}

fn first_text_value(values: &[TextValue]) -> Option<&str> {
    values.first().map(|item| item.value.as_str())
}

pub async fn get_weather(city: &str) -> Result<String, Error> {
    let url = format!("https://wttr.in/{}?format=j1", city);

    let client = reqwest::Client::new();

    let response = client.get(&url).send().await?;

    response.error_for_status_ref()?;

    let weather = response.json::<WeatherResponse>().await?;

    Ok(weather.summary(city))
}

#[derive(Debug, Clone, Deserialize)]
pub struct WeatherArg {
    city: String,
}

#[derive(Serialize, Deserialize)]
pub struct Weather;

impl Tool for Weather {
    const NAME: &'static str = "get_weather";
    type Error = Error;
    type Args = WeatherArg;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_weather".to_string(),
            description: "Get the weather for a given city".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "city": {
                        "type": "string",
                        "description": "The city to get the weather for"
                    }
                },
                "required": ["city"]
            }),
        }
    }

    async fn call(&self, args: WeatherArg) -> Result<String, Error> {
        get_weather(&args.city).await
    }
}
