# System Capabilities

System information, health checks, metrics, and time.

## Overview

| Capability | Permission | Purpose |
|------------|:----------:|---------|
| [`system.info`](#systeminfo) | 🟢 Autonomous | Returns system-level information including OS version, uptime, agent count, and resource overview. |
| [`system.health`](#systemhealth) | 🟢 Autonomous | Returns a comprehensive health report for all KruxOS services, including resource usage and queue depths. |
| [`system.metrics`](#systemmetrics) | 🟢 Autonomous | Returns resource usage metrics for a configurable time window. |
| [`system.time`](#systemtime) | 🟢 Autonomous | Returns the current system time in multiple formats. |

## `system.info`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Returns system-level information including OS version, uptime, agent count, and resource overview.

### When to use

Use system.info to get an overview of the KruxOS instance: version, uptime, how many agents
are connected, and basic resource availability. Use system.health for detailed service status.
Use system.metrics for time-windowed resource measurements.

### Inputs

*No inputs required.*

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `os_version` | `String` | KruxOS version string. |
| `uptime_seconds` | `Integer` | System uptime in seconds. |
| `agent_count` | `Integer` | Number of currently connected agents. |
| `cpu_count` | `Integer` | Number of available CPU cores. |
| `memory_total_bytes` | `Integer` | Total system memory in bytes. |
| `memory_available_bytes` | `Integer` | Available system memory in bytes. |
| `disk_total_bytes` | `Integer` | Total disk space on the data partition in bytes. |
| `disk_available_bytes` | `Integer` | Available disk space on the data partition in bytes. |

### Common patterns

**Check system resource availability before a large operation**

1. `system.info() to check memory_available_bytes and disk_available_bytes`
2. `Proceed if sufficient resources are available`

### Errors

**`SystemInfoError`** — Failed to read system information.

- **retry**: Retry the operation.

**Tags:** `system` `info` `safe` `read`

---

## `system.health`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Returns a comprehensive health report for all KruxOS services, including resource usage and queue depths.

### When to use

Use system.health to diagnose problems or verify all services are running correctly.
Use system.info for a lighter overview. Use system.metrics for time-windowed measurements.

### Inputs

*No inputs required.*

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `overall_status` | `String` | Overall system health: 'healthy', 'degraded', or 'unhealthy'. |
| `services` | `Array` | Array of service status objects, each with: name, status ('healthy'/'degraded'/'unhealthy'), message. |
| `resource_usage` | `Object` | Current resource usage: {cpu_percent, memory_percent, disk_percent, open_files}. |
| `queue_depths` | `Object` | Queue depths: {approval_pending, alerts_pending, audit_buffer}. |

### Common patterns

**Diagnose a slow system**

1. `system.health() to check for degraded or unhealthy services`
2. `Check resource_usage for high cpu_percent or memory_percent`

### Errors

**`HealthCheckError`** — Failed to collect health information from one or more services.

- **retry**: Retry the operation. Some services may be temporarily unresponsive.

**Tags:** `system` `health` `diagnostics` `safe` `read`

---

## `system.metrics`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Returns resource usage metrics for a configurable time window.

### When to use

Use system.metrics to understand resource consumption trends over time.
Use system.health for current point-in-time status.
Use system.info for basic system overview.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `window` | `String` | No | `5m` | Time window for metrics: '1m', '5m', '15m', '1h', '24h'. Metrics are averaged over this window. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `window` | `String` | The time window used (echo of input). |
| `cpu` | `Object` | CPU metrics: {average_percent, peak_percent}. |
| `memory` | `Object` | Memory metrics: {average_bytes, peak_bytes, average_percent}. |
| `disk_io` | `Object` | Disk I/O metrics: {read_bytes, write_bytes}. |
| `network_io` | `Object` | Network I/O metrics: {rx_bytes, tx_bytes}. |

### Common patterns

**Monitor resource trends**

1. `system.metrics(window='5m') for recent metrics`
2. `system.metrics(window='1h') for hourly comparison`

### Errors

**`MetricsError`** — Failed to collect metrics.

- **retry**: Retry the operation.

**Tags:** `system` `metrics` `diagnostics` `safe` `read`

---

## `system.time`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Returns the current system time in multiple formats.

### When to use

Use system.time to get the current time for timestamps, scheduling calculations,
or displaying time to users. Returns UTC and optionally a configured timezone.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `timezone` | `String` | No | — | IANA timezone name (e.g. 'America/New_York', 'Europe/London'). If provided, includes local time in that timezone. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `utc` | `DateTime` | Current time in UTC (ISO 8601). |
| `unix_epoch` | `Integer` | Current time as Unix epoch seconds. |
| `unix_epoch_ms` | `Integer` | Current time as Unix epoch milliseconds. |
| `local` | `DateTime` | Current time in the requested timezone (ISO 8601). Null if no timezone specified. |
| `timezone` | `String` | The timezone used, or 'UTC' if none specified. |

### Common patterns

**Get current timestamp for logging**

1. `system.time() to get UTC timestamp`

### Errors

**`InvalidTimezone`** — The specified timezone name is not a valid IANA timezone.

- **fix_timezone**: Use a valid IANA timezone name like 'America/New_York' or 'Europe/London'.

**Tags:** `system` `time` `safe` `read`

---
