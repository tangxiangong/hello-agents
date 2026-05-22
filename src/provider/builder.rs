//! Builder of Provider

use crate::{Completion, Error, Model, Provider, Result};
use rig::{
    agent::{Agent as RigAgent, AgentBuilder},
    client::CompletionClient,
    completion::CompletionModel,
    providers::{anthropic, deepseek, gemini, ollama, openai},
    tool::{Tool, ToolDyn},
};

/// OpenAI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiApi {
    Responses,
    Completions,
}

/// Anthropic API 版本头
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnthropicVersion {
    V2023_01_01,
    V2023_06_01,
    Latest,
}

impl AnthropicVersion {
    fn as_str(&self) -> &'static str {
        match self {
            AnthropicVersion::V2023_01_01 => "2023-01-01",
            AnthropicVersion::V2023_06_01 => "2023-06-01",
            AnthropicVersion::Latest => "2023-06-01",
        }
    }
}

/// 统一的 agent builder。公共方法对所有 provider 有效;
/// 专属方法仅对对应 provider 有效,用错 provider 时 `build()` 报
/// [`Error::InvalidProviderOption`]。
pub struct ProviderBuilder {
    provider: Provider,
    api_key: Option<String>,
    base_url: Option<String>,
    model: Option<Model>,
    preamble: Option<String>,
    temperature: Option<f64>,
    max_tokens: Option<u64>,
    tools: Vec<Box<dyn ToolDyn>>,
    openai_api: Option<OpenAiApi>,
    anthropic_version: Option<AnthropicVersion>,
}

impl ProviderBuilder {
    /// 新建一个针对 `provider` 的 builder。通常经 `Provider::builder` 调用。
    pub fn new(provider: Provider) -> Self {
        Self {
            provider,
            api_key: None,
            base_url: None,
            model: None,
            preamble: None,
            temperature: None,
            max_tokens: None,
            tools: Vec::new(),
            openai_api: None,
            anthropic_version: None,
        }
    }

    /// 设定模型;各 provider 子枚举均 `impl Into<Model>`。
    pub fn model(mut self, model: impl Into<Model>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// 系统提示词。
    pub fn preamble(mut self, preamble: impl Into<String>) -> Self {
        self.preamble = Some(preamble.into());
        self
    }

    /// 采样温度。
    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// 最大输出 token 数。
    pub fn max_tokens(mut self, max_tokens: u64) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// 显式提供 API key,优先级高于 `Config` 与环境变量。
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// 注册一个工具。
    pub fn tool(mut self, tool: impl Tool + 'static) -> Self {
        self.tools.push(Box::new(tool));
        self
    }

    /// OpenAI 专属:使用 Responses API(OpenAI 默认即此)。
    pub fn responses_api(mut self) -> Self {
        self.openai_api = Some(OpenAiApi::Responses);
        self
    }

    /// OpenAI 专属:使用 Chat Completions API。
    pub fn completions_api(mut self) -> Self {
        self.openai_api = Some(OpenAiApi::Completions);
        self
    }

    /// Anthropic 专属:设定 API 版本头。
    pub fn anthropic_version(mut self, version: AnthropicVersion) -> Self {
        self.anthropic_version = Some(version);
        self
    }

    /// Ollama / Compatible 专属:指定端点 base URL。
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// 校验配置并构造统一的 [`Agent`]。
    pub fn build(self) -> Result<Completion> {
        let ProviderBuilder {
            provider,
            api_key,
            base_url,
            model,
            preamble,
            temperature,
            max_tokens,
            tools,
            openai_api,
            anthropic_version,
        } = self;

        let model = model.ok_or(Error::MissingModel)?;
        if model.provider() != provider {
            return Err(Error::ModelProviderMismatch {
                model: model.provider(),
                provider,
            });
        }
        if openai_api.is_some() && provider != Provider::OpenAI {
            return Err(Error::InvalidProviderOption {
                provider,
                option: "openai_api",
            });
        }
        if anthropic_version.is_some() && provider != Provider::Anthropic {
            return Err(Error::InvalidProviderOption {
                provider,
                option: "anthropic_version",
            });
        }

        let agent = match model {
            Model::OpenAI(m) => {
                let key = resolve_key(provider, &api_key)?;
                match openai_api.unwrap_or(OpenAiApi::Responses) {
                    OpenAiApi::Responses => {
                        let client =
                            openai::Client::new(key).map_err(|e| Error::Rig(e.to_string()))?;
                        Completion::OpenAIResponses(finish(
                            client.agent(m.id()),
                            preamble,
                            temperature,
                            max_tokens,
                            tools,
                        ))
                    }
                    OpenAiApi::Completions => {
                        let client = openai::Client::new(key)
                            .map_err(|e| Error::Rig(e.to_string()))?
                            .completions_api();
                        Completion::OpenAICompletions(finish(
                            client.agent(m.id()),
                            preamble,
                            temperature,
                            max_tokens,
                            tools,
                        ))
                    }
                }
            }
            Model::Anthropic(m) => {
                let key = resolve_key(provider, &api_key)?;
                let version = anthropic_version.unwrap_or(AnthropicVersion::Latest);
                let client = anthropic::Client::builder()
                    .api_key(key)
                    .anthropic_version(version.as_str())
                    .build()
                    .map_err(|e| Error::Rig(e.to_string()))?;
                Completion::Anthropic(finish(
                    client.agent(m.id()),
                    preamble,
                    temperature,
                    max_tokens,
                    tools,
                ))
            }
            Model::Gemini(m) => {
                let key = resolve_key(provider, &api_key)?;
                let client = gemini::Client::new(key).map_err(|e| Error::Rig(e.to_string()))?;
                Completion::Gemini(finish(
                    client.agent(m.id()),
                    preamble,
                    temperature,
                    max_tokens,
                    tools,
                ))
            }
            Model::DeepSeek(m) => {
                let key = resolve_key(provider, &api_key)?;
                let client = deepseek::Client::new(key).map_err(|e| Error::Rig(e.to_string()))?;
                Completion::DeepSeek(finish(
                    client.agent(m.id()),
                    preamble,
                    temperature,
                    max_tokens,
                    tools,
                ))
            }
            Model::Ollama(m) => {
                let url = base_url
                    .clone()
                    .unwrap_or_else(|| "http://localhost:11434".to_string());
                let client = ollama::Client::builder()
                    .api_key(ollama::OllamaApiKey::default())
                    .base_url(&url)
                    .build()
                    .map_err(|e| Error::Rig(e.to_string()))?;
                Completion::Ollama(finish(
                    client.agent(m.id()),
                    preamble,
                    temperature,
                    max_tokens,
                    tools,
                ))
            }
            Model::Compatible(id) => {
                let key = resolve_key(provider, &api_key)?;
                let url = base_url.clone().ok_or(Error::MissingBaseURL)?;
                let client = openai::Client::builder()
                    .api_key(key)
                    .base_url(&url)
                    .build()
                    .map_err(|e| Error::Rig(e.to_string()))?
                    .completions_api();
                Completion::OpenAICompletions(finish(
                    client.agent(id),
                    preamble,
                    temperature,
                    max_tokens,
                    tools,
                ))
            }
        };
        Ok(agent)
    }
}

fn resolve_key(provider: Provider, api_key: &Option<String>) -> Result<String> {
    if let Some(key) = api_key
        && !key.is_empty()
    {
        return Ok(key.clone());
    }
    if let Some(var) = provider.api_key_env()
        && let Ok(value) = std::env::var(var)
        && !value.is_empty()
    {
        return Ok(value);
    }
    Err(Error::MissingApiKey(provider))
}

/// 把公共配置应用到 rig 的 [`AgentBuilder`] 并构建。
fn finish<M>(
    mut builder: AgentBuilder<M>,
    preamble: Option<String>,
    temperature: Option<f64>,
    max_tokens: Option<u64>,
    tools: Vec<Box<dyn ToolDyn>>,
) -> RigAgent<M>
where
    M: CompletionModel + 'static,
{
    if let Some(p) = preamble {
        builder = builder.preamble(&p);
    }
    if let Some(t) = temperature {
        builder = builder.temperature(t);
    }
    if let Some(mt) = max_tokens {
        builder = builder.max_tokens(mt);
    }
    builder.tools(tools).build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnthropicModel, Model, OpenAIModel, Provider};

