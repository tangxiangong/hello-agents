# 统一 rig 的 Provider 与 Model —— 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 `hello-agents` 内为 `rig-core` 0.37 加一层薄封装,把多个 provider 统一成 `Provider` 单元枚举、把模型常量统一成嵌套的 `Model` 枚举,并用枚举包裹的 `Agent` 与单结构体 `ProviderBuilder` 提供统一构造入口,同时保留各 provider 专属特性。

**Architecture:** `src/provider.rs` 拆成 `src/provider/` 目录(`mod.rs` / `model.rs` / `builder.rs`)。`Provider` 是纯单元枚举(身份标识);`Model` 是按 provider 分的嵌套枚举;`Agent` 是包裹各 provider `rig_core::agent::Agent` 的 7 变体枚举,全静态分发;`ProviderBuilder` 是单结构体,`build()` 内一处 `match` 完成 client 构造与校验。

**Tech Stack:** Rust(edition 2024)、`rig-core` 0.37(lib 名 `rig_core`)、`tokio`、`thiserror`。

**参考资料:** rig 0.37 源码位于 `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rig-core-0.37.0/`。设计文档:`docs/superpowers/specs/2026-05-22-unify-rig-providers-design.md`。rig client 构造签名以该源码为准,编译期按报错微调。

**约定:** 每个 Task 末尾提交一次。提交信息结尾加一行:
`Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>`。当前仓库尚无提交,直接在当前分支提交即可。

---

## Task 1: 把 `provider.rs` 重构为 `provider/` 目录模块

把单文件 `src/provider.rs` 换成目录模块,放入只含 `Provider` 单元枚举的 `mod.rs` 与两个空的子模块文件,保证编译通过。`Provider` 的方法在 Task 4 补。

**Files:**
- Delete: `src/provider.rs`
- Create: `src/provider/mod.rs`
- Create: `src/provider/model.rs`
- Create: `src/provider/builder.rs`
- 不改:`src/lib.rs`(`mod provider; pub use provider::*;` 对目录模块同样有效)

- [ ] **Step 1: 删除旧文件**

Run: `rm src/provider.rs`

旧文件里的 `Provider`(3 变体)与 `OpenAIModel`(2 变体)是占位代码,`main.rs` / `app.rs` / `tools/` 均未引用,可安全删除。

- [ ] **Step 2: 创建 `src/provider/model.rs`(暂为占位)**

```rust
//! 模型枚举:`Model` 及各 provider 子枚举。Task 3 填充。
```

- [ ] **Step 3: 创建 `src/provider/builder.rs`(暂为占位)**

```rust
//! `ProviderBuilder` 与构造逻辑。Task 6 填充。
```

- [ ] **Step 4: 创建 `src/provider/mod.rs`**

```rust
//! Provider / Model / Agent 统一封装层。

mod builder;
mod model;

pub use builder::*;
pub use model::*;

/// 受支持的 LLM 服务商。纯单元枚举,作为身份标识;
/// 专属配置经 [`ProviderBuilder`] 提供。方法见 Task 4。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provider {
    OpenAI,
    Anthropic,
    Gemini,
    DeepSeek,
    Xai,
    Ollama,
    Compatible,
}
```

- [ ] **Step 5: 编译验证**

Run: `cargo build`
Expected: 编译成功。可能出现 `builder`/`model` 模块为空、`Provider` 未使用的 `dead_code` 警告,允许存在。

- [ ] **Step 6: 提交**

```bash
git add src/provider/ src/provider.rs
git commit -m "refactor: convert provider.rs into provider/ module directory

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: 扩展 `Error` 枚举

为统一层新增错误变体。`Error` 已派生 `Clone`,故包裹 rig 错误时存 `String`。

**Files:**
- Modify: `src/error.rs`
- Test: `src/error.rs`(内联 `#[cfg(test)]`)

- [ ] **Step 1: 写失败的测试**

在 `src/error.rs` 末尾追加:

```rust
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
        assert_eq!(Error::MissingModel.to_string(), "no model was set on the builder");
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
    }
}
```

- [ ] **Step 2: 运行测试,确认失败**

Run: `cargo test error_messages_render`
Expected: 编译失败 —— `Error` 缺少这些变体。

- [ ] **Step 3: 实现 —— 替换 `src/error.rs` 全文**

