# Admin CLI Commands

By the end of this page, you'll know the three admin command groups that manage operator identity, host access, and CLI integrations.

These commands go beyond day-to-day agent management. Use them when you need to issue bearer tokens, give agents access to host directories, or regenerate host CLI seed configs.

For the full CLI reference, see the [CLI Guide](../quickstart/cli.md).

## `kruxos user-token` — bearer token management

Issue, list, and revoke `krx_user_*` bearer tokens for host CLIs, integrations, and CI.

| Command | Purpose |
|---------|---------|
| `kruxos user-token create <label>` | Mint a new token. Prints raw bearer **once**. |
| `kruxos user-token list` | Show all tokens (metadata only — no raw values). |
| `kruxos user-token revoke <id>` | Revoke by ID. Takes effect on next handshake. |

```bash
# Issue a token for a new laptop
kruxos user-token create laptop-2026

# Audit active tokens
kruxos user-token list

# Revoke a compromised token
kruxos user-token revoke 7e4c1d3a-...
```

See [User Identities & Tokens](user-identities.md) for the dashboard equivalent.

## `kruxos mount` — per-agent host mounts

Give an agent access to a host directory without expanding its default sandbox workspace. Mounts appear under `/mnt/<label>` inside the agent's sandbox.

| Command | Purpose |
|---------|---------|
| `kruxos mount add <agent> --source <path> --target /mnt/<label> --label <label> [--read-only]` | Add a mount |
| `kruxos mount list <agent>` | List mounts for an agent |
| `kruxos mount remove <agent> <mount_id>` | Remove a mount |
| `kruxos mount toggle-readonly <agent> <mount_id>` | Flip read-only flag |
| `kruxos mount relabel <agent> <mount_id> <new_label>` | Update label (path is immutable) |

```bash
# Give an agent access to a project repo
kruxos mount add AgentCode \
    --source $HOME/projects/myrepo \
    --target /mnt/myrepo \
    --label myrepo

# Mount a dataset read-only
kruxos mount add AgentResearch \
    --source /data/datasets/wiki-en \
    --target /mnt/wiki-en \
    --label wiki-en \
    --read-only
```

!!! tip "Restart hint"
    Adding or removing a mount prints a restart hint. The agent's sandbox must restart to pick up mount changes. Pause and resume the agent, or kill and reconnect.

The dashboard **Host Access** tab on each agent's detail page provides the same operations with a visual UI.

## `kruxos cli-config` — host CLI seed configs

Generate deterministic seed configs for Claude Code and Codex CLI that wire them to the KruxOS gateway.

| Command | Purpose |
|---------|---------|
| `kruxos cli-config generate [--write] [--claude] [--codex]` | Render seed config to stdout or write to disk |

```bash
# Generate and write both Claude Code and Codex configs
kruxos cli-config generate --write

# Claude Code only
kruxos cli-config generate --write --claude

# Preview without writing
kruxos cli-config generate --claude
```

Writes:

- `~/.claude/settings.json` — MCP server pointing at `mcp-bridge`
- `~/.codex/config.toml` — hooks and MCP config for Codex

The raw bearer token is **not** stored in these files — `mcp-bridge` reads it from the vault at launch time.

See [Host CLI Integrations](host-cli-integrations.md) for the full install walkthrough.

## Next steps

- [User Identities & Tokens](user-identities.md)
- [Managing Agents](managing-agents.md) — agent lifecycle including Host Access tab
- [CLI Guide](../quickstart/cli.md) — full command reference