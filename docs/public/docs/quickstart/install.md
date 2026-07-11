# Install KruxOS

By the end of this page, you'll have a running KruxOS instance ready to accept agent connections.

KruxOS ships as a self-hosted appliance with two distribution paths:

- **Docker image** on Docker Hub (`altvale/kruxos`) — fastest to try out
- **VM image** as `.img.gz` / `.qcow2` / `.vmdk` / Vagrant `.box` for x86_64 and aarch64 — full sandbox + Code Sessions

Either path drops you into the same first-boot dashboard wizard at `http://<host>:7800`.

## Option 1: Docker (recommended for trying out)

### Prerequisites

- Docker installed ([Get Docker](https://docs.docker.com/get-docker/))

### Run KruxOS

```bash
docker run -d --name kruxos --privileged \
  -e KRUXOS_VAULT_PASSPHRASE='choose-a-strong-passphrase' \
  -p 7800:7800 \
  -p 7700:7700 \
  -v kruxos-data:/data/kruxos \
  altvale/kruxos:latest
```

| Port | Service | Purpose |
|------|---------|---------|
| 7700 | Gateway | MCP-native (JSON-RPC fallback) — agents connect here |
| 7800 | Dashboard | First-boot wizard + web UI (HTTPS by default) |

Only these two ports need publishing. The User API (7703) and health endpoint (7704) bind to loopback inside the appliance and are reached through the dashboard, not exposed. The old supervision port (7701) is retired — supervision now rides an in-guest root-only control socket, not a TCP port.

!!! note "About `--privileged`"
    The KruxOS sandbox needs user/network namespaces, cgroup v2 and nftables. `--privileged` is the simplest way to grant those on Docker; if you'd rather use targeted capabilities, see the [Docker isolation guide](../guides/docker-isolation.md).

### Finish setup in the browser

Open <https://localhost:7800> — the first-boot wizard walks you through eight steps:

1. **Welcome** — orientation card explaining what the wizard sets up.
2. **Vault passphrase** — same value you passed via `KRUXOS_VAULT_PASSPHRASE`. Unlocks the vault, dashboard login, and console root login. A live strength meter scores the passphrase before submit.
3. **Workspace** — picks the AdminAgent's home directory. The default `/data/kruxos/users/admin` is auto-created. A click-through **directory browser** opens a modal listing subdirectories with writability dots and an inline "New folder" affordance (under `/data/`). A "Type path instead" fallback toggles a free-text input for clipboard pastes.
4. **AdminAgent (Identity)** — names the first agent and optionally configures its model provider inline. Five provider types are wired in — **Anthropic**, **OpenAI**, **OpenAI Codex** (OAuth device-code), **OpenRouter**, **Local** — plus a **Skip** tab that defers provider setup to Settings. Provider and agent are persisted atomically (provider first; if provider registration fails, the agent is not created).
5. **License activation** — paste a JWT or skip (skipping logs a warning but keeps serving).
6. **User token** — generates a `krx_user_*` bearer token; shown **once** for the loopback User API and CLI installs.
7. **Install CLI Tools** — optional. Installs Claude Code and/or Codex CLI seed configs in-process. Both can be installed later from Dashboard → Integrations.
8. **Done** — confirmation screen.

The dashboard auto-generates a self-signed TLS cert; browsers will prompt to accept it.

### Verify it's running

```bash
docker exec kruxos kruxos verify
```

Expected output (abbreviated):

```
KruxOS Verify
  [PASS] Gateway (MCP)               listening on 0.0.0.0:7700
  [PASS] Dashboard (HTTPS)           listening on 0.0.0.0:7800
  [PASS] Health endpoint             listening on 127.0.0.1:7704
  [PASS] Vault                       unlocked
  [PASS] Capability definitions      89 capabilities across 13 categories
```

!!! tip "CLI commands inside Docker"
    Run any `kruxos` command from your host by prefixing with `docker exec kruxos`:
    ```bash
    docker exec kruxos kruxos status
    docker exec kruxos kruxos agent list
    docker exec kruxos kruxos --help
    ```

!!! warning "Code Sessions are not supported in the Docker image"
    The dashboard `/code` page (xterm.js terminals through the sandbox) needs cgroup v2 delegation that isn't reliable through Docker even with `--privileged`. All other features — gateway, dashboard, agents, capabilities, vault, audit, comms — work normally. Use the VM image for code-session workloads.

Your KruxOS instance is ready. Continue to connect your AI model or CLI:

- [Connect Claude Code](claude-code.md) (recommended — MCP-native, zero adapter code)
- [Connect Claude Desktop or the Claude API](connect-claude.md)
- [Connect OpenAI Codex / GPT](connect-openai.md)
- [Connect Gemini](connect-gemini.md)
- [Connect local models](connect-local.md)

---

## Option 2: VM image (full appliance — Code Sessions + sandbox)

### Prerequisites

- A VM hypervisor (KVM / QEMU / libvirt, VirtualBox, or VMware) or bare-metal x86_64 / aarch64 hardware
- 2 GiB RAM minimum, 4 GiB recommended
- No fixed disk minimum — the shipped image is ~8 GiB and works out of the box (see [Disk sizing](#disk-sizing) below)

Validated on KVM and VirtualBox on x86_64. The aarch64 artefact ships, but the acceptance walkthrough was performed on x86_64 only. **Hyper-V Gen 2 is not supported.**

### Download

Release artefacts are published on GitHub Releases at <https://github.com/altvale/kruxos/releases>:

- `kruxos-x86_64.img.gz` / `kruxos-aarch64.img.gz` — raw disk image
- `kruxos-x86_64.qcow2` / `kruxos-aarch64.qcow2` — libvirt / KVM / QEMU
- `kruxos-x86_64.vmdk` / `kruxos-aarch64.vmdk` — VMware / VirtualBox
- `kruxos-x86_64.box` — Vagrant (libvirt; x86_64 only)
- `SHA256SUMS` + a per-artefact `.cosign.bundle` (Fulcio cert + Rekor inclusion proof) for offline verification

### Verify the download

```bash
# Hash check
sha256sum -c SHA256SUMS --ignore-missing

# Signature check (offline; bundle contains Fulcio cert + Rekor proof)
cosign verify-blob \
  --bundle kruxos-x86_64.qcow2.cosign.bundle \
  --certificate-identity-regexp '.*' \
  --certificate-oidc-issuer-regexp '.*' \
  kruxos-x86_64.qcow2
```

### Boot in QEMU

```bash
qemu-system-x86_64 \
  -m 2048 \
  -drive file=kruxos-x86_64.qcow2,format=qcow2,if=virtio \
  -netdev user,id=net0,hostfwd=tcp::7700-:7700,hostfwd=tcp::7800-:7800 \
  -device virtio-net-pci,netdev=net0
```

### Boot in VirtualBox

**Bridged networking is simplest for an appliance:** the VM gets a real LAN IP, so you reach the dashboard at `https://<vm-ip>:7800` with no port forwarding.

1. Create a new VM: Linux, Other Linux (64-bit)
2. Allocate 2048 MB RAM
3. Attach the `.vmdk` as the boot disk
4. Set the network adapter to **Bridged** (Settings → Network → Attached to: Bridged Adapter)
5. Start the VM

If you must stay on NAT instead, forward host ports **7700** and **7800** to the guest (the dashboard and gateway are the only ports you need).

### Boot via Vagrant (x86_64)

```bash
vagrant box add kruxos ./kruxos-x86_64.box
vagrant init kruxos
vagrant up
```

### First boot

The default firewall accepts TCP 7700 / 7702 / 7800 (SSH on 22 is opt-in and off by default). Open `https://<vm-ip>:7800` in your browser and run through the same dashboard wizard described in Option 1 (welcome, vault passphrase, workspace, AdminAgent, license, User token, Install CLI Tools, done).

Daily state backups (02:00 UTC) and audit-log rotation (03:00 UTC, 90-day retention) run on systemd timers out of the box.

### Disk sizing

The shipped image is a fixed ~8 GiB disk with a 4 GiB `/data` partition — enough to try KruxOS out of the box. On first boot, KruxOS **auto-expands `/data` to fill whatever disk it finds**, so you never have to repartition by hand: give the VM a bigger disk and the extra space is claimed on the next boot.

Enlarge the disk only if you need the room (local models, long agent history, large workspaces):

- **Cloud:** pick the disk size when you create the instance.
- **QEMU / libvirt:** `qemu-img resize kruxos-x86_64.qcow2 20G` **before** first boot.
- **VMware:** expand the disk in the VM's settings.
- **VirtualBox:** in the Virtual Media Manager, copy the `.vmdk` to a VDI and drag the size slider, or `VBoxManage modifymedium <disk>.vdi --resize 20480`.

`/data` fills the new space automatically on the next boot — the resize above is an optional power-user step, not part of a normal install.

### Verify

From inside the VM console (vault passphrase unlocks console root):

```bash
kruxos verify
kruxos sandbox diagnose
```

Or from your host, hit the dashboard at `https://<vm-ip>:7800`.

---

## Next steps

- [Connect Claude Code](claude-code.md) — the zero-config golden path
- [Connect Claude Desktop or API](connect-claude.md)
- [Connect OpenAI](connect-openai.md) — GPT models + Codex
- [Connect Gemini](connect-gemini.md)
- [Connect local models](connect-local.md) — Ollama, vLLM, LM Studio, llama.cpp
- [Web Dashboard](dashboard.md) — monitor agents from your browser
- [CLI Guide](cli.md) — manage KruxOS from the terminal
