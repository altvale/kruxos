# Getting Started with KruxOS

This page walks you from zero to a working KruxOS instance with a connected AI agent. Total time: **under 15 minutes**.

---

## Step 1: Run KruxOS (5 minutes)

=== "Docker (fastest)"

    ```bash
    docker run -d --name kruxos --privileged \
      -e KRUXOS_VAULT_PASSPHRASE='choose-a-strong-passphrase' \
      -p 7800:7800 \
      -p 7700:7700 \
      -p 7701:7701 \
      -v kruxos-data:/data/kruxos \
      altvale/kruxos:latest
    ```

    Open <https://localhost:7800> and finish the eight-step first-boot wizard: welcome, vault passphrase, workspace (with a click-through directory browser for the AdminAgent home directory), AdminAgent identity (with optional inline model-provider config — Anthropic / OpenAI / OpenAI Codex via OAuth / OpenRouter / Local, or Skip to defer), license, User token, Install CLI Tools, done.

    Verify it's running:

    ```bash
    docker exec kruxos kruxos verify
    ```

    !!! tip "Full Docker reference"
        See [Install KruxOS](quickstart/install.md) for port details, sandbox notes, and troubleshooting.

    !!! warning "Code Sessions not supported on Docker in v0.0.1"
        Code Sessions (`/code` page + `kruxos code …` subcommands) need cgroup v2 delegation that isn't reliable through Docker. All other features work normally. Docker-side fix ships in v0.0.2.

=== "VM image (full appliance)"

    Download the artefact for your hypervisor from <https://github.com/altvale/kruxos/releases>:

    - `kruxos-x86_64.img.gz` / `kruxos-aarch64.img.gz` — raw disk image
    - `kruxos-x86_64.qcow2` — libvirt / KVM / QEMU
    - `kruxos-x86_64.vmdk` — VMware / VirtualBox
    - `kruxos-x86_64.box` — Vagrant (libvirt; x86_64 only)

    All artefacts are cosign-signed; per-artefact `.cosign.bundle` files include the Fulcio cert + Rekor inclusion proof for offline verification.

    See [Install KruxOS](quickstart/install.md#option-2-vm-image-full-appliance--code-sessions--sandbox) for the boot walkthrough.

---

## Step 2: Connect your AI model (5 minutes)

### Claude Code (recommended — MCP-native, zero adapter code)

On the appliance:

```bash
kruxos cli-config generate --write
```

Writes `~/.claude/settings.json` referencing the bundled `/opt/kruxos/bin/mcp-bridge`, with the agent token pulled from the vault. Claude Code's native shell tool is disabled at the user-config and requirements layers so every tool call routes through the KruxOS approval queue.

Restart Claude Code. Ask: *"What tools do you have from KruxOS?"* — Claude should list the capabilities visible to your policy tier.

!!! tip "Other clients"
    - [Claude Desktop / Claude API](quickstart/connect-claude.md)
    - [OpenAI / Codex CLI](quickstart/connect-openai.md)
    - [Gemini](quickstart/connect-gemini.md)
    - [Local models](quickstart/connect-local.md) — Ollama, vLLM, LM Studio, llama.cpp

### Python SDK (programmatic access)

The Python SDK ships **bundled inside the appliance** at `/opt/kruxos/sdk/python/` (auto-importable via `/etc/profile.d/kruxos-sdk.sh`). The external `pip install kruxos` distribution to PyPI ships in **v0.0.3**.

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
        result = await os.call_async("system.time")
        print(f"Server time: {result.data['utc']}")

        caps = await os.capabilities.list_async()
        print(f"Available: {len(caps)} capabilities")
    finally:
        await os.close_async()

asyncio.run(main())
```

---

## Step 3: Explore (5 minutes)

### Web Dashboard

Open <https://localhost:7800> in your browser (HTTPS-by-default with an auto-generated self-signed cert — accept the prompt on first visit).

| Page | What it shows |
|------|--------------|
| **Home** | System health, active agents, recent activity, pending approvals |
| **Supervision** | Real-time stream of every capability invocation (TCP 7701 WS) |
| **Agents** | Templates (Coder / Researcher / DevOps / Email / General), model overrides, host mounts |
| **Approvals** | Pending / Approved / Rejected / Timed Out tabs; default 24-hour hold for User MCP calls |
| **Audit** | Searchable hash-chained audit log with Principal-aware filtering |
| **Chat** | Multi-model chat with persisted sessions, knowledge panel, inline approvals |
| **Code Sessions** (`/code`) | xterm.js terminals through the sandbox — VM image only in v0.0.1 |
| **Identities** | User token CRUD with one-time raw-token reveal |
| **Integrations** | Claude Code / Codex install, regenerate seed configs |
| **Policies** | Visual + YAML editor, per-agent overrides |
| **Settings** | One card per model provider (Anthropic, OpenAI, Codex, OpenRouter, Gemini, Local, DeepSeek, Grok, Mistral, Groq, GLM) |

### CLI

```bash
# Docker: prefix with `docker exec kruxos`
kruxos status                  # System overview
kruxos agent list              # Connected agents
kruxos audit query --last 1h   # Recent audit events
kruxos --help                  # Full command reference
```

To enumerate capabilities, query MCP `tools/list` or JSON-RPC `capabilities.list` over the Gateway — each entry is annotated with its policy tier; `blocked` capabilities are omitted.

### Try these prompts with Claude

Once connected, try these in a conversation:

- *"What time is it on the server?"* — calls `system.time`
- *"List files in the workspace"* — calls `filesystem.list`
- *"Create a file called hello.txt with 'Hello World'"* — calls `filesystem.write` (may require approval)
- *"What agent am I?"* — calls `agent.whoami`
- *"Show me system info"* — calls `system.info`

---

## What to do next

- **[Policies](guides/policies.md)** — four-tier permission model (`autonomous` / `notify` / `approval_required` / `blocked`)
- **[Approval Workflow](guides/approval-workflow.md)** — how gated calls flow through the queue
- **[Managing Agents](guides/managing-agents.md)** — create agents with different permission tiers
- **[Model Providers](guides/model-providers.md)** — add OpenAI / Codex / Gemini / local providers
- **[Email capabilities](quickstart/gmail.md)** — adapters ship today; operator-facing OAuth lands v0.0.2
- **[Autonomous Agents](guides/autonomous-agents.md)** — five-field cron schedules, one-shot delays, manual trigger
- **[Dashboard Chat](guides/dashboard-chat.md)** — chat with your agent from the web UI
- **[Pack Development](developers/packs/quickstart.md)** — local-path packs work today; registry + publishing flow ships v0.0.2
- **[Security Whitepaper](security/whitepaper.md)** — sandbox primitives, identity model, secrets vault
