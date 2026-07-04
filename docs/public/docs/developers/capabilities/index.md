# Capability Reference

Auto-generated from `definitions/*.yaml`. **89 typed capabilities across 13 categories** in v0.0.1. Every capability available to agents through the KruxOS Gateway is listed here.

Capabilities are surfaced over MCP (`tools/list`) and JSON-RPC (`capabilities.list`). Each entry is annotated with its policy tier; capabilities at the `blocked` tier are omitted from listing entirely.

## Permission tiers

| Tier | Meaning |
|------|---------|
| 🟢 `autonomous` | Agent can invoke freely — no supervisor notification |
| 🔵 `notify` | Agent can invoke, supervisor is notified after the fact |
| 🟡 `approval_required` | Agent must wait for supervisor approval before execution (default 24-hour hold for User MCP calls) |
| 🔴 `blocked` | Blocked unconditionally — requires explicit policy override to surface |

## Categories (89 capabilities)

| Category | Capabilities | Description |
|----------|:------------:|-------------|
| [Agent](agent.md) | 4 | Agent identity, capability discovery, session info, and policy inspection. |
| [Alerts](alerts.md) | 3 | Send, list, and acknowledge supervisor alerts. |
| [Communications](comms.md) | 4 | Agent-to-agent messaging, broadcast, pub/sub channels, and inbox polling. |
| [Email (Gmail)](email.md) | 7 | Search, read, send, delete, move, and draft emails via the Service Proxy. |
| [Filesystem](filesystem.md) | 12 | Read, write, search, copy, move, delete, watch, and restore files and directories. |
| [Git](git.md) | 8 | Clone, pull, push, commit, diff, log, branch, and stash management. |
| [Network](network.md) | 4 | HTTP requests, DNS lookups, file downloads, and port checks. |
| [Process](process.md) | 5 | Execute commands, monitor background processes, and retrieve output. |
| [Scheduler](scheduler.md) | 4 | Create, list, and delete cron-scheduled invocations, and delay execution. |
| [Secrets](secrets.md) | 3 | List available secrets and request rotation. Raw values are never exposed (use-not-read contract). |
| [Slack](slack.md) | 7 | Search, read, send, reply, react, and list Slack channels via the Service Proxy. |
| [State & Memory](state.md) | 24 | Session state, persistent state, shared cross-agent state, snapshots, briefings, and backups. |
| [System](system.md) | 4 | System information, health checks, metrics, and time. |

## Discovering capabilities at runtime

From inside an MCP client:

```jsonc
// MCP request
{ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }
```

From the JSON-RPC fallback path:

```jsonc
{ "jsonrpc": "2.0", "id": 1, "method": "capabilities.list" }
```

Each returned capability includes its `policy_tier` annotation. The v0.0.1 CLI does not ship a dedicated `kruxos cap` subcommand for listing — use `tools/list` / `capabilities.list` over the Gateway, or inspect the live registry from the dashboard `/agents` page where each agent's effective surface is rendered with its tier badges.
