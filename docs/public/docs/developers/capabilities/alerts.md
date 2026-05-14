# Alerts Capabilities

Send, list, and acknowledge supervisor alerts.

## Overview

| Capability | Permission | Purpose |
|------------|:----------:|---------|
| [`alerts.send`](#alertssend) | 🟢 Autonomous | Sends an alert to the supervisor with structured context. |
| [`alerts.list`](#alertslist) | 🟢 Autonomous | Lists alerts sent by the current agent, including their delivery status and any acknowledgements. |
| [`alerts.acknowledge`](#alertsacknowledge) | 🔵 Notify | Marks an alert as acknowledged. |

## `alerts.send`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Sends an alert to the supervisor with structured context. Alerts appear in the CLI and dashboard for human review.

### When to use

Use alerts.send to notify the supervisor about situations that need human attention:
errors that cannot be automatically resolved, unusual conditions, requests for expanded access,
or completion of important tasks. Do not send alerts for routine events — use audit logging instead.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `message` | `String` | Yes | — | Clear, concise alert message describing what happened and what action is needed. |
| `severity` | `String` | No | `info` | Alert severity: 'info' (informational), 'warning' (needs attention soon), 'critical' (needs immediate attention). |
| `context` | `Object` | No | — | Structured context to help the supervisor understand the alert. Include relevant data: file paths, error messages, attempted actions. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `alert_id` | `String` | Unique identifier for the alert. |
| `delivered_to` | `String` | Where the alert was delivered: 'file_queue' (community), 'dashboard' (enterprise). |
| `severity` | `String` | The severity level of the sent alert. |

### Side effects

- Creates an alert visible to the supervisor via CLI (kruxos alerts) and dashboard. *(not reversible)*

### Common patterns

**Report an error that needs human intervention**

1. `alerts.send(message='Failed to access external API after 3 retries', severity='warning', context={'url': 'https://api.example.com', 'last_error': 'connection timeout'})`

**Request expanded permissions**

1. `alerts.send(message='Need access to domain api.newservice.com for data sync', severity='info', context={'capability': 'network.http_request', 'domain': 'api.newservice.com'})`

### Errors

**`AlertError`** — Failed to send the alert.

- **retry**: Retry sending the alert.

**Tags:** `alerts` `supervision` `safe`

---

## `alerts.list`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Lists alerts sent by the current agent, including their delivery status and any acknowledgements.

### When to use

Use alerts.list to check if previous alerts have been acknowledged by the supervisor.
Use alerts.send to create new alerts.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `status` | `String` | No | — | Filter by status: 'pending' (unacknowledged), 'acknowledged', or 'all'. Default 'all'. |
| `limit` | `Integer` | No | `50` | Maximum number of alerts to return. Most recent first. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `alerts` | `Array` | Array of alert objects, each with: alert_id, message, severity, status, created_at, acknowledged_at, acknowledged_by. |
| `total` | `Integer` | Total number of matching alerts. |

### Common patterns

**Check if a previous alert was acknowledged**

1. `alerts.list(status='pending')`
2. `If the alert you are waiting on is no longer pending, the supervisor has responded`

### Errors

**`AlertError`** — Failed to query alerts.

- **retry**: Retry the query.

**Tags:** `alerts` `supervision` `safe` `read`

---

## `alerts.acknowledge`

**Permission:** 🔵 Notify · **Version:** 1.0

> Marks an alert as acknowledged. Typically called by the supervisor via CLI or dashboard, but agents can acknowledge their own informational alerts.

### When to use

Use alerts.acknowledge to mark an alert as handled after taking corrective action.
Primarily used by supervisors, but agents may acknowledge their own info-level alerts.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `alert_id` | `String` | Yes | — | The unique identifier of the alert to acknowledge. |
| `acknowledged_by` | `String` | No | — | Name of the person or agent acknowledging the alert. Defaults to the calling agent's name. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `acknowledged` | `Boolean` | True if the alert was found and acknowledged. False if the alert_id does not exist or was already acknowledged. |
| `alert_id` | `String` | The alert_id that was acknowledged (echo of input). |

### Side effects

- Updates the alert status to 'acknowledged' with a timestamp. *(not reversible)*

### Common patterns

**Acknowledge a resolved alert**

1. `alerts.list(status='pending') to find pending alerts`
2. `alerts.acknowledge(alert_id='...') after resolving the issue`

### Errors

**`AlertNotFound`** — No alert exists with the specified alert_id.

- **list_alerts**: Use alerts.list to see available alerts and their IDs.

**`AlertError`** — Failed to acknowledge the alert.

- **retry**: Retry the operation.

**Tags:** `alerts` `supervision` `acknowledge`

---
