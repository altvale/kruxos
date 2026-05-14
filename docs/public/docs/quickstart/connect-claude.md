# Connect Claude

By the end of this page, Claude will be connected to KruxOS and executing capabilities.

Claude connects to KruxOS via **MCP (Model Context Protocol)** — the same protocol Claude uses natively. A lightweight stdio bridge translates between Claude's local MCP transport and the KruxOS Gateway's WebSocket transport.

## Prerequisites

- A running KruxOS instance ([Install](install.md))
- An agent token (64-char hex) issued by `kruxos agent create` or by the first-boot wizard
- **The bundled mcp-bridge** at `/opt/kruxos/bin/mcp-bridge` (ships on the VM image) — or you can run the Python equivalent from the in-appliance SDK at `/opt/kruxos/sdk/python/`

!!! note "About the Python SDK in v0.0.1"
    The Python SDK ships **bundled inside the appliance** at `/opt/kruxos/sdk/python/` (auto-importable from interactive shells via `/etc/profile.d/kruxos-sdk.sh`). The external `pip install kruxos` distribution to PyPI ships in **v0.0.3** alongside the license-server cycle. For host-CLI integrations (Claude Code, Codex), use the native `mcp-bridge` and `cli-hook` binaries under `/opt/kruxos/bin/`.

## Choose your Claude client

