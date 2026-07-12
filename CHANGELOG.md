# Changelog

All notable user-facing changes to KruxOS are documented in this file.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Per-release notes with more narrative detail live under
[`docs/release-notes/`](docs/release-notes/).

## [Unreleased]

### Added

- Community inference-model catalog — the appliance now merges a
  contributor-maintained list of local GGUF models
  (`https://docs.kruxos.com/inference/models.json`) into its built-in set, so
  community models appear in the **Settings → Inference** picker. Every entry
  is hash-verified before use, is additive-only (it can never modify or replace
  a built-in model), and lands only via maintainer-reviewed pull request. New
  **Inference Model Catalog** developer guide covers the file format, the
  field-by-field rules, and how to obtain a real `sha256`/`size_bytes` from
  Hugging Face LFS metadata without downloading the weights. Seeded with
  TinyLlama 1.1B Chat (Q4_K_M, Apache-2.0).

- Documented the **`network.credentialed_request`** capability: the gateway
  attaches an operator-provisioned, vault-stored credential at the network
  boundary; the agent references the secret by name and never sees the
  value, and the secret must be scoped to the capability and host-bound.

- Remote Access guide — recipes for reaching the dashboard from outside
  the LAN with Tailscale, Cloudflare Tunnel, or Ngrok. Covers the
  security trade-offs (expose only the dashboard port; keep the loopback
  User API private; add a tunnel-level identity gate), cost, and
  troubleshooting.

- SSH access — the appliance now bundles an opt-in OpenSSH server, disabled
  by default and enabled from **Settings › System › SSH access**. Root,
  public-key only (password authentication is never enabled); `tcp/22` stays
  firewalled until you add an authorized key, and closes again when the last
  key is removed. Once enabled, SSH stays on across reboot until you disable
  it. SFTP and `scp` ride the same channel as a first-party file-transfer
  path. The File Transfer guide now walks through generating, pasting, and
  fingerprint-verifying an SSH key; documented there and in the security
  whitepaper.

### Changed

- GPU Drivers guide rewritten around the new **one-click Install Now** flow:
  after an NVIDIA license attestation the appliance fetches the version-pinned
  driver directly from NVIDIA and the KruxOS GPU kit from the signed GitHub
  Release, then verifies the kit signature and hash-checks the driver against a
  checksum signed into the kit (an auto-fetched mismatch blocks the install; a
  manual-upload mismatch only warns). The manual two-upload flow stays available
  as the air-gap path. Corrected the framing that KruxOS "never downloads" the
  driver — it never *hosts or redistributes* NVIDIA's driver, which always comes
  straight from NVIDIA.

### Fixed

- Python SDK guide corrected against the shipped `kruxos` client API. Capability
  invocation is `agent.call_async(name, **inputs)` (async) / `agent.call(...)`
  (sync) — there is no `capabilities.invoke`. Connections take an `agent_name`
  and a `ws://` endpoint and close via `close_async()` / `close()` (not
  `disconnect()`); discovery uses `list_async` / `describe_async`; state is
  `state.get_async` / `set_async` / `list_async` / `delete_async` with a `tier=`
  argument (not `state.session` / `persistent` / `shared` sub-objects); approvals
  resolve through `wait_for_approval_async(request_id, ...)`; transactions open
  via `agent.transaction()` with `tx.call_async(...)`; context briefings use
  `briefing_async()`; and the Claude Desktop MCP config is produced by
  `generate_claude_desktop_config(...)`. The Pydantic-model and error-class
  names in the examples now match the package, and every code sample was
  syntax-checked.

- Monitoring guide corrected against the shipped CLI: `kruxos alerts` takes no
  `--last`/`--severity` flags (it lists live resource alerts plus agent-raised
  alerts; use `--json` for machine-readable output); the `alerts.send` examples
  (Monitoring guide and SDK guide) use the real `message`/`severity`/`context`
  fields (there is no `title` field); and the Gmail sync-status section now
  points at `kruxos connect status` for connection state and the dashboard
  **Service Proxy** page for detailed sync / buffer / dead-letter status. The
  proxy overview-strip label matches the UI (**Buffered operations**).

