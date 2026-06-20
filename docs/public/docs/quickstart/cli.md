# CLI Guide

By the end of this page, you'll know how to manage KruxOS entirely from the command line.

The `kruxos` CLI is a single binary that ships on the appliance (at `/opt/kruxos/bin/kruxos`) and covers every operator surface — agents, approvals, secrets, audit logs, packs, code sessions, user tokens, mounts, sandbox diagnostics, state backup/restore, the migration path, and more.

## Quick reference

```bash
kruxos --help
```

Expected output (abbreviated):

```
KruxOS — operating system for AI agents

Usage: kruxos <COMMAND>

Commands:
  version       Show KruxOS version and build information
  status        System status summary
  config        Manage system configuration
  agent         Manage agent credentials and lifecycle
  approve       Manage approval queue
  watch         Live activity stream of all connected agents
  agents        Live agent dashboard (TUI)
  alerts        Show active alerts
  kill          Terminate an agent's session immediately
  pause         Freeze an agent's session (no capability calls processed)
  resume        Resume a paused agent's session
  state         Explore and manage agent state, plus backup/restore/backups
  model         Manage model providers (Claude, OpenAI, Gemini, Local)
  pack          Manage capability packs (install <local-path> only in v0.0.1)
  vault         Manage the secrets vault
  audit         Query audit logs
  user-token    Manage User bearer tokens (krx_user_*)
  mount         Manage per-agent host mounts under /mnt/<label>
  cli-config    Render host-CLI seed configs for Claude Code / Codex CLI
  code          Manage dashboard-embedded code sessions (list / kill / attach)
  sandbox       Sandbox diagnostics (diagnose)
  trash         List soft-deleted items, manually trigger cleanup
  activate      Register a KruxOS license JWT locally
  migrate       Migrate data between Docker and OS image deployments
  verify        Verify system health: gateway, definitions, policies, databases
  completions   Generate shell completions (bash / zsh / fish)
  man           Generate on-demand man pages
```

Shell completions for bash / zsh / fish ship out of the box; man pages are emitted on demand via `kruxos man <command>`.

## System status

```bash
kruxos status
```

Expected output:

```
KruxOS v0.0.1
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Gateway:    running (port 7700, MCP-native)
Supervision: running (port 7701, 30s ping / 10s timeout keepalive)
Dashboard:  running (port 7800, HTTPS)
Vault:      unlocked
Policy:     personal-permissive (AdminAgent)

Agents:     2 registered, 1 active
Uptime:     3h 42m
```

## Alerts

```bash
kruxos alerts
```

Lists active alerts — those an agent raises with `alerts.send` and those the system's automatic monitors raise (high CPU / memory, disk pressure, audit-write failures, a service going down). Narrow the list by recency or severity:

```bash
kruxos alerts --last 24h           # alerts from the last 24 hours
kruxos alerts --severity critical  # critical alerts only
```

The same alerts surface on the dashboard **Alerts** page (`/alerts`), where you can acknowledge them.

## Agent management

### Create an agent

```bash
kruxos agent create --name deploy-bot --purpose "CI/CD deployment agent"
```

Expected output:

```
Agent created successfully.

  Name:    deploy-bot
  Token:   7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c

Save this token — it will not be shown again.
```

### List agents

```bash
kruxos agent list
```

Expected output:

```
Name         Status    Last Connected     Invocations
my-agent     active    2s ago             147
deploy-bot   idle      never              0
```

### Show agent details

```bash
kruxos agent show my-agent
```

### Rotate / revoke

```bash
kruxos agent rotate my-agent
kruxos agent revoke deploy-bot
```

### Session control

`kruxos kill / pause / resume <agent-name>` operate at agent-session granularity:

```bash
kruxos pause my-agent       # freeze: no capability calls processed
kruxos resume my-agent      # unfreeze
kruxos kill my-agent        # terminate the current session
```

## User tokens

```bash
kruxos user-token create --label cli-laptop   # prints the raw token once
kruxos user-token list                        # digest-free metadata only
kruxos user-token revoke <id>
```

The raw token is shown exactly once at create time and also stored in the vault under `user/token/<label>` for launcher scripts (mcp-bridge, cli-hook) to load — keep it out of argv.

## Approval queue

