# Gateway — JSON-RPC Fallback

For clients that do not support MCP, the Gateway exposes a JSON-RPC 2.0 endpoint. This provides the same capabilities with a simpler protocol.

## Connection

```
Endpoint:  http://localhost:7700/rpc
Transport: HTTP POST (JSON body)
Auth:      Bearer token in Authorization header
```

## Discovery

### List capabilities

```http
POST /rpc HTTP/1.1
Content-Type: application/json
Authorization: Bearer 7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c

{
  "jsonrpc": "2.0",
  "method": "capabilities.list",
  "params": {
    "category": "filesystem"
  },
  "id": 1
}
```

```json
{
  "jsonrpc": "2.0",
  "result": {
    "capabilities": [
      {
        "name": "filesystem.read",
        "version": "1.0",
        "purpose": "Reads the content of a file...",
        "permission_tier": "autonomous",
        "inputs": [...],
        "outputs": [...]
      }
    ]
  },
  "id": 1
}
```

### Describe a capability

```http
POST /rpc HTTP/1.1
Content-Type: application/json
Authorization: Bearer 7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c

{
  "jsonrpc": "2.0",
  "method": "capabilities.describe",
  "params": {
    "name": "filesystem.read"
  },
  "id": 2
}
```

Returns the full capability definition including `when_to_use`, `common_patterns`, and `errors`.

## Invocation

### Invoke a capability

```http
POST /rpc HTTP/1.1
Content-Type: application/json
Authorization: Bearer 7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c

{
  "jsonrpc": "2.0",
  "method": "capabilities.invoke",
  "params": {
    "capability": "filesystem.read",
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
    "success": true,
    "data": {
      "content": "# My Project...",
      "encoding": "utf-8",
      "size_bytes": 1234,
      "modified_at": "2026-03-15T10:30:00Z"
    }
  },
  "id": 3
}
```

### Error response

```json
{
  "jsonrpc": "2.0",
  "result": {
    "success": false,
    "error": {
      "type": "FileNotFound",
      "message": "No file at /workspace/missing.txt",
      "recovery": [
        {
          "action": "list_directory",
          "description": "Use filesystem.list to see available files."
        }
      ]
    }
  },
  "id": 3
}
```

## Session management

### Create session

```json
{
  "jsonrpc": "2.0",
  "method": "session.create",
  "params": {},
  "id": 1
}
```

```json
{
  "jsonrpc": "2.0",
  "result": {
    "session_id": "sess-a1b2c3d4",
    "agent": "my-agent",
    "created_at": "2026-03-15T10:00:00Z"
  },
  "id": 1
}
```

### End session

```json
{
  "jsonrpc": "2.0",
  "method": "session.end",
  "params": {
    "session_id": "sess-a1b2c3d4"
  },
  "id": 10
}
```

## Differences from MCP

| Feature | MCP | JSON-RPC |
|---------|-----|----------|
| Transport | WebSocket | HTTP POST |
| Tool names | `filesystem__read` | `filesystem.read` |
| Discovery | `tools/list` | `capabilities.list` |
| Invocation | `tools/call` | `capabilities.invoke` |
| Streaming | Supported | Not supported |
| Notifications | Server → Client push | Polling required |
| Session | Implicit (WebSocket) | Explicit (session.create) |

## Batch requests

JSON-RPC supports batch requests — send an array of request objects:

```json
[
  {"jsonrpc": "2.0", "method": "capabilities.invoke", "params": {"capability": "filesystem.read", "arguments": {"path": "/a.txt"}}, "id": 1},
  {"jsonrpc": "2.0", "method": "capabilities.invoke", "params": {"capability": "filesystem.read", "arguments": {"path": "/b.txt"}}, "id": 2}
]
```

Responses are returned as an array in the same order.

## Error codes

| Code | Meaning |
|------|---------|
| -32700 | Parse error — invalid JSON |
| -32600 | Invalid request — missing required fields |
| -32601 | Method not found |
| -32602 | Invalid params |
| -32603 | Internal error |
| -32000 | Rate limit exceeded |
| -32001 | Policy denied |
| -32002 | Approval required (pending) |
| -32003 | Authentication failed |
