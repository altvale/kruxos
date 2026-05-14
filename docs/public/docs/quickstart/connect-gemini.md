# Connect Gemini

By the end of this page, a Google Gemini model will be connected to KruxOS via the function declaration adapter.

## Prerequisites

- A running KruxOS instance ([Install](install.md))
- An agent token (64-char hex) from `kruxos agent create` or the wizard
- A Google AI API key
- The bundled Python SDK at `/opt/kruxos/sdk/python/` (auto-importable on the appliance). The external `pip install kruxos` distribution ships in **v0.0.3**.
- The `google-generativeai` package — install on the appliance via `python3 -m pip install --user google-generativeai`

## Connect and use capabilities

The example below runs **on the appliance** so the bundled SDK is on `sys.path`. For host-side code, copy `/opt/kruxos/sdk/python/` off the appliance until the PyPI distribution lands in v0.0.3.

```python
import asyncio
import google.generativeai as genai
from kruxos import KruxOS
from kruxos.connectors.gemini import GeminiAdapter

async def main():
    # Connect to KruxOS
    os = await KruxOS.connect_async(
        endpoint="ws://localhost:7700",
        agent_name="my-agent",
        api_key="7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c",
        purpose="Gemini quickstart",
    )

    try:
        # Create adapter — converts capabilities to Gemini function declarations
        adapter = GeminiAdapter(os)
        tools = await adapter.as_tools()
        print(f"Registered {len(tools)} tools with Gemini format")

        # Use with Gemini
        genai.configure(api_key="your-google-api-key")
        model = genai.GenerativeModel(
            "gemini-pro",
            tools=tools,
        )

        chat = model.start_chat()
        response = chat.send_message("List files in /workspace")

        # Execute any function calls the model makes
        for part in response.parts:
            if fn := part.function_call:
                result = await adapter.execute(fn.name, dict(fn.args))
                print(f"Tool: {fn.name}")
                print(f"Result: {result}")
    finally:
        await os.close_async()

asyncio.run(main())
```

Expected output:

```
Registered 89 tools with Gemini format
Tool: filesystem__list
Result: {"entries": [{"name": "hello.txt", "type": "file", "size": 21}]}
```

## Tool format

The adapter converts KruxOS capabilities into Gemini function declarations with uppercase types:

```json
{
  "function_declarations": [
    {
      "name": "filesystem__read",
      "description": "Reads the contents of a file...",
      "parameters": {
        "type": "OBJECT",
        "properties": {
          "path": {"type": "STRING", "description": "Absolute path..."},
          "encoding": {"type": "STRING", "description": "Character encoding..."}
        },
        "required": ["path"]
      }
    }
  ]
}
```

## Next steps

- [Web Dashboard](dashboard.md) — monitor agent activity
- [Managing Agents](../guides/managing-agents.md) — create dedicated agents per model
- [Policies](../guides/policies.md) — configure what each agent can do