`Provider` 的 `Display`(Task 4 实现)会输出小写名;此处错误信息需要 `OpenAI` 这样的形态,故用 `{:?}`(`Debug`)渲染 provider。

```rust
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
}
```

- [ ] **Step 4: 运行测试,确认通过**

Run: `cargo test error_messages_render`
Expected: PASS。

- [ ] **Step 5: 提交**

```bash
git add src/error.rs
git commit -m "feat: add provider/model error variants

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: 实现 `Model` 嵌套枚举

`Model` 顶层枚举 + 6 个 provider 子枚举,每个有 `id()`;`Model` 有 `id()` 与 `provider()`;各子枚举 `impl From<_> for Model`。

**Files:**
- Modify: `src/provider/model.rs`
- Test: `src/provider/model.rs`(内联 `#[cfg(test)]`)

注意:子枚举变体名带下划线(如 `Gpt5_5`、`Opus4_7`)会触发 `non_camel_case_types` lint,故每个相关枚举加 `#[allow(non_camel_case_types)]`。

- [ ] **Step 1: 写失败的测试**

在 `src/provider/model.rs` 末尾追加:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Provider;

    #[test]
    fn submodel_ids_match_expected_strings() {
        assert_eq!(OpenAIModel::Gpt5.id(), "gpt-5");
        assert_eq!(OpenAIModel::Gpt5_5.id(), "gpt-5.5");
        assert_eq!(OpenAIModel::O3Mini.id(), "o3-mini");
        assert_eq!(AnthropicModel::Opus4_7.id(), "claude-opus-4-7");
        assert_eq!(GeminiModel::Gemini2_5Flash.id(), "gemini-2.5-flash");
        assert_eq!(DeepSeekModel::Reasoner.id(), "deepseek-reasoner");
        assert_eq!(XaiModel::Grok4.id(), "grok-4-0709");
        assert_eq!(OllamaModel::Llama3_2.id(), "llama3.2");
        assert_eq!(OllamaModel::Custom("phi4".into()).id(), "phi4");
    }

    #[test]
    fn model_id_delegates_to_submodel() {
        assert_eq!(Model::OpenAI(OpenAIModel::Gpt4o).id(), "gpt-4o");
        assert_eq!(Model::Compatible("my-model".into()).id(), "my-model");
    }

    #[test]
    fn model_provider_reports_owner() {
        assert_eq!(Model::OpenAI(OpenAIModel::Gpt5).provider(), Provider::OpenAI);
        assert_eq!(
            Model::Anthropic(AnthropicModel::Haiku4_5).provider(),
            Provider::Anthropic
        );
        assert_eq!(Model::Gemini(GeminiModel::Gemini2_0Flash).provider(), Provider::Gemini);
        assert_eq!(Model::DeepSeek(DeepSeekModel::Chat).provider(), Provider::DeepSeek);
        assert_eq!(Model::Xai(XaiModel::Grok3).provider(), Provider::Xai);
        assert_eq!(Model::Ollama(OllamaModel::Mistral).provider(), Provider::Ollama);
        assert_eq!(Model::Compatible("x".into()).provider(), Provider::Compatible);
    }

    #[test]
    fn submodels_convert_into_model() {
        let m: Model = OpenAIModel::Gpt5.into();
        assert_eq!(m, Model::OpenAI(OpenAIModel::Gpt5));
        let m: Model = AnthropicModel::Sonnet4_6.into();
        assert_eq!(m, Model::Anthropic(AnthropicModel::Sonnet4_6));
    }
}
```

- [ ] **Step 2: 运行测试,确认失败**

Run: `cargo test --lib provider::model`
Expected: 编译失败 —— `Model` 等类型未定义。

- [ ] **Step 3: 实现 —— 在 `src/provider/model.rs` 顶部(占位注释之后、`#[cfg(test)]` 之前)写入**

