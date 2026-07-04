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
mcp-bridge  (/opt/kruxos/bin/mcp-bridge, launched by Codex as a subprocess)
  │  WebSocket
  ▼
KruxOS Gateway  ──►  Policy Engine  ──►  Sandbox  ──►  Capabilities
           │
           └──►  Audit Log  ──►  Dashboard Approvals
```

Same bridge as Claude Desktop, Claude Code, and Hermes — MCP stdio is a
standard transport, so KruxOS ships a single bundled `mcp-bridge` and every
MCP-capable client launches it the same way.

## Prerequisites

- **KruxOS** running and reachable at `ws://HOST:7700`.
- **A KruxOS User token.** Codex CLI runs in the *User context* — the
  operator *is* the user, not a sandboxed agent — so the bridge authenticates
  with the shared User token, not a per-agent token. The first-boot wizard
  seeds one; you can create another any time:
  ```bash
  kruxos user-token create primary
  ```
  The token is stored in the vault under the label you pass (`primary` here)
  and is printed exactly once. In practice you rarely paste it: the bridge
  reads it from the vault by label, so it never lands on argv.
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

The MCP server entry alone is one `codex mcp add` away:

```bash
codex mcp add kruxos-gateway -- \
  /opt/kruxos/bin/mcp-bridge --label primary --gateway ws://127.0.0.1:7700/
```

That writes a `[mcp_servers.kruxos-gateway]` entry into
`~/.codex/config.toml`. But the shipping config also sets a few top-level
keys that `codex mcp add` won't touch, so the hand-edit below is the
complete picture. Open `~/.codex/config.toml` and make it look like this —
this is exactly what `kruxos cli-config generate --codex` emits:

```toml
approval_policy = "never"
sandbox_mode = "danger-full-access"

[shell_environment_policy]
inherit = "core"
exclude = ["*KEY", "*SECRET", "*TOKEN", "ANTHROPIC_*", "OPENAI_API_KEY"]

[features]
hooks = true
shell_tool = false
unified_exec = false

[mcp_servers.kruxos-gateway]
command = "/opt/kruxos/bin/mcp-bridge"
args = ["--label", "primary", "--gateway", "ws://127.0.0.1:7700/"]
tool_timeout_sec = 86400
startup_timeout_sec = 30
required = true

[mcp_servers.kruxos-gateway.env]
KRUXOS_USER_TOKEN = "krx_user_…"
```

What each piece does:

| Key | Purpose |
|-----|---------|
| `approval_policy = "never"` | Suppresses Codex's own per-tool "Always Allow" prompts. Approval still happens — in the KruxOS dashboard queue, which is the single approval surface. |
| `sandbox_mode = "danger-full-access"` | Codex runs in the User context (the operator *is* the user), so it gets full local access. Per-call gating happens at the gateway, not in Codex's own sandbox. |
| `shell_environment_policy.exclude` | Scrubs secrets/keys/tokens out of the environment the bridge subprocess inherits. |
| `features.hooks = true` | Keeps the KruxOS CLI-hook PreToolUse integration active. |
| `features.shell_tool = false` / `unified_exec = false` | Disables Codex's native shell + exec tools so **every** tool call routes through the `kruxos-gateway` MCP server rather than Codex's built-in shell path. |
| `mcp_servers.kruxos-gateway.env.KRUXOS_USER_TOKEN` | The User bearer the bridge presents to the gateway. |

The bridge reads exactly two things from its launch flags — `--label`
(which vault entry to read the User token from, default `primary`) and
`--gateway` (the gateway MCP WebSocket URL, default `ws://127.0.0.1:7700/`) —
plus one environment variable:

| Var                 | Purpose                                            | Default                   |
|---------------------|----------------------------------------------------|---------------------------|
| `KRUXOS_USER_TOKEN` | User bearer token (`krx_user_…`). If unset, the bridge reads it from the vault at `user/token/<label>`. | *(falls back to vault)* |

`kruxos cli-config generate --codex --write` seeds `KRUXOS_USER_TOKEN` from
the vault into the `env` block for you, so the generated file works
out of the box. If you'd rather not have the token in the file at all, drop
the `KRUXOS_USER_TOKEN` line: with the env var unset the bridge resolves the
token from the vault at `user/token/primary` (the `--label` value) on each
launch.

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

- **KruxOS audit log**: `kruxos audit query --capability filesystem.read`
- **Dashboard** (`http://localhost:7800`): the Codex tool call in the
  session view
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
      # Mount the bundled bridge binary from the appliance host (read-only).
      - /opt/kruxos/bin/mcp-bridge:/opt/kruxos/bin/mcp-bridge:ro
    environment:
      KRUXOS_USER_TOKEN: "${KRUXOS_USER_TOKEN}"
      OPENAI_API_KEY: "${OPENAI_API_KEY}"
    command: >
      bash -c "
        npm install -g @openai/codex &&
        mkdir -p /root/.codex &&
        cat > /root/.codex/config.toml <<'TOML'
approval_policy = \"never\"
sandbox_mode = \"danger-full-access\"

[features]
hooks = true
shell_tool = false
unified_exec = false

