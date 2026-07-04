# Install KruxOS

By the end of this page, you'll have a running KruxOS instance ready to accept agent connections.

KruxOS ships as a self-hosted appliance with three distribution paths:

| Path | Best for | Code Sessions | Typical time |
|------|----------|---------------|--------------|
| **Docker** | Evaluation, CI, quick try-out | No | ~30 seconds |
| **VM image** | KVM, VirtualBox, VMware, Vagrant | Yes | ~15 minutes |
| **Bare metal** | Dedicated hardware, air-gapped | Yes | ~30 minutes |

All paths drop you into the same eight-step first-boot dashboard wizard at `https://<host>:7800`.

Architectures: **x86_64** and **aarch64**. VM and bare metal require **2 GiB RAM** minimum (4 GiB recommended) and **20 GiB disk** minimum.

---

## Option 1: Docker (recommended for trying out)

### Prerequisites

- Docker installed ([Get Docker](https://docs.docker.com/get-docker/))

### Run KruxOS

```bash
docker run -d --name kruxos --privileged \
  -e KRUXOS_VAULT_PASSPHRASE='choose-a-strong-passphrase' \
  -p 7800:7800 \
  -p 7700:7700 \
  -p 7701:7701 \
  -v kruxos-data:/data/kruxos \
  altvale/kruxos:latest
```

| Port | Service | Purpose |
|------|---------|---------|
| 7700 | Gateway | MCP WebSocket — agents connect here (64-char hex API key) |
| 7701 | Supervision | WebSocket — dashboard live stream, audit events |
| 7703 | User API | Loopback HTTP — CLI / automation (`krx_user_*` bearer); **no `-p` mapping needed** |
| 7800 | Dashboard | First-boot wizard + web UI (HTTPS by default) |

!!! note "About `--privileged`"
    The KruxOS sandbox needs user/network namespaces, cgroup v2 and nftables. `--privileged` is the simplest way to grant those on Docker; if you'd rather use targeted capabilities, see the [Docker isolation guide](../guides/docker-isolation.md).

### Finish setup in the browser

Open <https://localhost:7800> — the first-boot wizard walks you through eight steps:

1. **Welcome** — orientation card explaining what the wizard sets up.
2. **Vault passphrase** — same value you passed via `KRUXOS_VAULT_PASSPHRASE`. Unlocks the vault, dashboard login, and console root login. A live strength meter scores the passphrase before submit.
3. **Workspace** — picks the AdminAgent's home directory. The default `/data/kruxos/users/admin` is auto-created. A click-through **directory browser** opens a modal listing subdirectories with writability dots and an inline "New folder" affordance (under `/data/`). A "Type path instead" fallback toggles a free-text input for clipboard pastes.
4. **AdminAgent (Identity)** — names the first agent and optionally configures its model provider inline. Five provider types are wired in — **Anthropic**, **OpenAI**, **OpenAI Codex** (OAuth device-code), **OpenRouter**, **Local** — plus a **Skip** tab that defers provider setup to Settings. Provider and agent are persisted atomically (provider first; if provider registration fails, the agent is not created). The **agent API key is shown once** at this step.
5. **License activation** — paste a JWT or skip (personal use is free).
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
  [PASS] Supervision WebSocket       listening on 0.0.0.0:7701
  [PASS] Dashboard (HTTPS)           listening on 0.0.0.0:7800
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
    The dashboard `/code` page (xterm.js terminals through the sandbox) needs cgroup v2 delegation that isn't reliable through Docker even with `--privileged`. All other features — gateway, dashboard, agents, capabilities, vault, audit, comms — work normally. Use a VM or bare-metal image for code-session workloads.

Your KruxOS instance is ready. Continue to connect your AI model or CLI:

- [Connect Claude Code](claude-code.md) (recommended — MCP-native, zero adapter code)
- [Connect Claude Desktop or the Claude API](connect-claude.md)
- [Connect OpenAI Codex / GPT](connect-openai.md)
- [Connect Gemini](connect-gemini.md)
- [Connect local models](connect-local.md)

---

## Option 2: VM image (full appliance — Code Sessions + sandbox) {#option-2-vm-image-full-appliance--code-sessions--sandbox}

### Prerequisites

- A VM hypervisor: KVM / QEMU / libvirt, VirtualBox, or VMware
- 2 GiB RAM minimum, 4 GiB recommended
- 20 GiB disk minimum

Tested: KVM and VirtualBox on x86_64. The aarch64 artefact ships, but acceptance walkthroughs were performed on x86_64 only. **Hyper-V Gen 2 is not supported.**

### Download

Release artefacts are published on [GitHub Releases](https://github.com/altvale/kruxos/releases):

| Format | Use with |
|--------|----------|
| `kruxos-x86_64.qcow2` / `kruxos-aarch64.qcow2` | KVM, QEMU, libvirt |
| `kruxos-x86_64.vmdk` / `kruxos-aarch64.vmdk` | VMware, VirtualBox |
| `kruxos-x86_64.box` | Vagrant (libvirt; x86_64 only) |

Each release includes `SHA256SUMS` and per-artefact `.cosign.bundle` files (Fulcio cert + Rekor inclusion proof) for offline verification.

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

### Boot in QEMU / KVM

```bash
curl -LO https://github.com/altvale/kruxos/releases/latest/download/kruxos-x86_64.qcow2

qemu-system-x86_64 \
  -m 2048 -smp 2 -enable-kvm \
  -drive file=kruxos-x86_64.qcow2,format=qcow2,if=virtio \
  -netdev user,id=net0,hostfwd=tcp::7700-:7700,hostfwd=tcp::7701-:7701,hostfwd=tcp::7800-:7800 \
  -device virtio-net-pci,netdev=net0
```

### Boot in VirtualBox

1. Create a new VM: Linux, Other Linux (64-bit)
2. Allocate 2048 MB RAM
3. Attach the `.vmdk` as the boot disk
4. Forward ports 7700, 7701, 7800
5. Start the VM — the console banner shows the dashboard URL

### Boot via Vagrant (x86_64)

```bash
vagrant box add kruxos ./kruxos-x86_64.box
vagrant init kruxos
vagrant up
```

### First boot

The default firewall accepts TCP 22 / 7700 / 7701 / 7702 / 7800. Open `https://<vm-ip>:7800` in your browser and run through the same dashboard wizard described in Option 1.

Daily state backups (02:00 UTC) and audit-log rotation (03:00 UTC, 90-day retention) run on systemd timers out of the box.

### Verify

From inside the VM console (vault passphrase unlocks console root):

```bash
kruxos verify
kruxos sandbox diagnose
```

Or from your host, hit the dashboard at `https://<vm-ip>:7800`.

---

## Option 3: Bare metal (dedicated hardware — air-gapped) {#option-3-bare-metal}

Write the raw disk image directly to USB, SSD, or NVMe. Best for dedicated agent infrastructure, maximum sandbox performance, and air-gapped deployments. See the [Deployment Guide](../enterprise/deployment-guide.md) for production network and TLS checklist.

### Prerequisites

- x86_64 or aarch64 hardware with UEFI or legacy BIOS
- 2 GiB RAM minimum, 4 GiB recommended
- 20 GiB disk minimum (target USB drive, SSD, or NVMe)

### Download

From [GitHub Releases](https://github.com/altvale/kruxos/releases):

- `kruxos-x86_64.img.gz` / `kruxos-aarch64.img.gz` — raw disk image (decompress before writing)

### Verify the download

```bash
sha256sum -c SHA256SUMS --ignore-missing

cosign verify-blob \
  --bundle kruxos-x86_64.img.gz.cosign.bundle \
  --certificate-identity-regexp '.*' \
  --certificate-oidc-issuer-regexp '.*' \
  kruxos-x86_64.img.gz
```

### Write to disk

```bash
gunzip kruxos-x86_64.img.gz

# Replace /dev/sdX with your target device — this erases the entire disk
sudo dd if=kruxos-x86_64.img of=/dev/sdX bs=4M status=progress conv=fsync
sync
```

!!! warning "Confirm the target device"
    Double-check `/dev/sdX` with `lsblk` (Linux) or `diskutil list` (macOS) before running `dd`. Writing to the wrong device will destroy data on that disk.

### Boot and network

1. Boot the machine from the USB/SSD/NVMe (UEFI or legacy BIOS)
2. The console banner shows the dashboard URL — typically `https://<host-ip>:7800`
3. Allow inbound **7700** (agent MCP) and **7800** (operator dashboard) on your management network
4. Keep **7701** (supervision WebSocket) restricted to localhost or a management VLAN — it provides session control and live activity streaming

### First boot

Run through the same eight-step dashboard wizard described in Option 1.

### Verify

From the appliance console:

```bash
kruxos verify
kruxos sandbox diagnose
```

---

## Next steps

- [Connect Claude Code](claude-code.md) — the zero-config golden path
- [Connect Claude Desktop or API](connect-claude.md)
- [Connect OpenAI](connect-openai.md) — GPT models + Codex
- [Connect Gemini](connect-gemini.md)
- [Connect local models](connect-local.md) — Ollama, vLLM, LM Studio, llama.cpp
- [Web Dashboard](dashboard.md) — monitor agents from your browser
- [CLI Guide](cli.md) — manage KruxOS from the terminal
- [Deployment Guide](../enterprise/deployment-guide.md) — production checklist (bare metal / enterprise)

Agent-oriented summary: [kruxos.com/agents/setup](https://kruxos.com/agents/setup/)