```rust
use crate::Provider;

/// 统一模型选择:按 provider 分组的嵌套枚举。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Model {
    OpenAI(OpenAIModel),
    Anthropic(AnthropicModel),
    Gemini(GeminiModel),
    DeepSeek(DeepSeekModel),
    Xai(XaiModel),
    Ollama(OllamaModel),
    /// 任意 OpenAI 兼容端点的原始 model id。
    Compatible(String),
}

impl Model {
    /// 模型 id 字符串。rig 的 `agent(impl Into<String>)` 接受任意字符串,
    /// 故这里直接返回字面量,不耦合 rig 的常量集合。
    pub fn id(&self) -> &str {
        match self {
            Model::OpenAI(m) => m.id(),
            Model::Anthropic(m) => m.id(),
            Model::Gemini(m) => m.id(),
            Model::DeepSeek(m) => m.id(),
            Model::Xai(m) => m.id(),
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
            Model::Xai(_) => Provider::Xai,
            Model::Ollama(_) => Provider::Ollama,
            Model::Compatible(_) => Provider::Compatible,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAIModel {
    Gpt5_5,
    Gpt5_2,
    Gpt5_1,
    Gpt5,
    Gpt5Mini,
    Gpt5Nano,
    Gpt4_1,
    Gpt4_1Mini,
    Gpt4o,
    Gpt4oMini,
    O3,
    O3Mini,
    O4Mini,
}

impl OpenAIModel {
    pub fn id(&self) -> &'static str {
        match self {
            OpenAIModel::Gpt5_5 => "gpt-5.5",
            OpenAIModel::Gpt5_2 => "gpt-5.2",
            OpenAIModel::Gpt5_1 => "gpt-5.1",
            OpenAIModel::Gpt5 => "gpt-5",
            OpenAIModel::Gpt5Mini => "gpt-5-mini",
            OpenAIModel::Gpt5Nano => "gpt-5-nano",
            OpenAIModel::Gpt4_1 => "gpt-4.1",
            OpenAIModel::Gpt4_1Mini => "gpt-4.1-mini",
            OpenAIModel::Gpt4o => "gpt-4o",
            OpenAIModel::Gpt4oMini => "gpt-4o-mini",
            OpenAIModel::O3 => "o3",
            OpenAIModel::O3Mini => "o3-mini",
            OpenAIModel::O4Mini => "o4-mini",
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnthropicModel {
    Opus4_7,
    Opus4_6,
    Sonnet4_6,
    Haiku4_5,
}

impl AnthropicModel {
    pub fn id(&self) -> &'static str {
        match self {
            AnthropicModel::Opus4_7 => "claude-opus-4-7",
            AnthropicModel::Opus4_6 => "claude-opus-4-6",
            AnthropicModel::Sonnet4_6 => "claude-sonnet-4-6",
            AnthropicModel::Haiku4_5 => "claude-haiku-4-5",
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeminiModel {
    Gemini3FlashPreview,
    Gemini2_5Flash,
    Gemini2_0Flash,
    Gemini2_0FlashLite,
}

impl GeminiModel {
    pub fn id(&self) -> &'static str {
        match self {
            GeminiModel::Gemini3FlashPreview => "gemini-3-flash-preview",
            GeminiModel::Gemini2_5Flash => "gemini-2.5-flash",
            GeminiModel::Gemini2_0Flash => "gemini-2.0-flash",
            GeminiModel::Gemini2_0FlashLite => "gemini-2.0-flash-lite",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeepSeekModel {
    Chat,
    Reasoner,
    V4Flash,
    V4Pro,
}

impl DeepSeekModel {
    pub fn id(&self) -> &'static str {
        match self {
            DeepSeekModel::Chat => "deepseek-chat",
            DeepSeekModel::Reasoner => "deepseek-reasoner",
            DeepSeekModel::V4Flash => "deepseek-v4-flash",
            DeepSeekModel::V4Pro => "deepseek-v4-pro",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XaiModel {
    Grok4,
    Grok3,
    Grok3Fast,
    Grok3Mini,
    Grok2,
}

impl XaiModel {
    pub fn id(&self) -> &'static str {
        match self {
            XaiModel::Grok4 => "grok-4-0709",
            XaiModel::Grok3 => "grok-3",
            XaiModel::Grok3Fast => "grok-3-fast",
            XaiModel::Grok3Mini => "grok-3-mini",
            XaiModel::Grok2 => "grok-2-1212",
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OllamaModel {
    Llama3_2,
    Mistral,
    Llava,
    /// Ollama 本地模型标签是开放集合,用 `Custom` 兜底。
    Custom(String),
}

impl OllamaModel {
    pub fn id(&self) -> &str {
        match self {
            OllamaModel::Llama3_2 => "llama3.2",
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
impl From<XaiModel> for Model {
    fn from(m: XaiModel) -> Self {
        Model::Xai(m)
    }
}
impl From<OllamaModel> for Model {
    fn from(m: OllamaModel) -> Self {
        Model::Ollama(m)
    }
}
```

