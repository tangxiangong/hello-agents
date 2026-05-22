# 设计文档:统一 rig 的 Provider 与 Model

- 日期:2026-05-22
- 状态:已批准设计,待实现
- 范围:`hello-agents` crate 中 `src/provider.rs` 的重构

## 1. 背景与目标

`hello-agents` 基于 `rig-core` 0.37 构建。rig 把每个 LLM 服务商做成独立的
provider 模块(约 25 个),每个 provider 暴露一个独立的 `Client` 类型,模型则以
`&str` 常量形式给出(如 `openai::GPT_4O`、`anthropic::CLAUDE_OPUS_4_7`)。

当前 `src/provider.rs` 只有占位用的 `Provider`(3 变体)与 `OpenAIModel`(2 变体)
枚举,尚未接入 rig。

目标:在 `hello-agents` 内构建一层薄封装,把 rig 的多个 provider 统一成枚举、把
`&str` 模型常量统一成枚举,同时**保留各 provider 的专属特性**,不把能力压成最小
公约数。

非目标(YAGNI):

- 不封装 embedding / image / audio / transcription 客户端,只做 completion + agent。
- 不照搬 rig 全部模型常量,只收录各 provider 的精选当代模型。
- 不重构 `Config`,只做让 builder 能按 provider 取 key / base_url 的最小适配。
- 不修改或 fork rig 本身。

## 2. 设计决策(已确认)

| 决策点 | 选择 |
| --- | --- |
| Provider 覆盖范围 | 主流精选集:OpenAI、Anthropic、Gemini、DeepSeek、xAI、Ollama,外加 Compatible 兼容入口 |
| Model 枚举结构 | 嵌套枚举:顶层 `Model` 按 provider 分,每个 provider 一个子枚举 |
| 运行时统一方式 | 枚举包裹 `rig::agent::Agent`,全静态、无 `dyn`,`match` 分发 |
| provider 特色保留 | `Provider` 保持纯单元枚举;特色做成各 provider 专属 builder 方法 |

## 3. 模块结构

`src/provider.rs` 拆为目录模块:

```
src/provider/
├── mod.rs       Provider 枚举、统一 Agent 枚举、工厂入口、re-export
├── model.rs     Model 嵌套枚举 + 各 provider 子枚举
└── builder.rs   ProviderBuilder 枚举与各 provider 专属 builder
```

`src/lib.rs` 中 `mod provider; pub use provider::*;` 维持不变。

每个文件的职责边界:

- `model.rs`:纯数据。只依赖 rig 的模型常量,不依赖 client / agent。
- `builder.rs`:构造逻辑。`ProviderBuilder` 结构体与 provider 专属方法,依赖
  `model.rs`、`Config`、rig 的 client 与 agent builder。
- `mod.rs`:对外门面。`Provider` 身份枚举、`Agent` 运行时枚举、错误校验。

## 4. `Provider` —— 纯单元枚举(身份标识)

```rust
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

保持 `Copy` + 无内嵌数据,使其适合作为配置 / 环境变量读出来的轻量标识。

方法:

- `impl FromStr`:`"openai"`、`"anthropic"` 等不区分大小写解析;未知值返回
  `Error::UnknownProvider`。
- `impl Display`:与 `FromStr` 对称的小写名。
- `fn api_key_env(&self) -> Option<&'static str>`:该 provider 的默认 API key
  环境变量名(如 `OPENAI_API_KEY`);Ollama 返回 `None`。
- `fn builder(&self, cfg: &Config) -> ProviderBuilder`:进入该 provider 的专属
  builder(见第 7 节)。
- `fn agent(&self, model: Model, cfg: &Config) -> Result<Agent>`:常见场景的一行
  快捷入口,内部走 `builder()` 的默认配置。

## 5. `Model` —— 嵌套枚举

```rust
pub enum Model {
    OpenAI(OpenAIModel),
    Anthropic(AnthropicModel),
    Gemini(GeminiModel),
    DeepSeek(DeepSeekModel),
    Xai(XaiModel),
    Ollama(OllamaModel),
    Compatible(String), // 任意 OpenAI 兼容端点的原始 model id
}
```

各子枚举收录该 provider 的精选当代模型(随 rig 0.37 实际常量为准):

```rust
pub enum OpenAIModel {
    Gpt5_5, Gpt5_2, Gpt5_1, Gpt5, Gpt5Mini, Gpt5Nano,
    Gpt4_1, Gpt4_1Mini, Gpt4o, Gpt4oMini,
    O3, O3Mini, O4Mini,
}

pub enum AnthropicModel {
    Opus4_7, Opus4_6, Sonnet4_6, Haiku4_5,
}

pub enum GeminiModel {
    Gemini3FlashPreview, Gemini2_5Flash, Gemini2_0Flash, Gemini2_0FlashLite,
}

pub enum DeepSeekModel {
    Chat, Reasoner, V4Flash, V4Pro,
}

pub enum XaiModel {
    Grok4, Grok3, Grok3Fast, Grok3Mini, Grok2,
}

pub enum OllamaModel {
    Llama3_2, Mistral, Llava,
    Custom(String), // Ollama 本地模型标签是开放集合
}
```

