# Gateway — MCP Protocol

The Agent Gateway speaks the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) natively on port **7700**. Claude connects with zero adapter code; other models use SDK adapters.

## Connection

```
Endpoint:  ws://localhost:7700/mcp
Transport: WebSocket (JSON frames)
Auth:      Bearer token (from `kruxos agent create`)
```

### Handshake

```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "clientInfo": {
      "name": "my-agent",
      "version": "1.0.0"
    },
    "capabilities": {}
  },
  "id": 1
}
```

### Response

```json
{
  "jsonrpc": "2.0",
  "result": {
    "protocolVersion": "2024-11-05",
    "serverInfo": {
      "name": "kruxos-gateway",
      "version": "1.0.0"
    },
    "capabilities": {
      "tools": { "listChanged": true }
    }
  },
  "id": 1
}
```

## Tool discovery

### List all tools

```json
{
  "jsonrpc": "2.0",
  "method": "tools/list",
  "params": {},
  "id": 2
}
```

Returns all capabilities the agent is permitted to use (filtered by policy):

```json
{
  "jsonrpc": "2.0",
  "result": {
    "tools": [
      {
        "name": "filesystem__read",
        "description": "Reads the content of a file at the given path...",
        "inputSchema": {
          "type": "object",
          "properties": {
            "path": { "type": "string", "description": "Absolute path..." },
            "encoding": { "type": "string", "default": "utf-8" }
          },
          "required": ["path"]
        }
      }
    ]
  },
  "id": 2
}
```

!!! note "Tool naming convention"
    MCP tool names use double underscores as separators: `filesystem__read`, `git__commit`, `email__send`. This maps to the dotted capability names (`filesystem.read` → `filesystem__read`).

## Tool invocation

### Call a tool

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "filesystem__read",
    "arguments": {
      "path": "/workspace/README.md"
    }
  },
  "id": 3
}
```

### Success response

```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "# My Project\nThis is the README..."
      }
    ]
  },
  "id": 3
}
```

### Error response

```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"error\": \"FileNotFound\", \"message\": \"No file at /workspace/missing.txt\", \"recovery\": [{\"action\": \"list_directory\", \"description\": \"Use filesystem.list to see available files.\"}]}"
      }
    ],
    "isError": true
  },
  "id": 3
}
```

## Approval flow

When a capability requires approval (`permission_tier: approval_required`), the Gateway returns a pending result:

```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"status\": \"pending_approval\", \"approval_id\": \"appr-a1b2c3\", \"capability\": \"process.run\", \"message\": \"Waiting for supervisor approval...\"}"
      }
    ]
  },
  "id": 4
}
```

The agent should poll or wait for the approval result. Once approved, the Gateway executes the capability and returns the real result.

## Session lifecycle

| Event | Method | Direction |
|-------|--------|-----------|
| Initialize | `initialize` | Client → Server |
| List tools | `tools/list` | Client → Server |
| Call tool | `tools/call` | Client → Server |
| Tool list changed | `notifications/tools/list_changed` | Server → Client |
| Session end | WebSocket close | Either |

## Rate limiting

Rate limits are applied per-agent based on policy configuration. When exceeded:

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "Rate limit exceeded: 100 calls/hour for filesystem.* capabilities",
    "data": {
      "retry_after_seconds": 45,
      "limit": 100,
      "window": "1h"
    }
  },
  "id": 5
}
```

## Authentication

Include the API key as a Bearer token in the WebSocket upgrade request:

```
GET /mcp HTTP/1.1
Host: localhost:7700
Upgrade: websocket
Connection: Upgrade
Authorization: Bearer 7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c
```

Alternatively, pass the key as a query parameter for environments that don't support custom headers:

```
ws://localhost:7700/mcp?token=7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c
```