- Security model page: documented the **enforced + audited** use-not-read
  secrets model (deny-by-default scope, a `secrets.use` audit entry per
  resolution, reflected-credential scrubbing) and operator secret
  provisioning (**Settings › Secrets** and `kruxos vault add` with
  capability/host scoping). Corrected the **network port map** — retired
  the supervision port, added the loopback User API (7703) and health
  (7704) ports, clarified that the agent gateway binds `0.0.0.0` (per-Agent
  bearer is the boundary), and added a **TLS reverse-proxy** note.

- Backup & Restore guide: corrected the encryption description — state
  backups are AES-256-GCM with a key derived from the **vault passphrase
  via Argon2id** (with a per-file salt), and older backups still restore
  (the format version is auto-detected).

- Security whitepaper: supervision is now described as a **root-only local
  control socket** (peer-credential, uid 0) instead of a network port with
  an admin passphrase — the AC-17 mapping row, the principal table, and the
  TLS/limitations sections were corrected. Also documents the **opt-in
  Tailscale tailnet** surface (userspace daemon, `tailscale serve`
  publishing only the dashboard — plus the opt-in SSH console when enabled —
  an egress guard fencing the loopback ports). Corrected three residual
  claims: the `/code` CLI tools mint their credential over the distinct
  **auth socket** (`/run/kruxos/auth.sock`), not the uid-0 control socket;
  and the agent gateway (7700) binds **`0.0.0.0`** by default (per-Agent
  bearer is the boundary; restrict via `server.host`), not `127.0.0.1` —
  fixed consistently across the threat model, §4.4, and the SSRF note.

- Model Providers guide: documented the unified **Local model
  (self-hosted)** provider — the engine sub-selector (Ollama native /
  OpenAI-compatible over `/v1` / KruxOS on-box engine), the optional API
  key for keyed OpenAI-compatible servers, and the system-managed on-box
  engine (added from **Settings › Local Models**). Also corrected the
  context-compaction descriptions: OpenAI/Codex use client-side, same-model
  compaction (there is no server-side `compact_threshold`), and Anthropic's
  native server-side compaction applies to its 4.6 models. Tightened the
  Anthropic tool-clearing note — native tool-clearing is **Haiku 4.5 /
  Sonnet 4+ / Opus 4+ only**; Claude 3.x is client-side only. Fixed the
  `models.yaml` key-registration example: model-provider keys are added
  through the managed flow (`kruxos model add` or **Settings › Models**),
  not `kruxos vault add` — which refuses the managed `model-provider:*`
  namespace.

- Install guide: removed the retired supervision port from the Docker,
  QEMU, VirtualBox and firewall steps and the port tables; recommended
  **bridged** networking for the VM (real LAN IP, no forwards); and
  replaced the misleading "20 GiB disk minimum" with the real model —
  the image is ~8 GiB and **`/data` auto-expands to fill the disk on
  first boot**. Clarified that grow-to-fill is a **one-time, first-boot**
  step: size the disk *before* first boot, because enlarging it later
  does not re-expand `/data` on its own and requires a manual partition
  + `resize2fs` grow (it is a required step, not the optional convenience
  it was first framed as). The first-boot firewall line now lists the
  ports actually accepted — **TCP 7700 + 7800 only** (7702 trigger-wake
  is UDP on loopback, with no inbound rule). Dropped stale
  v0.0.1-specific framing.

- Pack authoring guides: the manifest file is **`manifest.yaml`** (the
  Pack SDK scaffolds and requires it — `pack.yaml` is rejected), and the
  `capabilities:` list holds **definition-file paths** (e.g.
  `definitions/weather.yaml`), not capability name-strings. Updated the
  quickstart, publishing, and service-proxy pack guides.

