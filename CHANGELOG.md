# Changelog

All notable user-facing changes to KruxOS are documented in this file.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Per-release notes with more narrative detail live under
[`docs/release-notes/`](docs/release-notes/).

## [Unreleased]

### Added

- **Shared GitHub repository links now reuse the canonical KruxOS social header.**
- **The whole-disk fresh-install image now carries a keyless cosign supply-chain signature.**
- **The Docker container now warns at startup when its sandbox is degraded.**
- **Persistent license banner + nav attention marker across the whole dashboard.**
- **Audit and Activity rows now show which CLI initiated each action.**
- **Grok is now a model provider — sign in with your X Premium / SuperGrok subscription, or bring an xAI API key.**
- **Air-gapped appliances can now update KruxOS from a file — no internet required.**
- **Dashboard: one-click "Install Now" GPU driver install.**
- **One-click GPU driver install: the appliance now fetches the NVIDIA `.run` itself, verified against a pin signed into the kit /5201).**
- **The appliance can now fetch the GPU kit itself — and every fetched kit is signature-verified before anything touches disk.**
- **The GPU-driver install now presents a real NVIDIA license attestation, not a bare checkbox.**
- **Model-license ADVISORY badges on Local Models.**
- **Max Concurrent Code Sessions is now an operator control, not just a hidden setting.**
- **Pack runtime is now operator-configurable — `packs.python_path` and `packs.executor_timeout_secs` in `gateway.yaml`.**
- **Agents can call an authenticated API without ever seeing the credential — `network.credentialed_request`.**
- **Add a scoped, host-bound third-party API key from the dashboard or CLI.**
- **Settings › Local Models: a "Recommended" badge marks the suggested model, and the tab now shows your installed models with Enable / Disable / Remove controls.**
- **Pull any GGUF model straight from Hugging Face, and manage local models from the CLI.**
- **Remote access (Tailscale) — a dashboard card to turn on tailnet remote access, no command line (Phase 4).**
- **First-boot setup can turn on remote access (Tailscale) in one optional step (Phase 4b).**
- **Remote access to your dashboard over Tailscale, controllable from the appliance API (Phase 3).**
- **Hardware / Drivers page — install and manage a GPU driver from the dashboard.**
- **GPU acceleration loads automatically at boot.**
- **Opt-in SSH — a real headless console, off by default.**
- **The SSH access card now walks you through making an SSH key.**
- **One guided "Local model" provider — run a self-hosted model without the cloud-vendor friction.**
- **One-click VirtualBox import — `.ova` appliance with a resizable disk.**
- **Restart the gateway from the dashboard — new Settings › System tab.**
- **`filesystem.edit` — find-and-replace in one call.**
- **Agents list — Policy + Identity columns.**
- **`kruxos vault status`.**
- **Agent bearer tokens.**
- **Atomic pack upgrade.**
- **Pack dependency bundling — third-party Python libraries.**
- **Alerts page — agents can finally reach the operator.**
- **License tiers.**
- **Anonymous activation telemetry, opt-out.**
- **Offline-activated appliances register with your account — reliably.**
- **Activate / re-activate online from the dashboard.**
- **Activate online in the setup wizard.**
- **Online activation + `kruxos license status`.**
- **Live license-status surface.**
- **Settings › License tab.**
- **OTA update-signing key can now be rotated over the air.**

### Changed

