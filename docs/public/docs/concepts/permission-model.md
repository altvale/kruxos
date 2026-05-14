# Permission Model

KruxOS provides **one permission system** that controls everything an agent can do. There is no separate model-level permission prompt, no bolt-on sandbox, and no secondary authorization layer. The policy engine is the single gate between an agent and any action.

## One Permission Layer

When an agent calls a capability (read a file, send an email, access a secret), the policy engine evaluates the request against the configured rules and returns one of four decisions. The model never asks for permission separately — it either can do it or it can't.

This is fundamentally different from systems where the AI model has direct system access and a sandbox is bolted on afterward. In KruxOS, the model has *no* direct system access. Every action goes through:

```
Agent → Gateway → Policy Engine → Capability Executor → System
```

The policy engine sits between the agent and every operation. There is no escape hatch.

## How This Works Per Connection Method

### Server-side agents (autonomous)

The gateway controls everything. The model runs inside the reasoning loop, which routes every tool call through the policy engine. The model cannot bypass this — it has no shell access, no filesystem access, and no network access outside of the capabilities exposed to it.

```
Model (cloud API) → Gateway reasoning loop → Policy check → Execute
```

### Claude Desktop via MCP

When you connect Claude Desktop to KruxOS via MCP, the MCP tools are presented as user-authorized capabilities. Claude Desktop handles its own UI-level confirmations. KruxOS policy is evaluated server-side for every tool call.

```
Claude Desktop → MCP connection → Gateway → Policy check → Execute
```

### Claude Code via MCP

Similar to Claude Desktop — KruxOS tools appear as MCP tools in Claude Code. The policy engine is a separate permission domain from Claude Code's own tool approval.

### SDK agents (Python/TypeScript)

SDK agents make API calls to the gateway. The policy engine is the only gate — the SDK cannot execute capabilities without going through the gateway.

```
SDK → HTTP/WebSocket → Gateway → Policy check → Execute
```

## The Four Permission Tiers

Every capability invocation is classified into one of four tiers based on the policy rules configured for the agent:

### Autonomous

**The agent acts without asking.** The operation executes immediately, is logged to the audit trail, and the supervisor can see it in the activity stream.

**When to use:** Read operations, routine tasks, low-risk actions. Example: reading files in the agent's workspace, checking system status, reading email headers.

```yaml
# Policy rule example
- capability: "filesystem.read"
  tier: autonomous
  conditions:
    path_prefix: "/workspace/agent-name/"
```

### Notify

**The agent acts and the supervisor is notified.** The operation executes immediately, but a notification is sent to the supervision dashboard and CLI. The supervisor sees what happened in real time.

**When to use:** Actions that are generally safe but worth knowing about. Example: sending a non-urgent comms message, writing to shared state, modifying files in the workspace.

```yaml
- capability: "comms.send"
  tier: notify
```

### Approval Required

**The agent pauses and waits for human approval.** The operation is submitted to the approval queue. A supervisor must explicitly approve or reject it. The agent's reasoning loop blocks until a decision is made (with a configurable timeout).

**When to use:** High-impact or irreversible actions. Example: sending an email, deploying code, deleting files, making purchases.

```yaml
- capability: "email.send"
  tier: approval_required
  reason: "Email sends require human review"
```

### Blocked

**The agent cannot perform this action at all.** The capability call is rejected immediately with a structured error. The model receives the error and must find an alternative approach.

**When to use:** Actions that should never happen for this agent. Example: a bookkeeper agent should never access code repositories, a code agent should never send emails.

```yaml
- capability: "git.*"
  tier: blocked
  reason: "Bookkeeper agent has no access to code"
```

## How This Differs from OpenClaw

OpenClaw gives agents direct system access and adds sandboxing as a secondary constraint:

```
OpenClaw: Agent → Shell/System → Sandbox catches dangerous operations
```

KruxOS flips this model:

```
KruxOS: Agent → Policy Engine → Approved capability → System
```

The difference matters because:

1. **No ambient authority.** An agent in KruxOS cannot "try" to access something and get caught. It can only call defined capabilities, and each call is policy-checked before execution.

2. **Granular control.** You can allow an agent to read emails but not send them. You can allow file reads in one directory but not another. OpenClaw's sandbox is coarser — it's on or off per system resource.

3. **Audit everything.** Every capability call, every policy decision, every result is logged to the hash-chained audit trail. You have a complete, tamper-evident record of everything every agent did.

4. **One place to configure.** All permissions live in YAML policy files. There's no "also check the sandbox config" or "also check the model's system prompt." One system, one configuration surface.

## Configuring Permissions Per Agent

Policies are layered: system > organization > agent. Agent-specific policies override organization defaults, and system policies are immutable.

### Agent-specific policy

Create a file at `policies/agents/{agent-name}.yaml`:

```yaml
# policies/agents/bookkeeper.yaml
rules:
  # Can read and process emails autonomously
  - capability: "email.list"
    tier: autonomous
  - capability: "email.read"
    tier: autonomous

  # Email sends require approval
  - capability: "email.send"
    tier: approval_required
    reason: "All outgoing emails require review"

  # Can use shared state freely
  - capability: "state.shared.*"
    tier: notify

  # No access to code or filesystem outside workspace
  - capability: "git.*"
    tier: blocked
  - capability: "filesystem.*"
    tier: autonomous
    conditions:
      path_prefix: "/workspace/bookkeeper/"
  - capability: "filesystem.*"
    tier: blocked
    reason: "Cannot access files outside workspace"
```

### Organization-wide defaults

Set defaults in `policies/org.yaml`:

```yaml
defaults:
  tier: notify  # Default: execute but notify supervisor

overrides:
  # All agents can read their own state
  - capability: "state.persistent.*"
    tier: autonomous

  # Destructive operations always need approval
  - capability: "filesystem.delete"
    tier: approval_required
  - capability: "process.kill"
    tier: approval_required
```

## Summary

- **One permission system.** The policy engine is the only gate between agents and actions.
- **One place to configure.** YAML policy files control everything.
- **Four tiers.** Autonomous, Notify, Approval Required, Blocked.
- **No escape hatch.** Agents have no direct system access to bypass policy.
- **Full audit trail.** Every decision is logged and verifiable.