- Capability reference: corrected the counts to the current registry —
  **92 typed capabilities across 13 categories** (was 89). The Network
  row now shows **5** (adds the gateway-mediated credentialed request),
  Filesystem **13**, and Git **9**; the Git row also no longer lists a
  non-existent "stash" capability.

- Monitoring guide: corrected the health endpoint reference to the real
  API — port **7704** (was 7701), the `services` field is a JSON **array**
  (was an object), the overall status value is **`critical`** (was
  "unhealthy"), and the example response now matches the live field names.

- Documentation site dark mode: restored the code-block padding that was
  missing in the dark (slate) theme, so code samples no longer sit flush
  against their border. Light mode was unaffected.

## [0.0.2] - 2026-06-08

Second release. Still early beta — not for production use. The largest
cycle so far: a full dashboard redesign, capability Packs with a
sandboxed runtime and a public authoring SDK, Slack and Gmail Service
Proxy connectors, a self-updating appliance, and per-agent workspace
isolation across every capability.

### Added

**Dashboard redesign**

- Every dashboard page rebuilt on a shared design system, with light/dark
  theming, consistent iconography and typography, and the canonical
  KruxOS wordmark. Redesigned surfaces: Home, Chat, Code Sessions,
  Agents, Agent detail, Approvals, Activity, Audit, Identities,
  Integrations, Settings (now tabbed), Health, Service Proxy, and the
  new Packs page. The first-boot wizard, login, and sidebar were
  re-shelled in the same pass; all model-provider types and the OAuth
  device-code flow are preserved.
- Policy editor redesign — wildcard support, auto-save on visual
  changes, and an unsaved-changes warning before navigating away.

**Capability Packs**

- New Packs page to install a pack from the registry or by uploading a
  local pack file, list installed packs, and remove them. `kruxos pack
  install <name>` now resolves and fetches packs from the remote
  registry, not local paths only.
- Pack capabilities now execute inside the same forked, per-agent
  sandbox as the built-in capabilities — pack code is isolated exactly
  like first-party code.
- Installed pack capabilities are visible to the platform: they appear
  in the agent's capability list, the policy editor, and the tool
  listing. Installing or removing a pack from the dashboard takes effect
  immediately, with no gateway restart.
- Public Pack SDK — `@kruxos/pack-sdk` published to npm, with a
  getting-started guide and Capability Design Guidelines for authors.

**Service Proxy — Slack & Gmail**

- Operators can connect Slack and Gmail accounts from the dashboard. The
  Service Proxy syncs a local replica of your messages in the background
  and exposes a manual re-sync; a new Service Proxy page shows
  connection state and token expiry.
- Slack is the runtime-verified connector this release — its
  capabilities (channels, send, reply, react, search, read) operate
  against the synced replica and were exercised end-to-end on the
  appliance.
- Gmail connect ships on the same infrastructure but was not separately
  runtime-verified this cycle and requires your own Google Cloud OAuth
  app; treat it as early/experimental.

**Self-updating appliance**

- Settings → Updates dashboard flow: check, download, apply, and reboot
  into a new release entirely from the UI.
- Releases are Ed25519-signed and verified against the appliance's
  baked-in public key before applying. Updates are written to the
  inactive A/B root slot; a health monitor watches the freshly-booted
  slot and rolls back to the last known-good slot automatically if it
  fails its health checks.
- The `kruxos update check / download / apply / reboot` CLI is fully
  wired.

**Agents & capabilities**

- Per-agent workspace isolation now spans every filesystem-touching
  capability — filesystem, git, and process — through a single enforced
  chokepoint. Each agent gets its own workspace, created on add and
  backfilled at boot, and its system prompt advertises only that
  workspace.
- New `git.init` capability — agents can initialize a new repository in
  their workspace.

**Vault**

- `kruxos vault unlock` now unlocks the vault for sibling CLI commands,
  instead of each command seeing its own locked state.
- Re-authenticate a bad model provider from the dashboard — removing a
  provider clears its leftover secret, and a re-auth button lets you
  re-link an OAuth provider without hand-editing the vault.

**Code sessions & CLI integrations**

