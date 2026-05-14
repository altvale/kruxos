# Deployment Guide

Best practices for deploying KruxOS in production environments.

## Deployment options

| Method | Best for | Setup time |
|--------|----------|-----------|
| Docker | Quick evaluation, CI/CD environments | 5 minutes |
| ISO image (VM) | Dedicated agent infrastructure | 15 minutes |
| ISO image (bare metal) | Maximum performance, air-gapped environments | 30 minutes |

## Production checklist

### Before deployment

- [ ] Choose a policy template appropriate for your environment
- [ ] Prepare agent names and purposes for initial registration
- [ ] Decide on external service connections (Gmail, etc.)
- [ ] Plan backup strategy and retention period
- [ ] Configure network firewall rules for ports 7700, 7701, 7800

### System requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU | 2 cores | 4 cores |
| RAM | 2 GB | 4 GB |
| Disk | 20 GB | 50 GB |
| Network | 1 Mbps | 10 Mbps |
| OS | x86_64 Linux | KruxOS ISO or Docker |

### Network configuration

```
┌─────────────────────────────────────────────┐
│                  Firewall                    │
│                                              │
│  Allow inbound:                              │
│    7700/tcp  ← Agent connections (internal)  │
│    7800/tcp  ← Dashboard (admin network)     │
│                                              │
│  Block inbound:                              │
│    7701/tcp  ← Supervision (localhost only)  │
│    7702/tcp  ← OpenClaw bridge (if unused)   │
│                                              │
│  Allow outbound:                             │
│    443/tcp   → Gmail API, update server      │
│    53/udp    → DNS                           │
└─────────────────────────────────────────────┘
```

!!! warning "Supervision port"
    Port 7701 should **never** be exposed to untrusted networks. It provides session control (kill, pause) and live activity streaming. Restrict it to localhost or a management VLAN.

### TLS configuration

The dashboard serves HTTPS with a self-signed certificate by default. For production:

**Option 1: Let's Encrypt (recommended for public-facing)**

Configure in `/etc/kruxos/config.yaml`:

```yaml
dashboard:
  tls:
    mode: letsencrypt
    domain: agents.example.com
    email: admin@example.com
```

**Option 2: Reverse proxy**

Place nginx or Caddy in front of KruxOS:

```nginx
server {
    listen 443 ssl;
    server_name agents.example.com;
    ssl_certificate /etc/ssl/certs/agents.pem;
    ssl_certificate_key /etc/ssl/private/agents.key;

    # Dashboard
    location / {
        proxy_pass http://localhost:7800;
        proxy_set_header Host $host;
    }

    # Agent WebSocket
    location /ws {
        proxy_pass http://localhost:7700;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

## Policy configuration

### Choose a starting policy

| Environment | Recommended template | Philosophy |
|------------|---------------------|------------|
| Development / testing | `personal-permissive` | Maximum agent autonomy |
| Team use | `team-moderate` | Reads auto, writes notify, destructive = approval |
| Production | `enterprise-restrictive` | All writes need approval |

### Customize for your needs

Start with a template and customize. See the [Policies guide](../guides/policies.md) for full YAML syntax.

Key customizations for production:

```yaml
# Rate limit email sending
email.send:
  tier: notify
  rate_limit:
    max: 20
    window: 3600
    on_exceed: approval_required

# Block network access by default
network.*:
  tier: blocked

# Allow specific network capabilities
network.http_request:
  tier: approval_required
  reason: "External HTTP requests require review"
```

## Backup strategy

### Automated daily backups

Daily backups run **automatically** on the appliance via a systemd timer at 02:00 UTC out of the box — no setup needed. Verify with `systemctl list-timers '*kruxos*'`. Audit-log rotation runs at 03:00 UTC with a 90-day default retention.

To add a second schedule (e.g., hourly incrementals), use the host's cron:

```bash
0 * * * * /usr/local/bin/kruxos state backup --out /data/kruxos/backups/incr-$(date +\%H).tar.gz.enc
```

### Offsite backup

Copy backups to external storage:

```bash
# Example: sync to S3-compatible storage
0 3 * * * rclone copy /data/kruxos/backups/ remote:kruxos-backups/ --max-age 1d
```

### Retention

Configure backup retention in your backup rotation script. Recommended:

- Daily backups: keep 7 days
- Weekly backups: keep 4 weeks
- Monthly backups: keep 12 months

## Monitoring integration

### Health check endpoint

The `/health` endpoint on port 7701 returns HTTP 200 (healthy) or 503 (unhealthy):

```bash
# Nagios / monitoring check
curl -sf http://localhost:7701/health || echo "CRITICAL: KruxOS unhealthy"
```

### Alerting

Configure external alerting by polling the health endpoint or subscribing to the supervision WebSocket for real-time events.

## Update strategy

### Recommended process

1. **Back up** before updating: `kruxos state backup --out /data/kruxos/backups/pre-update.tar.gz.enc`
2. **Check** for updates: `kruxos update check`
3. **Apply** during maintenance window: `kruxos update apply`
4. **Verify** after reboot: `kruxos status` and `kruxos audit stats`
5. **Rollback** if issues: `kruxos update rollback`

!!! info "Health-driven A/B rollback ships in v0.0.2"
    The A/B partition layout and inactive-slot write path are in place in v0.0.1, but the **automated post-reboot health probe** that confirms the new slot (or fails back to the previous one) lands in **v0.0.2**. Until then, run step 4 manually before the boot flag is confirmed.

The A/B partition system ensures that failed updates automatically roll back. See [Updating](../guides/updating.md) for details.

## Scaling considerations

### v0.0.x (single-node)

The v0.0.x line is designed for single-node deployment. Practical limits:

| Metric | Tested capacity |
|--------|----------------|
| Concurrent agents | 50+ |
| Capability invocations/sec | 1,000+ |
| Audit entries/day | 1,000,000+ |
| State entries per agent | 10,000+ |

### Multi-node (future)

Multi-node clustering, PostgreSQL backend, and horizontal scaling are planned for a later v0.0.x release and are available under enterprise contracts. Contact [sales@altvale.com](mailto:sales@altvale.com).
