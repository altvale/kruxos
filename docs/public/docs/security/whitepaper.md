# KruxOS Security Whitepaper

**Version:** 0.0.1
**Date:** May 2026
**Classification:** Public
**Authors:** KruxOS Security Team

> **Status:** v0.0.1 is the first public release of KruxOS — early beta, not yet recommended for production. The architecture and controls described below are the target architecture; what's active *today* vs. *planned* is summarised below and called out inline.

> **What's active in v0.0.1:** Linux user/network namespaces, cgroup v2 resource limits, seccomp-bpf syscall filtering, nftables network policy, AES-256-GCM secrets vault, hash-chained audit log, deterministic policy engine, two-principal (User / Agent) identity model, mass-destruction command blocks, ws-proxy Origin pinning on code-session upgrades, cosign-signed release artefacts (per-artefact `.sig`, long-term keypair, public key at `https://kruxos.com/keys/cosign.pub` — see SECURITY.md for the full verification flow).
>
> **Active but with caveats:** the **per-call fork sandbox model** is applied to capability handlers; **stateless capabilities** (`system.time`, `system.info`, `system.health`, `agent.whoami`) execute in-process for latency. Per-agent host mounts are enforced under `/mnt/<label>` with path-escape detection in the gateway. The kernel ships built without `CRYPTO_USER_API_*` / `ALGIF_AEAD`, making CVE-2026-31431 structurally unreachable.
>
> **Deferred to v0.0.3 security architecture rework:** **Landlock filesystem confinement**, gateway/code-session privilege separation, per-agent seccomp / resource policy YAML, and the license-enforcement cycle (read-only mode, 14-day grace, machine fingerprinting, phone-home). Codex `apply_patch` and external (third-party) MCP servers don't yet flow through the KruxOS approval queue — that lands in **v0.0.4**.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Threat Model](#2-threat-model)
3. [Five-Layer Isolation Model](#3-five-layer-isolation-model)
4. [Cryptographic Architecture](#4-cryptographic-architecture)
5. [Audit Trail](#5-audit-trail)
6. [Policy Enforcement](#6-policy-enforcement)
7. [Deployment Modes](#7-deployment-modes)
8. [Known Limitations and Mitigations](#8-known-limitations-and-mitigations)
9. [Comparison with Alternatives](#9-comparison-with-alternatives)
10. [Responsible Disclosure](#10-responsible-disclosure)
11. [Appendix A: Compliance Mapping](#appendix-a-compliance-mapping)

---

## 1. Executive Summary

KruxOS is a purpose-built Linux-based operating system designed for AI agent deployment, execution, and governance. As organizations increasingly deploy autonomous AI agents — software entities that read files, execute code, make network requests, and interact with external services — the attack surface expands beyond what traditional operating systems were designed to contain. An AI agent with unrestricted system access can exfiltrate data, modify critical files, exhaust resources, or interfere with other agents, whether through malicious intent, a compromised underlying model, or simple programming error.

KruxOS addresses this by making **isolation the default, not an afterthought**. Every agent runs inside a kernel sandbox combining Linux namespaces, cgroup v2 resource controls, seccomp-bpf syscall filtering, and nftables network policy in v0.0.1; Landlock mandatory access control adds a fifth layer in the **v0.0.3** security architecture rework. All agent interactions pass through a typed capability API — agents never have direct shell access. A deterministic policy engine governs what each agent can do, with four permission tiers from fully autonomous to completely blocked. An encrypted secrets vault ensures agents can *use* credentials without ever *seeing* them. A hash-chained, append-only audit log records every action for forensic review.

### Key Security Properties

- **Process isolation**: Each agent runs in its own PID, mount, network, user, and UTS namespace. Agents cannot see or signal each other's processes.
- **Filesystem confinement (v0.0.1)**: Mount namespace isolation + per-agent host mounts under `/mnt/<label>` with gateway-side path-escape detection. **Landlock MAC adds kernel-enforced filesystem confinement in v0.0.3** — until then, the seccomp filter blocks the syscalls that would let an agent break out (`mount`, `umount2`, `pivot_root`, `unshare`, `setns`).
- **Resource containment**: cgroup v2 enforces per-agent limits on CPU, memory, I/O bandwidth, and process count. A runaway agent cannot starve the system.
- **Syscall restriction**: seccomp-bpf filters block dangerous syscalls (module loading, namespace escape, tracing, rebooting) at the kernel level. The filter is irrevocable once applied.
- **Network control**: Default-deny egress policy with per-agent domain allowlists. Agents cannot reach internal services or other agents' network stacks.
- **Secrets protection**: AES-256-GCM encrypted vault with Argon2id key derivation. Agents invoke capabilities that use secrets internally; raw values are never exposed to the agent.
- **Tamper-evident audit**: Every capability invocation, policy decision, and state change is recorded in a CBOR-format, SHA-256 hash-chained, append-only log.
- **Deterministic governance**: YAML-based policy engine with hierarchical evaluation (system > organization > agent). No LLM in the decision path — every policy decision is reproducible and auditable in under 1 millisecond.
- **Two-principal model**: KruxOS distinguishes the human operator (`User`) from autonomous code (`Agent`). User connections use bearer tokens on the dedicated User API (TCP 7703); agent connections use API keys on the MCP Gateway (TCP 7700). Each principal carries through to policy evaluation and audit attribution, so the operator's CLI tools and the agents they spawn never share an authority boundary.

---

## 2. Threat Model

### 2.1 Attacker Categories

KruxOS considers four attacker categories, ordered by assumed capability:

| Category | Description | Assumed Capability |
|----------|-------------|-------------------|
| **Malicious agent** | An agent deliberately attempting to escape its sandbox, access other agents' data, or exfiltrate information | Full control of the agent's code and prompts; ability to craft arbitrary capability calls |
| **Compromised AI model** | The underlying LLM has been manipulated (prompt injection, fine-tuning attack, model supply chain compromise) to produce harmful tool calls | Ability to generate any sequence of capability invocations, including unusual parameter combinations |
| **Network attacker** | An adversary on the same network segment attempting to intercept, modify, or inject traffic | Ability to sniff, modify, and inject network traffic; DNS poisoning |
| **Insider threat** | A human operator with legitimate system access attempting to cover their tracks or exceed their authorized access | Access to the CLI and potentially the data directory, but not the vault passphrase |

### 2.2 What KruxOS Protects Against

| Threat | Protection | Confidence |
|--------|-----------|------------|
| Agent reads files outside its workspace | Landlock deny-by-default + mount namespace isolation | **High** — kernel-enforced, irrevocable after application |
| Agent kills or traces other processes | PID namespace isolation + seccomp blocks `ptrace`, `kill` to host PIDs | **High** — kernel-enforced |
| Agent loads kernel modules | seccomp blocks `init_module`, `finit_module`, `delete_module` | **High** — BPF filter enforced by kernel |
| Agent escapes to host network | `CLONE_NEWNET` creates empty network stack for non-network capabilities | **High** — kernel namespace isolation |
| Agent exhausts system memory or CPU | cgroup v2 hard limits with OOM killer for memory, quota for CPU | **High** — kernel cgroup enforcement |
| Agent reads raw secret values | Use-not-read vault model; `SecretValue` type has no `Serialize`, `Display`, or `Clone` | **High** — architectural enforcement |
| Agent tampers with audit log | Append-only files with SHA-256 hash chain; agents have no filesystem access to audit directory | **High** — Landlock + hash chain integrity |
| Agent bypasses policy | Policy evaluated in the Gateway before capability dispatch; agent has no direct system access | **High** — Gateway is the only entry point |
| Unauthorized agent connects | API key authentication with SHA-256 hashing and constant-time comparison | **High** — cryptographic verification |
| Network eavesdropping on agent traffic | TLS for external connections; the dashboard serves HTTPS by default with a self-signed certificate; gateway WebSockets bind to `127.0.0.1` so they do not traverse the network | **Medium** — gateway-to-agent WebSocket TLS is configurable but not mandatory for localhost traffic in v0.0.1 |
| Audit log modification after the fact | Hash chain detects any modification, deletion, or reordering of entries | **High** — SHA-256 integrity verification |

### 2.3 What KruxOS Does NOT Protect Against

Honest disclosure of known boundaries is essential for security evaluation. The following threats are **outside the scope** of KruxOS v0.0.1:

**Compromised host kernel.** All five isolation layers (namespaces, cgroups, seccomp, Landlock, nftables) are kernel features. A kernel vulnerability that allows privilege escalation from within a namespace (e.g., CVE-2022-0185, CVE-2022-25636) would bypass all containment. **Mitigation:** KruxOS ships kernel 6.6 LTS with security patches. The seccomp filter reduces the attack surface by blocking ~85% of the syscall table, limiting the kernel surface available for exploitation.

**Physical access attacks.** An attacker with physical access to the machine can read the data partition, extract the vault database, and attempt offline passphrase cracking. **Mitigation:** Argon2id with 64 MiB memory cost makes brute-force expensive. Full-disk encryption (LUKS) is recommended for physical deployments but not enabled by default.

**Side-channel attacks between sandboxes.** Agents sharing the same physical CPU may be able to observe timing differences through shared microarchitectural state (cache, branch predictor, TLB). KruxOS does not implement side-channel mitigations such as core pinning or cache partitioning. **Mitigation:** Not addressed in v0.0.1. For high-security deployments requiring cross-agent confidentiality, run agents on separate physical machines or use VM-level isolation.

**Supply chain attacks in capability packs.** Community-contributed capability packs execute within the agent's sandbox, inheriting the agent's permissions. A malicious pack could exfiltrate data through allowed capability calls. **Mitigation:** Packs are sandboxed alongside the agent (same isolation layers apply). The pack registry requires cryptographic checksums. However, there is no static analysis or behavioral verification of pack code in v0.0.1.

**Denial of service by a legitimate agent.** While cgroup limits prevent system-wide resource exhaustion, an agent can still consume its full allocated quota (512 MB memory, 50% CPU by default), impacting performance of collocated agents on resource-constrained hardware. **Mitigation:** Resource limits are configurable per deployment. Dedicated hardware or VM-per-agent deployment eliminates contention.

**DNS rebinding attacks.** An attacker controlling an allowed domain could change its DNS records to resolve to internal IP addresses after the domain allowlist check passes. **Mitigation:** Network capabilities run in the Gateway process with application-level domain validation before each request. The forked sandbox child has no network access (`CLONE_NEWNET`). For enterprise deployments, adding private IP range filtering is recommended.

---

## 3. Five-Layer Isolation Model

Every agent on the KruxOS image runs inside a five-layer kernel sandbox. Each layer provides independent protection — compromising one layer does not disable the others.

```
┌──────────────────────────────────────────────────┐
│  Layer 5: nftables Per-Agent Network Rules       │  ← Network egress policy
├──────────────────────────────────────────────────┤
│  Layer 4: Landlock Filesystem Access Control     │  ← Path-based MAC
├──────────────────────────────────────────────────┤
│  Layer 3: seccomp-bpf Syscall Filtering          │  ← Syscall allowlist
├──────────────────────────────────────────────────┤
│  Layer 2: cgroup v2 Resource Controls            │  ← Resource limits
├──────────────────────────────────────────────────┤
│  Layer 1: Linux Namespaces                       │  ← Process/FS/net isolation
└──────────────────────────────────────────────────┘
```

### 3.1 Layer 1: Linux Namespaces

Linux namespaces provide the foundational isolation by giving each agent its own view of system resources. KruxOS uses five namespace types:

| Namespace | Flag | Effect |
|-----------|------|--------|
| **PID** | `CLONE_NEWPID` | Agent sees only its own process tree. PID 1 inside the sandbox is the sandbox init process, not the host's systemd. The agent cannot enumerate or signal host processes or other agents' processes. |
| **Mount** | `CLONE_NEWNS` | Agent has a private mount tree. The host filesystem is replaced via `pivot_root` with a minimal root containing only the agent's workspace, a read-only shared directory, and read-only capability definitions. |
| **Network** | `CLONE_NEWNET` | Non-network capabilities execute in an empty network namespace with only a loopback interface. The agent cannot make outbound connections or scan the local network. Network capabilities are proxied through the Gateway (see Section 3.5). |
| **User** | `CLONE_NEWUSER` | The agent process runs as an unprivileged user inside the namespace, mapped to a non-root UID/GID on the host. Even if an agent gains "root" inside its namespace, it has no capabilities on the host. |
| **UTS** | `CLONE_NEWUTS` | Each agent gets its own hostname (`agent-{name}`), preventing hostname-based fingerprinting of the host system. |

**Implementation detail:** Namespace creation occurs in the forked child process before capability handler execution. The order of operations matters: the child first joins its cgroup (requires `/sys/fs/cgroup` write access), then applies Landlock (which denies `/sys/fs/cgroup`), then applies seccomp (which is irrevocable). This ordering ensures each layer can be applied without conflicting with the others.

### 3.2 Layer 2: cgroup v2 Resource Controls

cgroup v2 provides hard resource limits that the kernel enforces regardless of what the agent does. Each agent gets a dedicated cgroup at `/sys/fs/cgroup/kruxos/sandbox-{id}/`.

| Resource | Controller | Default Limit | Enforcement |
|----------|-----------|---------------|-------------|
| **CPU** | `cpu.max` | 50,000 µs per 100,000 µs period (50% of one core) | Kernel throttles the process when quota is exhausted. The agent runs slower but does not crash. |
| **Memory** | `memory.max` | 512 MB (no swap: `memory.swap.max = 0`) | Kernel OOM-kills the process when the hard limit is reached. KruxOS detects this via `memory.events` and returns a structured `resource.exhausted` error to the agent with the limit value and recovery guidance. |
| **I/O bandwidth** | `io.max` | 50 MB/s read, 25 MB/s write | Kernel throttles I/O operations. The agent experiences slower disk access but continues running. |
| **Process count** | `pids.max` | 100 | Kernel returns `EAGAIN` on `fork()`/`clone()` when the limit is reached. KruxOS detects this via `pids.events` and returns a structured error. |

**OOM behavior:** When the kernel OOM-kills a sandbox child, the parent process (Gateway) detects the broken IPC pipe, reads the `memory.events` file from the cgroup, and maps the `oom_kill` counter to a structured `ResourceExhausted` error with the following fields:
- `resource`: `"memory"`
- `limit`: `"512 MB"`
- `recovery_actions`: `["Reduce data size", "Process files in smaller batches", "Request higher memory limit from administrator"]`

This enables AI agents to understand *why* their operation failed and adjust their strategy, rather than receiving an opaque crash.

**Freezer:** The cgroup freezer (`cgroup.freeze`) is used to pause agents during disconnect. A frozen agent's processes are suspended by the kernel (SIGSTOP equivalent) and consume no CPU. When the agent reconnects, the sandbox is unfrozen and execution resumes from exactly where it left off.

### 3.3 Layer 3: seccomp-bpf Syscall Filtering

seccomp-bpf (Secure Computing with Berkeley Packet Filter) restricts which system calls a process can make. KruxOS uses an **allowlist model**: approximately 160 safe syscalls are explicitly permitted, and all others return `EPERM`.

**Why allowlist, not denylist:** A denylist requires anticipating every dangerous syscall. New syscalls are added to the kernel in every release. An allowlist is fail-safe — any new syscall is automatically blocked until explicitly reviewed and permitted.

#### Default Profile (~160 allowed syscalls)

The default profile permits syscalls needed for normal application operation:

| Category | Examples | Purpose |
|----------|---------|---------|
| File I/O | `read`, `write`, `openat`, `close`, `fstat`, `lseek` | Read/write files within the Landlock-confined paths |
| Memory | `mmap`, `mprotect`, `munmap`, `brk`, `mremap` | Memory allocation for program execution |
| Process | `clone` (limited), `execve`, `exit_group`, `wait4` | Process creation (for `process.run` capability) |
| Networking | `socket`, `connect`, `bind`, `listen`, `accept4` | Network operations (effective only in network-enabled namespace) |
| I/O mux | `epoll_create1`, `epoll_ctl`, `epoll_wait`, `poll` | Async I/O for the tokio runtime |
| Time | `clock_gettime`, `clock_nanosleep`, `nanosleep` | Timing and sleep operations |
| Signals | `rt_sigaction`, `rt_sigprocmask`, `kill`, `tgkill` | Signal handling within the sandbox |

#### Blocked Syscalls (23 explicitly dangerous)

| Syscall | Reason for blocking |
|---------|-------------------|
| `mount`, `umount2` | Prevent filesystem remounting to escape Landlock |
| `pivot_root` | Prevent changing the root filesystem |
| `ptrace` | Prevent tracing/debugging other processes (information leak, code injection) |
| `init_module`, `finit_module`, `delete_module` | Prevent kernel module loading (rootkit installation) |
| `kexec_load`, `kexec_file_load` | Prevent loading a new kernel (full system compromise) |
| `reboot` | Prevent system shutdown/restart |
| `sethostname`, `setdomainname` | Prevent hostname spoofing (mitigated by UTS namespace but defense-in-depth) |
| `swapon`, `swapoff` | Prevent swap manipulation (bypass memory limits) |
| `syslog` | Prevent reading kernel log buffer (information disclosure) |
| `settimeofday` | Prevent time manipulation (audit log integrity) |
| `perf_event_open` | Prevent performance counter access (side-channel vector) |
| `bpf` | Prevent BPF program loading (potential kernel exploitation) |
| `userfaultfd` | Prevent userfaultfd creation (exploitation primitive) |
| `keyctl`, `request_key`, `add_key` | Prevent kernel keyring access |
| `unshare`, `setns` | Prevent namespace escape (re-joining host namespaces) |

#### Strict Profile (additional restrictions)

For untrusted agents, a strict profile additionally blocks:

| Syscall | Reason |
|---------|--------|
| `execve`, `execveat` | Prevents executing any binary (agent can only compute, not spawn processes) |
| `fork`, `vfork` | Prevents creating child processes |

The strict profile is selected via `gateway.yaml` → `sandbox.default_seccomp_profile: "strict"`.

**Architecture portability:** Syscall numbers differ between x86_64 and aarch64. KruxOS uses `libc::SYS_*` constants with `#[cfg(target_arch)]` conditional compilation to generate correct BPF filters for both architectures. The BPF program validates the architecture field of the seccomp data structure and rejects calls from unexpected architectures.

**Irrevocability:** Once applied via `prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER)`, the filter cannot be removed or relaxed. This is a kernel guarantee — even if the agent exploits a bug in the handler code, the syscall restrictions remain in effect.

### 3.4 Layer 4: Landlock Filesystem Access Control (v0.0.3)

!!! warning "Landlock is part of the v0.0.3 security architecture rework"
    Landlock filesystem confinement, gateway/code-session privilege separation, and per-agent seccomp / resource policy YAML all land together in **v0.0.3**. In v0.0.1, filesystem boundaries are enforced through mount-namespace isolation, per-agent host mounts under `/mnt/<label>` with path-escape detection in the gateway, and the seccomp filter blocking filesystem-escape syscalls (`mount`, `pivot_root`, `unshare`, `setns`). The description below documents the target v0.0.3 design.

Landlock is a Linux security module (available since kernel 5.13) that provides unprivileged, stackable filesystem access control. Unlike traditional DAC (ownership/permissions), Landlock cannot be bypassed by processes running as root within a user namespace.

KruxOS applies a **deny-by-default** Landlock ruleset to every sandboxed process:

| Path | Access | Purpose |
|------|--------|---------|
| `/data/kruxos/workspace/{name}/` | Read + Write | The agent's private workspace. All file operations are scoped here. |
| `/data/shared/` | Read only | Shared read-only data accessible to all agents. |
| `/etc/kruxos/definitions/` | Read only | Capability definition files (YAML). |
| All other paths | **Denied** | Including `/proc`, `/sys`, `/etc`, `/root`, other agents' workspaces, the vault database, and the audit log directory. |

**Critical protections provided by Landlock:**

- **Vault database (`/data/kruxos/vault.db`)**: Denied. Prevents agents from reading the encrypted vault database and attempting offline attacks against the master key.
- **Audit logs (`/data/kruxos/audit/`)**: Denied. Prevents agents from reading, modifying, or deleting audit entries.
- **Other agents' workspaces (`/data/kruxos/workspace/*/`)**: Denied. Only the agent's own workspace path is allowed.
- **Process environment (`/proc/*/environ`)**: Denied. Prevents agents from reading environment variables (which include the vault passphrase on the Gateway process).
- **System files (`/etc/shadow`, `/etc/passwd`)**: Denied. Prevents credential harvesting.

**Kernel version requirement:** Landlock requires kernel >= 5.13. KruxOS ships kernel 6.6 LTS with full Landlock support. On older kernels (when running the gateway binary directly on a host), Landlock returns `NotSupported` and the filesystem confinement layer is skipped. PID namespace mount isolation provides partial compensation, but Landlock is the authoritative filesystem MAC layer.

**Irrevocability:** Like seccomp, Landlock rulesets are irrevocable once applied. The sandboxed process cannot add new allowed paths or remove restrictions.

### 3.5 Layer 5: nftables Per-Agent Network Rules

Network egress is controlled through two complementary mechanisms:

**Application-level domain validation (primary):** The `SandboxCapabilityExecutor` in the Gateway validates domain names from `network.*` capability inputs against a per-agent allowed-domains list before any network request is made. This is the authoritative enforcement point because agents interact through typed capabilities (e.g., `network.http_request(url="https://api.example.com/data")`), not raw sockets.

**Kernel-level nftables rules (defense-in-depth):** Per-agent nftables chains provide IP/CIDR-based egress filtering as a second layer. Even if a process inside the sandbox obtains network access (e.g., through a network-enabled capability), nftables restricts which IP addresses it can reach.

**Default policy:** Default-deny. An agent with no network rules configured cannot make any outbound connections.

**Rule structure per agent:**

```
table inet kruxos {
    chain agent_{name}_output {
        type filter hook output priority 0; policy drop;
        
        # Always allow loopback
        oifname "lo" accept
        
        # Allow established/related connections
        ct state established,related accept
        
        # Auto-allow DNS when any egress rules exist
        udp dport 53 accept
        tcp dport 53 accept
        
        # Per-destination rules from policy
        ip daddr {allowed_ip} tcp dport {port} accept
        
        # Log and drop everything else
        log prefix "[kruxos-out-drop] " drop
    }
}
```

**Network namespace isolation for non-network capabilities:** When a forked sandbox child executes a non-network capability (e.g., `filesystem.read`, `process.run`), it calls `unshare(CLONE_NEWNET)` to create an empty network namespace with only a loopback interface. This means even if the handler code attempts outbound connections (through a bug or malicious capability pack), the kernel blocks them at the namespace level — there are no network interfaces to route through.

**Network capabilities (special case):** Capabilities like `network.http_request` require actual network access. These execute in the Gateway process after domain validation — the Gateway acts as the agent's network proxy. The forked child process never makes network requests directly. This proxy model is a deliberate tradeoff: it simplifies the network stack (no veth pair setup required) at the cost of sharing the Gateway's network context. See Section 8 for the SSRF risk implications.

---

## 4. Cryptographic Architecture

### 4.1 Secrets Vault

The secrets vault (`crates/vault/`) provides encrypted storage for all credentials with a use-not-read access model.

**Encryption at rest:**
- **Algorithm:** AES-256-GCM (authenticated encryption with associated data)
- **Key derivation:** Argon2id with the following parameters:
  - Memory cost: 64 MiB
  - Time cost: 3 iterations
  - Parallelism: 4 lanes
  - Output length: 256 bits (32 bytes)
  - Salt: Random 128-bit, stored in the `vault_meta` database table
- **Nonce:** Random 96-bit per secret, stored alongside the ciphertext
- **Verification:** A known plaintext value is encrypted during vault initialization. On unlock, this value is decrypted and compared to verify the passphrase is correct before deriving the master key.

**Master key lifecycle:**
1. The administrator provides a passphrase during vault initialization or unlock (typically via the `KRUXOS_VAULT_PASSPHRASE` environment variable, supplied by systemd at startup)
2. Argon2id derives a 256-bit master key from the passphrase + stored salt
3. The master key exists only in memory, in a `MasterKey` struct that implements `Zeroize` and `ZeroizeOnDrop`
4. When the vault is locked or the process exits, the master key is zeroed in memory
5. The master key is never written to disk, never logged, and never transmitted

**Dashboard login passphrase (separate from the vault master key):** To allow the operator to log in to the web dashboard *after* the gateway has unlocked the vault, KruxOS writes a `bcrypt` hash of the operator's chosen passphrase to `${data_dir}/vault_passphrase_hash` (mode `0600`) during the first-boot wizard. The supervision WebSocket (port 7701) and dashboard login form verify operator-supplied passphrases against this bcrypt hash with a constant-time comparison. The bcrypt hash is **only** for verifying the human operator at login time; it is never used to derive any encryption key. The vault's actual encryption key still derives from the original passphrase via Argon2id at gateway startup, before the dashboard is reachable.

**Use-not-read access model:**
Agents never receive raw secret values. Instead:
1. An agent calls a capability that requires a secret (e.g., `email.send` requires Gmail OAuth token)
2. The Gateway creates a per-invocation `SecretProvider` scoped to the calling agent
3. The capability handler calls `provider.get_handle("gmail-oauth")` to get an opaque `SecretHandle`
4. The handler calls `handle.resolve("email.send")` to obtain the decrypted value
5. The `resolve()` method verifies the requesting capability matches the secret's `allowed_capabilities` list
6. The decrypted `SecretValue` is used internally by the handler and never included in the response to the agent

The `SecretValue` type enforces this architecturally:
- No `Serialize` implementation — cannot be included in JSON responses
- No `Display` implementation — cannot be accidentally logged
- `Debug` shows only `SecretValue(N bytes)` — no value leakage in debug output
- No `Clone` — prevents copying the value to uncontrolled locations

**Capability scoping:** Each secret is bound to a list of capability patterns. A Gmail OAuth token might have `allowed_capabilities: ["email.*", "proxy.gmail.*"]`. A request to resolve this secret from `filesystem.read` is denied and audit-logged.

### 4.2 Principal Model and Authentication

KruxOS distinguishes two principal types — `User` (the human operator) and `Agent` (autonomous code) — and they authenticate against separate listeners with separate credential schemes. Every authenticated request carries its `Principal` through to policy evaluation and to every audit row, so a User-initiated tool call and an Agent-initiated tool call are never conflated.

| Principal | Listener | Credential | Notes |
|-----------|----------|-----------|-------|
| **User** (operator) | TCP **7703** (HTTP, loopback by default) | Bearer token | Issued via `kruxos user-token create`; one-time raw-token reveal; CLI tools (`mcp-bridge`, `cli-hook`) read the bearer from the vault, env, or stdin so it never appears in argv |
| **Agent** (autonomous) | TCP **7700** (MCP, JSON-RPC fallback) | API key | Issued via `kruxos agent create`; subject to per-agent policy; revocable via `kruxos agent revoke` |
| **Operator** (dashboard) | TCP **7800** (HTTPS, self-signed cert) | bcrypt-hashed passphrase | Verified against `${data_dir}/vault_passphrase_hash`; supervision WebSocket on 7701 uses the same hash |

**Token generation and storage:**

1. **Generation:** 32 random bytes (256 bits of entropy) from the OS CSPRNG, hex-encoded
2. **Display:** The plaintext token is shown once at creation time, then discarded
3. **Storage:** SHA-256 hash of the token stored in `agents.db` / the user-token table
4. **Verification:** On each connection, the gateway SHA-256-hashes the provided token and performs a constant-time comparison (`subtle::ConstantTimeEq`) against the stored hash

**Why SHA-256, not bcrypt, for User and Agent tokens:** Both token types are 32 random bytes — brute-force is computationally infeasible regardless of hash speed (2²⁵⁶ possible keys). bcrypt's intentional slowness is designed for low-entropy human passwords. SHA-256 is ~1000× faster, which matters because every gateway request verifies a token. The dashboard-login passphrase (§4.1) *is* low-entropy human input, which is why it uses bcrypt.

**Revocation:** Revoked agents and revoked user tokens are tracked in their respective SQLite tables. The gateway checks revocation status during authentication and returns a specific `auth.agent_revoked` / `auth.user_token_revoked` error (not the generic `auth.invalid_credentials`) so callers can distinguish "wrong key" from "principal has been disabled."

**Delegation chain:** When the operator (User) sends a chat turn that triggers an `AgentX` to send a `comms.send` message to `AgentY`, every audit row in the chain — User MCP call, AgentX execution, AgentX → AgentY message, AgentY execution — is correlated via `metadata.delegation` (originating-conversation id, from-agent), so the full provenance from human intent to final tool call is reconstructible from the audit log alone.

### 4.3 Audit Log Integrity

The audit log uses a SHA-256 hash chain for tamper detection:

- **Entry hash:** `SHA-256(canonical_json(entry_fields) || previous_hash)`
- **Canonical JSON:** All entry fields except `entry_hash` and `previous_hash`, serialized in alphabetical key order
- **Genesis hash:** `SHA-256("kruxos-audit-genesis-v1")` — a deterministic starting point
- **Cross-file chaining:** The first entry of each daily log file chains from the previous file's last entry hash

This chain detects three classes of tampering:
1. **Modification:** Changing any field in an entry invalidates its hash and breaks the chain
2. **Deletion:** Removing an entry causes the next entry's `previous_hash` to not match any preceding entry
3. **Reordering:** Swapping entries breaks the sequential hash chain

### 4.4 TLS Configuration

- **Agent-to-Gateway:** WebSocket over `ws://` (localhost) or `wss://` (remote). TLS is recommended for remote connections but not enforced by default in v0.0.1. The gateway binds to `127.0.0.1` by default; binding to `0.0.0.0` for remote access is an explicit configuration choice.
- **Dashboard:** **HTTPS by default** on port 7800. On first start, the dashboard generates a self-signed RSA certificate via `node:crypto` and persists it under the dashboard TLS directory; subsequent starts reuse it. Operators can replace the cert/key with files from a real CA by writing to `KRUXOS_TLS_CERT` / `KRUXOS_TLS_KEY`; user-supplied certificates are never overwritten. To opt out of TLS entirely (e.g., behind a reverse proxy that already terminates), set `KRUXOS_TLS_DISABLED=true`. For internet-facing deployments, fronting the dashboard with Caddy/nginx for a real certificate is still recommended.
- **External service connections:** The service proxy (Gmail, Slack, OpenAI Codex adapters) uses TLS for all API calls via `reqwest` with system certificate verification.

### 4.5 OAuth Token Management

For service proxy integrations (Gmail, Slack) and OAuth-based model providers (OpenAI Codex's `auth.openai.com` device-code flow), OAuth tokens are managed securely:

- **PKCE:** Authorization Code flow with Proof Key for Code Exchange prevents authorization code interception
- **Token storage:** Encrypted in the vault as `OAuthToken` secret type
- **Auto-refresh:** Background task checks all OAuth tokens every 60 seconds; tokens expiring within 5 minutes are automatically refreshed
- **Failure handling:** Failed refresh attempts mark the token as `needs_attention`; the supervisor is notified via the alerts system
- **Revocation on disconnect:** When a service is disconnected, OAuth tokens are revoked at the provider and deleted from the vault

---

## 5. Audit Trail

### 5.1 Format and Storage

Every capability invocation, policy decision, session event, and state change is recorded in the audit log. Entries are stored in **CBOR** (Concise Binary Object Representation) format in daily log files at `/data/kruxos/audit/audit-YYYY-MM-DD.log`.

**Framing:** Each entry is prefixed with a 4-byte big-endian length field, followed by the CBOR-encoded entry. This length-prefixed framing (not newline-delimited) is necessary because CBOR is a binary format that may contain bytes that look like newline characters.

**Entry structure:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique entry identifier |
| `timestamp` | RFC 3339 | When the event occurred |
| `actor` | Principal | `{type:"user"}` or `{type:"agent",name:"..."}` (see §4.2) |
| `session_id` | UUID | Which session the action occurred in |
| `capability` | String | The capability invoked (e.g., `filesystem.read`) |
| `inputs` | JSON (sanitized) | The inputs provided (with secrets redacted) |
| `output_summary` | String | Summary of the result |
| `status` | Enum | `success`, `denied`, `error`, `pending` |
| `policy_decision` | Object | Tier, rule reference, and reason for the decision |
| `duration_ms` | Integer | Execution time in milliseconds |
| `resource_usage` | Object | CPU time, memory peak, I/O bytes |
| `schema_version` | Integer | Audit-row schema version (legacy v1 entries verify alongside current entries) |
| `previous_hash` | Hex string | SHA-256 hash of the preceding entry |
| `entry_hash` | Hex string | SHA-256 hash of this entry |
| `metadata` | JSON | Additional context — error details, `approval_request_id` correlating back to the approval queue, `delegation` envelope on `comms.send` rows correlating User → AdminAgent → AgentX chains, etc. |

**Notable event types:**

- **`path_escape`** — recorded whenever a filesystem capability detects a symlink-escape or `..`-traversal attempt against the agent's workspace or a host mount. Carries the mount label and the resolved path so a reviewer can see which boundary was probed.
- **`cli_session.oom_killed`** — recorded when a `/code` session is killed by the cgroup OOM killer (distinct from a regular `exited` event).
- **`approval_required` → `approved` / `rejected` / `timed_out`** — the original tier is preserved on the row even when the queue resolution arrives later; dashboards and `kruxos audit` queries can reconstruct the timeline via `metadata.approval_request_id`.

### 5.2 Secret Redaction

Before writing to the audit log, the input sanitizer scans all fields for sensitive patterns and replaces matched values with `[REDACTED:field_name]`. Matched patterns include:

`api_key`, `token`, `password`, `secret`, `credential`, `authorization`, `private_key`, `access_key`, `refresh_token`, `client_secret`, `passphrase`

This prevents accidental secret leakage into the audit log, which may be readable by supervisors.

### 5.3 Tamper Detection

The hash chain (described in Section 4.3) provides cryptographic tamper detection. The `verify` command walks the entire chain and reports:

- **Intact:** All hashes verify, chain is unbroken
- **Modified at entry N:** Entry N's computed hash does not match its stored hash
- **Missing entry before N:** Entry N's `previous_hash` does not match entry N-1's hash
- **Truncated:** The chain ends before the expected final entry

### 5.4 Disk-Full Resilience

The audit writer prioritizes system availability over perfect audit completeness:

1. **Normal mode:** Entries are written to disk and fsynced immediately
2. **Degraded mode (disk full):** The writer detects fsync failure, fires a health alert (`audit writer degraded`), and switches to an in-memory ring buffer (default 10,000 entries; configurable). Operators are expected to alert on `WriterState::Degraded`.
3. **Recovery:** The writer retries disk writes every 30 seconds. On success, buffered entries are flushed to disk and the hash chain is maintained across the boundary.
4. **Configurable:** `audit.fail_mode: halt` changes this behavior to return errors to callers instead of degrading — appropriate for compliance environments where audit gaps are unacceptable.

### 5.5 Retention and Rotation

- **Default retention:** 90 days
- **Rotation:** Daily files (`audit-YYYY-MM-DD.log`) are rotated at **03:00 UTC** by a background task
- **Warning:** 7-day warning before the first deletion in a retention window
- **Index cleanup:** SQLite index entries for expired log files are removed in the same operation

### 5.6 Query Performance

A SQLite index database (`/data/kruxos/audit/audit-index.db`) provides fast querying:

- Indexed fields: `timestamp`, `actor`, `session_id`, `capability`, `status`
- Full entry retrieval reads from the CBOR files, grouped by log file to minimize I/O
- The index contains entry metadata only — the full entry (including inputs and outputs) is in the CBOR file

---

## 6. Policy Enforcement

### 6.1 Permission Model

KruxOS uses a four-tier permission model for capability governance:

| Tier | Behavior | Use Case |
|------|----------|----------|
| **Autonomous** | Execute immediately, log only | Low-risk read operations |
| **Notify** | Execute immediately, notify supervisor via dashboard/CLI | Moderate operations the supervisor should be aware of |
| **Approval Required** | Queue for human approval before execution | High-risk operations (deleting files, sending emails) |
| **Blocked** | Deny immediately with structured explanation | Operations that should never be performed |

### 6.2 Hierarchical Evaluation

Policies are evaluated in a three-layer hierarchy where the most restrictive tier always wins:

```
System policy (always enforced, cannot be overridden)
    ↓ most restrictive wins
Organization policy (organizational baseline)
    ↓ most restrictive wins
Agent policy (per-agent customization)
```

A lower-level policy can only be **more restrictive**, never less. If the system policy blocks `secrets.read_raw`, no organization or agent policy can unblock it.

**10 hardcoded system protections** (enforced even if all policy files are deleted):

1. `filesystem.write` to system paths → Blocked
2. `filesystem.delete` of system files → Blocked
3. `secrets.read_raw` (raw secret value access) → Blocked
4. `secrets.export` (bulk secret export) → Blocked
5. `audit.write` (injecting audit entries) → Blocked
6. `audit.delete` (deleting audit entries) → Blocked
7. `audit.truncate` (truncating audit log) → Blocked
8. `policy.modify_system` (changing system policy) → Blocked
9. `state.read_private` of another agent's data → Blocked
10. `state.write_private` to another agent's data → Blocked

### 6.3 Evaluation Performance

The policy compiler pre-indexes rules by capability category prefix (e.g., `filesystem`, `process`, `network`). At evaluation time, only the relevant category bucket plus wildcard rules are checked — not the full rule set. This achieves sub-millisecond evaluation on policy files with 100+ rules.

Pattern matching supports:
- **Exact match:** `filesystem.read` matches only `filesystem.read`
- **Category wildcard:** `filesystem.*` matches any capability in the `filesystem` category
- **Global wildcard:** `*` matches all capabilities

### 6.4 Rate Limiting

Policy rules can include rate limits that escalate the permission tier when exceeded:

```yaml
- match:
    capability: "email.send"
  tier: notify
  rate_limit:
    max_per_hour: 20
    escalate_to: approval_required
```

- Counters are in-memory, per-agent, per-rule
- Sliding 1-hour window
- When exceeded, the tier escalates (only to a more restrictive tier)
- Counters reset on gateway restart (acceptable for v0.0.1; persistent counters planned for enterprise)

### 6.5 Approval Queue

When a capability invocation requires approval:

1. An `ApprovalRequest` is created in SQLite (`/data/kruxos/approval_queue.db`)
2. The agent receives a `pending` response with the request ID
3. Supervisors view pending requests via the dashboard Approvals page (Pending / Approved / Rejected / Timed Out tabs) or via `kruxos audit`
4. The supervisor approves or rejects the request; the dashboard is the canonical approval surface — CLIs (Claude Code, Codex) route through it rather than prompting in-line
5. The agent polls for the decision; on approval, the capability executes
6. Requests expire after a configurable timeout (default: **24 hours** for User MCP calls; configurable via `approval.hold_timeout_seconds` in `/etc/kruxos/gateway.yaml`)
7. Timed-out requests cannot be approved retroactively — a late approve attempt returns HTTP `409 Conflict` with a status discriminator

---

## 7. Deployment Modes

KruxOS supports two deployment modes with different security characteristics:

### 7.1 OS Image (Full Isolation)

The KruxOS OS image is a minimal Linux distribution (Buildroot-based, kernel 6.6 LTS) purpose-built for agent execution.

| Security Feature | Status (v0.0.1) | Implementation |
|-----------------|-----------------|----------------|
| PID namespace | **Active** | Kernel `CLONE_NEWPID` |
| Mount namespace | **Active** | Kernel `CLONE_NEWNS` + `pivot_root` |
| Network namespace | **Active** | Kernel `CLONE_NEWNET` |
| User namespace | **Active** | Kernel `CLONE_NEWUSER` |
| cgroup v2 limits | **Active** | Kernel cgroup controllers |
| seccomp-bpf | **Active** | Kernel BPF filter (no libseccomp dependency) |
| Landlock MAC | **v0.0.3** | Kernel Landlock LSM — part of the v0.0.3 security architecture rework |
| nftables egress | **Active** | Kernel nftables |
| Policy engine | **Active** | Gateway application layer; four-tier deterministic YAML |
| Audit logging | **Active** | Gateway application layer; length-prefixed CBOR, SHA-256 hash chain |
| Secrets vault | **Active** | AES-256-GCM SQLite |
| Agent authentication | **Active** | 64-char hex token verification (MCP handshake on TCP 7700) |
| User authentication | **Active** | `krx_user_*` bearer tokens on the loopback User API (TCP 7703) |
| Gateway / code-session privilege separation | **v0.0.3** | Part of the security architecture rework |

### 7.2 Docker (Application-Level Isolation)

The Docker deployment provides application-level security with Docker's own container isolation replacing the per-agent kernel sandbox.

| Security Feature | Status | Implementation |
|-----------------|--------|----------------|
| PID namespace | Via Docker | Docker container boundary |
| Mount namespace | Via Docker | Docker container boundary |
| Network namespace | Via Docker | Docker bridge network |
| User namespace | Via Docker | Container user |
| cgroup v2 limits | Via Docker | `--memory`, `--cpus`, `--pids-limit` flags |
| seccomp-bpf | Via Docker | Docker's default seccomp profile |
| Landlock MAC | **Inactive** | Not available inside containers; ships in the v0.0.3 architecture rework regardless |
| nftables egress | **Inactive** | Docker network rules instead |
| Policy engine | **Active** | Gateway application layer |
| Audit logging | **Active** | Gateway application layer |
| Secrets vault | **Active** | Gateway application layer |
| Agent authentication | **Active** | Gateway application layer |

**Multi-agent isolation in Docker:** For agent-to-agent isolation, the recommended pattern is **one container per agent**. Each agent gets its own filesystem, process tree, and network namespace via Docker's isolation. This is the Docker equivalent of the OS image's per-agent kernel sandbox. Resource limits should be set via Docker flags (`--memory`, `--cpus`, `--pids-limit`) rather than cgroups inside the container.

**Dashboard:** In both deployment modes, the dashboard serves HTTPS by default on port 7800 with a self-signed certificate generated on first start (see §4.4). For internet-facing deployments, replace the self-signed cert with a CA-issued one (via `KRUXOS_TLS_CERT` / `KRUXOS_TLS_KEY`) or terminate TLS at a reverse proxy (Caddy, nginx) and set `KRUXOS_TLS_DISABLED=true` on the dashboard process.

### 7.3 Operator-Facing Surfaces

Three operator-facing surfaces deserve a security note because they handle interactive sessions, host-filesystem access, or operator credentials.

**Code Sessions (`/code` page).** The dashboard exposes browser-based xterm.js terminals running Claude Code or Codex through the gateway sandbox. Hardening:

- **Auth:** Subprocess auth via the `KRUXOS_USER_TOKEN` environment variable on a hardened env; argv self-check kills the child if the token leaks into argv. The dashboard ws-proxy pins the WebSocket `Origin` (rejecting mismatched / null / missing with HTTP 403); the gateway WebSocket itself uses bearer authentication.
- **Per-session cgroup:** 2 GiB memory cap (`code_sessions.memory_max_bytes`); OOM-killed sessions emit a distinct `cli_session.oom_killed` audit event.
- **Concurrency cap:** 4 sessions by default (`code_sessions.max`); the 5th spawn returns HTTP `429 code_session.too_many_sessions`.
- **Workdir validation:** Sessions can only run under the operator home or a registered agent-mount source — `/workspace/` is excluded.
- **Idle timeout:** 4 hours by default (`code_sessions.idle_timeout_seconds`).

**Host mounts (`/mnt/<label>`).** Operators can expose host directories to specific agents read-only or read-write via `kruxos mount add / list / remove / toggle-readonly / relabel`. Path-escape detection (symlink and `..`-traversal) emits `path_escape` audit events; trusted paths are bind-mounted into the sandbox under `/mnt/<label>` rather than into the agent's general workspace.

**Host-CLI integration (`mcp-bridge`, `cli-hook`).** When the operator runs Claude Code or Codex on the host (rather than inside the dashboard `/code` terminal), `mcp-bridge` and `cli-hook` route every gated tool call through the KruxOS approval queue — the CLI's native shell tool is disabled at the user-config and requirements layers (Codex `tool_timeout_sec=86400`, `required=true`; native `shell` and `unified_exec` off). The bearer token is read from the vault, env, or stdin so it never appears in `argv`; both binaries emit structured exit codes (`2 = argv leak`, `3 = vault locked`, `4 = token missing`).

### 7.4 Comparison Summary

| Property | OS Image | Docker |
|----------|----------|--------|
| Kernel isolation layers | 5 (all active) | Provided by Docker runtime |
| Per-agent sandbox | Yes (kernel namespaces per agent) | Yes (one container per agent) |
| Landlock filesystem MAC | Yes | No (Docker filesystem isolation instead) |
| Custom seccomp profile | Yes (KruxOS allowlist) | Docker default profile |
| Policy engine | Yes | Yes |
| Audit logging | Yes | Yes |
| Secrets vault | Yes | Yes |
| Resource overhead per agent | ~1 MB (cgroup + namespace metadata) | ~10-30 MB (container overhead) |
| Startup time per agent | <100 ms (namespace creation) | 1-5 seconds (container start) |
| Recommended for | Production, high-security environments | Development, single-agent deployments, Mac/Windows hosts |

---

## 8. Known Limitations and Mitigations

This section documents known security limitations in KruxOS v0.0.1 with complete transparency. Each limitation includes the planned mitigation milestone in the `v0.0.x` series, or "future" for items without a fixed slot yet. All planned items are tracked in the public issue tracker.

### 8.1 Vault Is In-Process

**Limitation:** The vault runs as a library within the Gateway process, not as a separate process or hardware security module. If the Gateway process memory is dumped (e.g., via a kernel exploit or physical access), the decrypted master key could be extracted.

**Mitigation:** The `MasterKey` struct uses the `zeroize` crate to clear memory on drop. The seccomp filter blocks `ptrace` (preventing memory debugging from other processes), and PID namespace isolation prevents agents from accessing `/proc/{gateway_pid}/mem`.

**Planned (v0.0.4):** Separate vault process with Unix domain socket IPC, reducing the attack surface to the IPC protocol rather than shared memory.

### 8.2 No TPM Integration

**Limitation:** The vault master key is derived from a passphrase stored in a file on the data partition (`/data/kruxos/.vault-env`). There is no hardware-backed key protection.

**Mitigation:** The passphrase file is `chmod 600` (owner-only readable). The data partition is a separate filesystem from the root partition. Argon2id with 64 MiB memory cost makes offline brute-force expensive.

**Planned (future / v0.0.6+):** TPM 2.0 integration for hardware-sealed key storage, with PCR-based boot attestation.

### 8.3 DNS Rebinding Risk

**Limitation:** An attacker controlling a domain on an agent's allowlist could change the domain's DNS records to point to an internal IP address (e.g., `169.254.169.254` for cloud metadata, `127.0.0.1` for localhost services) after passing the domain allowlist check.

**Mitigation:** Network capabilities execute in the Gateway process, which validates the domain before each request. The URL parser resolves the domain independently of the HTTP client. Forked sandbox children have no network access (`CLONE_NEWNET`), so only the Gateway's network stack is reachable.

**Planned (v0.0.4):** Private IP range filtering (RFC 1918 blocks, link-local, loopback) applied after DNS resolution. Cloud metadata endpoint blocking.

### 8.4 Side-Channel Timing Attacks Between Sandboxes

**Limitation:** Agents sharing the same physical CPU can observe timing variations through shared microarchitectural state (L1/L2/L3 caches, branch predictor, TLB). This could theoretically leak information between agents via cache-timing attacks.

**Mitigation:** Not addressed in v0.0.1. The practical impact is low for most deployments because: (a) agent workloads are I/O-bound (file operations, API calls), not compute-bound; (b) exploiting cache-timing attacks requires precise measurement that is difficult through the capability API abstraction.

**Planned (future / v0.0.6+):** Optional core pinning (`cpuset` cgroup controller) and cache partitioning (Intel CAT/AMD L3QoS) for high-security deployments.

### 8.5 Fork-Based Execution Model Shares Parent Memory at Fork Time

**Limitation:** The `fork()` call creates a copy-on-write snapshot of the Gateway's memory in the child process. At the moment of fork, the child has a read-only view of the parent's entire address space, including the vault master key. The child cannot read the parent's memory *after* fork (copy-on-write semantics mean modifications are not shared), but the master key bytes are present in the child's address space until they are overwritten by the handler's execution.

**Mitigation:** The child immediately applies seccomp and Landlock, preventing it from dumping its own memory to files or sending it over the network. The child's only IPC mechanism is a write-only pipe back to the parent, and the pipe protocol only accepts `CapabilityResponse` structures. An agent would need to exploit a bug in the handler code to extract the master key bytes from the process address space before seccomp is applied — a window of microseconds.

**Planned (v0.0.4):** Move to a pre-forked worker pool model where workers are forked before the vault is unlocked, eliminating the key-in-child-memory window entirely.

### 8.6 In-Process Network Execution (SSRF Risk)

**Limitation:** Network capabilities (`network.http_request`, `network.download`, etc.) execute in the Gateway process, sharing the Gateway's full network permissions. The domain allowlist is the only defense. If a URL parsing quirk allows a crafted URL to pass the domain check but resolve to an internal IP, the Gateway would make the request with its own network context.

**Mitigation:** The domain allowlist uses the `url` crate's parser (WHATWG URL standard compliant) to extract the hostname before the request. The `reqwest` HTTP client does not follow redirects to different domains. The Gateway binds to `127.0.0.1` by default, limiting exposure to the local machine.

**Planned (v0.0.4):** veth-pair model where network capabilities execute inside the sandbox's network namespace with nftables egress filtering, eliminating the shared-network-context issue entirely.

### 8.7 Rate Limit State Is Volatile

**Limitation:** Rate limit counters are in-memory and reset on Gateway restart. A restart clears all rate limit state, allowing agents to exceed intended hourly limits.

**Mitigation:** Rate limits are a safety net, not a billing mechanism. The approval queue (persistent in SQLite) provides the hard enforcement for high-risk operations. Gateway restarts are infrequent in production.

**Planned (enterprise):** Persistent rate limit counters in SQLite for compliance environments.

### 8.8 Internal Traffic Not Encrypted by Default

**Limitation:** Agent-to-Gateway traffic over WebSocket uses `ws://` (unencrypted) when connecting via localhost. The supervision WebSocket (port 7701) also uses `ws://` by default.

**Mitigation:** The Gateway binds to `127.0.0.1` by default, so traffic does not traverse the network. Remote access requires explicit configuration to bind to `0.0.0.0`, at which point TLS should be configured. For remote access, deploy a TLS-terminating reverse proxy such as Caddy. Built-in Let's Encrypt support is planned for v0.0.4.

**Planned (v0.0.4):** Automatic TLS for all WebSocket connections when binding to non-localhost addresses.

---

## 9. Comparison with Alternatives

### 9.1 KruxOS vs. Docker Alone

| Property | Docker alone | KruxOS (OS image) |
|----------|-------------|-------------------|
| Process isolation | Yes (container boundary) | Yes (per-agent PID namespace + container) |
| Filesystem MAC | No (DAC only) | Yes (Landlock deny-by-default) |
| Syscall filtering | Default Docker profile | Custom allowlist tuned for agent workloads |
| Per-agent resource limits | Manual (`--memory`, `--cpus`) | Automatic per-agent cgroup with configurable defaults |
| Capability governance | None | 4-tier policy engine with approval workflow |
| Secrets management | Env vars or Docker secrets | Encrypted vault with use-not-read access model |
| Audit logging | Docker daemon logs | Hash-chained, CBOR-format, queryable audit trail |
| Network policy per agent | Docker network rules | Per-agent domain allowlist + nftables |
| AI-native API | No (shell access, text parsing) | Yes (typed MCP/JSON-RPC capabilities) |

**Summary:** Docker provides container-level isolation but leaves governance, secrets, and AI-specific concerns to the user. KruxOS adds governance, secrets, audit, and agent-specific isolation layers on top of kernel primitives.

### 9.2 KruxOS vs. Virtual Machines

| Property | VMs (KVM/Xen) | KruxOS (OS image) |
|----------|---------------|-------------------|
| Isolation strength | Hardware-level (hypervisor) | Kernel-level (namespaces, seccomp, Landlock) |
| Side-channel resistance | Strong (separate address spaces) | Weak (shared CPU microarchitecture) |
| Resource overhead | 256 MB+ RAM per VM | ~1 MB per agent (cgroup metadata) |
| Startup time | 5-30 seconds | <100 ms |
| Agent density | 10-50 per host | 100+ per host |
| Governance layer | None built-in | Integrated policy, audit, secrets |
| Operational complexity | High (VM lifecycle, images) | Low (single OS, capability API) |

**Summary:** VMs provide stronger isolation (especially against side-channel attacks) but at significantly higher resource cost and operational complexity. KruxOS is designed for density and developer experience, trading some isolation depth for agent-native governance and 100x lower overhead per agent. For environments requiring VM-grade isolation, KruxOS can run inside a VM for defense-in-depth.

### 9.3 KruxOS vs. No Isolation (Bare Metal / Host OS)

| Risk | Bare metal | KruxOS |
|------|-----------|---------|
| Agent reads `/etc/shadow` | Possible (if running as root) | Blocked (Landlock) |
| Agent kills system services | Possible | Blocked (PID namespace + seccomp) |
| Agent installs rootkit | Possible (if running as root) | Blocked (seccomp blocks module loading) |
| Agent exfiltrates data via network | Unrestricted | Per-agent domain allowlist |
| Agent exhausts system memory | Unrestricted (OOM kills random processes) | cgroup limit, only the agent is killed |
| Compromised agent spreads laterally | Full system access | Confined to agent workspace |
| Audit trail of agent actions | None (unless manually implemented) | Comprehensive, hash-chained, tamper-evident |

**Summary:** Running AI agents on bare metal with no isolation is analogous to giving every employee root access to every server. KruxOS applies the principle of least privilege at the kernel level.

---

## 10. Responsible Disclosure

### 10.1 Reporting Security Vulnerabilities

If you discover a security vulnerability in KruxOS, please report it responsibly:

**Email:** security@altvale.com  
**GitHub Security Advisories:** https://github.com/altvale/kruxos/security/advisories/new  
**security.txt:** https://docs.kruxos.com/.well-known/security.txt (RFC 9116; PGP encryption deferred to a later release)  
**Disclosure policy:** https://github.com/altvale/kruxos/blob/main/SECURITY.md

**What to include:**
- Description of the vulnerability
- Steps to reproduce
- Impact assessment
- Affected versions (if known)
- Suggested fix (if available)

**Please do NOT:**
- Open a public GitHub issue for security vulnerabilities
- Exploit the vulnerability against production systems
- Share the vulnerability publicly before a fix is available

### 10.2 Response Timeline

| Phase | Target Timeline |
|-------|----------------|
| Acknowledgement | Within 48 hours |
| Initial assessment | Within 7 days |
| Fix development | Within 30 days for critical, 90 days for moderate |
| Public disclosure | Coordinated with the reporter, typically 90 days after report |
| Security advisory | Published on GitHub Security Advisories and the project website |

### 10.3 Scope

The following are in scope for security reports:
- Sandbox escape (bypassing any of the five isolation layers)
- Vault secret leakage (raw secret values accessible to agents)
- Audit log tampering (bypassing hash chain integrity)
- Policy bypass (executing blocked capabilities)
- Authentication bypass (connecting without valid credentials)
- Privilege escalation (gaining capabilities beyond the agent's policy tier)

The following are out of scope:
- Denial of service via legitimate resource consumption within limits
- Social engineering of human administrators
- Vulnerabilities in third-party dependencies (report these to the upstream project, but let us know so we can patch)

---

## Appendix A: Compliance Mapping

This appendix maps KruxOS security controls to common compliance frameworks. **This is not a certification claim** — KruxOS has not undergone SOC 2 audit, CIS benchmark assessment, or NIST evaluation. This mapping helps security reviewers orient quickly by relating KruxOS controls to frameworks they already understand.

### A.1 SOC 2 Type II Controls

| SOC 2 Trust Service Criteria | KruxOS Control | Status |
|-----------------------------|----------------|--------|
| **CC6.1** Logical access security | API key authentication, SHA-256 hashing, constant-time comparison, revocation via agents.db | **Addressed** |
| **CC6.2** Access provisioned based on authorization | 4-tier policy engine: autonomous/notify/approval/blocked per capability per agent | **Addressed** |
| **CC6.3** Access removed when no longer needed | `kruxos agent revoke` removes access; Gateway checks revocation status on every connection | **Addressed** |
| **CC6.6** Security events logged | Every capability invocation, policy decision, and state change audit-logged with CBOR + hash chain | **Addressed** |
| **CC6.7** Unauthorized access detected | Policy engine denies blocked capabilities; audit log records all denials; supervisor notified for `notify` tier | **Addressed** |
| **CC6.8** Boundaries to external threats | 5-layer kernel sandbox; default-deny network egress; nftables per-agent rules | **Addressed** |
| **CC7.1** Monitoring for anomalies | Rate limiting with escalation; real-time supervision WebSocket; health monitoring | **Partial** — no ML-based anomaly detection |
| **CC7.2** Incident response | Audit log query/replay for forensics; agent revocation; sandbox freeze/destroy | **Partial** — no automated incident playbooks |
| **CC8.1** Change management | Policy hot-reload from YAML; capability definition updates without restart | **Partial** — no change approval workflow for policy files |
| **CC9.1** Risk mitigation | Threat model documented; 94 adversarial security tests; 5-layer defense-in-depth | **Addressed** |

**Out of scope for KruxOS (organization-level controls):**
- CC1.x (Control environment) — organizational governance, not software
- CC2.x (Communication) — organizational processes
- CC3.x (Risk assessment) — organizational risk management
- CC5.x (Control activities) — organizational policies and procedures

### A.2 CIS Benchmark Alignment (Linux Hardening)

| CIS Control | KruxOS Implementation | Status |
|------------|----------------------|--------|
| **1.1** Filesystem configuration | Immutable root partition (read-only); separate data partition; Landlock MAC | **Addressed** |
| **1.4** Secure boot | UEFI boot with GRUB; A/B partition scheme for rollback | **Partial** — no Secure Boot signature verification |
| **2.1** Minimize installed services | Minimal Buildroot image; only KruxOS services running | **Addressed** |
| **3.1** Network parameters | Default-deny nftables egress; per-agent network namespace | **Addressed** |
| **4.1** Logging and auditing | Hash-chained CBOR audit log; SQLite index; 90-day retention | **Addressed** |
| **4.2** Log integrity | SHA-256 hash chain with tamper detection; agents cannot access audit directory | **Addressed** |
| **5.1** Access control | API key authentication; policy-based authorization; 4-tier model | **Addressed** |
| **5.2** SSH configuration | Opt-in OpenSSH, disabled by default; when enabled: root key-only, password authentication disabled, `tcp/22` firewalled until enabled | **Addressed** (hardened when enabled) |
| **5.3** Privilege escalation | seccomp blocks privilege-related syscalls; user namespace maps to unprivileged | **Addressed** |
| **6.1** System file integrity | Immutable root filesystem (ext4 mounted read-only) | **Addressed** |

### A.3 NIST 800-53 Mapping (Selected Controls)

| NIST Control | Description | KruxOS Implementation |
|-------------|-------------|----------------------|
| **AC-2** Account management | Agent identity in agents.db; create/revoke/rotate via CLI | Addressed |
| **AC-3** Access enforcement | Policy engine evaluates every capability call; 4-tier model | Addressed |
| **AC-6** Least privilege | Agents confined to workspace; capabilities are the only API; default-deny network | Addressed |
| **AC-17** Remote access | WebSocket with configurable TLS; admin passphrase for supervision port | Addressed |
| **AU-2** Audit events | Every capability invocation, policy decision, session event logged | Addressed |
| **AU-3** Audit content | Agent ID, session ID, capability, inputs (redacted), outputs, duration, policy decision | Addressed |
| **AU-9** Protection of audit information | Landlock blocks agent access to audit directory; hash chain detects tampering | Addressed |
| **AU-11** Audit retention | Configurable retention (default 90 days); automated rotation | Addressed |
| **IA-2** Identification and authentication | 256-bit API keys; SHA-256 hash storage; constant-time verification | Addressed |
| **IA-5** Authenticator management | Key rotation via `kruxos agent rotate`; revocation via `kruxos agent revoke` | Addressed |
| **SC-4** Information in shared resources | PID/mount/network namespace isolation; Landlock filesystem MAC; cgroup resource limits | Addressed |
| **SC-7** Boundary protection | 5-layer kernel sandbox; Gateway as single entry point; default-deny network | Addressed |
| **SC-12** Cryptographic key management | AES-256-GCM; Argon2id KDF; master key in memory only (zeroized on drop) | Addressed |
| **SC-13** Cryptographic protection | Vault encryption at rest; SHA-256 audit chain; TLS for external connections | Addressed |
| **SC-28** Protection of information at rest | Vault secrets encrypted with AES-256-GCM; backup files encrypted | Addressed |
| **SI-4** System monitoring | Real-time supervision WebSocket; health checks; resource metrics | Addressed |
| **SI-7** Software integrity | Immutable root filesystem; A/B partitions for safe updates; pack checksums | Partial |

---

*This document reflects the security architecture of KruxOS v0.0.1. It will be updated as the platform evolves. For the latest version, see https://docs.kruxos.com/security/.*
