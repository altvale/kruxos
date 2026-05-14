# Connect Local Models

The local model adapter connects KruxOS to Ollama and any OpenAI-compatible inference server.

## Ollama

### Setup

```python
from ollama import Client
from kruxos import KruxOS
from kruxos.connectors.local import LocalAdapter

ollama = Client()
agent = KruxOS.connect("localhost:7700", api_key="<64-char hex>")
adapter = OllamaAdapter(agent)
```

### Usage

```python
tools = adapter.as_tools()

response = ollama.chat(
    model="llama3.1:70b",
    messages=[{"role": "user", "content": "Read /workspace/README.md"}],
    tools=tools,
)

if response.message.tool_calls:
    for tool_call in response.message.tool_calls:
        result = adapter.handle_tool_call(tool_call)
```

### Agentic loop

```python
messages = [{"role": "user", "content": "List all Python files in /workspace"}]

while True:
    response = ollama.chat(
        model="llama3.1:70b",
        messages=messages,
        tools=tools,
    )

    messages.append(response.message)

    if not response.message.tool_calls:
        print(response.message.content)
        break

    for tool_call in response.message.tool_calls:
        result = adapter.handle_tool_call(tool_call)
        messages.append({
            "role": "tool",
            "content": str(result),
        })
```

## OpenAI-compatible servers

Any server that implements the OpenAI chat completions API with function calling works with the OpenAI adapter:

### vLLM

```python
from openai import OpenAI
from kruxos.connectors.openai import OpenAIAdapter

client = OpenAI(base_url="http://localhost:8000/v1", api_key="unused")
adapter = OpenAIAdapter(agent)
tools = adapter.as_tools()

response = client.chat.completions.create(
    model="meta-llama/Llama-3.1-70B-Instruct",
    messages=[{"role": "user", "content": "..."}],
    tools=tools,
)
```

### LM Studio

```python
client = OpenAI(base_url="http://localhost:1234/v1", api_key="lm-studio")
# Same adapter usage as above
```

### llama.cpp server

```python
client = OpenAI(base_url="http://localhost:8080/v1", api_key="unused")
# Same adapter usage as above
```

## Model recommendations

| Model | Size | Tool-calling quality | Notes |
|-------|------|---------------------|-------|
| Llama 3.1 70B | 70B | Good | Best open-source option for tool use |
| Llama 3.1 8B | 8B | Moderate | Acceptable for simple tasks |
| Mistral Large | 123B | Good | Strong function calling |
| Qwen2.5 72B | 72B | Good | Strong reasoning |
| Command R+ | 104B | Good | Built for tool use |

!!! tip "Structured errors help smaller models"
    KruxOS's structured error responses with explicit recovery suggestions significantly improve smaller models' ability to recover from failures — they don't have to guess what went wrong.

## Verifying connection

```bash
kruxos status
```

Local model agents appear the same as any other agent in the status output.
