# Agent Capabilities

Agent identity, capability discovery, session info, and policy inspection.

## Overview

| Capability | Permission | Purpose |
|------------|:----------:|---------|
| [`agent.whoami`](#agentwhoami) | 🟢 Autonomous | Returns the current agent's identity, permissions, and workspace path. |
| [`agent.capabilities`](#agentcapabilities) | 🟢 Autonomous | Lists all capabilities available to the current agent with their permission tiers. |
| [`agent.session`](#agentsession) | 🟢 Autonomous | Returns detailed metadata about the current session: duration, invocation count, resource usage, and workspace boundaries. |
| [`agent.policy`](#agentpolicy) | 🟢 Autonomous | Returns the compiled policy rules that apply to the current agent. |

## `agent.whoami`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Returns the current agent's identity, permissions, and workspace path.

### When to use

Use agent.whoami at the start of a session to discover your identity, what workspace
you have access to, and your policy group. Use agent.capabilities to see what capabilities
are available. Use agent.session for session-specific details.

### Inputs

*No inputs required.*

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `agent_name` | `AgentId` | The agent's registered name. |
| `purpose` | `String` | The agent's stated purpose (set at creation). |
| `policy_group` | `String` | The policy group this agent belongs to (e.g. 'default', 'restricted'). |
| `workspace_path` | `FilesystemPath` | Absolute path to the agent's workspace directory. |
| `created_at` | `DateTime` | When the agent identity was created. |

### Common patterns

**Orient at session start**

1. `agent.whoami() to discover identity and workspace`
2. `agent.capabilities() to see available tools`
3. `agent.session() for session state`

### Errors

**`SessionError`** — No active session context available.

- **reconnect**: Ensure the agent is properly connected and authenticated.

**Tags:** `agent` `identity` `safe` `read`

---

## `agent.capabilities`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Lists all capabilities available to the current agent with their permission tiers.

### When to use

Use agent.capabilities to discover what tools you can use and what permission level
each one requires. Use this before attempting an operation to check if you have access.
Use agent.policy for the full compiled policy rules.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `category` | `String` | No | — | Filter by capability category (e.g. 'filesystem', 'network', 'git'). Omit to list all. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `capabilities` | `Array` | Array of capability objects, each with: name, purpose, permission_tier ('autonomous', 'notify', 'approval_required', 'blocked'), tags. |
| `total` | `Integer` | Total number of capabilities matching the filter. |

### Common patterns

**Discover available filesystem operations**

1. `agent.capabilities(category='filesystem')`

**Check permission tier before a risky operation**

1. `agent.capabilities(category='git')`
2. `If git.push is 'approval_required', prepare the commit first then push`

### Errors

**`SessionError`** — No active session context available.

- **reconnect**: Ensure the agent is properly connected and authenticated.

**Tags:** `agent` `discovery` `safe` `read`

---

## `agent.session`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Returns detailed metadata about the current session: duration, invocation count, resource usage, and workspace boundaries.

### When to use

Use agent.session to check your session state, resource consumption, and quota usage.
Use agent.whoami for identity-level information. Use agent.capabilities for available tools.

### Inputs

*No inputs required.*

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `session_id` | `SessionId` | The current session's unique identifier. |
| `connected_at` | `DateTime` | When this session was established. |
| `duration_seconds` | `Integer` | Session duration in seconds. |
| `invocation_count` | `Integer` | Total number of capability invocations in this session. |
| `resource_usage` | `Object` | Session resource usage: {cpu_ms, memory_bytes, io_bytes_read, io_bytes_written}. |
| `workspace_path` | `FilesystemPath` | Absolute path to the agent's workspace directory. |
| `state_quota` | `Object` | Persistent state quota: {used_bytes, limit_bytes, percent}. |

### Common patterns

**Monitor resource usage during a long operation**

1. `agent.session() to check resource_usage and state_quota`
2. `If quota percent is high, clean up unused state keys`

### Errors

**`SessionError`** — No active session context available.

- **reconnect**: Ensure the agent is properly connected and authenticated.

**Tags:** `agent` `session` `safe` `read`

---

## `agent.policy`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Returns the compiled policy rules that apply to the current agent.

### When to use

Use agent.policy to understand what rules govern your behaviour: which capabilities
require approval, which are blocked, and what rate limits apply. Use agent.capabilities
for a simpler view of just the permission tiers.

### Inputs

*No inputs required.*

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `policy_group` | `String` | The agent's policy group. |
| `rules` | `Array` | Array of rule objects applying to this agent, each with: capability (pattern), tier, reason, rate_limit. |
| `default_tier` | `String` | The default permission tier for capabilities not covered by a specific rule. |

### Common patterns

**Understand restrictions before starting work**

1. `agent.policy() to see all rules`
2. `Check for blocked capabilities or rate limits`

### Errors

**`SessionError`** — No active session context available.

- **reconnect**: Ensure the agent is properly connected and authenticated.

**Tags:** `agent` `policy` `safe` `read`

---
