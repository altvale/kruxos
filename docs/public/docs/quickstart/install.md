# Install KruxOS

By the end of this page, you'll have a running KruxOS instance ready to accept agent connections.

KruxOS v0.0.1 ships as a self-hosted appliance with two distribution paths:

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
  -p 7701:7701 \
  -v kruxos-data:/data/kruxos \
  altvale/kruxos:latest
```

| Port | Service | Purpose |
|------|---------|---------|
| 7700 | Gateway | MCP-native (JSON-RPC fallback) — agents connect here |
| 7701 | Supervision | WebSocket — dashboard live stream, audit events |
| 7800 | Dashboard | First-boot wizard + web UI (HTTPS by default) |

!!! note "About `--privileged`"
    The KruxOS sandbox needs user/network namespaces, cgroup v2 and nftables. `--privileged` is the simplest way to grant those on Docker; if you'd rather use targeted capabilities, see the [Docker isolation guide](../guides/docker-isolation.md).

### Finish setup in the browser

Open <https://localhost:7800> — the first-boot wizard walks you through:

1. **Vault passphrase** — same value you passed via `KRUXOS_VAULT_PASSPHRASE`. Unlocks the vault, dashboard login, and console root login.
2. **AdminAgent** — creates the first agent with a personal-permissive policy.
3. **License activation** — paste a JWT or skip (v0.0.1 logs a warning but keeps serving).
4. **User token** — generates a `krx_user_*` bearer token; shown **once** for the loopback User API and CLI installs.
5. **CLI install** — emits Claude Code / Codex seed configs via `kruxos cli-config generate`.

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

!!! warning "Code Sessions are not supported in the Docker image (v0.0.1)"
    The dashboard `/code` page (xterm.js terminals through the sandbox) needs cgroup v2 delegation that isn't reliable through Docker even with `--privileged`. All other features — gateway, dashboard, agents, capabilities, vault, audit, comms — work normally. Use the VM image for code-session workloads. Docker-side fix ships in **v0.0.2**.

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
- 20 GiB disk minimum

Tested in v0.0.1: KVM and VirtualBox on x86_64. The aarch64 artefact ships, but the v0.0.1 acceptance walkthrough was performed on x86_64 only. **Hyper-V Gen 2 is not supported.**

### Download

Release artefacts for v0.0.1 are published on GitHub Releases at <https://github.com/altvale/kruxos/releases>:

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
  -netdev user,id=net0,hostfwd=tcp::7700-:7700,hostfwd=tcp::7701-:7701,hostfwd=tcp::7800-:7800 \
  -device virtio-net-pci,netdev=net0
```

### Boot in VirtualBox

1. Create a new VM: Linux, Other Linux (64-bit)
2. Allocate 2048 MB RAM
3. Attach the `.vmdk` as the boot disk
4. Forward ports 7700, 7701, 7800
5. Start the VM

### Boot via Vagrant (x86_64)

```bash
vagrant box add kruxos ./kruxos-x86_64.box
vagrant init kruxos
vagrant up
```

### First boot

The default firewall accepts TCP 22 / 7700 / 7701 / 7702 / 7800. Open `https://<vm-ip>:7800` in your browser and run through the same dashboard wizard described in Option 1 (vault passphrase, AdminAgent, license, User token, CLI install).

Daily state backups (02:00 UTC) and audit-log rotation (03:00 UTC, 90-day retention) run on systemd timers out of the box.

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
