# Dashboard REST API

The community dashboard exposes REST endpoints for the web UI. These same endpoints can be used by custom tools and integrations.

The Gateway's three machine-facing surfaces in v0.0.1 are:

| Port | Surface | Auth |
|------|---------|------|
| 7700 | Gateway — MCP-native (`tools/list` / `tools/call`) with JSON-RPC fallback | Agent token (64-char hex) |
| 7701 | Supervision WebSocket — live activity stream consumed by the dashboard | Operator session / User token |
| 7702 | UDP trigger-wake on `127.0.0.1` — loopback only | None (loopback) |
| 7703 | User API — bearer-auth loopback for CLI / mcp-bridge / cli-hook | User token (`krx_user_*`) |
| 7800 | Dashboard — HTTPS-by-default, serves the REST endpoints documented below | Operator session |

!!! note "v0.0.1 token formats"
    v0.0.1 issues two kinds of bearer credentials:

      - **User tokens** — `krx_user_<base64url>` prefix.
      - **Agent tokens** — 64 hex characters (no prefix).

    The `krx_agent_<base64url>` prefix scheme is reserved and will become
    the issued agent-token format in v0.0.2, once the bearer-auth
    dispatcher (Gate C) lands. Today, agent tokens are verified only via
    the MCP handshake on port 7700, not via HTTP bearer auth.

## Base URL

```
https://localhost:7800/api
```

The dashboard serves HTTPS with an auto-generated self-signed certificate (browsers will prompt on first visit). All endpoints require an authenticated operator session.

## Authentication

Pass a User token as a Bearer token, or use a logged-in dashboard session cookie:

```http
GET /api/agents HTTP/1.1
Authorization: Bearer krx_user_...
```

## Endpoints

### System overview

#### `GET /api/status`

Returns system-wide status summary.

```json
{
  "version": "1.0.0",
  "uptime_seconds": 86400,
  "active_agents": 3,
  "active_sessions": 2,
  "pending_approvals": 1,
  "capabilities_invoked_24h": 1247,
  "policy_violations_24h": 3,
  "health": "healthy"
}
```

### Agents

#### `GET /api/agents`

Lists all registered agents.

```json
{
  "agents": [
    {
      "name": "research-agent",
      "purpose": "Research and analysis",
      "status": "active",
      "created_at": "2026-03-10T08:00:00Z",
      "last_connected": "2026-03-15T10:00:00Z",
      "total_invocations": 4521
    }
  ]
}
```

#### `GET /api/agents/:name`

Returns details for a specific agent including session history and invocation stats.

#### `POST /api/agents/:name/pause`

Pauses an agent (rejects new capability invocations).

#### `POST /api/agents/:name/resume`

Resumes a paused agent.

### Activity stream

#### `GET /api/activity`

Returns recent activity events (paginated).

```http
GET /api/activity?limit=50&offset=0&agent=research-agent
```

```json
{
  "events": [
    {
      "type": "capability_invoked",
      "timestamp": "2026-03-15T10:30:00.123Z",
      "agent": "research-agent",
      "capability": "filesystem.read",
      "success": true,
      "duration_ms": 12
    }
  ],
  "total": 1247,
  "limit": 50,
  "offset": 0
}
```

### Approvals

#### `GET /api/approvals`

Lists pending approval requests.

```json
{
  "approvals": [
    {
      "id": "appr-x1y2z3",
      "agent": "deploy-agent",
      "capability": "process.run",
      "arguments": { "command": "kubectl apply -f deploy.yaml" },
      "requested_at": "2026-03-15T10:32:00Z",
      "status": "pending"
    }
  ]
}
```

#### `POST /api/approvals/:id/approve`

Approves a pending request.

#### `POST /api/approvals/:id/reject`

Rejects a pending request. Optional body:

```json
{ "reason": "Not authorized to deploy to production" }
```

### Audit log

#### `GET /api/audit`

Queries the audit log with filters.

```http
GET /api/audit?agent=research-agent&capability=filesystem.*&since=2026-03-15T00:00:00Z&limit=100
```

```json
{
  "entries": [
    {
      "id": "aud-001",
      "timestamp": "2026-03-15T10:30:00.123Z",
      "agent": "research-agent",
      "capability": "filesystem.read",
      "arguments": { "path": "/workspace/data.csv" },
      "result": "success",
      "policy_decision": "autonomous",
      "hash": "sha256:abc123..."
    }
  ],
  "total": 500,
  "integrity": "verified"
}
```

### Service Proxy

#### `GET /api/proxy/status`

Returns Service Proxy status for all connected services.

```json
{
  "services": [
    {
      "name": "gmail",
      "status": "connected",
      "last_sync": "2026-03-15T10:28:00Z",
      "replica_entries": 1500,
      "buffered_writes": 2,
      "rollback_points": 15
    }
  ]
}
```

#### `GET /api/proxy/writes`

Lists buffered writes pending execution.

#### `POST /api/proxy/writes/:id/cancel`

Cancels a buffered write before it executes.

### Health

#### `GET /api/health`

Returns component-level health status.

```json
{
  "status": "healthy",
  "components": {
    "gateway": { "status": "healthy", "uptime": 86400 },
    "vault": { "status": "healthy", "locked": false },
    "audit": { "status": "healthy", "entries": 50000 },
    "proxy": { "status": "healthy", "services": 1 },
    "dashboard": { "status": "healthy" }
  },
  "resources": {
    "cpu_percent": 12.5,
    "memory_percent": 45.2,
    "disk_percent": 23.1
  }
}
```

### State explorer

#### `GET /api/state/:agent`

Lists persistent state keys for an agent.

#### `GET /api/state/:agent/:key`

Returns the value of a specific state key.

#### `DELETE /api/state/:agent/:key`

Deletes a state key (audit-logged).

#### `GET /api/state/shared`

Lists shared state keys.
