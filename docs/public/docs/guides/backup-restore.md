# Backup & Restore

By the end of this page, you'll know how to back up KruxOS data and restore from a backup.

In v0.0.1 the backup CLI surface is `kruxos state backup / restore / backups` — there is **no `kruxos backup` namespace**. Daily backups also run automatically via a systemd timer at 02:00 UTC, and audit-log rotation runs at 03:00 UTC with a 90-day default retention.

## What gets backed up

| Data | Location | Included |
|------|----------|----------|
| Agent state (persistent) | `/data/kruxos/agents/*/state.db` | Yes |
| Shared state | `/data/kruxos/shared/state.db` | Yes |
| Agent database | `/data/kruxos/agents.db` | Yes |
| Approval queue | `/data/kruxos/approval_queue.db` | Yes |
| Audit logs | `/data/kruxos/audit/` | Yes |
| Vault (encrypted secrets) | `/data/kruxos/vault.db` | Yes |
| Policy files | `/data/kruxos/policies/{system,org,agents/<name>}.yaml` | Yes |
| Configuration | `/data/kruxos/config.yaml` | Yes |
| Gmail read-replica | `/data/kruxos/proxy/gmail/sync.db` | No (re-syncs) |
| Session state | In-memory | No (ephemeral) |

## Create a backup

### Full backup

```bash
kruxos state backup --out /tmp/state-2026-05-11.tar.gz.enc
```

Expected output:

```
Creating backup...
  Agent state:     ✓ (3 agents, 2.4 MB)
  Shared state:    ✓ (128 KB)
  Agent database:  ✓ (45 KB)
  Approval queue:  ✓ (12 KB)
  Audit logs:      ✓ (15.2 MB)
  Vault:           ✓ (encrypted)
  Policies:        ✓ (4 files)
  Configuration:   ✓

Backup saved: /data/kruxos/backups/backup-2026-03-29T14-30-00.tar.gz.enc
Size: 18.1 MB (encrypted)
```

!!! info "Encryption"
    Backups are encrypted with AES-256-GCM using a key derived from the vault master key. You need the vault passphrase to restore.

### Scheduled backups

Daily backups run **automatically** on the appliance via a systemd timer at 02:00 UTC — see `systemctl list-timers '*kruxos*'`. No manual cron setup needed in v0.0.1.

If you want to add a second schedule (e.g., hourly increments), use the host's cron:

```bash
echo "0 * * * * /usr/local/bin/kruxos state backup --out /data/kruxos/backups/incr-$(date +%H).tar.gz.enc" | crontab -
```

### Backup to external storage

Copy the backup file to external storage:

```bash
# To a remote server
scp /data/kruxos/backups/backup-2026-03-29T14-30-00.tar.gz.enc user@backup-server:/backups/

# To cloud storage (example with rclone)
rclone copy /data/kruxos/backups/backup-2026-03-29T14-30-00.tar.gz.enc remote:kruxos-backups/
```

### Docker volume backup

For Docker installations, back up the entire data volume:

```bash
docker run --rm -v kruxos-data:/data/kruxos -v $(pwd):/backup alpine \
  tar czf /backup/kruxos-data-backup.tar.gz /data/kruxos
```

## Restore from backup

### Prerequisites

- A running KruxOS instance (fresh install or existing)
- The backup file
- The vault passphrase used when the backup was created

### Restore

```bash
kruxos state restore /path/to/backup-2026-03-29T14-30-00.tar.gz.enc
```

Use `kruxos state backups` to list the available backup files first.

You'll be prompted for the vault passphrase:

```
Enter vault passphrase: ********

Restoring from backup-2026-03-29T14-30-00.tar.gz.enc...
  Agent state:     ✓ (3 agents restored)
  Shared state:    ✓
  Agent database:  ✓ (3 agents)
  Approval queue:  ✓ (2 pending)
  Audit logs:      ✓ (15.2 MB, chain verified)
  Vault:           ✓ (secrets decrypted and re-encrypted)
  Policies:        ✓ (4 files)
  Configuration:   ✓

Restore complete. Restart services to apply:
  systemctl restart kruxos-gateway
```

### Verify after restore

```bash
# Check system status
kruxos status

# Verify agents
kruxos agent list

# Verify audit chain integrity
kruxos audit stats
```

## Backup retention

Backups accumulate in `/data/kruxos/backups/`. Clean up old backups:

```bash
# List backups
ls -la /data/kruxos/backups/

# Remove backups older than 30 days
find /data/kruxos/backups/ -name "*.tar.gz.enc" -mtime +30 -delete
```

## Next steps

- [Updating KruxOS](updating.md) — apply updates with automatic rollback
- [Monitoring](monitoring.md) — health checks and alerts
- [Troubleshooting](troubleshooting.md) — common issues and solutions