- [ ] **Step 4: 运行测试,确认通过**

Run: `cargo test --lib provider::model`
Expected: 4 个测试全部 PASS。

- [ ] **Step 5: 提交**

```bash
git add src/provider/model.rs
git commit -m "feat: add nested Model enum with per-provider submodels

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: 给 `Provider` 加 `FromStr` / `Display` / `api_key_env`

**Files:**
- Modify: `src/provider/mod.rs`
- Test: `src/provider/mod.rs`(内联 `#[cfg(test)]`)

- [ ] **Step 1: 写失败的测试**

在 `src/provider/mod.rs` 末尾追加:

```rust
#[cfg(test)]
mod tests {
    use super::Provider;
    use std::str::FromStr;

    #[test]
    fn parses_provider_case_insensitively() {
        assert_eq!(Provider::from_str("openai").unwrap(), Provider::OpenAI);
        assert_eq!(Provider::from_str("Anthropic").unwrap(), Provider::Anthropic);
        assert_eq!(Provider::from_str("OLLAMA").unwrap(), Provider::Ollama);
        assert_eq!(Provider::from_str("compatible").unwrap(), Provider::Compatible);
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
            Provider::Xai,
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
}
```

- [ ] **Step 2: 运行测试,确认失败**

Run: `cargo test --lib provider::tests`
Expected: 编译失败 —— `FromStr` / `Display` / `api_key_env` 未实现。

- [ ] **Step 3: 实现 —— 在 `src/provider/mod.rs` 的 `Provider` 枚举定义之后追加**

```rust
use crate::Error;
use std::fmt;
use std::str::FromStr;

impl Provider {
    /// 该 provider 的默认 API key 环境变量名;Ollama 无需 key,返回 `None`。
    pub fn api_key_env(&self) -> Option<&'static str> {
        match self {
            Provider::OpenAI => Some("OPENAI_API_KEY"),
            Provider::Anthropic => Some("ANTHROPIC_API_KEY"),
            Provider::Gemini => Some("GEMINI_API_KEY"),
            Provider::DeepSeek => Some("DEEPSEEK_API_KEY"),
            Provider::Xai => Some("XAI_API_KEY"),
            Provider::Ollama => None,
            Provider::Compatible => Some("LLM_API_KEY"),
        }
    }

    /// 规范小写名,与 [`FromStr`] 对称。
    fn canonical_name(&self) -> &'static str {
        match self {
            Provider::OpenAI => "openai",
            Provider::Anthropic => "anthropic",
            Provider::Gemini => "gemini",
            Provider::DeepSeek => "deepseek",
            Provider::Xai => "xai",
            Provider::Ollama => "ollama",
            Provider::Compatible => "compatible",
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_name())
    }
}

impl FromStr for Provider {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "openai" => Ok(Provider::OpenAI),
            "anthropic" => Ok(Provider::Anthropic),
            "gemini" => Ok(Provider::Gemini),
            "deepseek" => Ok(Provider::DeepSeek),
            "xai" => Ok(Provider::Xai),
            "ollama" => Ok(Provider::Ollama),
            "compatible" => Ok(Provider::Compatible),
            _ => Err(Error::UnknownProvider(s.to_string())),
        }
    }
}
```

- [ ] **Step 4: 运行测试,确认通过**

Run: `cargo test --lib provider::tests`
Expected: 4 个测试全部 PASS。

- [ ] **Step 5: 提交**

```bash
git add src/provider/mod.rs
git commit -m "feat: add FromStr/Display/api_key_env for Provider

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: 实现 `Agent` 枚举与统一调用方法

`Agent` 包裹各 provider 的 `rig_core::agent::Agent`。`prompt` / `multi_turn` 需要真实网络,无法离线单测;本 Task 只做实现 + `cargo build`,其变体接线在 Task 6 经 `build()` 测试间接覆盖。

**Files:**
- Modify: `src/provider/mod.rs`

- [ ] **Step 1: 实现 —— 在 `src/provider/mod.rs` 顶部 `mod` 声明之后、`Provider` 定义之前插入 import,并在文件中追加 `Agent` 定义**

文件顶部 import 区(`mod builder; mod model;` 与 `pub use` 之后)加入:

```rust
use rig_core::agent::Agent as RigAgent;
use rig_core::client::CompletionClient;
use rig_core::completion::Prompt;
use rig_core::providers::{anthropic, deepseek, gemini, ollama, openai, xai};

