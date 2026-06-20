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

On a fresh install, the dashboard opens into an eight-step wizard:

1. **Welcome** — orientation card explaining the four things the wizard sets up (secrets, identity, CLIs, policy).
2. **Vault passphrase** — initialises or unlocks the encrypted vault. A live strength meter scores the passphrase before submit.
3. **Workspace** — picks the AdminAgent's home directory. The default `/data/kruxos/users/admin` is auto-created. A **click-through directory browser** opens a modal that lists subdirectories with writability dots and an inline "New folder" affordance (under `/data/`). A "Type path instead" fallback toggles a free-text input for clipboard pastes or pre-known paths.
4. **AdminAgent (Identity)** — names the first agent and optionally configures its model provider inline. Five provider types are wired into the wizard — **Anthropic**, **OpenAI**, **OpenAI Codex** (OAuth device-code), **OpenRouter**, **Local** — plus a **Skip** tab that defers provider setup to Settings. Provider credentials and the agent record are persisted atomically (provider first; if provider registration fails, the agent is not created).
5. **Licence** — paste a JWT or skip. KruxOS is free for personal use.
6. **User token** — generates a `krx_user_*` bearer token; shown **once** for CLI installs and the loopback User API. Acknowledge-and-continue is gated on a checkbox.
7. **Install CLI Tools** — optional. Installs Claude Code and/or Codex CLI seed configs in-process. Both can be installed later from Dashboard → Integrations.
8. **Done** — confirmation screen with a link into the main dashboard.

The progress rail at the top of the wizard supports backward navigation by clicking any completed dot.

### Home — System Overview

The Home page renders three rows of cards:

- **Row 1 — Status cards.** System health (healthy / degraded / unhealthy), Active agents (count + names + session duration), Pending approvals (count), and Service Proxy (sync state across Gmail / Slack adapters).
- **Row 2 — Today's metrics.** Capability invocations, approvals decided, errors, and a queue depth strip.
- **Row 3 — Recent activity.** Last 20 capability invocations across all agents, with status dots (ok / warn / error), agent name, capability, and relative timestamp.

On a fresh appliance with zero agents and zero activity, the page renders an "empty" subline ("No agents connected yet · waiting for first check-in") instead of the metric rows.

### Agents

The Agents list at `/agents` renders a typed table of all agents with status badges, autonomous-pulse indicator, model-provider override (inline-editable), and quick actions. Create flow is multi-step: pick a template (Coder / Researcher / DevOps / Email / General), set name and purpose, stage host mounts under `/mnt/<label>`, then submit — `${HOME}` placeholders must be resolved before the create button enables.

