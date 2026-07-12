# Monitoring

By the end of this page, you'll know how to monitor system health, view metrics, and configure alerts.

## Health checks

### HTTP endpoint

KruxOS exposes a health endpoint on port 7704 (bound to loopback):

```bash
curl -s http://localhost:7704/health | python3 -m json.tool
```

Expected output:

```json
{
    "status": "degraded",
    "services": [
        {
            "name": "sandbox-confinement",
            "status": "healthy",
            "latency_ms": 0,
            "details": "Landlock enforced at kernel ABI ceiling",
            "checked_at": "2026-07-11T14:03:21.482Z"
        },
        {
            "name": "inference-engine",
            "status": "degraded",
            "latency_ms": 3,
            "details": "engine configured but socket not responding",
            "checked_at": "2026-07-11T14:03:21.485Z"
        }
    ],
    "resources": {
        "cpu_percent": 12.5,
        "memory_used_bytes": 268435456,
        "memory_total_bytes": 2147483648,
        "memory_percent": 12.5,
        "disk_used_bytes": 128027547648,
        "disk_total_bytes": 512110190592,
        "disk_percent": 25.0,
        "load_average_1m": 0.42,
        "load_average_5m": 0.37,
        "load_average_15m": 0.29,
        "collected_at": "2026-07-11T14:03:21.485Z"
    },
    "agent_metrics": {
        "active_sessions": 2,
        "total_invocations": 1487,
        "approval_queue_depth": 0,
        "dead_letter_queue_depth": 0,
        "comms_queue_depth": 1,
        "collected_at": "2026-07-11T14:03:21.485Z"
    },
    "generated_at": "2026-07-11T14:03:21.485Z",
    "total_latency_ms": 4
}
```

`services` is an **array** — one object per registered service check (`name`, `status`, `latency_ms`, `details`, `checked_at`). The top-level `status` is the worst status across all services.

Health status values:

| Status | Meaning |
|--------|---------|
| `healthy` | All services operating normally |
| `degraded` | Some services have issues but the system is functional |
| `critical` | Critical services are down |

`GET /health/ready` returns a minimal readiness body instead — `{"ready": true, "status": "healthy"}` (or `"ready": false` / `"status": "critical"` when not ready).

### CLI health check

```bash
kruxos status
```

Expected output:

```
KruxOS Status

  Gateway:             running (port 7700)
  Health:              healthy

Services
  sandbox-confinement  healthy      <1ms  Landlock enforced (kernel ABI v4)
  inference-engine     healthy      <1ms  on-appliance inference not installed (not configured)

Resources
  CPU:                 12.3%
  Memory:              7.5 GiB / 14.9 GiB (50.3%)
  Disk:                111.8 GiB / 465.7 GiB (24.0%)

Agents
  Total:               0
  Active:              0
  Revoked:             0

Approval Queue
  Pending:             0
  Approved today:      0
  Rejected today:      0
```

The **Services** list shows the gateway's registered health checks — currently **sandbox confinement** and the **on-appliance inference engine** (a not-installed inference engine is the expected opt-out default and still reports `healthy`). Each line reads `healthy`, `degraded`, or `critical`, and the top **Health** line reflects the worst of them; on an interactive terminal the status words and the CPU / memory / disk percentages are colour-coded green / yellow / red (percentages turn yellow at or above 60% and red at or above 80%). The **Services** and **Resources** blocks come from the health endpoint, so they are omitted when the gateway is unreachable — the **Gateway**, **Health**, **Agents**, and **Approval Queue** sections still render from local data.

### Dashboard

The **Health** page at `https://localhost:7800/health` auto-refreshes every 15 seconds and surfaces:

- **Status banner** at the top (Healthy / Degraded / Critical / Unknown) with the issue count, generation timestamp, and total-latency right rail.
- **Services table** — one row per backend service (gateway / vault / proxy / audit / state) with a status dot, latency cell colour-coded by threshold, details column, and last-checked column.
- **Resources grid** — Memory / CPU / Disk cards with progress bars that change colour at the 60 % and 80 % thresholds.
- **Agent metrics grid** — active agents, total sessions (lifetime cumulative — see the tooltip on the Total cell), invocations per minute, and error rate.

If the gateway is unreachable, the page renders an explicit error banner ("Can't reach gateway: …. Retrying …") with a **Retry now** button so a transient failure no longer presents as a silent blank page.

## Activity and audit from the dashboard

Two dashboard pages share the same event-row chrome but answer different questions: **Activity** is the live feed, **Audit** is the forensic query surface. Both render each entry with a status dot · time · actor · capability · policy-tier chip · duration · expand chevron. Expanding a row shows the result summary, a key-value grid for the request, and copy-to-clipboard handles for the `entry_hash` and `log_file`.

### Activity — live feed (`/activity`)

A live-updating feed driven by Server-Sent Events from `/api/activity/stream`. New entries stream in at the top, capped at the 200 most recent.