/// 取某 provider client 的 `CompletionModel` 关联类型,避免硬编码 rig 内部路径。
type Cm<C> = <C as CompletionClient>::CompletionModel;
```

在 `Provider` 的 impl 之后追加:

```rust
/// 统一的 agent 句柄:枚举包裹各 provider 的 [`rig_core::agent::Agent`]。
pub enum Agent {
    OpenAIResponses(RigAgent<Cm<openai::Client>>),
    OpenAICompletions(RigAgent<Cm<openai::CompletionsClient>>),
    Anthropic(RigAgent<Cm<anthropic::Client>>),
    Gemini(RigAgent<Cm<gemini::Client>>),
    DeepSeek(RigAgent<Cm<deepseek::Client>>),
    Xai(RigAgent<Cm<xai::Client>>),
    Ollama(RigAgent<Cm<ollama::Client>>),
}

impl Agent {
    /// 单轮提示:把回复作为字符串返回。
    pub async fn prompt(&self, input: &str) -> crate::Result<String> {
        let res = match self {
            Agent::OpenAIResponses(a) => a.prompt(input).await,
            Agent::OpenAICompletions(a) => a.prompt(input).await,
            Agent::Anthropic(a) => a.prompt(input).await,
            Agent::Gemini(a) => a.prompt(input).await,
            Agent::DeepSeek(a) => a.prompt(input).await,
            Agent::Xai(a) => a.prompt(input).await,
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
            Agent::Xai(a) => a.prompt(input).max_turns(max_turns).await,
            Agent::Ollama(a) => a.prompt(input).max_turns(max_turns).await,
        };
        res.map_err(|e| Error::Rig(e.to_string()))
    }
}
```

说明:`crate::Result` 见 Step 2。`a.prompt(input)` 走 rig 的 `Prompt` trait,返回 `PromptRequest`,经 `IntoFuture` 可直接 `.await`;`.max_turns(n)` 设多轮上限。若编译报 `Prompt` / `max_turns` 路径不符,对照 `rig-core-0.37.0/src/completion/request.rs` 与 `src/agent/prompt_request/mod.rs` 微调。

- [ ] **Step 2: 在 `src/error.rs` 增加 `Result` 类型别名(供整 crate 复用)**

在 `src/error.rs` 的 `Error` 定义之后追加:

```rust
/// 本 crate 统一的 `Result`。
pub type Result<T, E = Error> = std::result::Result<T, E>;
```

`src/lib.rs` 已有 `pub use error::*;`,`crate::Result` 与 `crate::Error` 均可用。

- [ ] **Step 3: 编译验证**

Run: `cargo build`
Expected: 编译成功。若 `openai::CompletionsClient` 名称不符,查 `rig-core-0.37.0/src/providers/openai/client.rs`(应由 `pub use client::*` 导出)。

- [ ] **Step 4: 提交**

```bash
git add src/provider/mod.rs src/error.rs
git commit -m "feat: add Agent enum wrapping rig agents with unified prompt

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: 实现 `ProviderBuilder`

单结构体 `ProviderBuilder`,链式公共方法 + provider 专属方法,`build()` 内一处 `match` 完成校验与 client 构造。

**Files:**
- Modify: `src/provider/builder.rs`
- Test: `src/provider/builder.rs`(内联 `#[cfg(test)]`)

- [ ] **Step 1: 写失败的测试**

