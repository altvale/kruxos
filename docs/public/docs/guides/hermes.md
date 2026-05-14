# Running Hermes Agent on KruxOS

[Hermes Agent](https://github.com/NousResearch/hermes-agent) is an open-source,
self-improving AI agent framework from Nous Research with built-in MCP client
support. Connecting Hermes to KruxOS gives every file, process, network, and
git operation that Hermes performs the same governance the rest of your agents
get: sandboxing, policy enforcement, approval flows, and audit logging.

This guide walks through the one-time setup — about five minutes if KruxOS is
already running.

!!! info "Full Hermes integration ships in v0.0.3"
    v0.0.1 supports Hermes through the same generic `mcp-bridge` path documented
    below — any client that speaks MCP stdio can use it. **Full Hermes
    integration** (a dedicated `kruxos cli-config` adapter for Hermes,
    on-appliance pip installation, ACP gating for web/browser/messaging tools)
    is tracked separately and lands in **v0.0.3**. Until then, follow the
    generic MCP setup below.

## Why connect Hermes to KruxOS?

Hermes ships with 40+ built-in tools covering the same surface area KruxOS
governs: `terminal`, `read_file`, `write_file`, `list_directory`, shell, git,
web. Those built-in tools run directly on the host with no sandbox and no
audit trail. Routing them through KruxOS turns every Hermes OS call into a
policy-checked, audit-logged capability invocation — without changing the way
you write prompts or deploy Hermes.

Architecture:

```
Hermes Agent (python)
  │  stdio (newline-delimited JSON-RPC, MCP 2024-11-05)
  ▼
kruxos.connectors.claude_bridge  (launched by Hermes as a subprocess)
  │  WebSocket
  ▼
KruxOS Gateway  ──►  Policy Engine  ──►  Sandbox  ──►  Capabilities
           │
           └──►  Audit Log  ──►  Dashboard Approvals
```

The bridge is the same `kruxos.connectors.claude_bridge` stdio-to-WebSocket
bridge that Claude Desktop and Claude Code use — KruxOS does not ship a
Hermes-specific adapter because MCP stdio is a standard transport.

## Prerequisites

- **KruxOS** running and reachable at `ws://HOST:7700`
  (Docker via `altvale/kruxos:latest`, the VM image, or a native build all work).
- **A KruxOS agent token.** Create one with:
  ```bash
  kruxos agent create --name hermes
  ```
  Write down the 64-char hex token it prints. You will pass this to Hermes as
  an environment variable.
- **The bundled `mcp-bridge`** at `/opt/kruxos/bin/mcp-bridge` on the appliance,
  or a host-side copy of the in-appliance Python SDK at `/opt/kruxos/sdk/python/`
  (until the PyPI distribution lands in v0.0.3).
- **Hermes Agent v0.7.0 or newer** installed via the project's install script
  (see the Hermes README for current instructions).

## Step 1 — Add KruxOS as an MCP server in Hermes

Edit `~/.hermes/config.yaml` and add an entry under `mcp_servers`:

```yaml
mcp_servers:
  kruxos:
    command: /opt/kruxos/bin/mcp-bridge
    args: []
    env:
      KRUXOS_ENDPOINT: wss://localhost:7700
      KRUXOS_AGENT_NAME: hermes
      KRUXOS_AGENT_TOKEN: 7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c
```

If KruxOS is running on a different host (for example, in a Docker container
on your dev machine while Hermes runs locally), replace `localhost` with the
gateway host. Hermes launches the bridge as a child process on startup, and
the bridge proxies MCP messages over the WebSocket connection.

!!! tip "Config location"
    Hermes reads `~/.hermes/config.yaml` by default. If you run Hermes with
    a custom config (`hermes --config path/to/config.yaml`), add the entry
    to that file instead.

## Step 2 — Disable Hermes's built-in OS tools

Hermes's native `terminal`, `read_file`, `write_file`, `list_directory`, and
similar tools run directly on the host and bypass KruxOS governance. Once you
connect KruxOS, you want Hermes to use KruxOS equivalents
(`filesystem.read`, `filesystem.write`, `filesystem.list`, `process.run`, …)
so every call is sandboxed and audited.

Hermes manages tool enablement through the `hermes tools` CLI, not through
`config.yaml` keys. Disable the overlapping native tools in one shot:

```bash
hermes tools disable terminal read_file write_file list_directory shell_exec git_clone git_commit
```

You can also run `hermes tools` with no arguments to get the interactive
picker and toggle tools on and off visually.

Leave the non-OS tools enabled — for example Hermes's web search, markdown
renderer, and planner are fine alongside KruxOS. List what's currently
enabled with:

```bash
hermes tools list
```

The goal is a clean split: KruxOS owns anything that touches the host OS;
Hermes owns reasoning, planning, and in-process tools.

## Step 3 — Verify the connection

Start Hermes and ask the agent to list its tools:

```
$ hermes
> What tools do you have from the kruxos MCP server?
```

Hermes should report the KruxOS capabilities it discovered. You should see
names such as `filesystem.read`, `filesystem.write`, `process.run`,
`network.http_request`, and so on — roughly 80+ tools depending on which
packs you have loaded.

If you see tools like `filesystem_read` with underscores in the Hermes
output, that is expected: MCP tool names must match `^[a-zA-Z0-9_-]{1,128}$`,
so the bridge translates dots to underscores on the way out and back again
on the way in. You can invoke either form — Hermes will call the mangled
name, and the bridge will restore the original dotted name before forwarding
to the gateway.

Try a concrete tool call:

```
> Read the file /workspace/README.md and summarise it.
```

Hermes will dispatch a `filesystem_read` call through the bridge. On the
KruxOS side you will see the call appear in the gateway audit log and — if
the capability is sensitive — in the dashboard approval queue.

## Approval flow

KruxOS and Hermes both have approval flows, and it is worth understanding
the distinction:

- **KruxOS approvals** happen at the OS level. When Hermes attempts a
  capability whose policy is `require_approval`, the gateway holds the tool
  call and queues a request in the dashboard. The bridge polls for the
  decision and returns the real result (approved) or a STOP error (rejected)
  to Hermes. Hermes does not need to know anything about this — the tool
  call simply takes longer to return.
- **Hermes approvals** (v0.7.0) happen at the agent level via Slack or
  Telegram buttons. They gate whether a command *becomes* a tool call at all.

Running both approval systems in parallel causes double-approval friction:
the user has to confirm once in Slack/Telegram before Hermes fires the tool
call, then again in the KruxOS dashboard before the call executes.

Command approval is a **global** Hermes setting — it is not configurable
per MCP server. You have two reasonable choices:

- **Rely on KruxOS alone** — disable Hermes command approval globally via
  `hermes config set` (see `hermes config --help` for the exact key in your
  installed version). KruxOS will hold every sensitive tool call in the
  dashboard queue, and the bridge will block until the decision lands.
  This is the simplest story for teams who want a single source of truth
  for governance.
- **Keep both for defense-in-depth** — leave Hermes command approval on.
  You will see Slack/Telegram prompts before the tool call is dispatched,
  and then again in the dashboard before the capability executes. More
  friction, but two independent stop-buttons.

For v0.0.1 we recommend the first option: one approval path, one audit log,
one place to change your mind.

## Tool filtering: KruxOS policy vs Hermes include/exclude

Hermes supports per-server `tools.include` / `tools.exclude` lists. This is a
client-side filter that controls which tools Hermes *sees*. KruxOS policy is
a server-side filter that controls which tools an agent is *allowed to call*.

The two filters compose:

| KruxOS policy | Hermes include/exclude | Result |
|---------------|-----------------------|--------|
| Allowed       | Included              | Hermes can see and call the tool |
| Allowed       | Excluded              | Tool hidden from Hermes, but a direct MCP call would still succeed (Hermes won't make one) |
| Blocked       | Included              | Tool hidden by the gateway — Hermes never sees it |
| Blocked       | Excluded              | Tool hidden by both — same result |

**Recommendation:** let KruxOS handle filtering via policies and leave Hermes
unfiltered. That way the governance decision lives in one place (`policies/`),
and changing a policy updates every client automatically. Only use Hermes's
filter when you want to hide a KruxOS-allowed tool from a specific Hermes
profile for UX reasons.

## Production mode: running Hermes inside the KruxOS container

The setup above is a good starting point but assumes Hermes runs on the host
with its own process space. For full governance, run Hermes **inside** a
KruxOS agent container so that even Hermes's reasoning-time file access
(working directory writes, temp files, cache) is inside the sandbox:

1. Build a Docker image that layers Hermes on top of `altvale/kruxos:latest`.
2. Point Hermes's MCP server entry at `ws://gateway:7700` (the compose-network
   hostname) instead of `localhost`.
3. Mount the agent's workspace volume at `/workspace` — that becomes Hermes's
   working directory and KruxOS's sandbox root simultaneously.

Production mode is the same story as Codex in a KruxOS container — see the
Codex guide for a full compose example. The Hermes-specific bit is just the
image build.

## Troubleshooting

### Hermes reports "MCP server kruxos failed to start"

The bridge is failing before it can accept messages on stdin. Run the bridge
manually to see the error:

```bash
KRUXOS_ENDPOINT=wss://localhost:7700 \
KRUXOS_AGENT_NAME=hermes \
KRUXOS_AGENT_TOKEN=7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c \
/opt/kruxos/bin/mcp-bridge
```

Structured exit codes (10 = auth, 11 = network) point at the cause.

Common causes:

- **`mcp-bridge` binary missing.** `ls -l /opt/kruxos/bin/mcp-bridge` — on the
  appliance it's always present; on a host machine, copy it off the VM image
  or run it from inside a `docker exec`.
- **Environment variables missing.** The bridge exits immediately if
  `KRUXOS_AGENT_NAME` or `KRUXOS_AGENT_TOKEN` is unset.
- **Gateway unreachable.** Test with
  `curl -sk -o /dev/null -w '%{http_code}\n' https://localhost:7800`. If the
  dashboard is not responding either, the gateway is down — check
  `docker compose ps` or `systemctl status kruxos-gateway`.

### "Authentication failed" in the bridge logs

The token the bridge is using does not match a registered agent. Re-create
the agent (`kruxos agent create --name hermes`) and copy the token exactly —
agent tokens are 64-char hex strings in v0.0.1. Set the `KRUXOS_BRIDGE_LOG_LEVEL=DEBUG`
env var for verbose output.

### Hermes sees tools but every call errors with "Session not initialized"

Hermes is skipping the MCP `initialize` handshake. Upgrade to the current
Hermes release — older builds (< 0.6) had a bug where they sent `tools/list`
before `initialize`. If you cannot upgrade, raise a Hermes issue; the bridge
faithfully forwards whatever Hermes sends and the gateway requires
`initialize` first per the MCP spec.

### Long-running tool calls time out

The bridge holds tool calls for up to `_EXECUTION_TIMEOUT_SECS` (120 seconds
by default) when an approval is pending. If you frequently approve requests
that take longer to run after approval, raise the timeout by forking the
bridge module or open an issue.

### Approval requests never appear in the dashboard

Check that the gateway sees the tool call at all:

```bash
kruxos audit tail --agent hermes
```

If nothing shows up, Hermes is not dispatching the call (check Hermes logs).
If the call is logged but no approval queues, the matching policy is not set
to `require_approval` — inspect `policies/` for the agent's effective
policy group.

## Next steps

- [Approval Workflow](approval-workflow.md) — how dashboard approvals work
- [Policies](policies.md) — write the policy that governs what Hermes can do
- [Managing Agents](managing-agents.md) — rotate the Hermes API key, set
  rate limits, assign policy groups
- [Running Codex on KruxOS](codex.md) — same pattern for the OpenAI Codex CLI
