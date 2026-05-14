# Running Codex CLI on KruxOS

[OpenAI Codex CLI](https://github.com/openai/codex) is the command-line agent
interface OpenAI ships with ChatGPT and API plans. It has first-class MCP
client support baked in, which means you can bring every file, process,
network, and git operation Codex performs under KruxOS governance with a
single `codex mcp add` command.

This guide is the direct counterpart to the [Hermes Agent guide](hermes.md):
same bridge, different client, a few Codex-specific twists around the
Landlock sandbox and project-scoped config.

## Why connect Codex to KruxOS?

Codex already runs every tool call inside a Landlock-based sandbox — that's
the right instinct, and it's the same security philosophy KruxOS takes. What
Codex does not give you is:

- A **policy engine** that can express "this agent may read `/workspace` but
  never write to `/etc`" as data, not shell arguments.
- A **dashboard approval queue** so a human can stop a dangerous tool call
  before it executes, with full input context.
- An **audit log** that chains every capability invocation across every
  agent in your organisation into a single tamper-evident stream.
- A **uniform governance surface** across Claude, Hermes, Codex, and your
  own Python SDK agents — all three clients hit the same capabilities
  through the same policies.

Connecting Codex to KruxOS layers these on top of Codex's existing sandbox
without replacing anything Codex already does well.

Architecture:

```
Codex CLI  (Rust binary)
  │  stdio (newline-delimited JSON-RPC, MCP 2024-11-05)
  ▼
kruxos.connectors.claude_bridge  (launched by Codex as a subprocess)
  │  WebSocket
  ▼
KruxOS Gateway  ──►  Policy Engine  ──►  Sandbox  ──►  Capabilities
           │
           └──►  Audit Log  ──►  Dashboard Approvals
```

Same bridge as Claude Desktop, Claude Code, and Hermes — MCP stdio is a
standard transport, so KruxOS ships a single `kruxos.connectors.claude_bridge`
Python module and every MCP-capable client launches it the same way.

## Prerequisites

- **KruxOS** running and reachable at `ws://HOST:7700`.
- **A KruxOS agent token**:
  ```bash
  kruxos agent create --name codex
  ```
  Save the 64-char hex token.
- **The bundled `mcp-bridge`** at `/opt/kruxos/bin/mcp-bridge` — ships on the
  appliance (VM or Docker image).
- **Codex CLI installed** (`npm install -g @openai/codex`, or whichever
  install method your plan uses). Either an API key or a ChatGPT
  subscription — KruxOS does not care which, the bridge runs downstream of
  Codex's auth.

!!! tip "Use the seed-config generator instead of hand-editing"
    On the appliance, `kruxos cli-config generate --write` emits the right
    `~/.codex/config.toml` + `~/.codex/hooks.json` stanzas (and, with root,
    `/etc/codex/requirements.toml`) referencing the bundled `mcp-bridge`,
    with the token pulled from the vault — never on argv. The hand-written
    approach below is provided for reference if you're managing Codex's
    config outside the appliance.

!!! warning "apply_patch routing"
    In v0.0.1, Codex's built-in `apply_patch` tool is **not yet routed through
    the KruxOS approval queue** — it's an upstream limitation. The MCP-proxy
    fix that closes that gap lands in **v0.0.4**. Other Codex tools (`shell`,
    `unified_exec`) are disabled at the user-config and requirements layers,
    so all governable surfaces route through KruxOS already.

## Step 1 — Add KruxOS as an MCP server in Codex

The hand-edit alternative (when `kruxos cli-config generate` isn't an option):

```bash
codex mcp add kruxos -- /opt/kruxos/bin/mcp-bridge
```

This writes a `[mcp_servers.kruxos]` entry into `~/.codex/config.toml`.
You'll need to add the environment variables by hand. Open
`~/.codex/config.toml` and make the entry look like this:

```toml
[mcp_servers.kruxos]
command = "/opt/kruxos/bin/mcp-bridge"
args = []

[mcp_servers.kruxos.env]
KRUXOS_ENDPOINT = "wss://localhost:7700"
KRUXOS_AGENT_NAME = "codex"
KRUXOS_AGENT_TOKEN = "7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c"
```

These are the three env vars the bridge reads:

| Var                  | Purpose                                  | Default                  |
|----------------------|------------------------------------------|--------------------------|
| `KRUXOS_ENDPOINT`    | Gateway WebSocket URL                    | `wss://localhost:7700`   |
| `KRUXOS_AGENT_NAME`  | Agent name (from `kruxos agent create`)  | *(required)*             |
| `KRUXOS_AGENT_TOKEN` | Agent token (64-char hex)                | *(required)*             |

Codex launches the bridge as a child process on startup, pipes MCP messages
over its stdin/stdout, and the bridge proxies everything to the gateway.

## Step 2 — Verify the connection

List the configured MCP servers:

```bash
codex mcp list
```

The `kruxos` entry should appear and show as running. If it does not start,
jump to [Troubleshooting](#troubleshooting).

Then ask Codex to use a KruxOS tool:

```
$ codex
> Use the kruxos filesystem_read tool to read /workspace/README.md.
```

Codex will dispatch the call through the bridge. Tool names that contain
dots on the KruxOS side (`filesystem.read`, `network.http_request`) are
rewritten to underscores (`filesystem_read`, `network_http_request`) on
the way out because MCP tool names must match `^[a-zA-Z0-9_-]{1,128}$`.
The bridge restores the dotted name before forwarding the call to the
gateway, so the audit log records `filesystem.read`, not the mangled form.

You should see the call in:

- **KruxOS audit log**: `kruxos audit tail --agent codex`
- **Dashboard** (`http://localhost:7800`): a new session for agent `codex`
- **Approval queue** (if policy is `require_approval`): a pending request

## Sandbox layering: Codex Landlock + KruxOS sandbox

Codex has its own Landlock-based sandbox for tool execution, and KruxOS has
its own sandbox at the gateway level (namespaces, cgroups, seccomp,
Landlock, nftables — see [security-model](../enterprise/security-model.md)).
When you run Codex against KruxOS via MCP, both sandboxes apply in series:

```
Codex tool dispatch  ─►  Codex Landlock  ─►  bridge stdio
                                                │
                                                ▼
                                          KruxOS Gateway
                                                │
                                                ▼
                                          KruxOS Sandbox  ─►  Capability handler
```

Each layer catches different things. Codex's Landlock is scoped to the
Codex process and prevents *Codex itself* from touching files outside its
working directory. KruxOS's sandbox is scoped to *the capability handler*
that runs on the gateway side of the bridge, and enforces the agent's
workspace policy regardless of where the client is running. They are not
redundant — they defend against different threat models:

| Threat                                             | Caught by     |
|----------------------------------------------------|---------------|
| Prompt injection makes Codex decide to `rm -rf /`  | Codex Landlock + KruxOS policy |
| Bug in Codex lets a tool call bypass Codex's sandbox | KruxOS sandbox |
| Compromised bridge or gateway process              | Codex Landlock (nothing else) |
| Agent operating outside its assigned workspace     | KruxOS policy + sandbox |

For the getting-started setup below (Mode A — Codex on the host, KruxOS in
a container), you get both. For the production setup (Mode B — Codex
*inside* a KruxOS agent container), Codex's Landlock becomes redundant with
KruxOS's container isolation. Both modes are valid; pick based on how much
operational complexity you want.

## Mode A: Codex on the host (getting started)

This is what Step 1 above sets up. Codex runs on your laptop or dev VM,
connects to a KruxOS gateway that runs wherever — Docker, the OS image, a
remote host. This is the simplest mode and gives you the full governance
picture on day one.

Pros: zero container orchestration, fastest path to a working demo, works
with `codex mcp add` out of the box.

Cons: Codex still has direct host access for its own internal state (working
directory, temp files, cache), so KruxOS only governs what goes through MCP
tool calls — not what Codex does on its own.

## Mode B: Codex inside a KruxOS agent container (production)

For full governance, run Codex as an agent *inside* a KruxOS agent
container. Every file Codex touches — working directory, temp files, cache,
output — is then inside the KruxOS sandbox, not just the tool calls it makes.

KruxOS already ships a multi-container agent isolation pattern in
`docker-compose.agent.yml`. Add a Codex service on top:

```yaml
# docker-compose.codex.yml
services:
  codex-agent:
    image: node:20-bookworm
    container_name: kruxos-codex-agent
    depends_on:
      kruxos:
        condition: service_healthy
    working_dir: /workspace
    networks:
      codex-net:
    volumes:
      - codex-workspace:/workspace
    environment:
      KRUXOS_ENDPOINT: "ws://gateway:7700"
      KRUXOS_AGENT_NAME: "codex"
      KRUXOS_API_KEY: "${CODEX_API_KEY}"
      OPENAI_API_KEY: "${OPENAI_API_KEY}"
    command: >
      bash -c "
        apt-get update && apt-get install -y python3 python3-pip &&
        pip install --break-system-packages kruxos &&
        npm install -g @openai/codex &&
        mkdir -p /root/.codex &&
        cat > /root/.codex/config.toml <<'TOML'
[mcp_servers.kruxos]
command = \"python3\"
args = [\"-m\", \"kruxos.connectors.claude_bridge\"]

[mcp_servers.kruxos.env]
KRUXOS_ENDPOINT = \"ws://gateway:7700\"
KRUXOS_AGENT_NAME = \"codex\"
KRUXOS_API_KEY = \"${KRUXOS_API_KEY}\"
TOML
        tail -f /dev/null
      "

volumes:
  codex-workspace:

networks:
  codex-net:
    driver: bridge
```

Start it alongside the gateway:

```bash
docker compose -f docker-compose.yml \
               -f docker-compose.agent.yml \
               -f docker-compose.codex.yml up -d
```

Then `docker exec -it kruxos-codex-agent codex` drops you into Codex inside
the sandboxed container. The gateway is reachable at `ws://gateway:7700`
on the compose network, and Codex's filesystem is confined to the
`codex-workspace` volume mounted at `/workspace`.

In Mode B you can disable Codex's own Landlock sandbox if you want — the
KruxOS container already isolates Codex from the host — but leaving it on
costs nothing and gives you defense-in-depth.

## Project-scoped config

Codex supports `.codex/config.toml` inside a project directory, and it
takes precedence over `~/.codex/config.toml` when Codex is launched from
that directory. This is the most underrated Codex feature for team setups:

1. One engineer sets up KruxOS and the agent API key.
2. They commit `.codex/config.toml` to the project repo (with the API key
   read from an env var, not hardcoded — see below).
3. Everyone who clones the repo gets KruxOS governance automatically the
   first time they run `codex` in that directory.

Example `.codex/config.toml` for a repo:

```toml
[mcp_servers.kruxos]
command = "/opt/kruxos/bin/mcp-bridge"
args = []

[mcp_servers.kruxos.env]
KRUXOS_ENDPOINT = "wss://localhost:7700"
KRUXOS_AGENT_NAME = "codex-team"
# Token resolved from the shell environment at Codex launch time
KRUXOS_AGENT_TOKEN = "${CODEX_KRUXOS_TOKEN}"
```

Add `CODEX_KRUXOS_TOKEN` to `.envrc` / `direnv` / your shell profile, and
*do not* commit the actual token. This pattern gives you centralised
governance for a whole team with one commit.

!!! warning "Don't commit tokens"
    `~/.codex/config.toml` is fine for personal use, but a project-scoped
    `.codex/config.toml` lives in the repo. Use environment-variable
    interpolation (`${VAR}`) for `KRUXOS_AGENT_TOKEN`, never a literal
    64-char hex token.

## Codex as an MCP server (not covered here)

Codex can also run *as* an MCP server via `codex mcp-server`, which means
another agent could in theory call Codex as a tool. That's interesting for
multi-agent orchestration, but it's the reverse direction of this guide
(KruxOS governing Codex's tool calls, not Codex exposing itself as a tool
to KruxOS). It is out of scope for the v0.0.x line.

## Troubleshooting

### `codex mcp list` shows `kruxos` as failed

Run the bridge by hand with the same env vars to see the error:

```bash
KRUXOS_ENDPOINT=wss://localhost:7700 \
KRUXOS_AGENT_NAME=codex \
KRUXOS_AGENT_TOKEN=7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c \
/opt/kruxos/bin/mcp-bridge
```

Structured exit codes (10 = auth, 11 = network) point at the cause.

Most common causes:

- **`mcp-bridge` binary not on the path Codex sees.** Codex launches the
  command verbatim; verify `/opt/kruxos/bin/mcp-bridge` is executable and
  reachable from the Codex working directory.
- **Env vars missing.** `~/.codex/config.toml` has to have the
  `[mcp_servers.kruxos.env]` subsection — `codex mcp add` does not create
  it for you. Edit the file and add the env vars manually.
- **Gateway unreachable.** `curl -sk -o /dev/null -w '%{http_code}\n'
  https://localhost:7800` — if the dashboard isn't responding the gateway
  is down.

### Tools appear but every call errors with "Session not initialized"

The gateway requires an MCP `initialize` handshake before `tools/list` or
`tools/call`. Codex should do this automatically. If it doesn't, you are on
a very old Codex build — upgrade (`npm install -g @openai/codex`) and try
again. The bridge faithfully forwards whatever the client sends and the
gateway rejects out-of-order traffic per the MCP spec.

### "Authentication failed" in the bridge logs

The `KRUXOS_AGENT_TOKEN` value does not match a registered agent. Re-check
the token you saved from `kruxos agent create --name codex`. Agent tokens
are 64-char hex strings in v0.0.1. Set `KRUXOS_BRIDGE_LOG_LEVEL=DEBUG` in the env
section to see more.

### Tools return "blocked by policy" immediately

This is not a bug — KruxOS is doing its job. Inspect the policy:

```bash
kruxos audit tail --agent codex
```

You will see the blocked call with the matching policy rule. Edit the
policy file in `policies/` or assign the `codex` agent to a different
policy group:

```bash
kruxos agent set-policy codex --group permissive
```

### Approval requests appear in the dashboard but Codex times out

The bridge waits up to 5 minutes (`_APPROVAL_TIMEOUT_SECS`) for an approval
decision, and 2 minutes (`_EXECUTION_TIMEOUT_SECS`) for execution after
approval is granted. If your reviewers are slower than that, increase both
constants in `claude_bridge.py` or approve faster. A timed-out approval
returns a STOP error to Codex — the call is not silently dropped.

### Codex's own sandbox blocks a legitimate tool call

If a tool call *reaches the gateway* and gets a real result but fails
somewhere in Codex's own handling, that's Codex's Landlock sandbox catching
something, not KruxOS. Check Codex's logs; KruxOS's audit log will show the
call succeeded.

## Next steps

- [Approval Workflow](approval-workflow.md) — dashboard approvals in detail
- [Policies](policies.md) — write the policy that governs the `codex` agent
- [Model Providers](model-providers.md) — Codex as a *model* (the other
  direction: KruxOS calling the OpenAI Codex API to back its own chat
  sessions — not to be confused with this guide, which is Codex calling
  KruxOS as a tool)
- [Running Hermes on KruxOS](hermes.md) — same integration pattern for
  Hermes Agent
- [Docker Agent Isolation](docker-isolation.md) — the compose file Mode B
  builds on