校验类逻辑在构造 client 之前返回,无网络;成功路径用假 key 走通构造(rig client 构造不发请求)。在 `src/provider/builder.rs` 末尾追加:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnthropicModel, Config, Model, OpenAIModel, Provider};

    fn test_config() -> Config {
        // Config 没有便于测试的构造器时,经 from_env 读取;
        // 这些测试都显式 .api_key(),不依赖 Config 里的 key。
        unsafe {
            std::env::set_var("LLM_API_KEY", "test-key");
        }
        Config::from_env()
    }

    #[test]
    fn build_without_model_errors() {
        let err = ProviderBuilder::new(Provider::OpenAI, test_config())
            .build()
            .unwrap_err();
        assert!(matches!(err, crate::Error::MissingModel));
    }

    #[test]
    fn build_with_mismatched_model_errors() {
        let err = ProviderBuilder::new(Provider::OpenAI, test_config())
            .model(AnthropicModel::Opus4_7)
            .build()
            .unwrap_err();
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
        let err = ProviderBuilder::new(Provider::OpenAI, test_config())
            .model(OpenAIModel::Gpt5)
            .anthropic_version(AnthropicVersion::Latest)
            .build()
            .unwrap_err();
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
        let agent = ProviderBuilder::new(Provider::OpenAI, test_config())
            .api_key("fake-key")
            .model(OpenAIModel::Gpt5)
            .preamble("hi")
            .build()
            .expect("build should succeed with a fake key");
        assert!(matches!(agent, crate::Agent::OpenAIResponses(_)));
    }

    #[test]
    fn build_openai_completions_agent() {
        let agent = ProviderBuilder::new(Provider::OpenAI, test_config())
            .api_key("fake-key")
            .model(OpenAIModel::Gpt4o)
            .completions_api()
            .build()
            .expect("build should succeed");
        assert!(matches!(agent, crate::Agent::OpenAICompletions(_)));
    }

    #[test]
    fn build_compatible_agent() {
        let agent = ProviderBuilder::new(Provider::Compatible, test_config())
            .api_key("fake-key")
            .base_url("https://example.com/v1")
            .model(Model::Compatible("some-model".into()))
            .build()
            .expect("build should succeed");
        assert!(matches!(agent, crate::Agent::OpenAICompletions(_)));
    }
}
```

注意:`std::env::set_var` 在 edition 2024 是 `unsafe`,测试里已用 `unsafe` 块包裹。

- [ ] **Step 2: 运行测试,确认失败**

Run: `cargo test --lib provider::builder`
Expected: 编译失败 —— `ProviderBuilder` / `AnthropicVersion` / `OpenAiApi` 未定义。

- [ ] **Step 3: 实现 —— 在 `src/provider/builder.rs` 占位注释之后、`#[cfg(test)]` 之前写入**

```rust
use crate::{Agent, Config, Error, Model, Provider, Result};
use rig_core::agent::AgentBuilder;
use rig_core::client::CompletionClient;
use rig_core::completion::CompletionModel;
use rig_core::providers::{anthropic, deepseek, gemini, ollama, openai, xai};
use rig_core::tool::{Tool, ToolDyn};

/// OpenAI 的两套 API。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiApi {
    Responses,
    Completions,
}

/// Anthropic API 版本头。
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
    config: Config,
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
    pub fn new(provider: Provider, config: Config) -> Self {
        Self {
            provider,
            config,
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
    pub fn build(self) -> Result<Agent> {
        let ProviderBuilder {
            provider,
            config,
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
                let key = resolve_key(provider, &api_key, &config)?;
                match openai_api.unwrap_or(OpenAiApi::Responses) {
                    OpenAiApi::Responses => {
                        let client =
                            openai::Client::new(key).map_err(|e| Error::Rig(e.to_string()))?;
                        Agent::OpenAIResponses(finish(
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
                        Agent::OpenAICompletions(finish(
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
                let key = resolve_key(provider, &api_key, &config)?;
                let version = anthropic_version.unwrap_or(AnthropicVersion::Latest);
                let client = anthropic::Client::builder()
                    .api_key(key)
                    .anthropic_version(version.as_str())
                    .build()
                    .map_err(|e| Error::Rig(e.to_string()))?;
                Agent::Anthropic(finish(
                    client.agent(m.id()),
                    preamble,
                    temperature,
                    max_tokens,
                    tools,
                ))
            }
            Model::Gemini(m) => {
                let key = resolve_key(provider, &api_key, &config)?;
                let client = gemini::Client::new(key).map_err(|e| Error::Rig(e.to_string()))?;
                Agent::Gemini(finish(
                    client.agent(m.id()),
                    preamble,
                    temperature,
                    max_tokens,
                    tools,
                ))
            }
            Model::DeepSeek(m) => {
                let key = resolve_key(provider, &api_key, &config)?;
                let client = deepseek::Client::new(key).map_err(|e| Error::Rig(e.to_string()))?;
                Agent::DeepSeek(finish(
                    client.agent(m.id()),
                    preamble,
                    temperature,
                    max_tokens,
                    tools,
                ))
            }
            Model::Xai(m) => {
                let key = resolve_key(provider, &api_key, &config)?;
                let client =
                    xai::Client::new(key).map_err(|e| Error::Rig(e.to_string()))?;
                Agent::Xai(finish(
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
                Agent::Ollama(finish(
                    client.agent(m.id()),
                    preamble,
                    temperature,
                    max_tokens,
                    tools,
                ))
            }
            Model::Compatible(id) => {
                let key = resolve_key(provider, &api_key, &config)?;
                let url = base_url
                    .clone()
                    .unwrap_or_else(|| config.base_url().to_string());
                let client = openai::Client::builder()
                    .api_key(key)
                    .base_url(&url)
                    .build()
                    .map_err(|e| Error::Rig(e.to_string()))?
                    .completions_api();
                Agent::OpenAICompletions(finish(
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

/// 解析 API key:显式 `.api_key()` > `Config` 里的 key > 环境变量。
fn resolve_key(provider: Provider, api_key: &Option<String>, config: &Config) -> Result<String> {
    if let Some(key) = api_key {
        if !key.is_empty() {
            return Ok(key.clone());
        }
    }
    let cfg_key = config.llm_api_key();
    if !cfg_key.is_empty() {
        return Ok(cfg_key.to_string());
    }
    if let Some(var) = provider.api_key_env() {
        if let Ok(value) = std::env::var(var) {
            if !value.is_empty() {
                return Ok(value);
            }
        }
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
) -> rig_core::agent::Agent<M>
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
```

