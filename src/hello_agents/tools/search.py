from serpapi import SerpApiClient
import os

def search(query: str) -> str:
    print(f"🔍 正在执行 [SerpApi] 网页搜索: {query}")

    try:
        api_key = os.getenv("SERPAPI_API_KEY")
        if not api_key:
            return "❌ 错误: SERPAPI_API_KEY 未在 .env 文件中设置"
        
        params = {
            "engine": "google",
            "q": query,
            "api_key": api_key,
            "gl": "cn",
            "hl": "zh-cn",
        }

        client = SerpApiClient(params)
        results = client.get_dict()

        if "answer_box_list" in results:
            return "\n".join(results["answer_box_list"])
        if "answer_box" in results and "answer" in results["answer_box"]:
            return results["answer_box"]["answer"]
        if "knowledge_graph" in results and "description" in results["knowledge_graph"]:
            return results["knowledge_graph"]["description"]
        if "organic_results" in results and results["organic_results"]:
            snippets = [
                f"[{i+1}] {res.get('title', '')}\n{res.get('snippet', '')}"
                for i, res in enumerate(results["organic_results"][:5])
            ]
            return "\n\n".join(snippets)
        
        return "❌ 未找到相关结果"
        
    except Exception as e:
        return f"❌ 搜索时发生错误: {e}"

if __name__ == '__main__':
    from hello_agents.tools import ToolExecutor

    tool_executor = ToolExecutor()
    search_description = "一个网页搜索引擎。当你需要回答关于时事、事实以及在你的知识库中找不到的信息时，应使用此工具。"
    tool_executor.register("Search", search_description, search)
    
    print("\n--- 可用的工具 ---")
    print(tool_executor.list())

    print("\n--- 执行 Action: Search['英伟达最新的GPU型号是什么'] ---")
    tool_name = "Search"
    tool_input = "英伟达最新的GPU型号是什么"

    tool_function = tool_executor.get(tool_name)
    if tool_function:
        observation = tool_function(tool_input)
        print("👀 --- 观察 (Observation) ---")
        print(observation)
    else:
        print(f"❌ 错误:未找到名为 '{tool_name}' 的工具。")
        
