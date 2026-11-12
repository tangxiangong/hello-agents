from hello_agents.react import ReActAgent
from hello_agents.tools import ToolExecutor, search
import os
from openai import OpenAI
from dotenv import load_dotenv
from openai.types.chat import ChatCompletionMessageParam

load_dotenv()

class HelloAgentsLLM:
    def __init__(self, model: str | None = None, api_key: str | None = None, base_url: str | None = None, timeout: int | None = None) -> None:
        model = model or os.getenv("LLM_MODEL_ID")
        api_key = api_key or os.getenv("LLM_API_KEY")
        base_url = base_url or os.getenv("LLM_BASE_URL")
        timeout = timeout or int(os.getenv("LLM_TIMEOUT", 60))

        if not (model and api_key and base_url):
            raise ValueError("模型 ID、API 密钥和服务地址必须被提供或者在 .env 文件中配置")

        self.model: str = model
        self.client = OpenAI(api_key=api_key, base_url=base_url, timeout=timeout)
    
    def think(self, messages: list[ChatCompletionMessageParam], temperature: float = 0.0) -> str | None:
        print(f"🧠 正在调用 {self.model} 模型...")
        try:
            response = self.client.chat.completions.create(
                model=self.model,
                messages=messages,
                temperature=temperature,
                stream=True,
            )

            print("✅ LLM 响应成功:")
            collected_content = []
            for chunk in response:
                if not chunk.choices:
                    continue
                content = chunk.choices[0].delta.content or ""
                print(content, end="", flush=True)
                collected_content.append(content)
            print()
            return "".join(collected_content)
    
        except Exception as e:
            print(f"❌ LLM 调用失败: {e}")
            return None


def main() -> None:
    try:
        llm = HelloAgentsLLM()
        tool_executor = ToolExecutor()
        search_description = "一个网页搜索引擎。当你需要回答关于时事、事实以及在你的知识库中找不到的信息时，应使用此工具。"
        tool_executor.register("Search", search_description, search)

        react_agent = ReActAgent(llm, tool_executor)
        react_agent.run("iPhone 的最新款")
    
    except ValueError as e:
        print(e)