- **Status vocabulary** — `active` (connected), `paused` (session frozen), `revoked` (disabled), `disconnected` (registered but not currently connected).
- **Create** — opens an inline form. Templates pre-populate identity, policy, and mounts; you can edit any field before submitting.
- **Credentials modal** — after create, the new agent's API key + connection string are shown **once** in a modal with copy buttons.
- **Revoke** — opens a confirm modal. Revoked agents move to the bottom of the list with the `revoked` badge.
- **Restore** — revoked agents can be restored from the same row (restore preserves the agent's state and audit history).

### Agent detail (`/agents/<name>`)

Clicking an agent opens a five-tab detail page:

| Tab | What's there |
|-----|--------------|
| **Overview** | Stats grid (last seen / invocations / errors), model-provider selector + default-effort + token-budget config, context-management presets, standing instructions |
| **Identity** | `Agent.md` editor with char/token meter, draft → save (PUT) flow with revert |
| **Policy** | Summary card, per-agent trash-retention quick input, visual policy editor + YAML preview/edit toggle, delete-confirm modal |
| **Host Access** | Per-agent mount points under `/mnt/<label>` with staged-mode add-mount dialog |
| **State** | Searchable key-value explorer with quota meter, expandable entries, version history, edit + delete modals |

The action bar (Pause / Resume / Run Now / Kill / Rotate Key / Revoke) sits above the tabs. When the agent's status is `revoked`, all controls become read-only.

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

A single-pane User-principal surface at `/identities` with two stacked sections:

- **User tokens** — list of issued `krx_user_*` bearer tokens (name, created, last used, status). **New token** opens a modal that takes a friendly name and reveals the raw bearer **once** with a copy button. **Revoke** uses a type-the-name confirmation guard before calling `DELETE /api/user/tokens/<id>`.
- **User policy** — a "Has policy" / "Default" chip, a trash-retention quick input (positive integer hours, blank to clear back to the 168 h default), and a read-only YAML viewer for the current `/api/user/policy`. **Edit** opens a YAML editor modal with client-side `YAML.parse` validation before save; closing with unsaved changes prompts for confirm.

When the vault is locked, both sections render an inline "Vault is locked" banner with the gateway's hint message in place of their data.

### Integrations

Cards for **Claude Code** and **Codex CLI**, plus a **KruxOS Loopback** card describing the User API. Each external-CLI card shows a status badge (Loading / Installed / Not installed / Vault locked / Error), the detected version line, and action buttons:

- **View config** — opens a modal with the bundled seed config (`~/.claude/settings.json` for Claude Code, `~/.codex/config.toml` for Codex), copy-to-clipboard included.
- **Install** / **Regenerate** — atomic operation that installs the CLI via `npm i -g` (if needed) and writes the seed config in one step. Regenerate opens a confirm modal listing the destination paths before overwriting.

An **External Tools** section below the cards links out to Codex Cloud and the Claude Code docs (both `target="_blank"` with `rel="noopener noreferrer"`).

### Policies

Visual + YAML editor. Hot-reloadable from `/data/kruxos/policies/{system,org,agents/<name>}.yaml`. Per-agent overrides; User Rules surface on the Identities page.

### Settings

The page surfaces a **System defaults** summary card at the top (current chat / autonomous / fallback choices) and one **Provider card** per registered provider below. **+ Add Provider** opens a form supporting six provider types with auth-conditional fields:

- **Anthropic** — API key
- **OpenAI** — API key (also covers OpenAI-compatible upstreams via a Base URL override — DeepSeek, Grok, GLM, Mistral, Groq, custom)
- **OpenAI Codex** — OAuth device code (Sign in opens the ChatGPT subscription flow with a verification URL + copy-to-clipboard code, then polls until you approve in the browser)
- **Gemini** — API key
- **OpenRouter** — API key
- **Local** — none; runtime preset dropdown auto-fills the endpoint

Each provider card shows a credentials-status dot, default-model selector, Base URL, agent assignments, and three actions — **Test** (probes the upstream and renders the result inline), **Set Default** (per role), and **Remove**. If the vault is locked, the page renders a banner gating the cards.

### Health (`/health`)

Operator-facing health summary that auto-refreshes every 15 seconds:

- **Status banner** at the top of the page (Healthy / Degraded / Critical / Unknown) with the issue count, generation timestamp, and total-latency right rail.
- **Services table** — one row per backend service (gateway / vault / proxy / audit / state) with a status dot, latency cell colour-coded by threshold, details column, and last-checked column. Narrow viewports collapse the trailing columns automatically.
- **Resources grid** — Memory / CPU / Disk cards with progress bars that change colour at the 60 % and 80 % thresholds.
- **Agent metrics grid** — active agents, total sessions (lifetime cumulative), invocations / minute, and error rate.

If the gateway is unreachable, the page now renders an explicit error banner reading "Can't reach gateway: …. Retrying …" with a **Retry now** button so a transient failure no longer presents as a silent blank page.

### Alerts (`/alerts`)

The operator's surface for alerts — both those an agent raises with `alerts.send` and those the system's automatic monitors raise (high CPU / memory, disk pressure, audit-write failures, a service going down). Each row shows the severity (info / warning / critical), the source agent or monitor, the message, the timestamp, and whether it has been acknowledged; **Acknowledge** marks an alert as handled. The sidebar **Alerts** entry carries a count badge, and critical alerts also raise a banner across the top of every page — so an alert an agent sends now reaches the operator wherever they are in the dashboard rather than going unnoticed.

### Service Proxy (`/proxy`)

Per-service sync status for the proxy backends (Gmail / Slack adapters), auto-refreshing every 10 seconds. The top of the page carries a five-cell **overview strip** — Total services, Synced, With errors, Buffered ops, **Dead letters**. The Dead-letters cell is now first-class at the top of the page rather than buried inside each per-service card.

Below the overview strip, each service renders a card with the sync header, last-started / last-completed timestamps, buffered-write and dead-letter counts, and lists of pending buffered writes (with countdown + **Cancel**) and dead-letter writes (with **Retry** / **Discard**). When a service's sync is failing, the card surfaces how many consecutive failures it has seen and the last sync error (for example, a Slack `missing_scope`), so a misconfigured connection is diagnosable from the tile instead of showing a bare "Never / Unknown" with no explanation. All three write actions go through a confirm modal before posting to `/api/proxy/status`.

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
