# Connect Local Models

By the end of this page, a locally-running model (via Ollama, vLLM, LM Studio, or llama.cpp) will be connected to KruxOS.

KruxOS v0.0.1 ships local-provider support via the same OpenAI-compatible function-calling adapter — point any server that speaks that format at the appliance.

## Prerequisites

- A running KruxOS instance ([Install](install.md))
- An agent token (64-char hex) from `kruxos agent create` or the wizard
- One of: [Ollama](https://ollama.ai), [vLLM](https://docs.vllm.ai), LM Studio, or llama.cpp's `server` running and reachable from the appliance
- The bundled Python SDK at `/opt/kruxos/sdk/python/` (auto-importable on the appliance). The external `pip install kruxos` distribution ships in **v0.0.3**.

## Pull a model

```bash
ollama pull llama3.1
```

Expected output:

```
pulling manifest
pulling 8eeb52dfb3bb... 100% ▕████████████████▏ 4.7 GB
...
success
```

## Connect and use capabilities

```python
import asyncio
import ollama
from kruxos import KruxOS
from kruxos.connectors.local import LocalAdapter

async def main():
    # Connect to KruxOS
    os = await KruxOS.connect_async(
        endpoint="ws://localhost:7700",
        agent_name="my-agent",
        api_key="7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c",
        purpose="Local model quickstart",
    )

    try:
        # Create adapter — uses OpenAI-compatible format
        adapter = OllamaAdapter(os)
        tools = adapter.as_tools()
        print(f"Registered {len(tools)} tools")

        # Use with Ollama
        response = ollama.chat(
            model="llama3.1",
            messages=[{"role": "user", "content": "List files in /workspace"}],
            tools=tools,
        )

        # Execute any tool calls
        for tool_call in response["message"].get("tool_calls", []):
            result = await adapter.execute(
                tool_call["function"]["name"],
                tool_call["function"]["arguments"],
            )
            print(f"Tool: {tool_call['function']['name']}")
            print(f"Result: {result}")
    finally:
        await os.close_async()

asyncio.run(main())
```

Expected output:

```
Registered 89 tools
Tool: filesystem__list
Result: {"entries": [{"name": "hello.txt", "type": "file", "size": 21}]}
```

## Other OpenAI-compatible servers

The Ollama adapter works with any server that speaks the OpenAI tool-calling format:

- **vLLM**: `pip install vllm` and point to its endpoint
- **llama.cpp server**: use `--api-like-oai` flag
- **LM Studio**: enable the server and use the Ollama adapter

The adapter produces standard OpenAI function-calling tool definitions, compatible with any server that accepts that format.

## Next steps

- [Web Dashboard](dashboard.md) — monitor agent activity
- [Managing Agents](../guides/managing-agents.md) — create dedicated agents per model
- [Policies](../guides/policies.md) — configure what each agent can do
