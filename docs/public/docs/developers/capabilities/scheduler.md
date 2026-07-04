# Scheduler Capabilities

Create, list, and delete recurring cron-scheduled capability invocations, plus one-shot delayed invocations. **4 capabilities in v0.0.1.**

## Overview

| Capability | Permission | Purpose |
|------------|:----------:|---------|
| [`scheduler.cron_create`](#schedulercron_create) | 🔵 Notify | Schedules a recurring capability invocation using a five-field cron expression. |
| [`scheduler.cron_list`](#schedulercron_list) | 🟢 Autonomous | Lists all scheduled tasks for the current agent. |
| [`scheduler.cron_delete`](#schedulercron_delete) | 🔵 Notify | Removes a scheduled task by its identifier. |
| [`scheduler.delay`](#schedulerdelay) | 🔵 Notify | Schedules a one-shot delayed capability invocation that fires once after the specified delay and is then auto-removed. |

!!! info "scheduler.delay schema"
    The full input/output schema for `scheduler.delay` is emitted via MCP `tools/list` / JSON-RPC `capabilities.list`; the YAML source of truth lives at [`definitions/scheduler.yaml`](https://github.com/altvale/kruxos/blob/main/definitions/scheduler.yaml).

## `scheduler.cron_create`

**Permission:** 🔵 Notify · **Version:** 1.0

> Schedules a recurring capability invocation using a cron expression. The capability will be invoked automatically on the defined schedule.

### When to use

Use scheduler.cron_create to automate periodic tasks such as backups, health checks,
data syncing, or cleanup operations. The cron expression follows standard 5-field format:
minute hour day-of-month month day-of-week.
Use scheduler.cron_list to view existing scheduled tasks.
Use scheduler.cron_delete to remove a scheduled task.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `capability` | `String` | Yes | — | The capability to invoke on each scheduled run (e.g. 'filesystem.list', 'system.health'). |
| `inputs` | `Object` | No | — | Inputs to pass to the capability on each invocation. Defaults to empty object. |
| `schedule` | `CronExpression` | Yes | — | Cron expression defining the schedule. Standard 5-field format: 'minute hour day-of-month month day-of-week'. Examples: '0 * * * *' (hourly), '*/5 * * * *' (every 5 min), '0 2 * * *' (daily at 2am). |
| `description` | `String` | No | — | Human-readable description of what this scheduled task does. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `cron_id` | `String` | Unique identifier for the scheduled task. Use this to delete or reference it. |
| `schedule` | `CronExpression` | The cron expression that was set (echo of input). |
| `next_run` | `DateTime` | ISO 8601 timestamp of the next scheduled execution. |
| `capability` | `String` | The capability that will be invoked (echo of input). |

### Side effects

- Creates a persistent scheduled task that will invoke the specified capability on each trigger. Tasks survive restarts. *(reversible)*

### Common patterns

**Schedule hourly health checks**

1. `scheduler.cron_create(capability='system.health', schedule='0 * * * *', description='Hourly health check')`

**Schedule daily backup**

1. `scheduler.cron_create(capability='system.info', schedule='0 2 * * *', description='Daily system snapshot at 2am')`

### Errors

**`InvalidCronExpression`** — The cron expression is malformed or could not be parsed.

- **fix_expression**: Use standard 5-field cron format: minute(0-59) hour(0-23) day(1-31) month(1-12) weekday(0-6). Use * for any, */N for every N, and comma-separated lists.

**`CapabilityNotFound`** — The specified capability does not exist.

- **list_capabilities**: Use agent.capabilities to see available capabilities.

**`SchedulerError`** — Internal scheduler error.

- **retry**: Retry the operation.

**Tags:** `scheduler` `cron` `automation`

---

## `scheduler.cron_list`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Lists all scheduled tasks for the current agent.

### When to use

Use scheduler.cron_list to view your currently scheduled tasks, their schedules,
and when they will next run. Use scheduler.cron_delete to remove a task.

### Inputs

*No inputs required.*

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `jobs` | `Array` | Array of scheduled job objects, each with: cron_id, capability, inputs, schedule, next_run, description, enabled, created_at. |
| `total` | `Integer` | Total number of scheduled tasks. |

### Common patterns

**Check existing scheduled tasks**

1. `scheduler.cron_list()`
2. `Review jobs array for duplicates or stale schedules`

### Errors

**`SchedulerError`** — Internal scheduler error.

- **retry**: Retry the operation.

**Tags:** `scheduler` `cron` `safe` `read`

---

## `scheduler.cron_delete`

**Permission:** 🔵 Notify · **Version:** 1.0

> Removes a scheduled task by its identifier. The task will no longer execute.

### When to use

Use scheduler.cron_delete to stop a scheduled task from running.
Use scheduler.cron_list first to find the cron_id of the task to delete.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `cron_id` | `String` | Yes | — | The unique identifier of the scheduled task to delete. Obtain from scheduler.cron_list or scheduler.cron_create output. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `deleted` | `Boolean` | True if the task was found and deleted. False if the cron_id did not exist. |
| `cron_id` | `String` | The cron_id that was requested for deletion (echo of input). |

### Side effects

- Permanently removes the scheduled task. It will not execute again. *(not reversible)*

### Common patterns

**Remove a scheduled task**

1. `scheduler.cron_list() to find the cron_id`
2. `scheduler.cron_delete(cron_id='...') to remove it`

### Errors

**`SchedulerError`** — Internal scheduler error.

- **retry**: Retry the operation.

**Tags:** `scheduler` `cron` `delete`

---
