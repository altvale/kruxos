# Compliance

KruxOS is designed with compliance requirements in mind. This page maps KruxOS capabilities to common compliance frameworks.

## SOC 2 readiness

SOC 2 Type II evaluates controls across five Trust Services Criteria. Here is how KruxOS addresses each:

### Security

| Control | KruxOS Implementation |
|---------|----------------------|
| Access control | Per-agent API key authentication, admin passphrase for supervision |
| Least privilege | 4-tier policy engine (autonomous → blocked), per-agent policy overrides |
| Network segmentation | Per-agent nftables rules, default-deny egress |
| Encryption at rest | AES-256-GCM vault, encrypted backups |
| Encryption in transit | HTTPS dashboard, WSS optional for agent connections |
| Vulnerability management | Immutable root filesystem, A/B partition updates with rollback |

### Availability

| Control | KruxOS Implementation |
|---------|----------------------|
| System monitoring | Health endpoint (/health), automatic alerts, resource metrics |
| Incident response | Real-time activity stream, audit log replay, session pause/kill |
| Backup and recovery | Encrypted backups, automated restore, A/B partition rollback |
| Capacity management | Per-agent cgroup resource limits, state quotas |

### Processing integrity

| Control | KruxOS Implementation |
|---------|----------------------|
| Input validation | Schema validation on all capability inputs (type, range, required) |
| Error handling | Structured errors with typed codes, descriptions, and recovery actions |
| Transaction integrity | Atomic multi-operation transactions with commit/rollback |
| Audit trail | Hash-chained append-only logs with tamper detection |

### Confidentiality

| Control | KruxOS Implementation |
|---------|----------------------|
| Data classification | Secrets vault with capability-scoped access |
| Access restriction | Use-not-read model — agents never see raw secret values |
| Data disposal | Configurable audit retention, secure vault key zeroization |
| Encryption | AES-256-GCM for secrets, Argon2id KDF for passphrase |

### Privacy

| Control | KruxOS Implementation |
|---------|----------------------|
| Data minimization | Audit log secret redaction before write |
| Access logging | Every capability invocation logged with agent identity |
| Consent management | Service Proxy write buffer with cancellation window |

## ISO 27001 alignment

KruxOS supports ISO 27001 Annex A controls in these areas:

| Control area | Relevant KruxOS features |
|-------------|--------------------------|
| A.5 Information security policies | YAML policy files, policy hierarchy, version-controlled |
| A.6 Organization of information security | Role separation (agent vs admin), supervision port isolation |
| A.8 Asset management | Agent database, capability registry, pack manifest |
| A.9 Access control | Policy engine, API key authentication, vault scoping |
| A.10 Cryptography | AES-256-GCM vault, Argon2id KDF, Ed25519 update signing |
| A.12 Operations security | Audit logging, health monitoring, change management via A/B updates |
| A.13 Communications security | Per-agent network policy, default-deny egress |
| A.14 System development | Immutable root filesystem, signed updates |
| A.16 Incident management | Activity stream, audit replay, session control |
| A.18 Compliance | Audit export, retention policies, hash chain verification |

## Audit capabilities for compliance

### Export and retention

```bash
# Export audit logs for a time range
kruxos audit query --from 2026-01-01 --to 2026-03-31 --format json > q1-audit.json

# Verify hash chain integrity
kruxos audit stats
# Output: Hash chain: verified ✓ (142,847 entries across 90 files)

# Configure retention
# In /etc/kruxos/config.yaml:
# audit:
#   retention_days: 365
```

### Evidence for auditors

| Auditor request | KruxOS command |
|----------------|----------------|
| All actions by agent X | `kruxos audit query --agent X --format json` |
| All policy denials | `kruxos audit query --outcome denied --format json` |
| All approval decisions | `kruxos audit query --capability '*.approval' --format json` |
| System integrity proof | `kruxos audit stats` (hash chain verification) |
| Access control configuration | `kruxos config show policy` |
| Encryption configuration | `kruxos vault list` (shows algorithm, no raw values) |

## Compliance gaps (v0.0.1)

These areas are not fully addressed in v0.0.1 and are available under enterprise contracts or planned for later v0.0.x releases:

| Gap | Status | Plan |
|-----|--------|------|
| Multi-factor authentication | Not implemented | Enterprise contract / post-v0.0.x |
| RBAC (role-based access control) | Agent-level only | Enterprise contract / post-v0.0.x |
| SIEM integration | Manual export | Enterprise contract / post-v0.0.x |
| SSO / SAML / OIDC | Not implemented | Enterprise contract / post-v0.0.x |
| Data residency controls | Single-node only | Enterprise contract / post-v0.0.x |
| Automated compliance reporting | Manual | Enterprise contract / post-v0.0.x |

!!! info
    KruxOS v0.0.1 provides the foundational security controls. Enterprise contracts add the management and reporting layers that large organizations need for formal compliance programs — contact [sales@altvale.com](mailto:sales@altvale.com).