实现注意点(编译期对照 `rig-core-0.37.0` 源码微调):

- `Config` 的字段私有、访问器为 `pub(crate)`,`builder.rs` 同 crate 可用 `config.llm_api_key()` / `config.base_url()`。`Config` 派生了 `Clone`。
- `ollama::OllamaApiKey` 由 `pub type ClientBuilder = client::ClientBuilder<OllamaBuilder, OllamaApiKey, _>` 间接导出,应可达;若不可达,改用 `ollama::Client::from_env()` 并接受其默认 URL。
- `AgentBuilder` 的 `.preamble` 收 `&str`、`.tools` 收 `Vec<Box<dyn ToolDyn>>` 并切换类型态后 `.build()`;`finish` 因此末尾统一调 `.tools(tools).build()`。
- 各 `Client::new` / `Client::builder().build()` 返回 `rig_core::http_client::Result<_>`,统一 `map_err(|e| Error::Rig(e.to_string()))`。

- [ ] **Step 4: 运行测试,确认通过**

Run: `cargo test --lib provider::builder`
Expected: 7 个测试全部 PASS。

- [ ] **Step 5: 提交**

```bash
git add src/provider/builder.rs
git commit -m "feat: add ProviderBuilder with unified build and provider options

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 7: 接通 `Provider::builder` 与 `Provider::agent`

给 `Provider` 加两个入口方法,打通"选 provider → builder → Agent"的端到端链路。

**Files:**
- Modify: `src/provider/mod.rs`
- Test: `src/provider/mod.rs`(扩充已有 `#[cfg(test)] mod tests`)

- [ ] **Step 1: 写失败的测试**

在 `src/provider/mod.rs` 的 `mod tests` 内追加:

```rust
    #[test]
    fn provider_agent_shortcut_builds() {
        use crate::{Config, Model, OpenAIModel};
        unsafe {
            std::env::set_var("LLM_API_KEY", "fake-key");
        }
        let cfg = Config::from_env();
        let agent = Provider::OpenAI
            .agent(Model::OpenAI(OpenAIModel::Gpt5), &cfg)
            .expect("shortcut build should succeed");
        assert!(matches!(agent, crate::Agent::OpenAIResponses(_)));
    }

    #[test]
    fn provider_builder_entrypoint_builds() {
        use crate::{AnthropicModel, Config};
        unsafe {
            std::env::set_var("LLM_API_KEY", "fake-key");
        }
        let cfg = Config::from_env();
        let agent = Provider::Anthropic
            .builder(&cfg)
            .model(AnthropicModel::Sonnet4_6)
            .max_tokens(1024)
            .build()
            .expect("builder entrypoint should succeed");
        assert!(matches!(agent, crate::Agent::Anthropic(_)));
    }
```