[mcp_servers.kruxos-gateway]
command = \"/opt/kruxos/bin/mcp-bridge\"
args = [\"--label\", \"primary\", \"--gateway\", \"ws://gateway:7700/\"]
required = true

[mcp_servers.kruxos-gateway.env]
KRUXOS_USER_TOKEN = \"${KRUXOS_USER_TOKEN}\"
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

1. One engineer sets up KruxOS and the User token.
2. They commit `.codex/config.toml` to the project repo (with the token
   read from an env var, not hardcoded — see below).
3. Everyone who clones the repo gets KruxOS governance automatically the
   first time they run `codex` in that directory.

Example `.codex/config.toml` for a repo:

```toml
approval_policy = "never"
sandbox_mode = "danger-full-access"

[features]
hooks = true
shell_tool = false
unified_exec = false

[mcp_servers.kruxos-gateway]
command = "/opt/kruxos/bin/mcp-bridge"
args = ["--label", "primary", "--gateway", "ws://127.0.0.1:7700/"]
required = true

[mcp_servers.kruxos-gateway.env]
# Token resolved from the shell environment at Codex launch time
KRUXOS_USER_TOKEN = "${KRUXOS_USER_TOKEN}"
```

Add `KRUXOS_USER_TOKEN` to `.envrc` / `direnv` / your shell profile, and
*do not* commit the actual token. This pattern gives you centralised
governance for a whole team with one commit.

!!! warning "Don't commit tokens"
    `~/.codex/config.toml` is fine for personal use, but a project-scoped
    `.codex/config.toml` lives in the repo. Use environment-variable
    interpolation (`${VAR}`) for `KRUXOS_USER_TOKEN`, never a literal
    token. Better still, drop the `KRUXOS_USER_TOKEN` line entirely and let
    the bridge resolve the token from the vault by `--label`.

## Codex as an MCP server (not covered here)

Codex can also run *as* an MCP server via `codex mcp-server`, which means
another agent could in theory call Codex as a tool. That's interesting for
multi-agent orchestration, but it's the reverse direction of this guide
(KruxOS governing Codex's tool calls, not Codex exposing itself as a tool
to KruxOS). It is out of scope for the v0.0.x line.

## Troubleshooting

### `codex mcp list` shows `kruxos-gateway` as failed

Run the bridge by hand with the same flags to see the error:

```bash
KRUXOS_USER_TOKEN=krx_user_… \
/opt/kruxos/bin/mcp-bridge --label primary --gateway ws://127.0.0.1:7700/
```

(Omit `KRUXOS_USER_TOKEN` to exercise the vault path — the bridge then reads
`user/token/primary` and the vault must be unlocked.) Structured exit codes
point at the cause: `3` vault locked, `4` no token at that label, `5` vault
unavailable, `6` gateway unreachable, `7` token rejected.

Most common causes:

- **`mcp-bridge` binary not on the path Codex sees.** Codex launches the
  command verbatim; verify `/opt/kruxos/bin/mcp-bridge` is executable and
  reachable from the Codex working directory.
- **Wrong server entry.** `~/.codex/config.toml` must carry a
  `[mcp_servers.kruxos-gateway]` entry with the `--label` / `--gateway`
  args — `codex mcp add` writes the entry but not the surrounding
  `approval_policy` / `sandbox_mode` / `[features]` keys, so finish the
  hand-edit from [Step 1](#step-1-add-kruxos-as-an-mcp-server-in-codex).
- **Gateway unreachable.** `curl -sk -o /dev/null -w '%{http_code}\n'
  https://localhost:7800` — if the dashboard isn't responding the gateway
  is down.

### Tools appear but every call errors with "Session not initialized"

The gateway requires an MCP `initialize` handshake before `tools/list` or
`tools/call`. Codex should do this automatically. If it doesn't, you are on
a very old Codex build — upgrade (`npm install -g @openai/codex`) and try
again. The bridge faithfully forwards whatever the client sends and the
gateway rejects out-of-order traffic per the MCP spec.

### "Authentication failed" in the bridge logs (exit code 7)

The User token the bridge presented does not match a live token (revoked, or
the wrong value). Re-check the token under label `primary`, or mint a fresh
one with `kruxos user-token create primary` and re-seed the config. Set
`KRUXOS_LOG=debug` in the `env` section to see more.

### Tools return "blocked by policy" immediately

This is not a bug — KruxOS is doing its job. Inspect the blocked call:

```bash
kruxos audit query --capability filesystem.read
```

You will see the blocked call with the matching policy rule. Adjust the
policy that governs the User principal to allow it — see
[Policies](policies.md) for how rules are written and applied.

### Approval requests appear in the dashboard but Codex times out

A tool call that needs approval is held at the gateway until a reviewer acts.
The Codex side waits up to `tool_timeout_sec` (the shipping config sets this
to 86400 — 24 hours — so human-in-the-loop approvals don't time out the call
under normal use). A timed-out approval returns a STOP error to Codex — the
call is not silently dropped. If you want a shorter leash, lower
`tool_timeout_sec` in the `[mcp_servers.kruxos-gateway]` entry.

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
