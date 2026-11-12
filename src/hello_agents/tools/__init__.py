from typing import Dict, Any, Callable
from .search import search

class ToolExecutor:
    def __init__(self):
        self.tools: Dict[str, Dict[str, Any]] = {}

    def register(self, name: str, description: str, func: Callable):
        if name in self.tools:
            print(f"⚠️ 警告: 工具 {name} 已存在，将被覆盖")
        self.tools[name] = {"description": description, "func": func}
        print(f"✅ 工具 {name} 注册成功")

    def get(self, name: str):
        return self.tools.get(name, {}).get("func")

    def list(self):
        return "\n".join([
            f"- {name}: {info['description']}"
            for name, info in self.tools.items()
        ])


__all__ = ["search", "ToolExecutor"]

