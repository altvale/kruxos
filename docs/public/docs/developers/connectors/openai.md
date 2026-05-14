# Connect OpenAI (GPT-4 / GPT-4o)

The OpenAI adapter translates KruxOS capabilities into OpenAI's function-calling format.

## Setup

```python
from openai import OpenAI
from kruxos import KruxOS
from kruxos.connectors.openai import OpenAIAdapter

client = OpenAI()
agent = KruxOS.connect("localhost:7700", api_key="<64-char hex>")
adapter = OpenAIAdapter(agent)
```

## Usage

```python
# Convert capabilities to OpenAI function format
tools = adapter.as_tools()

# Chat completion with function calling
response = client.chat.completions.create(
    model="gpt-4o",
    messages=[{"role": "user", "content": "Read /workspace/README.md"}],
    tools=tools,
)

# Handle function calls
for choice in response.choices:
    if choice.message.tool_calls:
        for tool_call in choice.message.tool_calls:
            result = adapter.handle_tool_call(tool_call)
            # Feed result back to the conversation
```

## Agentic loop

```python
import json

messages = [{"role": "user", "content": "Summarise all .py files in /workspace"}]

while True:
    response = client.chat.completions.create(
        model="gpt-4o",
        messages=messages,
        tools=tools,
    )

    msg = response.choices[0].message
    messages.append(msg)

    if not msg.tool_calls:
        print(msg.content)
        break

    for tool_call in msg.tool_calls:
        result = adapter.handle_tool_call(tool_call)
        messages.append({
            "role": "tool",
            "tool_call_id": tool_call.id,
            "content": json.dumps(result),
        })
```

## Tool name format

OpenAI function names use double underscores (same as MCP):

| Capability | OpenAI function name |
|------------|---------------------|
| `filesystem.read` | `filesystem__read` |
| `git.log` | `git__log` |
| `email.search` | `email__search` |

## Function schema

The adapter generates OpenAI-compatible function schemas from capability definitions:

```json
{
  "type": "function",
  "function": {
    "name": "filesystem__read",
    "description": "Reads the content of a file at the given path.",
    "parameters": {
      "type": "object",
      "properties": {
        "path": {
          "type": "string",
          "description": "Absolute path to the file to read."
        },
        "encoding": {
          "type": "string",
          "description": "Character encoding. Defaults to utf-8."
        }
      },
      "required": ["path"]
    }
  }
}
```

## Error handling

Errors are returned as structured JSON in the function result:

```json
{
  "success": false,
  "error": {
    "type": "FileNotFound",
    "message": "No file at /workspace/missing.txt",
    "recovery": [
      {"action": "list_directory", "description": "Use filesystem.list to see available files."}
    ]
  }
}
```

GPT models can read these structured errors and take appropriate recovery actions.
