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

A live-updating feed streamed over Server-Sent Events from `/api/activity/stream`:

- Capability invocations with parameters and results
- Policy decisions (allowed, denied, approval required)
- Session events (connect, disconnect, reconnect)
- Error events with structured details

A **LiveIndicator** pill in the top-right shows the stream state — **Live**, **Paused**, or **Disconnected**. Click to pause (closes the SSE connection); click again to resume (reopens it). A warning banner appears across the top if the connection drops mid-session. A filter bar provides substring search plus Agent / Status / Capability filters; the feed buffers the most recent 200 entries.

### Approvals

Operations gated as `approval_required` appear here with five tabs — **Pending / Approved / Rejected / Timed Out / All**. The Pending tab carries a count badge (also mirrored next to the page title). Auto-refresh polls every 5 s; when the pending count grows between polls, a toast slides in reading "N new approval request(s) pending" so operators don't need to keep the tab visually focused. Default 24-hour hold for User MCP calls (configurable). Timed-out approvals can't be approved retroactively (HTTP 409 with a status discriminator); the audit log preserves the original `approval_required` tier through the decision chain.

!!! tip
    Approvals can also be managed from the CLI: `kruxos approve list` and `kruxos approve accept <id>`.

### Audit

Search and filter the hash-chained audit log:

- **Actor filter** — Principal-tagged dropdown; selecting **User** filters to operator-initiated entries, selecting an agent name filters to that agent's entries.
- **Capability** text input + **Status** dropdown + **From / To** date pickers (default range: last 7 days).
- View full request and response details for any entry; copy `entry_hash` or `log_file` from the expanded row.
- Configurable page size (**25 / 50 / 100 / 200**) with a "Showing N–M of T" summary at the top.
- **Export JSON** of the active filtered result set.
- Verify hash chain integrity.

### Chat

Four-column desktop layout — Agents · Conversations · Messages · Knowledge — plus a `⌘K` / `Ctrl+K` Search overlay. Multi-model with per-message Model + Thinking overrides above the composer, persisted sessions, tool-call cards with policy-tier colouring, and an inline approval flow. Collapses to a 3-state mobile navigation under 768 px.

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