    #[test]
    fn build_without_model_errors() {
        let err = ProviderBuilder::new(Provider::OpenAI)
            .build()
            .err()
            .expect("should be an error");
        assert!(matches!(err, crate::Error::MissingModel));
    }

    #[test]
    fn build_with_mismatched_model_errors() {
        let err = ProviderBuilder::new(Provider::OpenAI)
            .model(AnthropicModel::Opus_4_7)
            .build()
            .err()
            .expect("should be an error");
        assert!(matches!(
            err,
            crate::Error::ModelProviderMismatch {
                model: Provider::Anthropic,
                provider: Provider::OpenAI,
            }
        ));
    }

    #[test]
    fn anthropic_option_on_openai_errors() {
        let err = ProviderBuilder::new(Provider::OpenAI)
            .model(OpenAIModel::GPT_5)
            .anthropic_version(AnthropicVersion::Latest)
            .build()
            .err()
            .expect("should be an error");
        assert!(matches!(
            err,
            crate::Error::InvalidProviderOption {
                provider: Provider::OpenAI,
                option: "anthropic_version",
            }
        ));
    }

    #[test]
    fn build_openai_responses_agent() {
        let agent = ProviderBuilder::new(Provider::OpenAI)
            .api_key("fake-key")
            .model(OpenAIModel::GPT_5)
            .preamble("hi")
            .build()
            .expect("build should succeed with a fake key");
        assert!(matches!(agent, crate::Completion::OpenAIResponses(_)));
    }

    #[test]
    fn build_openai_completions_agent() {
        let agent = ProviderBuilder::new(Provider::OpenAI)
            .api_key("fake-key")
            .model(OpenAIModel::GPT_4o)
            .completions_api()
            .build()
            .expect("build should succeed");
        assert!(matches!(agent, crate::Completion::OpenAICompletions(_)));
    }

    #[test]
    fn build_compatible_agent() {
        let agent = ProviderBuilder::new(Provider::Compatible)
            .api_key("fake-key")
            .base_url("https://example.com/v1")
            .model(Model::Compatible("some-model".into()))
            .build()
            .expect("build should succeed");
        assert!(matches!(agent, crate::Completion::OpenAICompletions(_)));
    }
}