- [ ] **Step 2: 运行测试,确认失败**

Run: `cargo test --lib provider::tests`
Expected: 编译失败 —— `Provider::builder` / `Provider::agent` 未定义。

- [ ] **Step 3: 实现 —— 在 `src/provider/mod.rs` 的 `impl Provider` 块内(`api_key_env` 之后)追加**

需要引用 `Config` / `Model`,在文件顶部 import 区补 `use crate::{Config, Model};`(若 `Error` 已 `use`,合并即可)。

```rust
    /// 进入该 provider 的 [`ProviderBuilder`],用于设置专属选项。
    pub fn builder(&self, config: &Config) -> ProviderBuilder {
        ProviderBuilder::new(*self, config.clone())
    }

    /// 常见场景的快捷入口:用默认选项直接构造 [`Agent`]。
    pub fn agent(&self, model: Model, config: &Config) -> crate::Result<Agent> {
        self.builder(config).model(model).build()
    }
```

- [ ] **Step 4: 运行测试,确认通过**

Run: `cargo test --lib`
Expected: 全部测试 PASS(model / provider / builder / error 各组)。

- [ ] **Step 5: 提交**

```bash
git add src/provider/mod.rs
git commit -m "feat: wire Provider::builder and Provider::agent entrypoints

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 8: 在 `app.rs` 用新 API 构造 agent

把占位的 `App::run` 改成:读环境选 provider/model、构造 `Agent`、打印就绪信息。`main` 改为 tokio 异步入口。不实际发起 `prompt`(需真实 key/网络)。

**Files:**
- Modify: `src/app.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 改写 `src/app.rs` 的 `App::run`**

把 `impl App` 整段替换为:

```rust
use crate::{Config, Model, OpenAIModel, Provider};
use std::str::FromStr;

pub struct App;

impl App {
    pub async fn run() -> crate::Result<()> {
        let config = Config::from_env();
        let _ = AGENT_SYSTEM_PROMPT;

        // 默认 OpenAI;可用 PROVIDER 环境变量覆盖。
        let provider = match std::env::var("PROVIDER") {
            Ok(name) => Provider::from_str(&name)?,
            Err(_) => Provider::OpenAI,
        };

        // 演示:用默认模型构造一个 agent。各 provider 的专属选项
        // 可改用 `provider.builder(&config)....build()` 链式设置。
        let model: Model = match provider {
            Provider::OpenAI => OpenAIModel::Gpt5.into(),
            other => {
                println!("provider {other} selected; using OpenAI gpt-5 for this demo");
                OpenAIModel::Gpt5.into()
            }
        };

        let _agent = Provider::OpenAI.agent(model, &config)?;
        println!(
            "agent ready: provider={}, tavily_configured={}",
            provider,
            config.tavily_api_key().is_some(),
        );
        Ok(())
    }
}
```

保留文件顶部的 `AGENT_SYSTEM_PROMPT` 常量不动。

- [ ] **Step 2: 改写 `src/main.rs` 为异步入口**

```rust
use hello_agents::App;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    if let Err(err) = App::run().await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
```

- [ ] **Step 3: 编译并跑全部测试**

Run: `cargo build`
Expected: 编译成功,无 error。

Run: `cargo test`
Expected: 全部测试 PASS。

- [ ] **Step 4: 冒烟运行**

Run: `LLM_API_KEY=fake cargo run`
Expected: 打印 `agent ready: provider=openai, tavily_configured=false`(或按 `.env` 实际值),进程退出码 0。

- [ ] **Step 5: 提交**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: build agent via unified Provider API in App::run

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## 完成标准

- `cargo build` 与 `cargo test` 均通过。
- `Provider` 是 7 变体单元枚举,有 `FromStr` / `Display` / `api_key_env` / `builder` / `agent`。
- `Model` 是嵌套枚举,6 个子枚举各有 `id()`,顶层有 `id()` / `provider()`。
- `Agent` 是 7 变体枚举,有统一的 `prompt` / `multi_turn`。
- `ProviderBuilder` 单结构体,公共方法 + `responses_api` / `completions_api` / `anthropic_version` / `base_url` 专属方法,`build()` 做三类校验。
- `app.rs` 经新 API 构造 agent。
