use rig::{completion::ToolDefinition, tool::Tool};
use serde::Deserialize;
use serpapi::serpapi::Client;
use std::collections::HashMap;

use crate::{Error, Result};

pub struct GoogleSearch {
    api_key: String,
}

impl GoogleSearch {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }

    pub async fn search(&self, query: &str) -> Result<String> {
        let mut setting = HashMap::new();
        setting.insert("api_key".to_owned(), self.api_key.clone());
        setting.insert("engine".to_owned(), "google".to_owned());

        let client = Client::new(setting).map_err(|e| Error::SerpApiError(e.to_string()))?;

        let mut parameter = HashMap::<String, String>::new();
        parameter.insert("q".into(), query.into());
        parameter.insert("location".into(), "Austin, Texas, United States".into());
        parameter.insert("hl".into(), "en".into());
        parameter.insert("gl".into(), "us".into());
        parameter.insert("google_domain".into(), "google.com".into());

        let results = client
            .search(parameter)
            .await
            .map_err(|e| Error::SerpApiError(e.to_string()))?
            .to_string();

        Ok(results)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleSearchArg {
    query: String,
}

impl Tool for GoogleSearch {
    const NAME: &'static str = "google_search";
    type Error = Error;
    type Args = GoogleSearchArg;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "google_search".to_string(),
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

    async fn call(&self, args: GoogleSearchArg) -> Result<String, Error> {
        self.search(&args.query).await
    }
}
