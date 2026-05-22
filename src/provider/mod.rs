//! Provider / Model / Completion 统一封装层

mod builder;
pub use builder::*;
mod model;
pub use model::*;

use crate::{Error, Result};
use rig::{
    agent::Agent as RigAgent,
    client::CompletionClient,
    completion::Prompt,
    providers::{anthropic, deepseek, gemini, ollama, openai},
};
use std::str::FromStr;

type CompletionModel<C> = <C as CompletionClient>::CompletionModel;

/// LLM Provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provider {
    OpenAI,
    Anthropic,
    Gemini,
    DeepSeek,
    Ollama,
    Compatible,
}

impl Provider {
    /// 该 provider 的默认 API key 环境变量名; Ollama 无需 key,返回 `None`
    pub fn api_key_env(&self) -> Option<&'static str> {
        match self {
            Provider::OpenAI => Some("OPENAI_API_KEY"),
            Provider::Anthropic => Some("ANTHROPIC_API_KEY"),
            Provider::Gemini => Some("GEMINI_API_KEY"),
            Provider::DeepSeek => Some("DEEPSEEK_API_KEY"),
            Provider::Ollama => None,
            Provider::Compatible => Some("LLM_API_KEY"),
        }
    }

    pub fn builder(&self) -> ProviderBuilder {
        ProviderBuilder::new(*self)
    }

    pub fn agent(&self, model: model::Model) -> Result<Completion> {
        self.builder().model(model).build()
    }

    fn canonical_name(&self) -> &'static str {
        match self {
            Provider::OpenAI => "openai",
            Provider::Anthropic => "anthropic",
            Provider::Gemini => "gemini",
            Provider::DeepSeek => "deepseek",
            Provider::Ollama => "ollama",
            Provider::Compatible => "compatible",
        }
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.canonical_name())
    }
}

impl FromStr for Provider {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "openai" => Ok(Provider::OpenAI),
            "anthropic" => Ok(Provider::Anthropic),
            "gemini" => Ok(Provider::Gemini),
            "deepseek" => Ok(Provider::DeepSeek),
            "ollama" => Ok(Provider::Ollama),
            "compatible" => Ok(Provider::Compatible),
            _ => Err(Error::UnknownProvider(s.to_string())),
        }
    }
}

pub enum Completion {
    OpenAIResponses(RigAgent<CompletionModel<openai::Client>>),
    OpenAICompletions(RigAgent<CompletionModel<openai::CompletionsClient>>),
    Anthropic(RigAgent<CompletionModel<anthropic::Client>>),
    Gemini(RigAgent<CompletionModel<gemini::Client>>),
    DeepSeek(RigAgent<CompletionModel<deepseek::Client>>),
    Ollama(RigAgent<CompletionModel<ollama::Client>>),
}

impl Completion {
    /// 单轮提示:把回复作为字符串返回。
    pub async fn prompt(&self, input: &str) -> Result<String> {
        let res = match self {
            Completion::OpenAIResponses(a) => a.prompt(input).await,
            Completion::OpenAICompletions(a) => a.prompt(input).await,
            Completion::Anthropic(a) => a.prompt(input).await,
            Completion::Gemini(a) => a.prompt(input).await,
            Completion::DeepSeek(a) => a.prompt(input).await,
            Completion::Ollama(a) => a.prompt(input).await,
        };
        res.map_err(|e| Error::Rig(e.to_string()))
    }

    /// 多轮提示:允许最多 `max_turns` 轮工具调用循环。
    pub async fn multi_turn(&self, input: &str, max_turns: usize) -> crate::Result<String> {
        let res = match self {
            Completion::OpenAIResponses(a) => a.prompt(input).max_turns(max_turns).await,
            Completion::OpenAICompletions(a) => a.prompt(input).max_turns(max_turns).await,
            Completion::Anthropic(a) => a.prompt(input).max_turns(max_turns).await,
            Completion::Gemini(a) => a.prompt(input).max_turns(max_turns).await,
            Completion::DeepSeek(a) => a.prompt(input).max_turns(max_turns).await,
            Completion::Ollama(a) => a.prompt(input).max_turns(max_turns).await,
        };
        res.map_err(|e| Error::Rig(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::Provider;
    use std::str::FromStr;

    #[test]
    fn parses_provider_case_insensitively() {
        assert_eq!(Provider::from_str("openai").unwrap(), Provider::OpenAI);
        assert_eq!(
            Provider::from_str("Anthropic").unwrap(),
            Provider::Anthropic
        );
        assert_eq!(Provider::from_str("OLLAMA").unwrap(), Provider::Ollama);
        assert_eq!(
            Provider::from_str("compatible").unwrap(),
            Provider::Compatible
        );
    }

    #[test]
    fn rejects_unknown_provider() {
        let err = Provider::from_str("nope").unwrap_err();
        assert!(matches!(err, crate::Error::UnknownProvider(s) if s == "nope"));
    }

    #[test]
    fn display_roundtrips_with_fromstr() {
        for p in [
            Provider::OpenAI,
            Provider::Anthropic,
            Provider::Gemini,
            Provider::DeepSeek,
            Provider::Ollama,
            Provider::Compatible,
        ] {
            let parsed = Provider::from_str(&p.to_string()).unwrap();
            assert_eq!(parsed, p);
        }
    }

    #[test]
    fn api_key_env_known() {
        assert_eq!(Provider::OpenAI.api_key_env(), Some("OPENAI_API_KEY"));
        assert_eq!(Provider::Ollama.api_key_env(), None);
    }

    #[test]
    fn provider_agent_shortcut_builds() {
        use crate::{Model, OpenAIModel};
        unsafe {
            std::env::set_var("LLM_API_KEY", "fake-key");
        }
        let agent = Provider::OpenAI
            .agent(Model::OpenAI(OpenAIModel::GPT_5))
            .expect("shortcut build should succeed");
        assert!(matches!(agent, crate::Completion::OpenAIResponses(_)));
    }

    #[test]
    fn provider_builder_entrypoint_builds() {
        use crate::AnthropicModel;
        unsafe {
            std::env::set_var("LLM_API_KEY", "fake-key");
        }
        let agent = Provider::Anthropic
            .builder()
            .model(AnthropicModel::Sonnet_4_6)
            .max_tokens(1024)
            .build()
            .expect("builder entrypoint should succeed");
        assert!(matches!(agent, crate::Completion::Anthropic(_)));
    }
}
