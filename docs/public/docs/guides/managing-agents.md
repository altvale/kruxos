# Managing Agents

By the end of this page, you'll know how to create, monitor, and manage agents throughout their lifecycle.

## Agent lifecycle

```mermaid
stateDiagram-v2
    [*] --> Registered: kruxos agent create
    Registered --> Active: Agent connects
    Active --> Paused: kruxos pause <name>
    Paused --> Active: kruxos resume <name>
    Active --> Idle: Agent disconnects
    Idle --> Active: Agent reconnects
    Active --> Revoked: kruxos agent revoke
    Idle --> Revoked: kruxos agent revoke
    Revoked --> [*]
```

## Create an agent

Each agent gets a unique name and API key. Create agents for different purposes:

```bash
# General-purpose agent
kruxos agent create --name my-agent --purpose "General assistant"

# Dedicated deployment agent
kruxos agent create --name deploy-bot --purpose "CI/CD automation"

# Monitoring agent
kruxos agent create --name monitor --purpose "System health monitoring"
```

!!! tip "One agent per purpose"
    Create separate agents for different tasks. Each agent gets its own state, audit trail, and policy scope. This makes it easy to revoke access or adjust permissions for a specific use case.

## Monitor agents

### CLI

```bash
# Quick overview
kruxos agent list

# Detailed view for one agent
kruxos agent show deploy-bot

# Live dashboard (auto-refreshing)
kruxos agents
```

### Dashboard

Navigate to **Agents** in the web dashboard at `http://localhost:7800/agents`. You'll see:

- Connection status (active, idle, revoked)
- Session duration
- Invocation count and last activity
- Quick actions (pause, resume, kill, revoke)

### Live activity

Watch what an agent is doing in real time:

```bash
# All agents
kruxos watch

# Filter to one agent
kruxos watch --agent deploy-bot
```

## Session management

In v0.0.1 the session-control subcommands live at the top level of the CLI (`kruxos pause / resume / kill <name>`), not under a `kruxos session` namespace.

### Pause an agent

Temporarily freeze an agent's session. All capability calls return `SessionPaused` errors until resumed:

```bash
kruxos pause my-agent
```

The agent stays connected but cannot invoke capabilities.

### Resume an agent

```bash
kruxos resume my-agent
```

### Kill a session

Force-disconnect an agent:

```bash
kruxos kill my-agent
```

The agent can reconnect with the same token. To permanently block reconnection, revoke the agent.

## Credential management

### Rotate API key

If an API key may be compromised, rotate it immediately:

```bash
kruxos agent rotate my-agent
```

This invalidates the old key and issues a new one. The agent will need to be reconfigured with the new key.

### Revoke an agent

Permanently disable an agent. Active sessions are terminated, the API key is invalidated, and the agent cannot reconnect:

```bash
kruxos agent revoke deploy-bot
```

!!! warning
    Revocation is permanent. The agent's state and audit logs are preserved, but the agent cannot be re-activated. Create a new agent if you need to restore access.

## Agent state

Each agent has its own persistent state (key-value store) that survives disconnections:

```bash
# List state keys
kruxos state list my-agent

# Read a value
kruxos state get my-agent last_deploy

# Set a value (for debugging)
kruxos state set my-agent debug_flag '{"enabled": true}'

# Delete a key
kruxos state delete my-agent debug_flag
```

State modifications via the CLI are audit-logged. This is a supervisory tool for debugging, not the primary way agents interact with state.

### State quotas

Each agent has a configurable state quota (default: 100 MB). Check usage:

```bash
kruxos state quota my-agent
```

Expected output:

```
Agent:     my-agent
Used:      2.4 MB / 100.0 MB (2.4%)
Keys:      47
```

## Context briefings

When an agent reconnects after a disconnection, KruxOS generates a **context briefing** — a summary of what changed while the agent was away:

- New approval decisions
- State changes by other agents (shared state)
- Policy updates
- System events (updates, restarts, service changes)

Briefings are rule-based and deterministic — no AI summarisation required. The agent receives the briefing automatically on reconnect via `agent.briefing`.

## Next steps

- [Approval Workflow](approval-workflow.md) — handle operations that need human review
- [Policies](policies.md) — control what each agent can do
- [Monitoring](monitoring.md) — health checks and alerts
