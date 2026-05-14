# Monitoring

By the end of this page, you'll know how to monitor system health, view metrics, and configure alerts.

## Health checks

### HTTP endpoint

KruxOS exposes a health endpoint on port 7701:

```bash
curl -s http://localhost:7701/health | python3 -m json.tool
```

Expected output:

```json
{
    "status": "healthy",
    "version": "1.0.0",
    "uptime_seconds": 14400,
    "services": {
        "gateway": "healthy",
        "vault": "healthy",
        "proxy": "healthy",
        "audit": "healthy",
        "state": "healthy"
    },
    "resources": {
        "cpu_percent": 12.5,
        "memory_used_mb": 256,
        "memory_total_mb": 2048,
        "disk_used_percent": 34.2
    }
}
```

Health status values:

| Status | Meaning |
|--------|---------|
| `healthy` | All services operating normally |
| `degraded` | Some services have issues but the system is functional |
| `unhealthy` | Critical services are down |

### CLI health check

```bash
kruxos alerts
```

Expected output:

```
System Health: HEALTHY
━━━━━━━━━━━━━━━━━━━━━━━
  CPU:     12.5% (ok)
  Memory:  256 MB / 2048 MB (ok)
  Disk:    34.2% (ok)
  Gateway: running
  Vault:   unlocked
  Proxy:   syncing (last: 2m ago)
  Audit:   writing (chain: verified)
```

### Dashboard

The **Health** page at `http://localhost:7800/health` shows:

- Real-time CPU, memory, and disk graphs
- Per-service health status with history
- Active alerts
- Resource trend lines

## Metrics

### System metrics

Query system metrics via the CLI or SDK:

```bash
kruxos audit stats --last 24h
```

Agents can query metrics programmatically:

```python
# System-level metrics
result = await os.call_async("system.metrics", category="system")
# Returns: cpu_percent, memory_used_mb, disk_used_percent, uptime_seconds

# Agent-level metrics
result = await os.call_async("system.metrics", category="agents")
# Returns: active_count, total_sessions, invocations_per_minute

# Policy metrics
result = await os.call_async("system.metrics", category="policy")
# Returns: evaluations_total, denied_count, approval_pending_count

# HTTP metrics
result = await os.call_async("system.metrics", category="http")
# Returns: requests_total, latency_p50, latency_p99
```

## Alerts

### Automatic alerts

KruxOS automatically monitors for these conditions:

| Condition | Threshold | Alert |
|-----------|-----------|-------|
| High CPU | > 90% for 5 min | Warning |
| High memory | > 85% | Warning |
| Disk space | > 90% | Critical |
| Audit write failure | Any failure | Critical |
| Service down | Health check fail | Critical |
| Approval waiting | > 30 min | Info |
| Rate limit exceeded | Any agent | Warning |

### Agent-triggered alerts

Agents can send alerts to supervisors:

```python
await os.call_async(
    "alerts.send",
    severity="warning",
    title="Deployment failed",
    message="Tests failed on commit abc1234. Manual review needed.",
)
```

### Viewing alerts

```bash
# Recent alerts
kruxos alerts --last 24h

# Critical only
kruxos alerts --severity critical
```

On the dashboard, alerts appear as banners on every page and in detail on the Health page.

### Alert deduplication

KruxOS deduplicates identical alerts. If the same condition triggers repeatedly, you'll see one alert with a count and the time range, not a flood of notifications.

## Monitoring the Service Proxy

### Gmail sync status

```bash
kruxos status
```

The status output includes proxy health:

```
Proxy:      syncing (last: 2m ago)
  Gmail:    connected, 2347 messages synced
  Buffer:   0 pending writes
```

On the dashboard, navigate to **Service Proxy** for detailed sync status, write buffer contents, and error history.

## External monitoring integration

### Health endpoint for load balancers

The `/health` endpoint returns HTTP 200 when healthy and HTTP 503 when unhealthy. Use this for:

- Load balancer health checks
- Kubernetes liveness/readiness probes
- Uptime monitoring services

### Prometheus-compatible metrics

KruxOS exposes metrics in a format suitable for collection:

```bash
curl -s http://localhost:7701/health | python3 -c "
import json, sys
data = json.load(sys.stdin)
print(f'kruxos_cpu_percent {data[\"resources\"][\"cpu_percent\"]}')
print(f'kruxos_memory_used_mb {data[\"resources\"][\"memory_used_mb\"]}')
print(f'kruxos_status {{status=\"{data[\"status\"]}\"}} 1')
"
```

## Next steps

- [Backup & Restore](backup-restore.md) — protect your data
- [Updating KruxOS](updating.md) — apply updates safely
- [Troubleshooting](troubleshooting.md) — common issues and solutions
