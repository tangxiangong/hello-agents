use crate::Error;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tavily::Tavily;

#[derive(Debug, Clone, Deserialize)]
pub struct SearchAttractionArg {
    city: String,
    weather: String,
}

#[derive(Serialize, Deserialize)]
pub struct SearchAttraction {
    api_key: String,
}

impl SearchAttraction {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }

    async fn query(&self, args: &SearchAttractionArg) -> Result<String, Error> {
        let tavily = Tavily::builder(&self.api_key)
            .timeout(Duration::from_secs(60))
            .max_retries(5)
            .build()?;
        let query = format!(
            "'{}' 在 '{}'天气下最值得去的旅游景点推荐及推荐",
            args.city, args.weather
        );
        let results = tavily.search(query).await?;

        Ok(format!("{:?}", results))
    }
}

impl Tool for SearchAttraction {
    const NAME: &'static str = "search_attraction";
    type Error = Error;
    type Args = SearchAttractionArg;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "search_attraction".to_string(),
            description: "Search for attractions in a given city under a given weather condition"
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "city": {
                        "type": "string",
                        "description": "The city to search for attractions"
                    },
                    "weather": {
                        "type": "string",
                        "description": "The weather condition to search for"
                    },

                },
                "required": ["city", "weather"]
            }),
        }
    }

    async fn call(&self, args: SearchAttractionArg) -> Result<String, Error> {
        self.query(&args).await
    }
}
