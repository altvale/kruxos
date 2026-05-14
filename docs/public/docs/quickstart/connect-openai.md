# Connect OpenAI

By the end of this page, a GPT model will be connected to KruxOS via the function-calling adapter.

OpenAI models use function calling to invoke KruxOS capabilities. The bundled SDK translates capability schemas into OpenAI's tool format automatically.

## OpenAI Codex CLI

If you want **Codex CLI** (subscription / device-code OAuth) rather than direct API access, use the seed config generator on the appliance:

```bash
kruxos cli-config generate --write   # writes ~/.codex/config.toml + hooks.json
```

The generator wires Codex through `mcp-bridge` + `cli-hook` so every tool call routes via the KruxOS approval queue. Codex's native `shell` and `unified_exec` tools are disabled at the user-config and requirements layers (`/etc/codex/requirements.toml`).

!!! info "Codex apply_patch routing"
    In v0.0.1, Codex's built-in `apply_patch` tool is **not yet routed through the KruxOS approval queue** — it's an upstream limitation. The MCP-proxy fix that closes that gap lands in **v0.0.4**.

## Prerequisites (direct API use)

- A running KruxOS instance ([Install](install.md))
- An agent token (64-char hex) from `kruxos agent create` or the wizard
- An OpenAI API key
- The bundled Python SDK at `/opt/kruxos/sdk/python/` (auto-importable on the appliance). The external `pip install kruxos` distribution ships in **v0.0.3**.

## Connect and use capabilities

The example below runs **on the appliance** so the bundled SDK is on `sys.path`. For host-side code, copy `/opt/kruxos/sdk/python/` off the appliance until the PyPI distribution lands in v0.0.3.

```python
import asyncio
from openai import OpenAI
from kruxos import KruxOS
from kruxos.connectors.openai import OpenAIAdapter

async def main():
    # Connect to KruxOS
    os = await KruxOS.connect_async(
        endpoint="ws://localhost:7700",
        agent_name="my-agent",
        api_key="7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c",
        purpose="OpenAI quickstart",
    )

    try:
        # Create adapter — converts KruxOS capabilities to OpenAI tools
        adapter = OpenAIAdapter(os)
        tools = await adapter.as_tools()
        print(f"Registered {len(tools)} tools with OpenAI format")

        # Use with OpenAI chat completion
        client = OpenAI()
        messages = [{"role": "user", "content": "List files in /workspace"}]

        response = client.chat.completions.create(
            model="gpt-4o",
            messages=messages,
            tools=tools,
        )

        # Execute any tool calls the model makes
        for tool_call in response.choices[0].message.tool_calls or []:
            result = await adapter.execute(
                tool_call.function.name,
                tool_call.function.arguments,
            )
            print(f"Tool: {tool_call.function.name}")
            print(f"Result: {result}")
    finally:
        await os.close_async()

asyncio.run(main())
```

Expected output (tool count reflects the policy tier on the calling agent — `blocked` capabilities are omitted, so totals vary):

```
Registered 89 tools with OpenAI format
Tool: filesystem__list
Result: {"entries": [{"name": "hello.txt", "type": "file", "size": 21}]}
```

## Tool format

The adapter converts each KruxOS capability into an OpenAI function tool:

```json
{
  "type": "function",
  "function": {
    "name": "filesystem__read",
    "description": "Reads the contents of a file at the specified path...",
    "parameters": {
      "type": "object",
      "properties": {
        "path": {"type": "string", "description": "Absolute path to the file..."},
        "encoding": {"type": "string", "description": "Character encoding..."}
      },
      "required": ["path"]
    }
  }
}
```

Capability names use `__` (double underscore) as the separator since OpenAI function names cannot contain dots.

## Next steps

- [Web Dashboard](dashboard.md) — monitor agent activity
- [Managing Agents](../guides/managing-agents.md) — create dedicated agents per model
- [Policies](../guides/policies.md) — configure what each agent can do
