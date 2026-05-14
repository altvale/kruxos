# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in KruxOS, please report it responsibly.

**Email:** security@altvale.com
**GitHub Security Advisories:** https://github.com/altvale/kruxos/security/advisories/new
**Machine-readable disclosure metadata:** [security.txt](.well-known/security.txt) (RFC 9116; mirrored at https://docs.kruxos.com/.well-known/security.txt)

**Do NOT** open a public GitHub issue for security vulnerabilities.

### What to include

- Description of the vulnerability
- Steps to reproduce
- Affected versions
- Potential impact assessment

### Response timeline

- **Acknowledgment:** within 48 hours
- **Initial assessment:** within 5 business days
- **Fix or mitigation:** depends on severity, targeting 30 days for critical issues

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.0.x   | Yes       |

## Security Architecture

KruxOS is designed with defense-in-depth for AI agent execution:

- **Agent isolation** — each agent runs in a sandboxed environment with restricted capabilities (Linux user / network namespaces, cgroup v2 limits, seccomp BPF allowlist, nftables defense-in-depth).
- **Policy enforcement** — deterministic YAML-based policy engine evaluates every capability invocation; no LLM in the policy path.
- **Secrets vault** — AES-256-GCM encrypted vault with use-not-read semantics; agents invoke capabilities that use secrets internally, raw values are never exposed.
- **Audit trail** — length-prefixed CBOR framing, hash-chained for tamper evidence; bounded ring-buffer with disk-full retry to prevent silent loss.
- **Approval flows** — configurable human-in-the-loop approval for sensitive operations.
- **Per-principal soft-delete trash** — destructive operations are recoverable for the configured retention window (168 h for User, 24 h for Agent; both configurable via per-policy `trash_retention_hours`).
- **Network isolation** — nftables firewall with default-deny and per-agent network policies.
- **Systemd hardening** — `ProtectSystem=strict`, narrow `ReadWritePaths=`, `NoNewPrivileges=true` on all services.

For a complete security analysis, see the [Security Whitepaper](docs/public/docs/security/whitepaper.md).

## Verifying Release Artifacts

Every KruxOS release artifact (`.vmdk`, `.qcow2`, `.img.gz`, `.box`) is cryptographically signed with a long-term cosign keypair. Verify before installing — especially if you obtained the artifact from a mirror or third-party source.

### One-time setup

Fetch the public key:

```bash
curl -fsSL https://kruxos.com/keys/cosign.pub -o cosign.pub
```

Or copy it from the release page (also attached as a release asset). The public key is:

```
-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEbO2on7fSKPZRtPMisWyu/YfHge0d
kpq6s3BdEL3KjT6LUg4lywXPqGgLESuoeZ8nE/J7nJpgue+vrlKHlKowiA==
-----END PUBLIC KEY-----
```

### Per-artifact verification

For each artifact you download, also download the matching `.sig` file from the same release. Then:

```bash
cosign verify-blob \
  --key cosign.pub \
  --signature kruxos-x86_64.vmdk.sig \
  kruxos-x86_64.vmdk
```

`Verified OK` means the file matches what was signed by the KruxOS release key. Any other output (or a verification error) means the file has been altered, the signature is wrong, or you're using the wrong public key.

### Optional: SHA256 integrity check

The release also includes a `SHA256SUMS` file listing the SHA-256 hash of every artifact. After downloading, run:

```bash
sha256sum -c SHA256SUMS --ignore-missing
```

This catches accidental download corruption. It does **not** authenticate provenance — only the cosign signature does that.

### Why key-based (not Sigstore keyless)

KruxOS releases use a long-term cosign keypair rather than Sigstore keyless signing. This means:

- No OAuth identity (email, GitHub login) is published to a public transparency log per release.
- Verifiers fetch one public key once and re-use it across every KruxOS release.
- No third-party signing infrastructure dependency at verification time.

If our release key is ever compromised or lost, we will publish a rotation notice here and at `kruxos.com/keys/` with a new key for subsequent releases. Older releases remain verifiable with the original key.

## Scope

The following are in scope for security reports:

- Gateway authentication and authorization bypass
- Policy engine bypass or non-deterministic behavior
- Vault encryption weaknesses or key exposure
- Sandbox escape or privilege escalation
- Audit log tampering or integrity bypass
- Cross-agent data leakage
- Service proxy credential exposure

The following are out of scope:

- Vulnerabilities in upstream dependencies (report to the upstream project)
- Issues requiring physical access to the host machine
- Social engineering attacks
- Denial of service against the local gateway (it's a local service)
