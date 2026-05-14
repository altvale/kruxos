# Supervision WebSocket Events

The Gateway exposes a supervision WebSocket on port **7701**, separate from the agent port (7700). This streams real-time events to the dashboard and CLI tools.

## Connection

```
Endpoint:  wss://localhost:7701/events
Transport: WebSocket (JSON frames)
Auth:      User token (krx_user_*) as a Bearer header, or operator session
Keepalive: 30 s server ping / 10 s read timeout
Max frame: 16 MiB by default
```

```
Authorization: Bearer krx_user_...
```

!!! warning "Security"
    Port 7701 is for supervisors only. Agents (64-char hex tokens, authenticated on port 7700) cannot connect to this port — only Users (`krx_user_*`). User tokens are issued by `kruxos user-token create` or the first-boot dashboard wizard.

## Event stream

Once connected, the server streams JSON events. Each event has a standard envelope:

```json
{
  "type": "capability_invoked",
  "timestamp": "2026-03-15T10:30:00.123Z",
  "agent": "research-agent",
  "session_id": "sess-a1b2c3d4",
  "data": { ... }
}
```

## Event types

### `capability_invoked`

Fired when an agent invokes a capability.

```json
{
  "type": "capability_invoked",
  "timestamp": "2026-03-15T10:30:00.123Z",
  "agent": "research-agent",
  "session_id": "sess-a1b2c3d4",
  "data": {
    "capability": "filesystem.read",
    "arguments": { "path": "/workspace/data.csv" },
    "permission_tier": "autonomous",
    "duration_ms": 12
  }
}
```

### `capability_result`

Fired when a capability invocation completes.

```json
{
  "type": "capability_result",
  "timestamp": "2026-03-15T10:30:00.135Z",
  "agent": "research-agent",
  "session_id": "sess-a1b2c3d4",
  "data": {
    "capability": "filesystem.read",
    "success": true,
    "duration_ms": 12
  }
}
```

### `capability_error`

Fired when a capability invocation fails.

```json
{
  "type": "capability_error",
  "timestamp": "2026-03-15T10:30:01.000Z",
  "agent": "research-agent",
  "session_id": "sess-a1b2c3d4",
  "data": {
    "capability": "filesystem.write",
    "error_type": "PolicyDenied",
    "message": "Agent 'research-agent' is not permitted to write to /etc/",
    "tier": "blocked"
  }
}
```

### `policy_violation`

Fired when an agent attempts an action denied by policy.

```json
{
  "type": "policy_violation",
  "timestamp": "2026-03-15T10:31:00.000Z",
  "agent": "research-agent",
  "session_id": "sess-a1b2c3d4",
  "data": {
    "capability": "process.run",
    "rule": "system.deny_dangerous_commands",
    "reason": "Command 'rm -rf /' matches blocked pattern",
    "tier": "blocked"
  }
}
```

### `approval_requested`

Fired when an agent invokes a capability that requires approval.

```json
{
  "type": "approval_requested",
  "timestamp": "2026-03-15T10:32:00.000Z",
  "agent": "deploy-agent",
  "session_id": "sess-e5f6g7h8",
  "data": {
    "approval_id": "appr-x1y2z3",
    "capability": "process.run",
    "arguments": { "command": "kubectl apply -f deploy.yaml" },
    "reason": "process.run requires approval for deploy-agent"
  }
}
```

### `approval_resolved`

Fired when an approval request is accepted or rejected.

```json
{
  "type": "approval_resolved",
  "timestamp": "2026-03-15T10:33:00.000Z",
  "agent": "deploy-agent",
  "data": {
    "approval_id": "appr-x1y2z3",
    "decision": "approved",
    "decided_by": "admin",
    "decided_at": "2026-03-15T10:33:00.000Z"
  }
}
```

### `session_started`

```json
{
  "type": "session_started",
  "timestamp": "2026-03-15T10:00:00.000Z",
  "agent": "research-agent",
  "data": {
    "session_id": "sess-a1b2c3d4",
    "agent_purpose": "Research and analysis agent"
  }
}
```

### `session_ended`

```json
{
  "type": "session_ended",
  "timestamp": "2026-03-15T11:00:00.000Z",
  "agent": "research-agent",
  "data": {
    "session_id": "sess-a1b2c3d4",
    "duration_seconds": 3600,
    "capabilities_invoked": 47
  }
}
```

### `rate_limit_hit`

```json
{
  "type": "rate_limit_hit",
  "timestamp": "2026-03-15T10:45:00.000Z",
  "agent": "scraper-agent",
  "data": {
    "capability_pattern": "network.*",
    "limit": 100,
    "window": "1h",
    "retry_after_seconds": 120
  }
}
```

### `write_buffered`

Fired when a Service Proxy write is buffered (email send, delete, etc.).

```json
{
  "type": "write_buffered",
  "timestamp": "2026-03-15T10:50:00.000Z",
  "agent": "assistant-agent",
  "data": {
    "write_id": "wr-m1n2o3",
    "service": "gmail",
    "operation": "email.send",
    "buffer_until": "2026-03-15T10:55:00.000Z",
    "cancellable": true
  }
}
```

### `health_alert`

```json
{
  "type": "health_alert",
  "timestamp": "2026-03-15T10:55:00.000Z",
  "agent": null,
  "data": {
    "component": "disk",
    "severity": "warning",
    "message": "Disk usage at 85%",
    "threshold": "80%"
  }
}
```

## Filtering

Add query parameters to filter the event stream:

```
ws://localhost:7701/events?passphrase=...&agent=research-agent
ws://localhost:7701/events?passphrase=...&type=capability_invoked,policy_violation
ws://localhost:7701/events?passphrase=...&agent=deploy-agent&type=approval_requested
```

| Parameter | Description |
|-----------|-------------|
| `agent` | Filter to events from a specific agent |
| `type` | Comma-separated list of event types to include |
| `session` | Filter to a specific session ID |

## Event summary

| Event | Fired when | Typical frequency |
|-------|------------|-------------------|
| `capability_invoked` | Agent calls a capability | High (every invocation) |
| `capability_result` | Capability returns successfully | High |
| `capability_error` | Capability fails | Low-medium |
| `policy_violation` | Policy blocks an action | Low |
| `approval_requested` | Capability needs approval | Low |
| `approval_resolved` | Approval decided | Low |
| `session_started` | Agent connects | Low |
| `session_ended` | Agent disconnects | Low |
| `rate_limit_hit` | Agent hits rate limit | Low |
| `write_buffered` | Proxy buffers a write | Low |
| `health_alert` | System health issue | Rare |
