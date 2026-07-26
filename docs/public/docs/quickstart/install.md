# Install KruxOS

By the end of this page, you'll have a running KruxOS instance ready to accept agent connections.

KruxOS ships as a self-hosted appliance with three distribution paths:

| Path | Best for | Code Sessions | Typical time |
|------|----------|---------------|--------------|
| **Docker** | Evaluation, CI, quick try-out | No | ~30 seconds |
| **VM image** | KVM, VirtualBox, VMware, Vagrant | Yes | ~15 minutes |
| **Bare metal** | Dedicated hardware, air-gapped | Yes | ~30 minutes |

All paths drop you into the same first-boot dashboard wizard at `https://<host>:7800`. The dashboard serves HTTPS by default; a plain `http://<host>:7800` request is permanently redirected (`308`) to the HTTPS URL on the same port, so accept the self-signed certificate on first visit.

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
  -v kruxos-data:/data/kruxos \
  altvale/kruxos:latest
```

| Port | Service | Purpose |
|------|---------|---------|
| 7700 | Gateway | MCP WebSocket — agents connect here (64-char hex API key) |
| 7703 | User API | Loopback HTTP — CLI / automation (`krx_user_*` bearer); **no `-p` mapping needed** |
| 7800 | Dashboard | First-boot wizard + web UI (HTTPS by default) |

Only these two ports need publishing. The User API (7703) and health endpoint (7704) bind to loopback inside the appliance and are reached through the dashboard, not exposed. The old supervision port (7701) is retired — supervision now rides an in-guest root-only control socket, not a TCP port.

!!! note "About `--privileged`"
    The KruxOS sandbox needs user/network namespaces, cgroup v2 and nftables. `--privileged` is the simplest way to grant those on Docker; if you'd rather use targeted capabilities, see the [Docker isolation guide](../guides/docker-isolation.md).

### Finish setup in the browser

Open <https://localhost:7800> — the first-boot wizard walks you through ten steps:

1. **Welcome** — orientation card explaining what the wizard sets up.
2. **Vault passphrase** — same value you passed via `KRUXOS_VAULT_PASSPHRASE`. Unlocks the vault, dashboard login, and console root login. A live strength meter scores the passphrase before submit.
3. **Workspace** — picks the AdminAgent's home directory. The default `/data/kruxos/users/admin` is auto-created. A click-through **directory browser** opens a modal listing subdirectories with writability dots and an inline "New folder" affordance (under `/data/`). A "Type path instead" fallback toggles a free-text input for clipboard pastes.
4. **AdminAgent (Identity)** — names the first agent and optionally configures its model provider inline. Five provider types are wired in — **Anthropic**, **OpenAI**, **OpenAI Codex** (OAuth device-code), **OpenRouter**, **Local** — plus a **Skip** tab that defers provider setup to Settings. Provider and agent are persisted atomically (provider first; if provider registration fails, the agent is not created). The **agent API key is shown once** at this step.
5. **License activation** — paste a JWT or skip (personal use is free).
6. **User token** — generates a `krx_user_*` bearer token; shown **once** for the loopback User API and CLI installs.
7. **Install CLI Tools** — optional. Installs Claude Code and/or Codex CLI seed configs in-process. Both can be installed later from Dashboard → Integrations.
8. **SSH key** — optional. Pre-seed a public key so the opt-in SSH console is ready to turn on later from Settings › System.
9. **Remote access** — optional. Turn on Tailscale so you can reach the dashboard from your own devices anywhere; shows a one-time login link. Can also be enabled later from Settings › System (see the [Remote Access guide](../guides/remote-access.md)).
10. **Done** — confirmation screen.

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
  [PASS] Capability definitions      92 capabilities across 13 categories
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
- No fixed disk minimum — the shipped image is ~8 GiB and works out of the box (see [Disk sizing](#disk-sizing) below)

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

!!! tip "Faster local inference in VirtualBox (AVX2)"
    KruxOS also ships a `kruxos-x86_64.ova` for VirtualBox's **File → Import
    Appliance** one-click path. The OVA opts the guest into the host CPU's real
    vector features (AVX2 and the AVX/FMA features it builds on), which lets the
    appliance's built-in local inference engine pick its fastest CPU code path
    instead of falling back to the slow baseline. Features are only ever exposed
    when the host CPU actually has them, so the import is safe on older hardware.
    Two host requirements for AVX2 to reach the guest:

    - **VirtualBox 7.1.4 or newer** — earlier releases can't pass AVX2 through to
      the guest at all.
    - **On Windows with Hyper-V enabled** (this includes any machine running
      WSL2), VirtualBox masks the host's CPU features unless you're on
      **VirtualBox 7.1.12 or newer**. On such a host, either disable Hyper-V or
      use VirtualBox 7.1.12+.

    After booting, confirm the feature reached the guest with
    `grep avx2 /proc/cpuinfo` inside the appliance.

### Boot via Vagrant (x86_64)

```bash
vagrant box add kruxos ./kruxos-x86_64.box
vagrant init kruxos
vagrant up
```

### First boot

The default firewall accepts TCP 7700 (agent gateway) and 7800 (dashboard) only — SSH on 22 is opt-in and off by default, and port 7702 (trigger-wake) is UDP on loopback only, so it has no inbound firewall rule by design. Open `https://<vm-ip>:7800` in your browser and run through the same dashboard wizard described in Option 1 (welcome, vault passphrase, workspace, AdminAgent, license, User token, Install CLI Tools, SSH key, Remote access, done).

Daily state backups (02:00 UTC) and audit-log rotation (03:00 UTC, 90-day retention) run on systemd timers out of the box.

### Disk sizing

The shipped image is a fixed ~8 GiB disk with a 4 GiB `/data` partition — enough to try KruxOS out of the box. Grow-to-fill is a **one-time, first-boot step**: on the very first boot KruxOS auto-expands `/data` to fill whatever disk it finds, records a marker, and then never runs the auto-grow again. So the rule is simple — **size the disk generously _before_ first boot** and you never have to repartition by hand: whatever size the disk is at first boot, `/data` claims all of it automatically.

To start bigger (local models, long agent history, large workspaces), enlarge the virtual disk **before** you boot the image for the first time:

- **Cloud:** pick the disk size when you create the instance.
- **QEMU / libvirt:** `qemu-img resize kruxos-x86_64.qcow2 20G` **before** first boot.
- **VMware:** expand the disk in the VM's settings.
- **VirtualBox:** in the Virtual Media Manager, copy the `.vmdk` to a VDI and drag the size slider, or `VBoxManage modifymedium <disk>.vdi --resize 20480`.

**Enlarging after first boot is a manual, required step.** Because the auto-grow only runs once, resizing the virtual disk *later* does **not** re-expand `/data` on its own. From the VM console, resize the disk (one of the commands above), then grow partition 4 and run `resize2fs` on the `/data` filesystem to claim the new space.

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
4. Supervision no longer uses a network port — it rides an in-guest **root-only control socket** (`/run/kruxos/control.sock`), so there is no supervision port to firewall

!!! note "Wi-Fi-only machines: join the network at the console"
    If the machine has no Ethernet port and no cable, the banner will show no address and there is no dashboard to open yet. Log in at the console and run `kruxos setup` — step 2 scans for wireless networks and joins one. Full walkthrough: [Joining Wi-Fi from the console](../guides/first-boot-wizard.md#joining-wi-fi-from-the-console).

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