- `/code` improvements — rename tabs, a "gateway restarted, refresh"
  banner, a re-attach hint on tab close, a directory picker for the
  working directory, and dark-mode-visible spawn-modal pickers.
- Claude Code / Codex sign-in now persists across code sessions.
- Integrations page — per-CLI Update and Uninstall buttons, a working
  View Config, and corrected config-path text.

**Approvals & audit**

- Approval rows surface which CLI (Claude Code or Codex) initiated the
  request. The audit row-expand view surfaces the related
  approval-request id.

**Operator file transfer**

- A new Uploads page and file API give operators a documented way to
  move files to and from the appliance. (The page ships this release; it
  was not separately runtime-verified in the pre-tag walk.)

### Changed

- First-boot wizard refinements — the license step reflects the current
  pricing model; OpenAI/Codex OAuth no longer blocks first boot
  (retry-or-disable); a OAuth-completed provider is linked as the new
  agent's default model; clearer Skip/Continue on the CLI-install step;
  the "Seed config" button now reads "Install"; and the pre-login banner
  merges first-boot info with the login banner.
- The `/chat` "No model provider" error now points to the Agents page
  instead of a CLI command.
- The `git` CLI now ships on the appliance, so a raw-shell fallback no
  longer hits `git: not found`. (The `gh` GitHub CLI is intentionally
  not bundled — KruxOS exposes a curated `github.*` capability surface
  instead.)
- The appliance version is now derived from a single source, so it no
  longer reports a stale value that blocked updates from registering as
  newer.
- The update server moved onto GitHub Releases of the public KruxOS
  repository.

### Fixed

- The appliance firewall now loads correctly on boot on both
  architectures (a missing kernel option previously aborted the entire
  ruleset, leaving the appliance with no firewall).
- Slack search/read returned no messages despite messages being synced;
  unfiltered search now returns synced messages, and a single failing
  channel during sync no longer aborts the whole sync.
- Dark-theme dropdown options were invisible (white-on-white) on several
  pages.
- `kruxos code list` / `kill` no longer panics.
- The console pre-login and message-of-the-day banners no longer claim a
  stale version after an OS update — query the live version via `kruxos
  status` or the dashboard; ASCII art refreshed.
- Numerous dashboard polish fixes across the directory browser, add-mount
  modal, approvals layout, sign-in button, agents create button, sidebar,
  and activity/audit filters.

## [0.0.1] - 2026-05-14

First public release. Early beta — not for production use.

KruxOS is a purpose-built execution layer for AI agents: a gateway that
mediates every tool call against a deterministic policy engine, an
approval queue for the operations the operator wants to see, and a
capability registry that gives agents structured access to filesystem,
process, network, git, scheduler, comms, state, secrets, email, and
Slack. v0.0.1 ships as a self-hosted appliance — bootable VM image
(x86_64 + aarch64), Docker image, dashboard, CLI, and an in-appliance
Python SDK.

### Added

- **Agent gateway** — MCP-native on TCP 7700 with JSON-RPC fallback;
  supervision WebSocket on 7701; UDP trigger-wake on 127.0.0.1:7702;
  bearer-auth User API on 7703 (distinct principal from token-auth
  agents); dashboard on 7800 (HTTPS by default). Bridges for Claude Code
  and OpenAI Codex. `KRUXOS_ENV=production` enables fail-closed startup.
- **89 typed capabilities across 13 categories** — filesystem, process,
  network, git, scheduler, system, agent, state, comms, secrets, email,
  Slack, alerts. MCP `tools/list` and JSON-RPC `capabilities.list`
  annotate each with its policy tier; `blocked` capabilities are omitted.
- **Deterministic policy engine** — four permission tiers (`autonomous`
  / `notify` / `approval_required` / `blocked`), no LLM in the
  evaluation path, hot-reloadable YAML, per-agent overrides, and a
  visual + YAML editor on the dashboard.
- **Approval queue** — persistent, surfaced on the dashboard with
  Pending / Approved / Rejected / Timed Out tabs; default 24-hour hold
  for User MCP calls; timed-out approvals cannot be approved
  retroactively.