| Client | Config file | Best for |
|--------|------------|----------|
| [Claude Desktop](#claude-desktop) | `claude_desktop_config.json` | Conversational use |
| [Claude Code](#claude-code) | `.mcp.json` or settings | Development workflows |
| [Claude API](#claude-api) | Python code | Programmatic use |

---

## Claude Desktop

### 1. Generate the MCP config

The fastest path is the appliance CLI:

```bash
# On the appliance (or via `docker exec kruxos ...` / SSH to the VM)
kruxos cli-config generate
```

This emits a Claude Desktop / Claude Code stanza referencing the bundled `mcp-bridge`, with the token pulled from the vault — never on argv:

```json
{
  "mcpServers": {
    "kruxos": {
      "command": "/opt/kruxos/bin/mcp-bridge",
      "args": [],
      "env": {
        "KRUXOS_ENDPOINT": "wss://YOUR_KRUXOS_HOST:7700",
        "KRUXOS_AGENT_NAME": "default-agent",
        "KRUXOS_AGENT_TOKEN": "<64-char hex> (loaded from vault by the bridge)"
      }
    }
  }
}
```

!!! warning "Never put the raw token on argv"
    `mcp-bridge` self-rejects if its own argv contains a `krx_user_` substring (visible to `ps` / `/proc/*/cmdline`). Pass tokens via the env var or stdin only.

### 2. Add to Claude Desktop config

Open the Claude Desktop config file:

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

If the file doesn't exist, create it. If it already has `mcpServers`, merge the `kruxos` entry into the existing object.

### 3. Restart Claude Desktop

Quit and relaunch Claude Desktop. The KruxOS capabilities should appear as available tools.

### 4. Verify

Start a new conversation and ask:

> "What tools do you have available?"

Claude should list KruxOS capabilities (system.time, filesystem.read, process.run, etc.).

### 5. Try these prompts

- "What time is it on the server?" → calls `system.time`
- "Show me the system information" → calls `system.info`
- "List all files in the workspace" → calls `filesystem.list`
- "Create a file called hello.txt with 'Hello World'" → calls `filesystem.write`
- "Read hello.txt" → calls `filesystem.read`
- "Run `echo hello` and show me the output" → calls `process.run`
- "What agent am I?" → calls `agent.whoami`

---

## Claude Code

See the dedicated [Claude Code quickstart](claude-code.md) for the full walkthrough. The short version:

```bash
# On the appliance — writes ~/.claude/settings.json with the mcp-bridge stanza
kruxos cli-config generate --write
```

The generator stores the raw token under `user/token/<label>` in the vault and references it indirectly from the seed config. Native shell-style tools that bypass KruxOS supervision are disabled at the user-config and requirements layers; every Claude Code tool call is routed through the KruxOS approval queue via `cli-hook`.

### Verify

Claude Code will automatically discover the KruxOS tools. Try development-oriented prompts:

- "Check the system health on KruxOS"
- "Read the deployment config and tell me if anything looks wrong"
- "Create a new Python file in the workspace"
- "Run the test suite and show me the results"

---

## Claude API (programmatic use)

For programmatic access, point the Anthropic SDK at the KruxOS Gateway as an MCP server:

```python
import anthropic

client = anthropic.Anthropic()

kruxos_mcp = {
    "type": "url",
    "url": "wss://YOUR_KRUXOS_HOST:7700",
    "name": "kruxos",
    "authorization_token": "<64-char hex>",  # your agent token
}

response = client.messages.create(
    model="claude-sonnet-4-6",
    max_tokens=1024,
    mcp_servers=[kruxos_mcp],
    messages=[
        {"role": "user", "content": "List the files in the workspace"}
    ],
)

print(response.content)
```

---

## Use with the in-appliance Python SDK

When you're running code on the appliance itself (e.g. inside an autonomous agent task), the bundled SDK is auto-importable:

```python
import asyncio
from kruxos import KruxOS

async def main():
    os = await KruxOS.connect_async(
        endpoint="ws://localhost:7700",
        agent_name="default-agent",
        api_key="<64-char hex>",
    )
    try:
        caps = await os.capabilities.list_async()
        print(f"Available: {len(caps)} capabilities")

        result = await os.call_async("system.time")
        print(f"Server time: {result.data['utc']}")
    finally:
        await os.close_async()

asyncio.run(main())
```

For host-side Python code: copy `/opt/kruxos/sdk/python/` off the appliance into your project until the PyPI distribution ships in **v0.0.3**.

## Verify the connection

Check that your agent is connected:

```bash
kruxos agent list
kruxos status
```

A healthy agent shows `active` after Claude makes its first tool call. The dashboard `/activity` page reflects the same connection in real time over the supervision WebSocket on TCP 7701.

## How it works

```
Claude Desktop/Code → stdio → bridge process → WebSocket → KruxOS Gateway
                                                              ↓
                                                    Policy Engine → Registry → Handler
                                                              ↓
                                                         Audit Log
```

1. Claude Desktop launches the bridge as a local MCP server process
2. The bridge connects to the KruxOS Gateway via WebSocket
3. It authenticates with your agent credentials
4. MCP messages flow bidirectionally: stdin/stdout ↔ WebSocket
5. Each capability call goes through: Gateway → Policy Engine → Registry → Handler
6. Every invocation is logged to the audit system

## Troubleshooting

### Claude doesn't see any tools

- **Check config path**: Ensure `claude_desktop_config.json` is in the correct location for your OS
- **Restart Claude**: Claude Desktop must be restarted after config changes
- **Check the bridge binary**: `/opt/kruxos/bin/mcp-bridge` must exist and be executable. On the VM image it ships under that path; on the Docker image it's bind-mounted into the container.
- **Test the bridge manually**: `KRUXOS_ENDPOINT=wss://localhost:7700 KRUXOS_AGENT_NAME=default-agent KRUXOS_AGENT_TOKEN=<64-char hex> /opt/kruxos/bin/mcp-bridge`. Structured exit codes (10 = auth, 11 = network) make failures easy to diagnose.

### Tools appear but calls fail

- **Check endpoint URL**: Ensure the Gateway is reachable at the configured endpoint
- **Check API key**: Verify the key with `kruxos agent list` — the agent should show `active` status
- **Check workspace**: Filesystem operations require the workspace directory to exist

### Approval-required tools hang

- Check the [web dashboard](dashboard.md) for pending approvals
- Or use the CLI: `kruxos approve list`
- Approve with: `kruxos approve accept <request-id>`

### Connection drops

- The SDK automatically reconnects on connection loss
- The bridge exits when Claude closes stdin — this is normal
- Check Gateway logs: `docker logs <container>` or `journalctl -u kruxos-gateway`

## Next steps

- [Web Dashboard](dashboard.md) — watch your agent's activity live
- [CLI Guide](cli.md) — manage agents and approvals from the terminal
- [Connect Gmail](gmail.md) — enable email capabilities
