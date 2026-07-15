# Updating KruxOS

By the end of this page, you'll know how to apply KruxOS updates safely with automatic rollback protection.

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

## Offline (air-gapped) update

If your appliance has no internet access, you can update it from files you
download on another device. The update goes through the exact same signed
A/B path as an online update — the image's Ed25519 signature is verified
against the appliance's trusted keys before anything is written, and
verification cannot be skipped.

1. On a device with internet access, download **both** files from the
   KruxOS release, unmodified and unrenamed:
    - `kruxos-<arch>-rootfs.img.gz` — the update image (this is the
      **rootfs** update artifact, *not* the full-disk installer image used
      for the initial flash)
    - `kruxos-<arch>-rootfs.img.gz.sig` — its signature
2. Open the dashboard at **Settings › Updates** and use the
   **Offline update — upload files** card to upload both files.
3. In the **Offline update — apply** card, select the uploaded image. A
   badge shows whether its signature is present — Apply stays disabled
   until the matching `.sig` is uploaded.
4. Click **Apply selected** and confirm. The signature is verified, the
   image is written to the inactive partition (30–60 seconds), and the
   Reboot card appears — reboot to activate, with the same automatic
   post-reboot health check and rollback protection as an online update.

If the image is unsigned, tampered with, or signed by a key the appliance
does not trust, the apply is rejected with an explicit error and nothing is
written to disk.

You can also apply an uploaded image from the appliance console:

```bash
kruxos update apply /data/kruxos/uploads/kruxos-<arch>-rootfs.img.gz
```

!!! warning "Skipping versions on an air-gapped appliance"
    An offline update cannot consult the online release feed, so it cannot
    warn you when a release requires stepping through an intermediate
    version first. Before skipping versions, check the release notes of the
    version you are installing: if they state a minimum required version,
    update to that intermediate release first.

!!! note "Free space"
    Applying decompresses the image next to the uploaded file, so the
    `/data` partition temporarily needs free space roughly equal to the
    decompressed image size. The uploaded files can be deleted from the
    **Uploads** page after the update is confirmed.

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
  -p 7800:7800 -p 7700:7700 \
  -v kruxos-data:/data/kruxos \
  altvale/kruxos:latest
```

!!! tip "Always use a named volume"
    The `-v kruxos-data:/data/kruxos` flag ensures your agents, state, audit logs, and vault survive container recreations.

## Update signing

All updates are signed with Ed25519. The public key is embedded in the OS image. The update mechanism verifies the signature before writing to disk — unsigned or tampered updates are rejected. The signing key can be rotated transparently when needed: a new public key travels inside an update signed by the current key, so the appliance learns to trust it automatically and operators don't need to do anything.

## Next steps

- [Backup & Restore](backup-restore.md) — back up before updating
- [Monitoring](monitoring.md) — verify health after updates
- [Troubleshooting](troubleshooting.md) — common update issues
