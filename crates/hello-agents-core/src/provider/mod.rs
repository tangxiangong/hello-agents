//! Provider / Model / Agent 统一封装层

mod builder;
pub use builder::*;
mod model;
pub use model::*;

use crate::{Error, Result};
use futures_util::StreamExt;
use rig::{
    agent::{Agent as RigAgent, MultiTurnStreamItem},
    client::CompletionClient,
    completion::{CompletionModel as RigCompletionModel, GetTokenUsage, Message, Prompt},
    providers::{anthropic, deepseek, gemini, ollama, openai},
    streaming::{StreamedAssistantContent, StreamingPrompt},
    wasm_compat::WasmCompatSend,
};
use std::{io::Write, str::FromStr};

type ProviderCompletionModel<C> = <C as CompletionClient>::CompletionModel;

/// 跨请求连续对话历史。调用方需要在同一个会话中复用它。
pub type ChatHistory = Vec<Message>;

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

    pub fn agent(&self, model: model::Model) -> Result<Agent> {
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

pub enum Agent {
    OpenAIResponses(RigAgent<ProviderCompletionModel<openai::Client>>),
    OpenAICompletions(RigAgent<ProviderCompletionModel<openai::CompletionsClient>>),
    Anthropic(RigAgent<ProviderCompletionModel<anthropic::Client>>),
    Gemini(RigAgent<ProviderCompletionModel<gemini::Client>>),
    DeepSeek(RigAgent<ProviderCompletionModel<deepseek::Client>>),
    Ollama(RigAgent<ProviderCompletionModel<ollama::Client>>),
}

impl Agent {
    /// 单轮提示:把回复作为字符串返回。
    pub async fn prompt(&self, input: &str) -> Result<String> {
        let res = match self {
            Agent::OpenAIResponses(a) => a.prompt(input).await,
            Agent::OpenAICompletions(a) => a.prompt(input).await,
            Agent::Anthropic(a) => a.prompt(input).await,
            Agent::Gemini(a) => a.prompt(input).await,
            Agent::DeepSeek(a) => a.prompt(input).await,
            Agent::Ollama(a) => a.prompt(input).await,
        };
        res.map_err(|e| Error::Rig(e.to_string()))
    }

    /// 多轮提示:允许最多 `max_turns` 轮工具调用循环。
    pub async fn multi_turn(&self, input: &str, max_turns: usize) -> crate::Result<String> {
        let res = match self {
            Agent::OpenAIResponses(a) => a.prompt(input).max_turns(max_turns).await,
            Agent::OpenAICompletions(a) => a.prompt(input).max_turns(max_turns).await,
            Agent::Anthropic(a) => a.prompt(input).max_turns(max_turns).await,
            Agent::Gemini(a) => a.prompt(input).max_turns(max_turns).await,
            Agent::DeepSeek(a) => a.prompt(input).max_turns(max_turns).await,
            Agent::Ollama(a) => a.prompt(input).max_turns(max_turns).await,
        };
        res.map_err(|e| Error::Rig(e.to_string()))
    }

    /// 连续对话:调用方复用同一个 `history`,即可让后续输入带上之前上下文。
    pub async fn chat(
        &self,
        input: &str,
        max_turns: usize,
        history: &mut ChatHistory,
    ) -> Result<String> {
        match self {
            Agent::OpenAIResponses(a) => chat_agent(a, input, max_turns, history).await,
            Agent::OpenAICompletions(a) => chat_agent(a, input, max_turns, history).await,
            Agent::Anthropic(a) => chat_agent(a, input, max_turns, history).await,
            Agent::Gemini(a) => chat_agent(a, input, max_turns, history).await,
            Agent::DeepSeek(a) => chat_agent(a, input, max_turns, history).await,
            Agent::Ollama(a) => chat_agent(a, input, max_turns, history).await,
        }
    }

    /// 流式提示:每收到一段文本增量就调用 `on_text`,并在结束时返回最终回复。
    pub async fn stream<F>(&self, input: &str, max_turns: usize, mut on_text: F) -> Result<String>
    where
        F: FnMut(&str) -> Result<()>,
    {
        match self {
            Agent::OpenAIResponses(a) => stream_agent(a, input, max_turns, &mut on_text).await,
            Agent::OpenAICompletions(a) => stream_agent(a, input, max_turns, &mut on_text).await,
            Agent::Anthropic(a) => stream_agent(a, input, max_turns, &mut on_text).await,
            Agent::Gemini(a) => stream_agent(a, input, max_turns, &mut on_text).await,
            Agent::DeepSeek(a) => stream_agent(a, input, max_turns, &mut on_text).await,
            Agent::Ollama(a) => stream_agent(a, input, max_turns, &mut on_text).await,
        }
    }

    /// 连续对话的流式提示:复用并自动更新 `history`。
    pub async fn stream_chat<F>(
        &self,
        input: &str,
        max_turns: usize,
        history: &mut ChatHistory,
        mut on_text: F,
    ) -> Result<String>
    where
        F: FnMut(&str) -> Result<()>,
    {
        match self {
            Agent::OpenAIResponses(a) => {
                stream_chat_agent(a, input, max_turns, history, &mut on_text).await
            }
            Agent::OpenAICompletions(a) => {
                stream_chat_agent(a, input, max_turns, history, &mut on_text).await
            }
            Agent::Anthropic(a) => {
                stream_chat_agent(a, input, max_turns, history, &mut on_text).await
            }
            Agent::Gemini(a) => stream_chat_agent(a, input, max_turns, history, &mut on_text).await,
            Agent::DeepSeek(a) => {
                stream_chat_agent(a, input, max_turns, history, &mut on_text).await
            }
            Agent::Ollama(a) => stream_chat_agent(a, input, max_turns, history, &mut on_text).await,
        }
    }

    /// 流式提示并把文本增量直接写到标准输出。
    pub async fn stream_to_stdout(&self, input: &str, max_turns: usize) -> Result<String> {
        let mut stdout = std::io::stdout();
        let response = self
            .stream(input, max_turns, |chunk| {
                print!("{chunk}");
                stdout.flush().map_err(Error::from)
            })
            .await?;
        println!();
        Ok(response)
    }

    /// 连续对话的流式提示并把文本增量直接写到标准输出。
    pub async fn stream_chat_to_stdout(
        &self,
        input: &str,
        max_turns: usize,
        history: &mut ChatHistory,
    ) -> Result<String> {
        let mut stdout = std::io::stdout();
        let response = self
            .stream_chat(input, max_turns, history, |chunk| {
                print!("{chunk}");
                stdout.flush().map_err(Error::from)
            })
            .await?;
        println!();
        Ok(response)
    }
}

async fn chat_agent<M>(
    agent: &RigAgent<M>,
    input: &str,
    max_turns: usize,
    history: &mut ChatHistory,
) -> Result<String>
where
    M: RigCompletionModel + 'static,
{
    let response = agent
        .prompt(input)
        .with_history(history.clone())
        .max_turns(max_turns)
        .extended_details()
        .await
        .map_err(|e| Error::Rig(e.to_string()))?;

    if let Some(messages) = response.messages {
        history.extend(messages);
    }

    Ok(response.output)
}

async fn stream_agent<M, F>(
    agent: &RigAgent<M>,
    input: &str,
    max_turns: usize,
    on_text: &mut F,
) -> Result<String>
where
    M: RigCompletionModel + 'static,
    M::StreamingResponse: GetTokenUsage + WasmCompatSend,
    F: FnMut(&str) -> Result<()>,
{
    let mut stream = agent.stream_prompt(input).multi_turn(max_turns).await;
    collect_stream(&mut stream, on_text, None).await
}

async fn stream_chat_agent<M, F>(
    agent: &RigAgent<M>,
    input: &str,
    max_turns: usize,
    history: &mut ChatHistory,
    on_text: &mut F,
) -> Result<String>
where
    M: RigCompletionModel + 'static,
    M::StreamingResponse: GetTokenUsage + WasmCompatSend,
    F: FnMut(&str) -> Result<()>,
{
    let mut stream = agent
        .stream_prompt(input)
        .with_history(history.clone())
        .multi_turn(max_turns)
        .await;
    collect_stream(&mut stream, on_text, Some(history)).await
}

async fn collect_stream<S, R, F>(
    stream: &mut S,
    on_text: &mut F,
    mut history: Option<&mut ChatHistory>,
) -> Result<String>
where
    S: futures_util::Stream<
            Item = std::result::Result<MultiTurnStreamItem<R>, rig::agent::StreamingError>,
        > + Unpin,
    F: FnMut(&str) -> Result<()>,
{
    let mut streamed_text = String::new();
    let mut final_response = None;

    while let Some(item) = stream.next().await {
        match item.map_err(|e| Error::Rig(e.to_string()))? {
            MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text)) => {
                on_text(&text.text)?;
                streamed_text.push_str(&text.text);
            }
            MultiTurnStreamItem::FinalResponse(response) => {
                if let Some(history) = history.as_deref_mut()
                    && let Some(messages) = response.history()
                {
                    history.extend_from_slice(messages);
                }
                final_response = Some(response.response().to_owned());
            }
            _ => {}
        }
    }

    Ok(final_response
        .filter(|response| !response.is_empty())
        .unwrap_or(streamed_text))
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
            std::env::set_var("OPENAI_API_KEY", "fake-key");
        }
        let agent = Provider::OpenAI
            .agent(Model::OpenAI(OpenAIModel::GPT_5))
            .expect("shortcut build should succeed");
        assert!(matches!(agent, crate::Agent::OpenAIResponses(_)));
    }

    #[test]
    fn provider_builder_entrypoint_builds() {
        use crate::AnthropicModel;
        unsafe {
            std::env::set_var("ANTHROPIC_API_KEY", "fake-key");
        }
        let agent = Provider::Anthropic
            .builder()
            .model(AnthropicModel::Sonnet_4_6)
            .max_tokens(1024)
            .build()
            .expect("builder entrypoint should succeed");
        assert!(matches!(agent, crate::Agent::Anthropic(_)));
    }
}
