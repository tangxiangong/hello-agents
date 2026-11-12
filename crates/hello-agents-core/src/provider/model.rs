//! 模型枚举:`Model` 及各 provider 子枚举。后续 Task 填充。

use crate::Provider;

/// LLM 模型
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Model {
    /// OpenAI
    OpenAI(OpenAIModel),
    /// Anthropic
    Anthropic(AnthropicModel),
    /// Gemini
    Gemini(GeminiModel),
    /// DeepSeek
    DeepSeek(DeepSeekModel),
    /// Ollama
    Ollama(OllamaModel),
    /// 任意 OpenAI 兼容端点的原始 model id
    Compatible(String),
}

impl Model {
    /// 模型 id
    pub fn id(&self) -> &str {
        match self {
            Model::OpenAI(m) => m.id(),
            Model::Anthropic(m) => m.id(),
            Model::Gemini(m) => m.id(),
            Model::DeepSeek(m) => m.id(),
            Model::Ollama(m) => m.id(),
            Model::Compatible(id) => id.as_str(),
        }
    }

    /// 该模型所属的 provider。
    pub fn provider(&self) -> Provider {
        match self {
            Model::OpenAI(_) => Provider::OpenAI,
            Model::Anthropic(_) => Provider::Anthropic,
            Model::Gemini(_) => Provider::Gemini,
            Model::DeepSeek(_) => Provider::DeepSeek,
            Model::Ollama(_) => Provider::Ollama,
            Model::Compatible(_) => Provider::Compatible,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAIModel {
    GPT_5_5,
    GPT_5_2,
    GPT_5_1,
    GPT_5,
    GPT_5_Mini,
    GPT_5_Nano,
    GPT_4_1,
    GPT_4_1_Mini,
    GPT_4o,
    GPT_4o_Mini,
    O3,
    O3_Mini,
    O4_Mini,
}

impl OpenAIModel {
    pub fn id(&self) -> &'static str {
        match self {
            OpenAIModel::GPT_5_5 => "gpt-5.5",
            OpenAIModel::GPT_5_2 => "gpt-5.2",
            OpenAIModel::GPT_5_1 => "gpt-5.1",
            OpenAIModel::GPT_5 => "gpt-5",
            OpenAIModel::GPT_5_Mini => "gpt-5-mini",
            OpenAIModel::GPT_5_Nano => "gpt-5-nano",
            OpenAIModel::GPT_4_1 => "gpt-4.1",
            OpenAIModel::GPT_4_1_Mini => "gpt-4.1-mini",
            OpenAIModel::GPT_4o => "gpt-4o",
            OpenAIModel::GPT_4o_Mini => "gpt-4o-mini",
            OpenAIModel::O3 => "o3",
            OpenAIModel::O3_Mini => "o3-mini",
            OpenAIModel::O4_Mini => "o4-mini",
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnthropicModel {
    Opus_4_7,
    Opus_4_6,
    Sonnet_4_6,
    Haiku_4_5,
}

impl AnthropicModel {
    pub fn id(&self) -> &'static str {
        match self {
            AnthropicModel::Opus_4_7 => "claude-opus-4-7",
            AnthropicModel::Opus_4_6 => "claude-opus-4-6",
            AnthropicModel::Sonnet_4_6 => "claude-sonnet-4-6",
            AnthropicModel::Haiku_4_5 => "claude-haiku-4-5",
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeminiModel {
    Gemini_3_Flash_Preview,
    Gemini_2_5_Flash,
    Gemini_2_0_Flash,
    Gemini_2_0_Flash_Lite,
}

impl GeminiModel {
    pub fn id(&self) -> &'static str {
        match self {
            GeminiModel::Gemini_3_Flash_Preview => "gemini-3-flash-preview",
            GeminiModel::Gemini_2_5_Flash => "gemini-2.5-flash",
            GeminiModel::Gemini_2_0_Flash => "gemini-2.0-flash",
            GeminiModel::Gemini_2_0_Flash_Lite => "gemini-2.0-flash-lite",
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeepSeekModel {
    Chat,
    Reasoner,
    V4_Flash,
    V4_Pro,
}

impl DeepSeekModel {
    pub fn id(&self) -> &'static str {
        match self {
            DeepSeekModel::Chat => "deepseek-chat",
            DeepSeekModel::Reasoner => "deepseek-reasoner",
            DeepSeekModel::V4_Flash => "deepseek-v4-flash",
            DeepSeekModel::V4_Pro => "deepseek-v4-pro",
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OllamaModel {
    Llama_3_2,
    Mistral,
    Llava,
    Custom(String),
}

impl OllamaModel {
    pub fn id(&self) -> &str {
        match self {
            OllamaModel::Llama_3_2 => "llama3.2",
            OllamaModel::Mistral => "mistral",
            OllamaModel::Llava => "llava",
            OllamaModel::Custom(id) => id.as_str(),
        }
    }
}

impl From<OpenAIModel> for Model {
    fn from(m: OpenAIModel) -> Self {
        Model::OpenAI(m)
    }
}

impl From<AnthropicModel> for Model {
    fn from(m: AnthropicModel) -> Self {
        Model::Anthropic(m)
    }
}

impl From<GeminiModel> for Model {
    fn from(m: GeminiModel) -> Self {
        Model::Gemini(m)
    }
}

impl From<DeepSeekModel> for Model {
    fn from(m: DeepSeekModel) -> Self {
        Model::DeepSeek(m)
    }
}

impl From<OllamaModel> for Model {
    fn from(m: OllamaModel) -> Self {
        Model::Ollama(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Provider;

    #[test]
    fn submodel_ids_match_expected_strings() {
        assert_eq!(OpenAIModel::GPT_5.id(), "gpt-5");
        assert_eq!(OpenAIModel::GPT_5_5.id(), "gpt-5.5");
        assert_eq!(OpenAIModel::O3_Mini.id(), "o3-mini");
        assert_eq!(AnthropicModel::Opus_4_7.id(), "claude-opus-4-7");
        assert_eq!(GeminiModel::Gemini_2_5_Flash.id(), "gemini-2.5-flash");
        assert_eq!(DeepSeekModel::Reasoner.id(), "deepseek-reasoner");
        assert_eq!(OllamaModel::Llama_3_2.id(), "llama3.2");
        assert_eq!(OllamaModel::Custom("phi4".into()).id(), "phi4");
    }

    #[test]
    fn model_id_delegates_to_submodel() {
        assert_eq!(Model::OpenAI(OpenAIModel::GPT_4o).id(), "gpt-4o");
        assert_eq!(Model::Compatible("my-model".into()).id(), "my-model");
    }

    #[test]
    fn model_provider_reports_owner() {
        assert_eq!(
            Model::OpenAI(OpenAIModel::GPT_5).provider(),
            Provider::OpenAI
        );
        assert_eq!(
            Model::Anthropic(AnthropicModel::Haiku_4_5).provider(),
            Provider::Anthropic
        );
        assert_eq!(
            Model::Gemini(GeminiModel::Gemini_2_0_Flash).provider(),
            Provider::Gemini
        );
        assert_eq!(
            Model::DeepSeek(DeepSeekModel::Chat).provider(),
            Provider::DeepSeek
        );
        assert_eq!(
            Model::Ollama(OllamaModel::Mistral).provider(),
            Provider::Ollama
        );
        assert_eq!(
            Model::Compatible("x".into()).provider(),
            Provider::Compatible
        );
    }

    #[test]
    fn submodels_convert_into_model() {
        let m: Model = OpenAIModel::GPT_5.into();
        assert_eq!(m, Model::OpenAI(OpenAIModel::GPT_5));
        let m: Model = AnthropicModel::Sonnet_4_6.into();
        assert_eq!(m, Model::Anthropic(AnthropicModel::Sonnet_4_6));
    }
}
