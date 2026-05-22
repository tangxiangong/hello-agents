use crate::Provider;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("API key not found")]
    APIKeyNotFound,

    #[error("unknown provider: {0}")]
    UnknownProvider(String),

    #[error("missing API key for provider {0:?}")]
    MissingApiKey(Provider),

    #[error("no model was set on the builder")]
    MissingModel,

    #[error("model belongs to {model:?} but builder is for {provider:?}")]
    ModelProviderMismatch { model: Provider, provider: Provider },

    #[error("option `{option}` is not valid for provider {provider:?}")]
    InvalidProviderOption {
        provider: Provider,
        option: &'static str,
    },

    #[error("rig error: {0}")]
    Rig(String),

    #[error("I/O error: {0}")]
    Io(String),

    #[error("missing base URL")]
    MissingBaseURL,

    #[error("reqwest error: {0}")]
    RequestError(String),

    #[error("tavily error: {0}")]
    TavilyError(String),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::RequestError(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

impl From<tavily::TavilyError> for Error {
    fn from(e: tavily::TavilyError) -> Self {
        Error::TavilyError(e.to_string())
    }
}

/// 本 crate 统一的 `Result`。
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::Error;
    use crate::Provider;

    #[test]
    fn error_messages_render() {
        assert_eq!(
            Error::UnknownProvider("foo".into()).to_string(),
            "unknown provider: foo"
        );
        assert_eq!(
            Error::MissingApiKey(Provider::OpenAI).to_string(),
            "missing API key for provider OpenAI"
        );
        assert_eq!(
            Error::MissingModel.to_string(),
            "no model was set on the builder"
        );
        assert_eq!(
            Error::ModelProviderMismatch {
                model: Provider::OpenAI,
                provider: Provider::Anthropic,
            }
            .to_string(),
            "model belongs to OpenAI but builder is for Anthropic"
        );
        assert_eq!(
            Error::InvalidProviderOption {
                provider: Provider::Anthropic,
                option: "openai_api",
            }
            .to_string(),
            "option `openai_api` is not valid for provider Anthropic"
        );
        assert_eq!(Error::Rig("boom".into()).to_string(), "rig error: boom");
        assert_eq!(
            Error::Io("broken pipe".into()).to_string(),
            "I/O error: broken pipe"
        );
    }
}