```bash
kruxos approve list                              # pending requests
kruxos approve show ap_001                       # full request details
kruxos approve accept ap_001 --reason "Deploy"
kruxos approve reject ap_002 --reason "Not needed"
kruxos approve watch                             # live stream
```

User MCP calls default to a **24-hour hold**; timed-out approvals cannot be approved retroactively (HTTP 409 with a status discriminator).

## Live activity stream

```bash
kruxos watch
```

Live-updating feed of every capability invocation. Press `q` to quit. Filter with `--agent <name>` or `--capability 'filesystem.*'`.

## Agent state

```bash
kruxos state list my-agent          # persistent state keys
kruxos state get my-agent last_deploy
kruxos state set my-agent key value
kruxos state delete my-agent key
kruxos state quota my-agent         # usage vs quota
```

Three state scopes: session / persistent / shared. Shared state is exposed under `kruxos state shared ...`.

## State backup / restore

```bash
kruxos state backup --out /tmp/state-2026-05-11.tar.gz.enc
kruxos state restore /tmp/state-2026-05-11.tar.gz.enc
kruxos state backups               # list available backups
```

Daily backups run automatically via systemd timer at 02:00 UTC.

## Audit log

```bash
kruxos audit query --agent my-agent --last 1h
kruxos audit replay sess_abc123      # full session replay
kruxos audit stats --last 24h
kruxos audit rotate                  # delete entries older than retention (default 90d)
```

The audit log is length-prefixed CBOR with a hash chain (tamper evidence). Daily rotation runs at 03:00 UTC.

## Vault management

```bash
kruxos vault list
kruxos vault add <key>
kruxos vault rotate <key>
kruxos vault revoke <key>
```

## Capability packs

```bash
kruxos pack list
kruxos pack install ./my-pack       # local-path install only in v0.0.1
```

!!! info "Pack registry ships in v0.0.2"
    `kruxos pack search` / `kruxos pack install <name-from-registry>` and the GitHub-based publishing flow land in **v0.0.2** alongside the seed packs and the standalone `pack-sdk` CLI.

## Mounts

```bash
kruxos mount add my-agent --source /home/op/data --target /mnt/data
kruxos mount list my-agent
kruxos mount remove <uuid>
kruxos mount toggle-readonly <uuid>
kruxos mount relabel <uuid> --label workspace
```

Targets must start with `/mnt/<label>`; sources are canonicalised and must exist. Path-escape detection is built in.

## Sandbox diagnostics

```bash
kruxos sandbox diagnose            # human-readable
kruxos sandbox diagnose --json     # machine-readable
```

Reports per-primitive status for landlock, seccomp, user_ns, net_ns, nftables, cgroups v2. Exit 0 only if all are active. Intended for in-VM release-smoke verification — no sudo required.

## CLI-config (host-CLI integrations)

```bash
kruxos cli-config generate            # preview seed configs for Claude Code + Codex
kruxos cli-config generate --write    # write ~/.claude/settings.json + ~/.codex/*
```

The generator never puts raw tokens on argv — it stores them in the vault and references them indirectly.

## Code Sessions

```bash
kruxos code list                   # active + parked sessions
kruxos code kill <uuid>            # terminate
kruxos code attach <uuid>          # attach stdio (scaffolded in Gate C; full attach lands in v0.0.2)
```

!!! warning "Code Sessions need the VM image in v0.0.1"
    Code Sessions (`/code` dashboard page + the `kruxos code` subcommands) are not supported on the Docker image in v0.0.1; the Docker-side cgroup v2 delegation fix ships in v0.0.2.

## Trash

```bash
kruxos trash list                  # per-principal soft-deleted items
kruxos trash cleanup --dry-run     # what hourly scheduler would remove
```

Restore is exposed as the `filesystem.restore` capability (any session can call it); the CLI surface stays focused on operator-side prune verification.

## Shell completions

```bash
kruxos completions bash > /etc/bash_completion.d/kruxos
kruxos completions zsh > ~/.zsh/completions/_kruxos
kruxos completions fish > ~/.config/fish/completions/kruxos.fish
```

## Next steps

- [Web Dashboard](dashboard.md) — visual alternative to the CLI
- [Managing Agents](../guides/managing-agents.md) — agent lifecycle in depth
- [Policies](../guides/policies.md) — write and manage policy rules
- [Monitoring](../guides/monitoring.md) — health checks and alerts
