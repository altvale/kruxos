# Connect Local Models

By the end of this page, a locally-running model (via Ollama, vLLM, LM Studio, or llama.cpp) will drive KruxOS tools.

This page covers the **SDK connector** direction: *your* code runs the model loop
(here, calling Ollama directly) and connects to KruxOS to fetch and execute tools.
KruxOS is the tool server; your script is the agent. The connector turns KruxOS
capabilities into OpenAI-format function-calling tool definitions that any
OpenAI-tool-calling-compatible client — including Ollama's — accepts.

!!! note "Connector vs. model provider — two different things"
    There are **two** ways to use a local model with KruxOS:

    - **This page (SDK connector):** your script runs the model and calls KruxOS for
      tools. Use the `kruxos` Python SDK's `LocalAdapter`.
    - **Model provider:** KruxOS runs the agent itself and calls your local model as
      its backend. For that, register an `ollama` provider — see
      [Model Providers → Ollama](../guides/model-providers.md#ollama-local). Note the
      provider talks to Ollama's **native** API (`http://host:11434`, no `/v1`);
      OpenAI-compatible servers (vLLM, LM Studio, llama.cpp) register as an `openai`
      provider with a `/v1` base URL.

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
        # Create adapter — emits OpenAI-format tool definitions
        adapter = LocalAdapter(os)
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

`LocalAdapter` works with any server that speaks the OpenAI tool-calling format:

- **vLLM**: `pip install vllm` and point to its endpoint
- **llama.cpp server**: use `--api-like-oai` flag
- **LM Studio**: enable the server and use `LocalAdapter`

The adapter produces standard OpenAI function-calling tool definitions, compatible with any server that accepts that format.

## Tuning the on-appliance inference engine

Separately from the connector and provider paths above, a KruxOS appliance can run
its **own** built-in inference engine — a bundled `llama.cpp` server — so dashboard
Chat and other features work with no external model service. When an operator has
enabled the engine, its runtime behaviour is tunable through an optional environment
file at `/data/kruxos/inference.env`.

A documented template ships read-only at `/opt/kruxos/inference/inference.env.example`.
Copy it, set the keys you want, then restart the engine:

```bash
cp /opt/kruxos/inference/inference.env.example /data/kruxos/inference.env
vi /data/kruxos/inference.env
systemctl restart kruxos-inference
```

Every key is optional — an unset or empty key falls back to the baked default shown
in brackets:

| Key | Default | Effect |
|-----|---------|--------|
| `KRUXOS_INFERENCE_PARALLEL` | `1` | Concurrent inference slots. The single-slot default avoids a multi-vCPU interrupt storm on small VMs; raise it only if you genuinely serve concurrent requests. |
| `KRUXOS_INFERENCE_THREADS` | auto | Worker threads. Unset auto-detects the host's physical cores (hyperthread siblings excluded); set a number (e.g. `2`) to be a quieter neighbour on a shared VM. Fewer threads is slower, not faster. |
| `KRUXOS_INFERENCE_POLL` | `50` | Threadpool busy-poll level, 0–100. `0` sleeps at the work barrier (lowest idle CPU); `100` spins a whole core. |
| `KRUXOS_INFERENCE_EXTRA_ARGS` | _(none)_ | Extra raw `llama-server` flags, appended verbatim and split on spaces — e.g. `--ctx-size 8192 --no-warmup`. A bad flag stops the engine from starting. |

!!! tip "If a chat turn hangs or the appliance gets laggy"
    Apply the levers in order, restarting after each: first set
    `KRUXOS_INFERENCE_THREADS=2`, then (if it still struggles) `KRUXOS_INFERENCE_POLL=100`.
    Re-test a chat turn and watch the softirq (`%si`) row in `top`.

## Next steps

- [Web Dashboard](dashboard.md) — monitor agent activity
- [Managing Agents](../guides/managing-agents.md) — create dedicated agents per model
- [Policies](../guides/policies.md) — configure what each agent can do
