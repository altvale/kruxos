# Claude Code Integration

Claude Code connects to KruxOS as an MCP server, giving it access to all registered capabilities — filesystem, process, git, network, scheduler, comms, state, secrets, email, Slack, alerts — through MCP tool use. Every tool call is routed through the KruxOS approval queue via `cli-hook`; Claude Code's native shell tool is disabled at the user-config and requirements layers so governance can't be bypassed.

## Prerequisites

- A running KruxOS instance ([Install](install.md))
- An agent token (64-char hex) from `kruxos agent create` or the first-boot wizard
- A User token (`krx_user_*`) for the loopback User API — issued by the wizard, also retrievable via `kruxos user-token list`
- The bundled `mcp-bridge` at `/opt/kruxos/bin/mcp-bridge` (ships on the appliance — VM image or Docker image)

## Setup

### Option A: Generate seed configs from the appliance (recommended)

`kruxos cli-config generate` emits Claude Code seed configs that reference the bundled `mcp-bridge` and pull tokens from the vault — never from argv.

Preview the output:

```bash
kruxos cli-config generate
```

Write it directly to your home directory:

```bash
kruxos cli-config generate --write
```

Written paths:

- `~/.claude/settings.json` — references `/opt/kruxos/bin/mcp-bridge`
- `~/.codex/config.toml`, `~/.codex/hooks.json` (for the Codex CLI, if installed)

### Option B: `.mcp.json` (per-project, hand-written)

Create `.mcp.json` in your project root and reference the bridge directly:

```json
{
  "mcpServers": {
    "kruxos": {
      "command": "/opt/kruxos/bin/mcp-bridge",
      "args": [],
      "env": {
        "KRUXOS_ENDPOINT": "wss://YOUR_KRUXOS_HOST:7700",
        "KRUXOS_AGENT_NAME": "default-agent",
        "KRUXOS_AGENT_TOKEN": "<64-char hex>"
      }
    }
  }
}
```

!!! warning "Tokens belong in env vars or the vault — never on argv"
    `mcp-bridge` self-rejects if its own argv contains a `krx_user_` substring (process listings leak via `ps` / `/proc/*/cmdline`). The seed-config generator stores raw tokens in the vault under `user/token/<label>` and references them indirectly; if you hand-edit `.mcp.json`, keep the agent token in the `env` block only.

## Verify

```
> "What tools do you have from KruxOS?"
```

Claude should list capabilities from the 13 categories — filesystem, process, network, git, scheduler, system, agent, state, comms, secrets, email, Slack, alerts. `blocked` capabilities are omitted from `tools/list` entirely; `approval_required` capabilities show with a policy-tier annotation.

## Development workflows

Claude Code excels at chaining multiple capability calls in a single task.

### Read → Analyze → Fix

> "Read the deployment config at /workspace/deploy.yaml and tell me if there are any issues. If you find problems, create a fixed version."

Claude will:
1. Call `filesystem.read` to read the config
2. Analyze the content
3. Call `filesystem.write` to create the fixed version (write may be `approval_required` depending on policy)

### Run → Inspect → Fix

> "Run the test suite and show me the results. If any tests fail, read the failing test file and suggest a fix."

Claude will:
1. Call `process.run` to execute tests
2. Call `filesystem.read` on failing test files
3. Call `filesystem.write` to create fixes

### Explore → Generate

> "List the files in /workspace/src, read the main module, then create a test file for it."

Claude will chain `filesystem.list`, `filesystem.read`, and `filesystem.write`.

### System health check

> "Check the system time, show me the system info, and tell me if everything looks healthy."

Stateless capabilities (`system.time`, `system.info`, `system.health`, `agent.whoami`) execute in-process for low latency; everything else runs in a forked child with the full sandbox applied.

## Differences from Claude Desktop

| Aspect | Claude Desktop | Claude Code |
|--------|---------------|-------------|
| Config file | `claude_desktop_config.json` | `~/.claude/settings.json` or `.mcp.json` |
| Config scope | Global | Per-project or global |
| Multi-step tasks | Single conversation turns | Chains many tools in one task |
| Best for | Conversational exploration | Code-aware development workflows |
| Transport | Same `mcp-bridge` (stdio↔WebSocket) | Same `mcp-bridge` |
| Pre-tool hooks | None | `cli-hook` maps native tools (`cat`, `grep`, `curl`) to KruxOS capabilities before they run |

## Troubleshooting

### Claude Code doesn't see KruxOS tools

- **Check the bridge binary**: `ls -l /opt/kruxos/bin/mcp-bridge`
- **Check the seed config**: `cat ~/.claude/settings.json` — the `kruxos` entry must point at the bridge
- **Test the bridge manually**: `KRUXOS_ENDPOINT=wss://localhost:7700 KRUXOS_AGENT_NAME=default-agent KRUXOS_AGENT_TOKEN=<64-char hex> /opt/kruxos/bin/mcp-bridge` — structured exit codes (10 = auth, 11 = network) point at the cause.

### Tools appear but calls fail

- **Check the Gateway is running**: `kruxos status` (on the appliance) or hit `https://localhost:7800` (dashboard) on the host.
- **Check the agent token**: `kruxos agent list` should show your agent as `active` once Claude Code makes its first call.
- **Approval-required calls hang**: a call gated as `approval_required` waits until the operator decides (default 24-hour hold for User MCP calls, configurable). Approve from the dashboard `/approvals` page or with `kruxos approve accept <id>`.

### Connection drops during long tasks

- The supervision WebSocket keepalives every 30 s with a 10 s timeout. The bridge auto-reconnects.
- The bridge exits when Claude Code closes stdin — this is normal between tasks.
- Gateway logs: `docker logs kruxos` (Docker) or `journalctl -u kruxos-gateway` (VM).

## Next steps

- [Connect Claude Desktop or API](connect-claude.md)
- [CLI Guide](cli.md) — manage agents and approvals from the terminal
- [Web Dashboard](dashboard.md) — watch agent activity live
- [Approval workflow](../guides/approval-workflow.md) — how gated tool calls flow through the queue
