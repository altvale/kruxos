# Architecture

## System overview

KruxOS is a layered system where every agent interaction follows a deterministic pipeline:

```mermaid
graph TB
    subgraph "Agents"
        Claude[Claude<br/>MCP native]
        GPT[GPT-4o<br/>Function calling]
        Gemini[Gemini<br/>Function declarations]
        Local[Ollama<br/>OpenAI-compatible]
    end

    subgraph "Layer 1: Gateway"
        GW[Agent Gateway<br/>Rust / Tokio async<br/>Port 7700]
        Auth[Authentication]
        Session[Session Management]
        GW --> Auth --> Session
    end

    subgraph "Layer 2: Capability Registry"
        Registry[47 Capabilities<br/>YAML definitions<br/>Schema validation]
    end

    subgraph "Layer 2a: Service Proxy"
        Sync[Sync Engine<br/>Read-replicas]
        Write[Write Proxy<br/>Buffer + cancel]
        Roll[Rollback Engine<br/>Point-in-time recovery]
    end

    subgraph "Layer 3: State System"
        SState[Session State<br/>In-memory]
        PState[Persistent State<br/>Per-agent SQLite]
        ShState[Shared State<br/>Cross-agent SQLite]
    end

    subgraph "Layer 4: Policy Engine"
        PE[YAML Rules<br/>Compiled evaluation tree<br/>< 1ms evaluation]
    end

    subgraph "Cross-cutting"
        Vault[Secrets Vault<br/>AES-256-GCM]
        Sandbox[Agent Sandbox<br/>5-layer kernel isolation]
        Audit[Audit Logs<br/>Hash-chained CBOR]
        Health[Health & Diagnostics<br/>HTTP /health endpoint]
        Comms[Agent Comms<br/>In-memory broker]
    end

    subgraph "Supervision"
        Dashboard[Web Dashboard<br/>Next.js + React<br/>Port 7800]
        CLI[kruxos CLI<br/>Rust / clap]
        Control[Control socket<br/>/run/kruxos/control.sock<br/>root-only · no TCP port]
    end

    subgraph "External Services"
        Gmail[Gmail API]
        Future[Future services...]
    end

    Claude & GPT & Gemini & Local --> GW
    Session --> PE
    Session --> Registry
    Registry --> Sync & Write & Roll
    Sync --> Gmail & Future
    Write --> Gmail & Future
    GW --> SState & PState & ShState
    GW --> Vault
    GW --> Sandbox
    GW --> Audit
    GW --> Health
    GW --> Comms
    Dashboard & CLI --> Control --> GW
```

## Request lifecycle

Every capability invocation follows this exact sequence:

```mermaid
sequenceDiagram
    participant Agent
    participant Gateway
    participant Policy
    participant Sandbox
    participant Registry
    participant Capability
    participant Vault
    participant Audit

    Agent->>Gateway: capabilities.call(name, inputs)
    Gateway->>Gateway: Authenticate (session lookup)
    Gateway->>Policy: evaluate(agent, capability, inputs)

    alt Blocked
        Policy-->>Gateway: Denied
        Gateway->>Audit: log(denied)
        Gateway-->>Agent: StructuredError(PolicyDenied)
    else Approval Required
        Policy-->>Gateway: ApprovalRequired
        Gateway->>Audit: log(pending)
        Gateway-->>Agent: StructuredError(ApprovalPending, request_id)
    else Autonomous / Notify
        Policy-->>Gateway: Allowed
        Gateway->>Sandbox: verify agent sandbox active
        Gateway->>Registry: dispatch(capability, inputs)
        Registry->>Capability: execute(inputs, secret_provider)
        Capability->>Vault: get_handle(secret_name)
        Vault-->>Capability: SecretHandle (opaque)
        Capability-->>Registry: CapabilityResponse
        Registry-->>Gateway: CapabilityResponse
        Gateway->>Audit: log(success, duration)
        Gateway-->>Agent: CapabilityResponse
    end
```

## Technology stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Gateway | Rust (tokio async) | Performance-critical hot path, memory safety |
| Registry | Rust + YAML definitions | Definitions as data, hot-reload without recompile |
| Policy Engine | Rust | Deterministic evaluation, < 1ms per decision |
| Sandbox | Rust + Linux kernel | Direct kernel API for minimal overhead |
| Vault | Rust | Security-critical, minimal attack surface |
| Audit | Rust | Append-only writes, hash chain computation |
| State System | SQLite (WAL mode) | Single-node, crash-safe, zero config |
| Service Proxy | Rust framework + Python adapters | Framework in Rust for safety, adapters in Python for flexibility |
| Agent Comms | Rust + Protocol Buffers | Low-latency in-memory message broker |
| Dashboard | Next.js 15 + TypeScript + Tailwind | Modern web stack, real-time via WebSocket |
| Agent SDK | Python 3.11+ | Primary AI agent ecosystem language |
| CLI | Rust (clap) | Single binary, fast startup, shell completions |

## Data storage