- **Secrets vault** — AES-256-GCM SQLite store with a use-not-read
  contract; OAuth provider tokens stored with auto-refresh.
- **Audit log** — CBOR framing, hash-chained for tamper evidence,
  Principal-aware actor field, daily rotation with 90-day default
  retention, and bounded-memory degraded mode on disk-full (no silent
  audit loss).
- **Agent runtime** — autonomous five-field cron schedules, one-shot
  delays, manual and UDP-wake triggers; per-agent state scopes; per-agent
  `Agent.md` identity; per-agent host mounts with path-escape detection;
  topic-based inter-agent comms; per-principal soft-delete trash with
  restore; OpenClaw compatibility bridge.
- **Per-agent sandbox** — Linux user/network namespaces, cgroup v2
  limits, seccomp BPF allowlist, nftables defense-in-depth, applied per
  capability call via a forked child. `kruxos sandbox diagnose` reports
  per-primitive status.
- **Model providers** — Anthropic, OpenAI (plus OpenAI-compatible
  endpoints), OpenAI Codex (device-code OAuth), OpenRouter, Google
  Gemini, and Local (Ollama / vLLM / LM Studio / llama.cpp).
  Provider-native prompt caching, two-tier context compaction with
  templates, and configurable thinking-effort levels.
- **Dashboard** — real-time supervision, approval queue, audit viewer,
  multi-model chat with persisted sessions, Agents page (templates,
  model overrides, identity, per-agent policy, host mounts), visual +
  YAML policy editor, Settings, Identities, Integrations, Code Sessions,
  and a first-boot wizard.
- **Host-CLI integrations** — `mcp-bridge` and `cli-hook` launchers, and
  `kruxos cli-config generate` to emit seed configs for Claude Code and
  Codex with MCP hardening.
- **Code sessions** — `kruxos code list / kill / attach` against the
  loopback User API, with concurrent-session caps, per-session memory
  caps, idle timeout, and workdir validation.
- **CLI** — a single `kruxos` binary covering activate, agent, audit,
  cli-config, code, migrate, model, mount, pack, sandbox, state backup,
  trash, user-token, vault, verify, version, status, and watch, with
  shell completions and on-demand man pages.
- **Distribution** — bootable image as `.img.gz`, `.qcow2`, `.vmdk`, and
  Vagrant `.box` for x86_64 and aarch64; Docker image; in-appliance
  Python SDK. All artefacts cosign-signed for offline verification.
- **Licensing** — Ed25519-signed JWT verifier; activate via
  `kruxos activate` or the dashboard wizard.

### Changed

- Approval queue is the single approval surface — Claude Code and Codex
  ship configured to defer all tool authorisation to KruxOS, with no
  per-tool prompts inside either CLI.
- `process.run` returns a `timed_out` state with partial output on
  timeout rather than a hard error; `git.status` returns a single
  structured response; `filesystem.read` defaults to a 1 MiB byte cap.
- Provider type "Ollama" renamed to "Local" — covers any
  OpenAI-compatible local endpoint.

### Security

- KruxOS is the single approval and policy-enforcement surface for AI
  tool calls; no in-CLI approval prompts, no governance bypass via a
  CLI's native shell tool.
- Mass-destruction commands blocked unconditionally (`rm -rf` of system
  paths, `dd of=/dev/sd*`, `mkfs.* /dev/...`, redirects to raw block
  devices).
- Per-principal soft-delete trash with retention; TLS on the dashboard
  by default; ws-proxy Origin pinning on code-session upgrades.
- Console root login is bound to the vault passphrase — the same secret
  unlocks the vault, the dashboard login, and console root.
- Release artefacts signed for offline verification; a security
  disclosure contact is published per RFC 9116.

[0.0.2]: https://github.com/altvale/kruxos/releases/tag/v0.0.2
[0.0.1]: https://github.com/altvale/kruxos/releases/tag/v0.0.1
