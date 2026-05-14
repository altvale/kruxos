# Web Dashboard

By the end of this page, you'll know how to use the KruxOS web dashboard to monitor agents, approve operations, and inspect audit logs.

## Access the dashboard

Open your browser to:

```
https://localhost:7800
```

The dashboard ships **HTTPS-by-default** with an auto-generated self-signed certificate; browsers will prompt to accept it on first visit. For production deployments, terminate TLS upstream (nginx, Caddy) with a trusted certificate.

### Docker users

If the dashboard is not loading, verify it started:

```bash
docker exec kruxos kruxos verify
```

## Dashboard pages

### First-boot Wizard

On a fresh install, the dashboard opens into a wizard that walks operators through vault passphrase, AdminAgent creation, license activation, User-token issuance, and CLI install.

### Home — System Overview

- **System health** — overall status (healthy/degraded/unhealthy) with service-level breakdown
- **Active agents** — count of connected agents with names and session duration
- **Recent activity** — last 20 capability invocations across all agents
- **Pending approvals** — count of operations waiting for human review

### Agents

Templates (Coder / Researcher / DevOps / Email / General) with per-agent model overrides, `Agent.md` identity, per-agent policy, and host mounts under `/mnt/<label>`.

- **Status** — active (connected), idle (registered but not connected), revoked
- **Session info** — current session duration, last connected time
- **Invocation count** — total capability calls made by this agent
- **Actions** — pause, resume, kill session, revoke credentials

### Activity / Supervision

A live-updating feed over the supervision WebSocket (TCP 7701, 30 s ping / 10 s timeout keepalive):

- Capability invocations with parameters and results
- Policy decisions (allowed, denied, approval required)
- Session events (connect, disconnect, reconnect)
- Error events with structured details

### Approvals

Operations gated as `approval_required` appear here with **Pending / Approved / Rejected / Timed Out** tabs. Default 24-hour hold for User MCP calls (configurable). Timed-out approvals can't be approved retroactively (HTTP 409 with a status discriminator); the audit log preserves the original `approval_required` tier through the decision chain.

!!! tip
    Approvals can also be managed from the CLI: `kruxos approve list` and `kruxos approve accept <id>`.

### Audit

Search and filter the hash-chained audit log:

- Principal-aware filter (`{type:"user"}` / `{type:"agent",name:...}`)
- View full request and response details for any entry
- Verify hash chain integrity

### Chat

Multi-model chat with persisted sessions, knowledge panel, inline approval flow.

### Code Sessions (`/code`)

xterm.js terminals through the Gateway sandbox. Concurrent-session cap defaults to 4; per-session memory cap 2 GiB; 4-hour idle timeout. **Not supported on the Docker image in v0.0.1** — use the VM image.

### Identities

User token CRUD with one-time raw-token reveal. The `krx_user_*` raw token is shown once at create time.

### Integrations

Claude Code / Codex install + regenerate seed configs (`~/.claude/CLAUDE.md`, `~/.codex/AGENTS.md`).

### Policies

Visual + YAML editor. Hot-reloadable from `/data/kruxos/policies/{system,org,agents/<name>}.yaml`. Per-agent overrides; User Rules surface on the Identities page.

### Settings

One card per model provider (Anthropic / OpenAI / Gemini / Local / OpenRouter / Codex / DeepSeek / Grok / Mistral / Groq / GLM) with connection tests.

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| `g h` | Go to Home |
| `g a` | Go to Agents |
| `g v` | Go to Activity |
| `g p` | Go to Approvals |

## Next steps

- [CLI Guide](cli.md) — command-line alternative to the dashboard
- [Managing Agents](../guides/managing-agents.md) — agent lifecycle management
- [Monitoring](../guides/monitoring.md) — set up alerts and health monitoring
