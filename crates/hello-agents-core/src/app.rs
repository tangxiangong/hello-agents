use crate::{
    ChatHistory, Config, Model, Provider, Result,
    tools::{GoogleSearch, SearchAttraction, Weather, WebSearch},
};

const DEFAULT_MAX_TURNS: usize = 8;

const AGENT_SYSTEM_PROMPT: &str = r#"
你是一个智能旅行助手。你的任务是分析用户的请求，并使用可用工具一步步地解决问题。

# 可用工具:
- `get_weather(city: str)`: 查询指定城市的实时天气。
- `get_attraction(city: str, weather: str)`: 根据城市和天气搜索推荐的旅游景点。

# 输出格式要求:
你的每次回复必须严格遵循以下格式，包含一对Thought和Action：

Thought: [你的思考过程和下一步计划]
Action: [你要执行的具体行动]

Action的格式必须是以下之一：
1. 调用工具：function_name(arg_name="arg_value")
2. 结束任务：Finish[最终答案]

# 重要提示:
- 每次只输出一对Thought-Action
- Action必须在同一行，不要换行
- 当收集到足够信息可以回答用户问题时，必须使用 Action: Finish[最终答案] 格式结束

请开始吧！
"#;

pub struct App {
    config: Config,
    model: Model,
}

impl App {
    pub fn new(config: Config, model: Model) -> Self {
        Self { config, model }
    }

    pub fn from_model(model: Model) -> Self {
        Self::new(Config::from_model(model.clone()), model)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn provider(&self) -> Provider {
        self.model.provider()
    }

    pub fn new_chat_history(&self) -> ChatHistory {
        Vec::new()
    }

    pub async fn run(&self, prompt: &str) -> Result<String> {
        self.build_agent()?.prompt(prompt).await
    }

    pub async fn chat(&self, prompt: &str, history: &mut ChatHistory) -> Result<String> {
        self.build_agent()?
            .chat(prompt, DEFAULT_MAX_TURNS, history)
            .await
    }

    pub async fn stream<F>(&self, prompt: &str, on_text: F) -> Result<String>
    where
        F: FnMut(&str) -> Result<()>,
    {
        self.build_agent()?
            .stream(prompt, DEFAULT_MAX_TURNS, on_text)
            .await
    }

    pub async fn stream_chat<F>(
        &self,
        prompt: &str,
        history: &mut ChatHistory,
        on_text: F,
    ) -> Result<String>
    where
        F: FnMut(&str) -> Result<()>,
    {
        self.build_agent()?
            .stream_chat(prompt, DEFAULT_MAX_TURNS, history, on_text)
            .await
    }

    pub async fn stream_to_stdout(&self, prompt: &str) -> Result<String> {
        self.build_agent()?
            .stream_to_stdout(prompt, DEFAULT_MAX_TURNS)
            .await
    }

    pub async fn stream_chat_to_stdout(
        &self,
        prompt: &str,
        history: &mut ChatHistory,
    ) -> Result<String> {
        self.build_agent()?
            .stream_chat_to_stdout(prompt, DEFAULT_MAX_TURNS, history)
            .await
    }

    fn build_agent(&self) -> Result<crate::Agent> {
        let mut provider = self
            .provider()
            .builder()
            .model(self.model.clone())
            .preamble(AGENT_SYSTEM_PROMPT)
            .tool(Weather);

        if let Some(api_key) = self.config.tavily_api_key() {
            provider = provider
                .tool(WebSearch::new(api_key))
                .tool(SearchAttraction::new(api_key));
        }

        if let Some(api_key) = self.config.serpapi_api_key() {
            provider = provider.tool(GoogleSearch::new(api_key));
        }

        provider.build()
    }
}
