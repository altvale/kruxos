# Policies

By the end of this page, you'll know how to write, manage, and debug YAML policy rules that control what agents can do.

## How policies work

KruxOS uses a **deterministic policy engine** — YAML rules are compiled into a fast evaluation tree at startup. No LLM is involved in policy decisions. Every evaluation is reproducible and auditable.

### Policy hierarchy

Policies are evaluated in layers, with higher layers overriding lower ones:

```
System policies (immutable, shipped with KruxOS)
  └── Organization policies (set by admin)
       └── Agent policies (per-agent overrides)
```

A lower layer **cannot** be more permissive than a higher layer. If the system policy blocks `secrets.read_raw`, no agent policy can allow it.

## Policy templates

KruxOS ships with three templates:

| Template | Best for | Philosophy |
|----------|----------|------------|
| `personal-permissive` | Single user, personal use | Most operations autonomous, minimal friction |
| `team-moderate` | Small teams | Reads autonomous, writes notify, destructive ops need approval |
| `enterprise-restrictive` | Production deployments | All writes need approval, strict sandboxing |

### View active policy

```bash
kruxos config show policy
```

Expected output:

```
Active policy: team-moderate
Source: /etc/kruxos/policies/team-moderate.yaml
```

### Switch policy template

```bash
kruxos config set policy enterprise-restrictive
```

## Writing policy rules

### Basic structure

```yaml
# /etc/kruxos/policies/my-policy.yaml
name: my-custom-policy
version: "1.0"
description: "Custom policy for the data team"

defaults:
  tier: notify

capabilities:
  # Category-level defaults
  filesystem.*:
    tier: autonomous

  # Capability-level overrides
  filesystem.delete:
    tier: approval_required
    reason: "File deletion requires approval"

  filesystem.write:
    tier: notify

  process.*:
    tier: approval_required
    reason: "All process execution needs human review"

  secrets.*:
    tier: blocked
    reason: "Agents cannot access secrets directly"
```

### Permission tiers

| Tier | YAML value | Behaviour |
|------|-----------|-----------|
| Autonomous | `autonomous` | Executes immediately |
| Notify | `notify` | Executes, supervisor notified |
| Approval Required | `approval_required` | Queued for human approval |
| Blocked | `blocked` | Always denied |

### Conditional rules

Rules can include conditions based on parameters:

```yaml
capabilities:
  filesystem.write:
    rules:
      # Small files are autonomous
      - condition:
          content_size_lt: 10240  # 10 KB
        tier: autonomous

      # Large files need approval
      - condition:
          content_size_gte: 10240
        tier: approval_required
        reason: "Large file writes need review"
```

### Rate limits

Add rate limits to any capability:

```yaml
capabilities:
  email.send:
    tier: notify
    rate_limit:
      max: 10
      window: 3600  # 1 hour
      on_exceed: approval_required
    reason: "Email sending is rate-limited"
```

When the rate limit is exceeded, the `on_exceed` tier applies. The agent receives a `RateLimitedError` with a `retry_after` value.

### Agent-specific overrides

```yaml
# Per-agent policy: /etc/kruxos/policies/agents/deploy-bot.yaml
agent: deploy-bot
extends: team-moderate

capabilities:
  process.run:
    tier: autonomous  # Deploy bot can run processes without approval
    reason: "Deployment automation has pre-approved process access"

  git.*:
    tier: autonomous
```

Agent policies must be **equal or more restrictive** than the organization policy. This override works because `team-moderate` already allows `process.run` at `approval_required` — the agent policy is relaxing within bounds the system allows.

## Validate a policy

```bash
kruxos config validate /path/to/my-policy.yaml
```

Expected output:

```
Policy 'my-custom-policy' is valid.

  Capabilities covered: 47/47
  Default tier: notify
  Agent overrides: 1 (deploy-bot)
  Rate limits: 1 (email.send)

  Warnings:
    - No explicit rule for network.* (using default: notify)
```

## Debug policy decisions

### Check what tier a capability gets

```bash
kruxos config check-policy --agent my-agent --capability filesystem.delete
```

Expected output:

```
Agent:       my-agent
Capability:  filesystem.delete
Tier:        approval_required
Source:      team-moderate.yaml (line 15)
Hierarchy:   system(allowed) → org(approval_required) → agent(no override)
```

### View recent policy decisions

```bash
kruxos audit query --last 1h --outcome denied
```

## Apply a custom policy

1. Write your policy YAML
2. Validate it: `kruxos config validate my-policy.yaml`
3. Copy to the policies directory: `cp my-policy.yaml /etc/kruxos/policies/`
4. Activate it: `kruxos config set policy my-custom-policy`
5. Verify: `kruxos config show policy`

Policy changes take effect immediately — no restart required. The policy engine recompiles the evaluation tree when the file changes.

## Next steps

- [Approval Workflow](approval-workflow.md) — how approvals work in practice
- [Monitoring](monitoring.md) — alert on policy violations
- [Managing Agents](managing-agents.md) — per-agent policy scoping