统一方法:

- `Model::id(&self) -> &str`:返回模型 id 字符串。单元变体返回 `&'static str`
  字面量(如 `"gpt-5"`);`OllamaModel::Custom` 与 `Model::Compatible` 返回内部
  `String` 的借用。返回类型 `&str` 借用 `self`,对两种情况都成立。
- `Model::provider(&self) -> Provider`:反查所属 provider。
- 每个子枚举自身也实现 `id()`,并作为挂载 provider 专属方法的位置(保留特色)。
- 每个子枚举 `impl Into<Model>`(`From<OpenAIModel> for Model` 等),便于
  `ProviderBuilder::model(impl Into<Model>)` 直接收子枚举值。

## 6. `Agent` —— 枚举包裹 rig 的 Agent

```rust
pub enum Agent {
    OpenAIResponses(rig::agent::Agent<openai::responses_api::CompletionModel>),
    OpenAICompletions(rig::agent::Agent<openai::completion::CompletionModel>),
    Anthropic(rig::agent::Agent<anthropic::completion::CompletionModel>),
    Gemini(rig::agent::Agent<gemini::completion::CompletionModel>),
    DeepSeek(rig::agent::Agent<deepseek::CompletionModel>),
    Xai(rig::agent::Agent<xai::completion::CompletionModel>),
    Ollama(rig::agent::Agent<ollama::CompletionModel>),
}
```

说明:

- rig 中每个 provider 的 `CompletionModel` 是独立具体类型,故每个变体显式列出。
  确切类型路径以 rig 0.37 实际为准,实现阶段核对。
- OpenAI 拆 `OpenAIResponses` / `OpenAICompletions` 两个变体,正是为保留其
  Responses / Completions 双 API 特色。
- `Compatible` provider 复用 `OpenAICompletions` 变体(OpenAI 兼容端点走
  Completions API),不单设变体。

统一方法(`match` 分发到内部 rig Agent):

- `async fn prompt(&self, input: &str) -> Result<String>`
- `async fn multi_turn(&self, input: &str, max_turns: usize) -> Result<String>`

rig Agent 调用产生的错误统一包成 `Error::Rig`。

## 7. `ProviderBuilder` —— 特色所在

`Provider::builder()` 返回单个结构体 `ProviderBuilder`,持有公共配置与 provider
专属配置:

```rust
pub struct ProviderBuilder {
    provider: Provider,
    api_key: Option<String>,
    base_url: Option<String>,          // Ollama / Compatible
    model: Option<Model>,
    preamble: Option<String>,
    temperature: Option<f64>,
    max_tokens: Option<u64>,
    tools: Vec<Box<dyn ToolDyn>>,
    openai_api: Option<OpenAiApi>,      // OpenAI 专属:Responses / Completions
    anthropic_version: Option<AnthropicVersion>, // Anthropic 专属
}
```

> 设计说明:此处采用单结构体,而非"7 个变体各包一个具体 builder 的枚举"。
> 原 spec 草稿的枚举方案与其用法示例自相矛盾(示例在 `ProviderBuilder` 上链式
> 调用了专属方法,而枚举方案下专属方法只在具体 builder 上)。各 provider 的专属
> 配置形状小且统一,单结构体可保留完全相同的公开 API,同时省去约 7× 样板。

所有方法都返回 `Self`,可链式调用。**公共方法**:

- `model(impl Into<Model>)` —— 设定模型(各子枚举 `impl Into<Model>`)
- `preamble(impl Into<String>)` —— 系统提示词
- `temperature(f64)` / `max_tokens(u64)`
- `tool(impl Tool + 'static)` —— 注册工具
- `api_key(impl Into<String>)` —— 显式提供 key
- `build(self) -> Result<Agent>`

**专属方法**(作用于对应 provider;若 provider 不匹配,`build()` 返回
`Error::InvalidProviderOption`):

- `responses_api()` / `completions_api()` —— OpenAI 专属,选择双 API
- `anthropic_version(AnthropicVersion)` —— Anthropic 专属,设定 API 版本头
- `base_url(impl Into<String>)` —— Ollama / Compatible 专属,指定端点地址

用法示例:

```rust
// 常见场景:一行
let agent = Provider::OpenAI.agent(Model::OpenAI(OpenAIModel::Gpt5), &cfg)?;

// 特色场景:OpenAI 专属
let agent = Provider::OpenAI
    .builder(&cfg)
    .responses_api()
    .model(OpenAIModel::Gpt5)
    .preamble("...")
    .tool(weather_tool)
    .build()?;

// 特色场景:Anthropic 专属
let agent = Provider::Anthropic
    .builder(&cfg)
    .anthropic_version(AnthropicVersion::Latest)
    .model(AnthropicModel::Opus4_7)
    .max_tokens(4096)
    .build()?;

// 特色场景:Ollama 专属
let agent = Provider::Ollama
    .builder(&cfg)
    .base_url("http://remote:11434")
    .model(OllamaModel::Llama3_2)
    .build()?;
```

