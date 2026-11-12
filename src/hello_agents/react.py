from __future__ import annotations
import re
from typing import TYPE_CHECKING
from openai.types.chat import ChatCompletionMessageParam
from hello_agents.tools import ToolExecutor

if TYPE_CHECKING:
    from hello_agents import HelloAgentsLLM

REACT_PROMPT_TEMPLATE = """
请注意，你是一个有能力调用外部工具的智能助手。

可用工具如下：
{tools}

请严格按照以下格式进行回应：

Thought: 你的思考过程，用于分析问题、拆解任务和规划下一步行动。
Action: 你决定采取的行动，必须是以下格式之一:
- `{{tool_name}}[{{tool_input}}]`:调用一个可用工具。
- `Finish[最终答案]`:当你认为已经获得最终答案时。
- 当你收集到足够的信息，能够回答用户的最终问题时，你必须在Action:字段后使用 Finish[最终答案] 来输出最终答案。

现在，请开始解决以下问题:
Question: {question}
History: {history}
"""

class ReActAgent:
    def __init__(self, client: HelloAgentsLLM, tool_executor: ToolExecutor, max_steps: int = 5):
        self.client = client
        self.tool_executor = tool_executor
        self.max_steps = max_steps
        self.history = []
        
    def run(self, question: str):
        self.history = []
        current_step = 0

        while current_step < self.max_steps:
            current_step += 1
            print(f"--- 第 {current_step} 步 ---")

            tools_desc = self.tool_executor.list()
            history_str = "\n".join(self.history)
            prompt = REACT_PROMPT_TEMPLATE.format(
                tools=tools_desc,
                question=question,
                history=history_str
            )
            
            messages: list[ChatCompletionMessageParam] = [{"role": "user", "content": prompt}]
            response_text = self.client.think(messages)

            if not response_text:
                print("❌ LLM 返回为空")
                break

            thought, action = self._parse_output(response_text)
            
            if thought:
                print(f"🧠 思考: {thought}")
            
            if not action:
                print(f"⚠️ 警告: 未能解析出有效的 Action, 流程终止")
                break

            finish_match = re.match(r"Finish\[(.*)\]", action, re.DOTALL)
            if finish_match:
                final_answer = finish_match.group(1)
                print(f"🎉 最终答案: {final_answer}")
                return final_answer

            tool_name, tool_input = self._parse_action(action)
            if not tool_name or not tool_input:
                # ... 处理无效Action格式 ...
                continue

            print(f"🎬 行动: {tool_name}[{tool_input}]")
            
            tool_function = self.tool_executor.get(tool_name)
            if not tool_function:
                observation = f"错误:未找到名为 '{tool_name}' 的工具。"
            else:
                observation = tool_function(tool_input) # 调用真实工具
            print(f"👀 观察: {observation}")

            self.history.append(f"Action: {action}")
            self.history.append(f"Observation: {observation}")

        print("已达到最大步数，流程终止")
        return None

    def _parse_output(self, text: str):
        """
        解析LLM输出，提取Thought和Action
        """
        thought_match = re.search(r"Thought:\s*(.*?)(?=\nAction:|$)", text, re.DOTALL)
        action_match = re.search(r"Action:\s*(.*?)$", text, re.DOTALL)
        thought = thought_match.group(1).strip() if thought_match else None
        action = action_match.group(1).strip() if action_match else None
        
        return thought, action

    def _parse_action(self, action_text: str):
        match = re.match(r"(\w+)\[(.*)\]", action_text)
        if match:
            return match.group(1), match.group(2)
        return None, None