- **Live indicator pill** in the top-right shows the stream state — **Live**, **Paused**, or **Disconnected**. Click to pause (closes the SSE connection); click again to resume (reopens it). A warning banner appears across the top if the connection drops mid-session.
- **Filter bar** — substring search, plus dropdowns for **Agent**, **Status**, and a dedicated **Capability** input (e.g. `shell.exec`). Filters apply to the server-side query; the substring search additionally narrows the visible 200-entry buffer.

### Audit — forensic query (`/audit`)

A point-in-time query against the hash-chained audit log at `/api/audit`. Default range is the last 7 days; results paginate with a configurable page size (**25 / 50 / 100 / 200**) and a "Showing N–M of T" summary at the top.

- **Actor filter** is a Principal-tagged dropdown — selecting **User** filters to operator-initiated entries (`actor_type=user`); selecting an agent name filters to that agent's entries (`agent_name=<name>`).
- **Capability** text input + **Status** dropdown + **From / To** date pickers stack alongside the actor filter.
- **Clear filters** resets every filter at once, including the actor selection.
- **Export JSON** downloads the current filtered result set as a JSON file (the export honours the active filters, not the page window).

The Audit page's URL parameters mirror the filter state, so any view is bookmarkable and shareable.

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
    message="Deployment failed: tests failed on commit abc1234. Manual review needed.",
    context={"commit": "abc1234", "stage": "tests"},
)
```

`alerts.send` takes `message` (required), `severity` (`info` / `warning` / `critical`, default `info`), and an optional structured `context` object.

### Viewing alerts

```bash
# Show active alerts — live resource alerts plus any alerts agents have raised
kruxos alerts

# Machine-readable output (for scripting)
kruxos alerts --json
```

`kruxos alerts` groups its output into **Resource Alerts** (disk-usage and CPU-load conditions checked live) and **Agent Alerts** (raised via `alerts.send`, newest first — each with its severity, message, structured context, and whether it's been acknowledged).

On the dashboard, the **Alerts** page (`/alerts`) lists every alert and lets you acknowledge it once handled; its sidebar entry carries a count badge, and critical alerts also raise a banner on every page — so an alert an agent sends reaches you wherever you are. See [Web Dashboard → Alerts](../quickstart/dashboard.md#alerts-alerts) for the full breakdown.

### Alert deduplication

KruxOS deduplicates identical alerts. If the same condition triggers repeatedly, you'll see one alert with a count and the time range, not a flood of notifications.

## Monitoring the Service Proxy

### Gmail sync status

To check whether Gmail (or Slack) is connected and its token healthy:

```bash
kruxos connect status
```

Example output:

```
● Gmail: connected (you@example.com)
    token expires: 2026-07-12T14:03:21Z
○ Slack: not connected — run `kruxos connect slack`
```

If a connection needs re-authorising, the line shows **needs attention — reconnect recommended**. For detailed sync status — last sync time, buffered writes, dead letters, and errors — use the dashboard **Service Proxy** page described below.

On the dashboard, navigate to **Service Proxy** at `/proxy` for detailed sync status, write buffer contents, and error history. The page auto-refreshes every 10 seconds and renders a five-cell overview strip at the top — **Total services**, **Synced**, **With errors**, **Buffered operations**, and **Dead letters** — so the dead-letter count is now visible at a glance instead of buried inside each per-service card.

Below the overview strip, each service card shows sync status, last-started / last-completed timestamps, buffered-write and dead-letter counts, and lists of pending writes with **Cancel** (for buffered) and **Retry** / **Discard** (for dead letters). When a service's sync is failing, the card also shows how many consecutive failures it has seen and the last sync error — so a missing scope, an expired token, or a network problem surfaces directly on the card instead of leaving the service stuck on "Never / Unknown" with no explanation. All three actions go through a confirm modal.

## External monitoring integration

### Health endpoint for load balancers

The `/health` endpoint returns HTTP 200 when the overall status is `healthy` or `degraded`, and HTTP 503 when it is `critical`. Use this for:

- Load balancer health checks
- Kubernetes liveness/readiness probes
- Uptime monitoring services

### Prometheus-compatible metrics

KruxOS exposes metrics in a format suitable for collection:

```bash
curl -s http://localhost:7704/health | python3 -c "
import json, sys
data = json.load(sys.stdin)
print(f'kruxos_cpu_percent {data[\"resources\"][\"cpu_percent\"]}')
print(f'kruxos_memory_used_bytes {data[\"resources\"][\"memory_used_bytes\"]}')
print(f'kruxos_status {{status=\"{data[\"status\"]}\"}} 1')
"
```

## Next steps

- [Backup & Restore](backup-restore.md) — protect your data
- [Updating KruxOS](updating.md) — apply updates safely
- [Troubleshooting](troubleshooting.md) — common issues and solutions