逃生舱:本封装是"加一层"而非"盖住" rig —— rig 的各 provider client 都是公开
类型,任何时候都能绕过 `ProviderBuilder` 直接用 `rig_core::providers::*` 构造
原始 client。故不额外定义 `into_rig_client()` 之类访问点,保持本层精简。

`AnthropicVersion` 为本 crate 定义的小枚举,映射到 rig 的
`ANTHROPIC_VERSION_*` 常量:

```rust
pub enum AnthropicVersion { V2023_01_01, V2023_06_01, Latest }
```

## 8. 错误处理

`src/error.rs` 的 `Error` 扩展以下变体:

```rust
UnknownProvider(String),                       // FromStr 解析失败
MissingApiKey(Provider),                       // 需要 key 但环境/配置缺失
MissingModel,                                  // build() 时未设定 model
ModelProviderMismatch { model: Provider, provider: Provider }, // model 与 provider 不一致
InvalidProviderOption { provider: Provider, option: &'static str }, // 专属选项用错 provider
Rig(String),                                   // 包裹 rig 构造 / 调用错误
```

注意现有 `Error` 派生了 `Clone`;rig 的错误类型未必 `Clone`,故 `Rig` 变体存
`String`(用 `to_string()` 包裹),而非持有原始错误对象。`&'static str` 不影响
`Clone`。

校验(在 `ProviderBuilder::build` 内):
- `model` 未设 → `MissingModel`。
- `model.provider() != self.provider` → `ModelProviderMismatch`。
- 设了 `openai_api` 但 provider 非 OpenAI、或设了 `anthropic_version` 但 provider
  非 Anthropic → `InvalidProviderOption`。

## 9. `Config` 最小适配

`Config` 当前持有单一 `base_url` / `llm_api_key` / `tavily_api_key`。本设计不重构
`Config`,只新增 builder 取值所需的最小访问:

- builder 构造 rig client 时,API key 来源优先级:`Config` 显式提供 > provider 的
  `api_key_env()` 环境变量。
- `base_url` 仅 `Compatible` / `Ollama` 用到,沿用 `Config::base_url()` 或 builder
  专属方法覆盖。

是否进一步把 `Config` 改成 per-provider 多 key 结构,留作后续,不在本次范围。

## 10. 取舍说明

- **样板代码**:`Agent` 枚举有 7 个 `match` 分支;`ProviderBuilder` 是单结构体,
  分发集中在 `build()` 一处 `match`。新增 provider 需改 `Provider`、`Model`、
  `Agent` 三处枚举与 `build()` 的 `match`。代价集中在 `provider/` 一个目录内。
  换来的是全静态分发、无 `dyn`、编译期类型安全。
- **OpenAI 双变体**:为保留 Responses / Completions 双 API,`Agent` 多一个变体,
  属有意为之。
- **精选而非穷举**:模型子枚举只收录当代主力模型;需要冷门 / 历史模型时,可经
  `Model::Compatible(String)` 或直接用 rig 原始 client 处理。
- **`Model::id()` 返回字面量**:rig 的 `agent(impl Into<String>)` 接受任意字符串,
  模型常量只是语法糖。故 `id()` 直接返回模型 id 字面量,不耦合 rig 的常量集合。

## 11. 测试策略

- `model.rs`:单元测试,断言每个 `Model` 变体的 `id()` 等于 rig 对应常量、
  `provider()` 返回正确 provider。这层纯数据、无 IO,可完整覆盖。
- `Provider` 的 `FromStr` / `Display`:往返一致性测试。
- 校验逻辑:`ModelProviderMismatch` 用错配的 `Model` + `Provider` 触发并断言。
- builder / `Agent` 的实际网络调用不做单元测试(依赖外部 API);只测到 `build()`
  能在给定假 key 下走通构造路径(或对构造逻辑做不联网的断言)。

## 12. 实现顺序建议

1. `model.rs`:`Model` 及子枚举、`id()` / `provider()`,配单元测试。
2. `error.rs`:扩展 `Error` 变体。
3. `Provider` 枚举:`FromStr` / `Display` / `api_key_env`。
4. `Agent` 枚举:变体定义 + `prompt` / `multi_turn` 分发。
5. `builder.rs`:各具体 builder 与 `ProviderBuilder` 枚举、公共与专属方法。
6. 接回 `Provider::builder` / `Provider::agent`,打通端到端。
7. 更新 `app.rs` 示例用新 API 构造 agent(替换当前占位逻辑)。
