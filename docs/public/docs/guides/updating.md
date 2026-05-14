# Updating KruxOS

By the end of this page, you'll know how to apply KruxOS updates safely with automatic rollback protection.

!!! info "v0.0.1 status of the update path"
    A/B partition layout, the inactive-slot write path, and the `kruxos migrate` data-portability flow all ship in v0.0.1. The **health-driven rollback automation** (post-reboot probe, automatic A/B fail-back) lands in **v0.0.2** — until then, the partition swap works but operators verify health manually before confirming the new slot.

## How updates work

KruxOS uses **A/B partitions** for safe updates:

```mermaid
graph LR
    subgraph Disk
        ESP[ESP<br/>Boot]
        A[Root A<br/>Current]
        B[Root B<br/>Inactive]
        Data[/data<br/>Persistent]
    end

    Update[New Version] -->|Written to| B
    B -->|On reboot| A2[Root B<br/>Now Active]
    A -->|Kept as| A3[Root A<br/>Rollback]
```

1. The update is written to the **inactive** root partition
2. The system reboots into the new partition
3. A health monitor verifies the new version works
4. If health checks fail, the system **automatically rolls back** to the previous partition

Your data (`/data`) is on a separate partition and is never touched during updates.

## Check for updates

```bash
kruxos update check
```

Expected output:

```
Current version: 1.0.0
Latest version:  1.0.1

Changes in 1.0.1:
  - Fixed rate limiter edge case with concurrent agents
  - Improved Gmail sync performance
  - Added filesystem.copy capability

Run 'kruxos update apply' to install.
```

## Apply an update

```bash
kruxos update apply
```

Expected output:

```
Downloading kruxos-1.0.1.img... (450 MB)
  ████████████████████ 100%

Verifying signature... ✓ (Ed25519)
Writing to inactive partition (sda3)...
  ████████████████████ 100%

Update staged. The new version will activate on reboot.

Reboot now? [y/N]: y

Rebooting...
```

### Post-reboot health check

After reboot, KruxOS runs an automatic health monitor:

1. Polls `/health/ready` every 5 seconds
2. Waits up to 2 minutes for all services to report healthy
3. If healthy: confirms the update (marks the new partition as good)
4. If unhealthy: **automatically reboots into the previous partition**

You can check the update status after reboot:

```bash
kruxos update status
```

Expected output (success):

```
Boot slot:    B (sda3)
Version:      1.0.1
Boot status:  CONFIRMED
Previous:     1.0.0 (sda2, available for rollback)
```

Expected output (rolled back):

```
Boot slot:    A (sda2)
Version:      1.0.0
Boot status:  CONFIRMED (rolled back from 1.0.1)
Rollback reason: Health check timeout — gateway failed to start
```

## Manual rollback

If you need to roll back after the health check confirmed the update:

```bash
kruxos update rollback
```

This reboots into the previous partition.

## Service-level updates

Some updates don't require a reboot. These update individual services:

```bash
kruxos update apply --service-only
```

Service updates restart affected services without rebooting the OS.

## Docker updates

For Docker installations:

```bash
# Pull the latest image
docker pull altvale/kruxos:latest

# Stop the current container
docker stop kruxos && docker rm kruxos

# Start with the new image (data volume preserved)
docker run -d --name kruxos --privileged \
  -e KRUXOS_VAULT_PASSPHRASE='your-vault-passphrase' \
  -p 7800:7800 -p 7700:7700 -p 7701:7701 \
  -v kruxos-data:/data/kruxos \
  altvale/kruxos:latest
```

!!! tip "Always use a named volume"
    The `-v kruxos-data:/data/kruxos` flag ensures your agents, state, audit logs, and vault survive container recreations.

## Update signing

All updates are signed with Ed25519. The public key is embedded in the OS image. The update mechanism verifies the signature before writing to disk — unsigned or tampered updates are rejected.

## Next steps

- [Backup & Restore](backup-restore.md) — back up before updating
- [Monitoring](monitoring.md) — verify health after updates
- [Troubleshooting](troubleshooting.md) — common update issues