All persistent data lives under `/data/kruxos/`:

| Database | Engine | Purpose | Scope |
|----------|--------|---------|-------|
| `agents.db` | SQLite | Agent identity, metadata | Global |
| `agents/{name}/state.db` | SQLite | Per-agent persistent state | Per-agent |
| `shared/state.db` | SQLite | Cross-agent shared state | Global |
| `approval_queue.db` | SQLite | Pending approval requests | Global |
| `vault.db` | SQLite | Encrypted secrets | Global |
| `audit/audit-index.db` | SQLite | Audit log query index | Global |
| `audit/audit-*.log` | CBOR files | Raw audit entries (hash-chained) | Daily files |
| `proxy/{service}/sync.db` | SQLite | Service read-replicas | Per-service |
| `proxy/{service}/write_buffer.db` | SQLite | Buffered outbound writes | Per-service |

All SQLite databases use WAL (Write-Ahead Logging) mode for concurrent read performance.

## Capability categories

| Category | Count | Examples |
|----------|-------|---------|
| `filesystem.*` | 10 | read, write, list, move, delete, search, stat, mkdir, copy, watch |
| `process.*` | 5 | run, list, kill, wait, info |
| `network.*` | 4 | http_request, dns_lookup, port_check, download |
| `git.*` | 7 | log, diff, status, commit, branch, checkout, clone |
| `scheduler.*` | 3 | cron_create, cron_list, cron_delete |
| `alerts.*` | 3 | send, list, acknowledge |
| `system.*` | 4 | metrics, health, info, shutdown |
| `agent.*` | 4 | session, capabilities, briefing, whoami |
| `secrets.*` | 3 | list, use, rotate |
| `comms.*` | 4 | send, receive, subscribe, publish |
| **Total** | **47** | |

Each capability is defined in YAML with: purpose, when_to_use, typed inputs/outputs, side effects, common patterns, and error types.

## Deployment topology

### Single-node (v0.0.x)

```
┌─────────────────────────────────┐
│         KruxOS Instance         │
│                                  │
│  Gateway ──── Registry           │
│     │                            │
│  Policy ──── Sandbox             │
│     │                            │
│  Vault ──── Audit ──── State     │
│     │                            │
│  Dashboard ──── Proxy            │
│                                  │
│  SQLite for all persistence      │
└─────────────────────────────────┘
```

The v0.0.x line is single-node with SQLite. All services run on one machine. This is the right architecture for personal use, small teams, and initial enterprise evaluation.

### Multi-node (future)

Multi-node clustering with PostgreSQL, distributed audit collection, and centralized policy management is planned for a later v0.0.x release and is available under enterprise contracts. Contact [sales@altvale.com](mailto:sales@altvale.com).

## Port map

| Port | Protocol | Bind | Service | Access |
|------|----------|------|---------|--------|
| 7700 | WebSocket (MCP + JSON-RPC) | `0.0.0.0` | Agent Gateway | Per-Agent bearer token (64-char hex API key) |
| 7702 | UDP | `127.0.0.1` | Trigger-wake | Loopback only |
| 7703 | HTTP | `127.0.0.1` | User API (bearer-auth) | User tokens (`krx_user_*`) |
| 7704 | HTTP | `127.0.0.1` | Health endpoint (`/health`, `/health/ready`) | Loopback only |
| 7800 | HTTPS | `0.0.0.0` | Web Dashboard | Operator passphrase / User tokens |

**Supervision has no TCP port.** Live activity, chat, and vault control ride a root-only local control socket (`/run/kruxos/control.sock`, mode `0600 root:root`, gated on peer-credential uid 0) that the dashboard and CLI reach on-box — an agent cannot connect to it. The former supervision WebSocket (port 7701) is retired.

### Network posture

- **The agent gateway (7700) binds `0.0.0.0` by default** so an agent on another host can connect — a supported topology. The bind address is not the security boundary: every connection is authenticated by a per-Agent bearer token. On a single-box deployment where all agents are local, restrict it with `server.host: "127.0.0.1"`.
- **The User API (7703) and health endpoint (7704) bind loopback (`127.0.0.1`)** and are never network-exposed — the dashboard proxies the User API on-box.
- **The shipped appliance firewall opens only TCP 7700 (agent gateway) and TCP 7800 (dashboard).** SSH (TCP 22) is opt-in and disabled by default; the trigger-wake port (7702) is loopback UDP with no firewall rule; there is no supervision port.

### Reaching the gateway from a remote agent

Port 7700 speaks plain WebSocket (`ws://`) by default. To connect an agent from another host safely, front 7700 with a **TLS-terminating reverse proxy** (e.g. Caddy or nginx) that serves `wss://` and forwards to the local gateway — the per-Agent bearer token still authenticates every connection. Keep the gateway on loopback (`server.host: "127.0.0.1"`) for single-box deployments where no remote agent connects. Built-in TLS for the gateway is planned for a later release.

For the full network security posture, TLS defaults, and threat model, see the [Security Model](security-model.md).
