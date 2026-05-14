# Connect Gemini

The Gemini adapter translates KruxOS capabilities into Google's function-calling format.

## Setup

```python
import google.generativeai as genai
from kruxos import KruxOS
from kruxos.connectors.gemini import GeminiAdapter

genai.configure(api_key="your-gemini-key")
agent = KruxOS.connect("localhost:7700", api_key="<64-char hex>")
adapter = GeminiAdapter(agent)
```

## Usage

```python
# Convert capabilities to Gemini tool format
tools = adapter.as_tools()

model = genai.GenerativeModel("gemini-2.0-flash", tools=tools)
chat = model.start_chat()

response = chat.send_message("List files in /workspace")

# Handle function calls
for part in response.parts:
    if hasattr(part, "function_call"):
        result = adapter.handle_function_call(part.function_call)
        response = chat.send_message(
            genai.protos.Content(
                parts=[genai.protos.Part(
                    function_response=genai.protos.FunctionResponse(
                        name=part.function_call.name,
                        response=result,
                    )
                )]
            )
        )
```

## Agentic loop

```python
response = chat.send_message("Find and summarise all Python files in /workspace")

while True:
    function_calls = [p for p in response.parts if hasattr(p, "function_call")]
    if not function_calls:
        print(response.text)
        break

    parts = []
    for fc in function_calls:
        result = adapter.handle_function_call(fc)
        parts.append(genai.protos.Part(
            function_response=genai.protos.FunctionResponse(
                name=fc.name,
                response=result,
            )
        ))

    response = chat.send_message(genai.protos.Content(parts=parts))
```

## Type mapping

Gemini uses uppercase type names. The adapter handles this automatically:

| KruxOS type | Gemini type |
|-------------|-------------|
| `String` | `STRING` |
| `Integer` | `INTEGER` |
| `Boolean` | `BOOLEAN` |
| `Object` | `OBJECT` |
| `Array` | `ARRAY` |

## Tool name format

Same convention as MCP and OpenAI — double underscores:

| Capability | Gemini function name |
|------------|---------------------|
| `filesystem.read` | `filesystem__read` |
| `git.commit` | `git__commit` |
| `state.persistent.get` | `state__persistent__get` |
