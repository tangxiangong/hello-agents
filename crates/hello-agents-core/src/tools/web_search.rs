use crate::Error;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tavily::Tavily;

#[derive(Debug, Clone, Deserialize)]
pub struct WebSearchArg {
    query: String,
}

#[derive(Serialize, Deserialize)]
pub struct WebSearch {
    api_key: String,
}

impl WebSearch {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }

    async fn query(&self, query: &str) -> Result<String, Error> {
        let tavily = Tavily::builder(&self.api_key)
            .timeout(Duration::from_secs(60))
            .max_retries(5)
            .build()?;
        let results = tavily.search(query).await?;

        Ok(format!("{:?}", results))
    }
}

impl Tool for WebSearch {
    const NAME: &'static str = "web_search";
    type Error = Error;
    type Args = WebSearchArg;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "web_search".to_string(),
            description: "Search the web for a given query".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The query to search for"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: WebSearchArg) -> Result<String, Error> {
        self.query(&args.query).await
    }
}
