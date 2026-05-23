use crate::Provider;

pub struct Config {
    tavily_api_key: Option<String>,
    serpapi_api_key: Option<String>,
    provider: Provider,
}

impl Config {
    pub fn new(provider: Provider) -> Self {
        Self {
            tavily_api_key: std::env::var("TAVILY_API_KEY").ok(),
            serpapi_api_key: std::env::var("SERPAPI_API_KEY").ok(),
            provider,
        }
    }

    pub fn from_model(model: crate::Model) -> Self {
        Self::new(model.provider())
    }

    pub fn provider(&self) -> Provider {
        self.provider
    }

    pub(crate) fn tavily_api_key(&self) -> Option<&str> {
        self.tavily_api_key.as_deref()
    }

    pub(crate) fn serpapi_api_key(&self) -> Option<&str> {
        self.serpapi_api_key.as_deref()
    }
}