- **Grok (Subscription) now reaches the full model catalog with the right reasoning behaviour.**
- **Cron jobs created before this release under the shared `"default"` attribution now fire confined to `<workspace_root>/default/` and are invisible to per-agent `scheduler.cron_list`.**
- **Appliances now send an anonymous activation ping to the deployed license server.**
- **OpenRouter app attribution now claims the wider (honest) category set.**
- **Seat-full activation is no longer a dead end in the dashboard.**
- **Refreshed the model dropdown catalogs for Grok, Codex, and Anthropic.**
- **`kruxos sandbox diagnose` now reports whether per-call Landlock confinement is actually being applied, not just the kernel ABI.**
- **`kruxos status` now renders the full health surface — per-service health and resource usage — reaching parity with the dashboard Health page.**
- **GPU kits now publish to the public `altvale/kruxos` releases.**
- **Root A/B slot size raised 640 MiB → 2048 MiB so a v0.0.3 appliance can OTA-update forward to future releases.**
- **v0.0.3 launches with two license tiers: Personal (free) and Enterprise (custom — contact us).**
- **Wizard's "Don't have a license key?" prompt now drives to signup, not pricing.**
- **Settings › Inference tab renamed "Local Models".**
- **The SSH access card now points to the shipped remote-access feature instead of saying VPN is "coming" (Phase 4).**
- **The Uploads page now shows a real progress bar while a file uploads.**
- **Hardware / Drivers page now links the exact NVIDIA `.run` file, not the generic portal (-followup).**
- **File uploads now stream to disk — multi-GB uploads (model GGUFs, GPU driver `.run` files) no longer risk running the appliance out of memory.**
- **Automatic license re-validation now runs ~every 6 h (was ~24 h).**
- **Local model provider is now `type: ollama`.**
- **`filesystem.search` can now roll up an inventory and find duplicates in one call (v1.2).**
- **`filesystem.search_content` can now count instead of streaming matches (v1.1).**
- **Setup wizard auto-navigates to the unlock screen after a restart.**
- **Leaner tool results and schemas for agents.**
- **The license signing key can now rotate without invalidating your license.**
- **The wizard's secondary license control is now "Enter license key".**
- **Seat-full online activation now points at both self-service paths.**
- **Dashboard pack upgrades are now atomic — no downtime, one audit entry.**
- **The `kruxos vault unlock` session now relocks after idle time.**
- **Pack publishing enforces the Capability Design Guidelines.**
- **OpenRouter requests now carry app-ranking categories.**
- **Activation is now required to finish setup.**
- **`process.run` sandbox is now a deny-by-default private rootfs.**
- **Privilege separation.**
- **`process.run` from `/code` and the User principal now runs non-root (completes 's `/code` confinement).**
- **Pack capabilities now run as non-root on every execution path (completes 's confinement for packs).**
- **Local IPC peer-credential authentication + plaintext vault passphrase removed (closes / ).**
- **User API (port 7703) is now bound to loopback only.**
- **Supervision/chat/vault moved off the cleartext-passphrase TCP port onto a root-only control socket; port 7701 retired (completes 's auth-transport hardening).**
- **Per-CLI attribution now reaches the audit log (with ).**
- **Installing a CLI no longer mints a per-CLI vault token.**

### Fixed

- **One-click GPU driver install completes the Staging step.**
- **Grok (Subscription) chat works again.**
- **One-click GPU driver install no longer fails at the "Staging files" step.**
- **Idle in-memory session state is now reclaimed instead of accumulating for the gateway's lifetime.**
- **Scheduled tasks are now attributed to the agent that created them, ending cross-agent cron bleed.**
- **The gateway no longer silently drops a client request that arrives while it is handling another.**
- **Agent state is now isolated per agent, and session state per session, across every path.**
- **The Python SDK now targets the real gateway capabilities — state, briefings, approvals, and error handling work end-to-end.**
- **`kruxos migrate` now carries your model configuration and on-box-engine state across a hardware move.**
- **Update downloads now survive a transient CDN hiccup instead of failing the whole update.**
- **`kruxos model add` no longer tells you to run `kruxos vault add`, which can't store the key.**
- **Context compaction now actually runs for non-Anthropic providers.**
- **Offline update: applying an uploaded image now works end-to-end.**
- **A stock appliance image now ships the GPU one-click install map.**
- **"Enable GPU" now works after an install + reboot — the accel loader is idempotent and no longer wipes a valid GPU marker.**
- **Max concurrent `/code` sessions default now scales with the appliance's actual RAM instead of a hardcoded `4`.**
- **A bare-metal reboot could no longer silently "forget" your license, vault, or first-boot state.**
- **The on-appliance local model now executes tool calls instead of leaking raw JSON into `/chat`, and its first turn no longer fails out-of-box on a fresh appliance.**
- **Slow pack installs and OS-update check/apply no longer fail with a spurious 502 from the dashboard.**
- **CLI output and `--help` text no longer show internal issue-tracker references.**
- **Approvals, Alerts, Health, Identities and the two newest Settings tabs are readable on a phone.**
- **No dashboard page forces sideways scrolling on a phone anymore.**
- **Dashboard buttons are visually uniform again, and the Integrations card actions no longer clip at narrow widths.**
- **The dashboard sidebar now auto-collapses to an icon rail on small screens.**
- **The on-appliance inference engine no longer reserves multiple gigabytes of RAM for a small model.**
- **Settings › Local Models: the "Disable" button now explains what it does.**
- **Settings › Local Models: an "Advanced: engine settings" panel lets you tune the inference engine without SSHing in to hand-edit `inference.env`.**
- **The Secrets list now warns about an allow-all key instead of showing it as a plain chip.**
- **Adding a secret with a reserved/managed name is now rejected.**
- **An "all hosts" host binding written as `*.` now trips the same confirmation as `*`.**
- **Revoked secrets now stay visible with a "revoked" badge instead of vanishing.**
- **`kruxos vault add` refuses to clobber a KruxOS-managed credential.**
- **A denied email read/send now tells the agent exactly how to recover.**
- **A revoked model-provider key no longer shows as "ready".**
- **Boot warning for an unscoped secret no longer mislabels an allow-all key as "denied".**
- **The "vault is locked" sign-in error now points to the real unlock path.**
- **Model downloads now survive rate-limits and dropped connections instead of failing the pull.**
- **Models you pull now stay in the model list — pulled and custom (BYOM) models no longer disappear after downloading.**
- **Enabling GPU inference now actually activates the GPU on a clean install.**
- **Turning on remote access no longer ends in a "timed out" error — the one-time login link now appears in a few seconds (Phase 4c).**
- **GPU offload can now open the device nodes: the NVIDIA `/dev` nodes are created group-readable/writable for the inference engine.**
- **GPU offload now finds NVIDIA's userspace driver library: `libcuda.so.1` is staged from the driver `.run`.**
- **GPU offload now reaches the device: the accel directory is traversable by the inference engine.**
- **GPU inference now actually offloads to the GPU — the inference engine picks up its accel drop-in at boot (the /etc-overlay drop-ins are merged via a daemon-reload once the overlay mounts).**
- **Enabling GPU now activates it without a reboot, and fails closed (clear error, drop-in rolled back) if the accelerator can't be loaded.**
- **The "GPU active" status now reflects the real accelerator marker, not just the enabled flag.**
- **Large dashboard uploads no longer fail with a 502 after several minutes.**
- **GPU driver install now completes — the sandboxed `.run` extractor no longer fails at Staging.**
- **Install-driver modal: an uploaded file no longer gets wiped, plus clearer upload UX.**
- **SSH setup: the "public key only" warning now reads as one sentence.**
- **SSH (and on-box inference) now stay on across a reboot once enabled.**
- **Internal issue numbers stripped from the remaining runtime log lines.**
- **Internal issue numbers no longer leak into operator-facing surfaces.**
- **The dashboard `/chat` now streams tokens as the model generates them — for every provider.**
- **Chat streaming is now per-conversation — switching conversations mid-reply no longer freezes every composer (ships with ).**
- **`/chat` no longer false-stalls a slow on-box (local) model on its first response.**
- **The first model provider you add now becomes the system default automatically — `/chat` works without a manual "Set Default".**
- **First-boot wizard no longer traps you on the "Create your AdminAgent" step.**
- **A plain `http://<host>:7800` now reaches the dashboard instead of dead-ending.**
- **The dashboard's self-signed certificate now has a SubjectAltName — no more hard cert error.**
- **The OVA ships with room for on-box inference — bigger default disk, RAM and CPU.**
- **OVA import is more turnkey — the dashboard is reachable, serial works, and no "Invalid settings" flag.**
- **On-appliance inference: a reworked engine config targeting the multi-vCPU hang.**
- **On-box inference is several times faster after a VirtualBox import — the appliance now sees your CPU's AVX2.**
- **On-appliance inference: the engine can now read the model you pull.**
- **On-appliance inference: a dropped download now auto-resumes.**
- **The on-box inference engine is now usable from Settings › Models.**
- **Settings › Inference tab is no longer unstyled text.**
- **Settings › System "Reboot system" now actually reboots the appliance.**
- **The appliance no longer boots reporting overall health `degraded`.**
- **The appliance now boots on NVMe disks.**
- **USB keyboards and USB storage now work on bare-metal installs.**
- **No more failed network unit and ~2-minute stall on first boot.**
- **Settings tab strip no longer widens the page on narrow screens.**
- **Personal licenses no longer show a misleading expiry countdown.**
- **License page "Refresh" now re-checks with the license server.**
- **`/data` now grows to fill the disk on first boot, so appliance state is no longer capped at the 4 GB image floor.**
- **License single-activation now works even when `/etc/machine-id` is empty.**
- **`/etc/machine-id` is now provisioned on the appliance.**
- **Installing/upgrading a real published pack now works — `manifest.yaml` is read correctly.**
- **Pack runner output is now bounded — OOM protection.**
- **Service Proxy dashboard now surfaces sync failures instead of "Never / Unknown".**
- **Pack capabilities now execute on the confined runtime.**
- **Console first-boot wizard now requires license activation, matching the dashboard.**
- **Agent in-process capabilities no longer fail under an active sandbox.**
- **Empty optional filters no longer silently match nothing.**
- **Integrations install no longer looks hung — and retrying is safe.**
- **Installing a host CLI from Integrations no longer fails with "Install failed (HTTP 502)".**
- **`/health` page no longer contradicts itself when all services are healthy.**
- **`/code` agent CLIs can reach the gateway again.**
- **Vault passphrase no longer echoes when typed.**
- **First-boot wizard now sets the system-default model provider.**
- **`/code` and other User-principal sessions can no longer read the vault or other KruxOS secrets through `filesystem.*` (CRITICAL).**
- **`filesystem.search_content` re-validates symlinked descendants when following links (Security).**
- **First-boot setup banner no longer shows the wrong version or old branding.**
- `process.
- **Codex CLI approval-gate hook now loads on Codex ≥ v0.140.0.**

### Security

- **Scheduled capability jobs now run confined to their creating agent's own workspace, like interactive and autonomous jobs.**
- **Reconnecting Gmail/Slack with a different account now purges the previous account's cached data.**
- **A poisoned server-supplied string can no longer inject terminal escape sequences into the operator CLI.**
- **An alert id can no longer traverse out of the alerts directory.**
- **A hostile allowlisted site can no longer OOM the gateway with a giant response body.**
- **A host binding can no longer be quietly widened to "any host" by a degenerate wildcard or a trailing dot.**
- **Reading an email body is now scoped and fully audited for operator/`/code` sessions too, not only agents.**
- **Credentialed egress never returns the credential — even on an error or via a reflecting server.**
- **Credentialed egress is fail-closed and cannot be aimed at an arbitrary host.**
- **Encrypted backups are now stretched with Argon2id, and the `state.backup` capability actually backs up your state.**
- **Agents can only use a stored credential for the capability it is scoped to — enforced, audited, and deny-by-default.**
- **The agent name `default` is now reserved, closing an audit-attribution collision.**
- **Reading an email body now respects the token's scope and is always audited.**
- **Opt-in SSH ships with guardrails for granting remote root.**
- **First boot: serial/headless boots no longer come up with passwordless root.**
- **Firewall: dropped the dead inbound TCP 7702 rule.**

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
