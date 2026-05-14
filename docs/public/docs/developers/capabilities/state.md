# State & Memory Capabilities

Session state, persistent state, shared cross-agent state, snapshots, briefings, and backups.

## Overview

| Capability | Permission | Purpose |
|------------|:----------:|---------|
| [`state.session.get`](#statesessionget) | 🟢 Autonomous | Retrieves a value from the current session's in-memory state by key. |
| [`state.session.set`](#statesessionset) | 🟢 Autonomous | Stores a key-value pair in the current session's in-memory state. |
| [`state.session.delete`](#statesessiondelete) | 🟢 Autonomous | Removes a key-value pair from the current session's in-memory state. |
| [`state.session.list`](#statesessionlist) | 🟢 Autonomous | Lists all keys in the current session's state, optionally filtered by prefix. |
| [`state.session.clear`](#statesessionclear) | 🟢 Autonomous | Removes all key-value pairs from the current session's state. |
| [`state.persistent.get`](#statepersistentget) | 🟢 Autonomous | Retrieves a value from the agent's persistent state by key. |
| [`state.persistent.set`](#statepersistentset) | 🟢 Autonomous | Stores a key-value pair in the agent's persistent state. |
| [`state.persistent.delete`](#statepersistentdelete) | 🟢 Autonomous | Removes a key and all its version history from the agent's persistent state. |
| [`state.persistent.list`](#statepersistentlist) | 🟢 Autonomous | Lists keys in the agent's persistent state with metadata summaries. |
| [`state.persistent.query`](#statepersistentquery) | 🟢 Autonomous | Queries persistent state by key prefix, returning full entries with values. |
| [`state.persistent.history`](#statepersistenthistory) | 🟢 Autonomous | Returns the version history of a specific key in persistent state. |
| [`state.shared.get`](#statesharedget) | 🟢 Autonomous | Retrieves a value from the cross-agent shared state by key. |
| [`state.shared.set`](#statesharedset) | 🔵 Notify | Stores a key-value pair in the cross-agent shared state with optimistic locking. |
| [`state.shared.delete`](#stateshareddelete) | 🔵 Notify | Removes a key from the cross-agent shared state. |
| [`state.shared.list`](#statesharedlist) | 🟢 Autonomous | Lists all keys in the cross-agent shared state with metadata. |
| [`state.shared.watch`](#statesharedwatch) | 🟢 Autonomous | Subscribes to change notifications for shared state keys. |
| [`state.snapshot.create`](#statesnapshotcreate) | 🔵 Notify | Creates a point-in-time snapshot of the entire system state. |
| [`state.snapshot.list`](#statesnapshotlist) | 🟢 Autonomous | Lists available system snapshots with timestamps and sizes. |
| [`state.snapshot.load`](#statesnapshotload) | 🟢 Autonomous | Loads a specific system snapshot by ID, returning the full captured state. |
| [`state.briefing.generate`](#statebriefinggenerate) | 🟢 Autonomous | Generates a context briefing summarising what changed since the agent's last activity. |
| [`state.backup.create`](#statebackupcreate) | 🟡 Approval Required | Creates an encrypted backup of all agent persistent state and shared state. |
| [`state.backup.list`](#statebackuplist) | 🟢 Autonomous | Lists available backups with timestamps, sizes, and scope. |
| [`state.backup.restore`](#statebackuprestore) | 🟡 Approval Required | Restores state from an encrypted backup. |
| [`state.backup.export`](#statebackupexport) | 🟡 Approval Required | Exports an encrypted backup to an external file path. |

## `state.session.get`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Retrieves a value from the current session's in-memory state by key.

### When to use

Use state.session.get to read temporary data stored earlier in the same session.
Session state is destroyed when the session ends. For data that must survive
across sessions, use state.persistent.get instead.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `key` | `String` | Yes | — | The key to look up. Keys are arbitrary strings. Use dotted namespaces (e.g. 'task.current') for organisation. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `value` | `Object` | The stored value, or null if the key does not exist. |
| `found` | `Boolean` | True if the key exists in session state. |

### Common patterns

**Check for cached computation result**

1. `state.session.get(key='analysis.result') to check if already computed`
2. `If found=false, perform computation and state.session.set the result`

**Read conversation context**

1. `state.session.get(key='context.last_action') to recall previous action`

### Errors

**`SessionNotFound`** — The session does not exist or has been terminated.

- **reconnect**: Establish a new session via the Gateway.

**Tags:** `state` `session` `read` `safe`

---

## `state.session.set`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Stores a key-value pair in the current session's in-memory state.

### When to use

Use state.session.set to store temporary data needed during this session.
Session state is destroyed on session end. Use state.persistent.set for
data that must survive across sessions.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `key` | `String` | Yes | — | The key to store under. Overwrites any existing value for the same key. |
| `value` | `Object` | Yes | — | Any JSON-serialisable value: string, number, boolean, object, or array. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `previous_value` | `Object` | The previous value if the key was overwritten, or null if new. |
| `overwritten` | `Boolean` | True if an existing key was overwritten. |

### Side effects

- Increases session memory usage. Session state is capped at a configurable limit (default 50 MB). *(not reversible)*

### Common patterns

**Cache a computation result for reuse within session**

1. `Perform expensive computation`
2. `state.session.set(key='analysis.result', value=result)`
3. `Later: state.session.get(key='analysis.result')`

### Errors

**`SessionNotFound`** — The session does not exist or has been terminated.

- **reconnect**: Establish a new session via the Gateway.

**`QuotaExceeded`** — Session state size limit exceeded.

- **delete_unused**: Use state.session.delete to remove unneeded keys.
- **clear**: Use state.session.clear to remove all session state.

**Tags:** `state` `session` `write`

---

## `state.session.delete`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Removes a key-value pair from the current session's in-memory state.

### When to use

Use state.session.delete to free memory by removing keys no longer needed.
Useful when session state approaches its quota limit.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `key` | `String` | Yes | — | The key to remove. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `previous_value` | `Object` | The value that was removed, or null if the key did not exist. |
| `deleted` | `Boolean` | True if the key existed and was removed. |

### Side effects

- Frees memory used by the removed key-value pair. *(not reversible)*

### Common patterns

**Clean up after finishing a task**

1. `state.session.delete(key='task.current')`
2. `state.session.delete(key='task.temp_data')`

### Errors

**`SessionNotFound`** — The session does not exist or has been terminated.

- **reconnect**: Establish a new session via the Gateway.

**Tags:** `state` `session` `write`

---

## `state.session.list`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Lists all keys in the current session's state, optionally filtered by prefix.

### When to use

Use state.session.list to discover what data is stored in the current session.
Use the prefix parameter to filter to a specific namespace.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `prefix` | `String` | No | — | If provided, only return keys starting with this prefix. For example, 'task.' returns all task-related keys. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `keys` | `Array` | List of matching keys, sorted alphabetically. |
| `count` | `Integer` | Number of keys returned. |

### Common patterns

**Discover all stored data**

1. `state.session.list() to see all keys`
2. `state.session.list(prefix='task.') to see only task-related keys`

### Errors

**`SessionNotFound`** — The session does not exist or has been terminated.

- **reconnect**: Establish a new session via the Gateway.

**Tags:** `state` `session` `read` `safe`

---

## `state.session.clear`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Removes all key-value pairs from the current session's state.

### When to use

Use state.session.clear to free all session memory at once.
This is irreversible within the session — all data is lost.

### Inputs

*No inputs required.*

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `removed_count` | `Integer` | Number of keys that were removed. |

### Side effects

- All session state is permanently deleted. *(not reversible)*

### Common patterns

**Reset session state for a fresh start**

1. `state.session.clear() to remove all session data`

### Errors

**`SessionNotFound`** — The session does not exist or has been terminated.

- **reconnect**: Establish a new session via the Gateway.

**Tags:** `state` `session` `write`

---

## `state.persistent.get`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Retrieves a value from the agent's persistent state by key. Persistent state survives across sessions.

### When to use

Use state.persistent.get to read data stored in a previous session or earlier in the current one.
Persistent state survives Gateway restarts. For session-only data, use state.session.get.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `key` | `String` | Yes | — | The key to look up. |
| `version` | `Integer` | No | — | Specific version number to retrieve. If omitted, returns the latest version. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `value` | `Object` | The stored value, or null if the key does not exist. |
| `version` | `Integer` | The version number of the returned value. |
| `found` | `Boolean` | True if the key exists. |
| `created_at` | `DateTime` | When this key was first created. |
| `updated_at` | `DateTime` | When this key was last updated. |

### Common patterns

**Resume from where a previous session left off**

1. `state.persistent.get(key='progress.checkpoint') to read last saved progress`
2. `Continue work from the saved checkpoint`

**Read a specific version for comparison**

1. `state.persistent.get(key='config', version=3) to read version 3`
2. `state.persistent.get(key='config') to read latest`
3. `Compare the two versions`

### Errors

**`KeyNotFound`** — The key does not exist in persistent state.

- **list_keys**: Use state.persistent.list to see available keys.
- **use_session**: The data may be in session state — try state.session.get.

**Tags:** `state` `persistent` `read` `safe`

---

## `state.persistent.set`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Stores a key-value pair in the agent's persistent state. Creates a new version of the key.

### When to use

Use state.persistent.set to store data that must survive across sessions
and Gateway restarts. Each set creates a new version — old versions are
retained for a configurable window (default: 100 versions).

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `key` | `String` | Yes | — | The key to store under. Overwrites any existing value (creating a new version). |
| `value` | `Object` | Yes | — | Any JSON-serialisable value. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `version` | `Integer` | The new version number assigned to this write. |
| `previous_version` | `Integer` | The previous version number, or 0 if this is a new key. |

### Side effects

- Creates a new version of the key in the agent's SQLite database. *(not reversible)*
- Old versions beyond the retention window are automatically pruned. *(not reversible)*
- Counts toward the agent's persistent state quota (default: 100 MB). *(not reversible)*

### Common patterns

**Save progress checkpoint**

1. `state.persistent.set(key='progress.checkpoint', value={step: 5, total: 10})`

**Store configuration that persists across sessions**

1. `state.persistent.set(key='preferences.output_format', value='json')`

### Errors

**`QuotaExceeded`** — Agent's persistent state quota exceeded.

- **delete_unused**: Use state.persistent.delete to remove unneeded keys.
- **request_increase**: Use alerts.send to request a quota increase from the supervisor.

**Tags:** `state` `persistent` `write`

---

## `state.persistent.delete`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Removes a key and all its version history from the agent's persistent state.

### When to use

Use state.persistent.delete to permanently remove data and free quota.
This deletes all versions of the key — the operation is not reversible.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `key` | `String` | Yes | — | The key to remove. All versions of this key are deleted. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `deleted` | `Boolean` | True if the key existed and was removed. |

### Side effects

- Permanently deletes the key and all its version history from SQLite. *(not reversible)*
- Frees quota used by this key. *(not reversible)*

### Common patterns

**Clean up completed task data**

1. `state.persistent.list(prefix='task.completed.') to find finished task keys`
2. `state.persistent.delete(key=...) for each completed task`

### Errors

**`KeyNotFound`** — The key does not exist.

- **list_keys**: Use state.persistent.list to see available keys.

**Tags:** `state` `persistent` `write`

---

## `state.persistent.list`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Lists keys in the agent's persistent state with metadata summaries.

### When to use

Use state.persistent.list to discover what persistent data exists.
Returns key names, versions, sizes, and timestamps — not full values.
Use state.persistent.get to read the full value of a specific key.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `prefix` | `String` | No | — | If provided, only return keys starting with this prefix. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `entries` | `Array` | List of objects with key, version, size_bytes, updated_at for each entry. |
| `count` | `Integer` | Number of entries returned. |
| `total_size_bytes` | `Integer` | Total size of all listed entries in bytes. |

### Common patterns

**Check quota usage**

1. `state.persistent.list() to see all keys and their sizes`
2. `Sum total_size_bytes to understand quota usage`

**Find keys in a namespace**

1. `state.persistent.list(prefix='deploy.') to find all deployment state`

### Errors

**`DatabaseError`** — Failed to read from the agent's persistent state database.

- **retry**: Retry the operation. If the error persists, check disk health.

**Tags:** `state` `persistent` `read` `safe`

---

## `state.persistent.query`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Queries persistent state by key prefix, returning full entries with values.

### When to use

Use state.persistent.query to retrieve multiple related entries at once.
Unlike state.persistent.list, this returns the full values.
Use for batch reads when you need the actual data, not just metadata.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `prefix` | `String` | Yes | — | Key prefix to match. Returns all keys starting with this prefix. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `entries` | `Array` | List of objects with key, value, version, created_at, updated_at. |
| `count` | `Integer` | Number of entries returned. |

### Common patterns

**Load all task state at once**

1. `state.persistent.query(prefix='task.') to get all task-related state`

**Batch read configuration**

1. `state.persistent.query(prefix='config.') to load all agent configuration`

### Errors

**`DatabaseError`** — Failed to query the agent's persistent state database.

- **retry**: Retry the operation. If the error persists, check disk health.

**Tags:** `state` `persistent` `read` `safe`

---

## `state.persistent.history`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Returns the version history of a specific key in persistent state.

### When to use

Use state.persistent.history to see how a value has changed over time.
Returns previous versions in reverse chronological order (newest first).
The number of retained versions depends on the retention configuration (default: 100).

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `key` | `String` | Yes | — | The key to get history for. |
| `limit` | `Integer` | No | — | Maximum number of versions to return. Defaults to all retained versions. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `versions` | `Array` | List of objects with value, version, created_at, updated_at. Ordered newest first. |
| `count` | `Integer` | Number of versions returned. |

### Common patterns

**Review recent changes to a configuration key**

1. `state.persistent.history(key='config.threshold', limit=5)`
2. `Compare values across versions to understand the change trajectory`

### Errors

**`KeyNotFound`** — The key does not exist or has no version history.

- **list_keys**: Use state.persistent.list to see available keys.

**Tags:** `state` `persistent` `read` `safe`

---

## `state.shared.get`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Retrieves a value from the cross-agent shared state by key. Returns the current version for use with optimistic locking.

### When to use

Use state.shared.get to read data shared between agents. The returned version
number is required for subsequent state.shared.set calls (optimistic locking).
For agent-private data, use state.persistent.get instead.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `key` | `String` | Yes | — | The shared state key to look up. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `value` | `Object` | The stored value, or null if the key does not exist. |
| `version` | `Integer` | Current version of this key. Pass this to state.shared.set to update. 0 if key does not exist. |
| `found` | `Boolean` | True if the key exists. |
| `owner_agent` | `String` | Name of the agent that last wrote this key. |
| `updated_at` | `DateTime` | When this key was last updated. |

### Common patterns

**Read shared configuration**

1. `state.shared.get(key='config.shared_threshold') to read value and version`
2. `Use the value in your logic`

**Read-modify-write with optimistic locking**

1. `state.shared.get(key='counter') to get current value and version`
2. `Compute new value`
3. `state.shared.set(key='counter', value=new_val, expected_version=version)`
4. `If VersionConflict, retry from step 1`

### Errors

**`KeyNotFound`** — The key does not exist in shared state.

- **list_keys**: Use state.shared.list to see available shared keys.

**Tags:** `state` `shared` `read` `safe`

---

## `state.shared.set`

**Permission:** 🔵 Notify · **Version:** 1.0

> Stores a key-value pair in the cross-agent shared state with optimistic locking. Requires the expected version to prevent lost updates.

### When to use

Use state.shared.set to write data visible to other agents. You must pass
expected_version (from a prior state.shared.get). If another agent updated
the key since you read it, a VersionConflict error is returned — re-read
and retry. For new keys, pass expected_version=0.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `key` | `String` | Yes | — | The shared state key to store under. |
| `value` | `Object` | Yes | — | Any JSON-serialisable value. |
| `expected_version` | `Integer` | Yes | — | The version you last read. Pass 0 to create a new key. If the current version does not match, the write is rejected with VersionConflict. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `version` | `Integer` | The new version number after this write. |

### Side effects

- Writes to the shared SQLite database. *(not reversible)*
- Notifies all agents subscribed via state.shared.watch. *(not reversible)*
- Counts toward the total shared state quota (default: 500 MB). *(not reversible)*

### Common patterns

**Create a new shared key**

1. `state.shared.set(key='config.mode', value='production', expected_version=0)`

**Atomic read-modify-write**

1. `entry = state.shared.get(key='counter')`
2. `state.shared.set(key='counter', value=entry.value+1, expected_version=entry.version)`
3. `On VersionConflict, loop back to get`

### Errors

**`VersionConflict`** — Another agent updated this key since you last read it. Your expected_version does not match the current version.

- **retry**: Re-read the key with state.shared.get and retry with the new version.

**`QuotaExceeded`** — Total shared state quota exceeded.

- **delete_unused**: Use state.shared.delete to remove unneeded shared keys.
- **request_increase**: Use alerts.send to request a quota increase.

**Tags:** `state` `shared` `write`

---

## `state.shared.delete`

**Permission:** 🔵 Notify · **Version:** 1.0

> Removes a key from the cross-agent shared state.

### When to use

Use state.shared.delete to remove shared data that is no longer needed.
This frees shared quota. The deletion is visible to all agents immediately.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `key` | `String` | Yes | — | The shared state key to remove. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `deleted` | `Boolean` | True if the key existed and was removed. |

### Side effects

- Permanently removes the key from shared state. *(not reversible)*
- Frees shared state quota. *(not reversible)*

### Common patterns

**Clean up after a shared task completes**

1. `state.shared.delete(key='task.shared_progress')`

### Errors

**`KeyNotFound`** — The key does not exist in shared state.

- **list_keys**: Use state.shared.list to see available shared keys.

**Tags:** `state` `shared` `write`

---

## `state.shared.list`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Lists all keys in the cross-agent shared state with metadata.

### When to use

Use state.shared.list to discover what shared data exists. Returns
key names, versions, sizes, and which agent last wrote each key.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `prefix` | `String` | No | — | If provided, only return keys starting with this prefix. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `entries` | `Array` | List of objects with key, version, size_bytes, owner_agent, updated_at. |
| `count` | `Integer` | Number of entries returned. |
| `total_size_bytes` | `Integer` | Total size of all listed entries in bytes. |

### Common patterns

**Discover shared configuration**

1. `state.shared.list(prefix='config.') to find all shared configuration keys`

### Errors

**`DatabaseError`** — Failed to read from the shared state database.

- **retry**: Retry the operation. If the error persists, check disk health.

**Tags:** `state` `shared` `read` `safe`

---

## `state.shared.watch`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Subscribes to change notifications for shared state keys.

### When to use

Use state.shared.watch to be notified when other agents modify shared state.
Returns a stream of change events. Useful for coordination between agents.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `prefix` | `String` | No | — | If provided, only receive notifications for keys matching this prefix. If omitted, all changes are reported. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `subscription_id` | `String` | A subscription identifier. Changes will be delivered to the agent's message queue. |

### Side effects

- Creates a subscription that will deliver change events until the session ends. *(not reversible)*

### Common patterns

**React to shared configuration changes**

1. `state.shared.watch(prefix='config.') to subscribe`
2. `When notification arrives, state.shared.get the updated key`
3. `Adjust behaviour based on new value`

### Errors

**`SubscriptionFailed`** — Failed to create the change subscription.

- **retry**: Retry the subscription.

**Tags:** `state` `shared` `read` `subscribe`

---

## `state.snapshot.create`

**Permission:** 🔵 Notify · **Version:** 1.0

> Creates a point-in-time snapshot of the entire system state.

### When to use

Use state.snapshot.create to capture the current state of all system
components: active agents, resource usage, pending approvals, alerts,
health status, and state statistics. Useful before making significant changes.

### Inputs

*No inputs required.*

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `snapshot_id` | `String` | Unique identifier for this snapshot (e.g. 'snap-20260325T140000'). |
| `timestamp` | `DateTime` | When the snapshot was taken. |
| `size_bytes` | `Integer` | Size of the snapshot file in bytes. |

### Side effects

- Creates a JSON file in the snapshots directory. *(not reversible)*
- Old snapshots beyond the retention limit are automatically removed. *(not reversible)*

### Common patterns

**Checkpoint before a risky operation**

1. `state.snapshot.create() to capture current state`
2. `Perform the operation`
3. `If something goes wrong, reference the snapshot for recovery`

### Errors

**`IoError`** — Failed to write snapshot to disk.

- **check_disk**: Use system.health to check disk space.

**Tags:** `state` `snapshot` `write`

---

## `state.snapshot.list`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Lists available system snapshots with timestamps and sizes.

### When to use

Use state.snapshot.list to see what snapshots are available.
Snapshots are listed newest first.

### Inputs

*No inputs required.*

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `snapshots` | `Array` | List of objects with snapshot_id, timestamp, size_bytes. |
| `count` | `Integer` | Number of snapshots available. |

### Common patterns

**Find the most recent snapshot**

1. `state.snapshot.list() — first entry is the most recent`

### Errors

**`IoError`** — Failed to read the snapshots directory.

- **check_disk**: Use system.health to check disk space and directory permissions.

**Tags:** `state` `snapshot` `read` `safe`

---

## `state.snapshot.load`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Loads a specific system snapshot by ID, returning the full captured state.

### When to use

Use state.snapshot.load to examine what the system looked like at a
specific point in time. Useful for debugging or comparing system state.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `snapshot_id` | `String` | Yes | — | The snapshot ID to load (from state.snapshot.list). |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `snapshot` | `Object` | Full snapshot data including agents, resources, approvals, alerts, health, and state summaries. |

### Common patterns

**Compare system state before and after a change**

1. `state.snapshot.load(snapshot_id='snap-before') to get before state`
2. `state.snapshot.load(snapshot_id='snap-after') to get after state`
3. `Compare the two snapshots`

### Errors

**`SnapshotNotFound`** — No snapshot exists with the given ID.

- **list_snapshots**: Use state.snapshot.list to see available snapshots.

**Tags:** `state` `snapshot` `read` `safe`

---

## `state.briefing.generate`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Generates a context briefing summarising what changed since the agent's last activity.

### When to use

Use state.briefing.generate when reconnecting after a period of inactivity
to understand what happened while the agent was offline. The briefing is
rule-based (no LLM required) and covers filesystem changes, process events,
pending approvals, unread messages, alerts, and health status.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `since` | `DateTime` | No | — | Gather changes after this timestamp. If omitted, uses the agent's last activity timestamp. |
| `detail_level` | `String` | No | — | 'summary' for counts only (default), or 'detailed' for full change lists. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `briefing` | `Object` | Structured briefing with since, filesystem_changes, process_events, pending_approvals, unread_messages, alerts, health, and a template-generated summary string. |

### Common patterns

**Reconnect and catch up**

1. `state.briefing.generate() to see what changed`
2. `Read the summary field for a quick overview`
3. `If needed, use detail_level='detailed' for full change lists`

**Check activity over a specific window**

1. `state.briefing.generate(since='2026-03-25T14:00:00Z') to review changes since a specific time`

### Errors

**`InvalidTimestamp`** — The since parameter is not a valid ISO 8601 timestamp.

- **fix_format**: Use ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ

**Tags:** `state` `briefing` `read` `safe`

---

## `state.backup.create`

**Permission:** 🟡 Approval Required · **Version:** 1.0

> Creates an encrypted backup of all agent persistent state and shared state.

### When to use

Use state.backup.create before major operations to create a recovery point.
Backups are encrypted at rest with AES-256-GCM using a key derived from
the vault master key. Old backups beyond the retention limit are automatically removed.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `agent` | `String` | No | — | If provided, back up only this agent's persistent state. If omitted, creates a full backup of all agents and shared state. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `backup_id` | `String` | Unique identifier for the created backup. |
| `created_at` | `DateTime` | When the backup was created. |
| `file_size_bytes` | `Integer` | Size of the encrypted backup file in bytes. |
| `scope` | `String` | 'full' for all state, or 'agent:{name}' for a single agent. |

### Side effects

- Creates an encrypted backup file in the backups directory. *(not reversible)*
- Old backups beyond the retention limit are automatically removed. *(not reversible)*

### Common patterns

**Full system backup before upgrade**

1. `state.backup.create() to back up everything`
2. `Perform the upgrade`
3. `If something breaks: state.backup.restore(backup_id=...)`

**Back up a specific agent before changes**

1. `state.backup.create(agent='my-agent')`

### Errors

**`IoError`** — Failed to write backup to disk.

- **check_disk**: Use system.health to check disk space.

**`VaultLocked`** — The vault must be unlocked to create encrypted backups.

- **unlock_vault**: Unlock the vault with the master passphrase.

**Tags:** `state` `backup` `write`

---

## `state.backup.list`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Lists available backups with timestamps, sizes, and scope.

### When to use

Use state.backup.list to see what backups are available for restoration.
Backups are listed newest first.

### Inputs

*No inputs required.*

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `backups` | `Array` | List of objects with backup_id, created_at, scope, file_size_bytes. |
| `count` | `Integer` | Number of backups available. |

### Common patterns

**Find the most recent backup**

1. `state.backup.list() — first entry is the most recent`

### Errors

**`IoError`** — Failed to read the backups directory.

- **check_disk**: Use system.health to check disk space and directory permissions.

**Tags:** `state` `backup` `read` `safe`

---

## `state.backup.restore`

**Permission:** 🟡 Approval Required · **Version:** 1.0

> Restores state from an encrypted backup. Requires the vault to be unlocked.

### When to use

Use state.backup.restore to recover state from a previous backup.
This overwrites current state with the backup contents. Can restore
a full backup or filter to a single agent.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `backup_id` | `String` | Yes | — | The backup ID to restore (from state.backup.list). |
| `agent` | `String` | No | — | If provided, only restore this agent's state from the backup. If omitted, restores all state. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `restored` | `Boolean` | True if the restore completed successfully. |
| `agents_restored` | `Array` | Names of agents whose state was restored. |
| `shared_state_restored` | `Boolean` | Whether shared state was restored. |

### Side effects

- Overwrites current persistent state with backup data. *(not reversible)*
- Overwrites current shared state (for full restores). *(not reversible)*
- This operation is logged in the audit trail. *(not reversible)*

### Common patterns

**Full system restore**

1. `state.backup.list() to find the backup`
2. `state.backup.restore(backup_id='bkp-20260325T140000') to restore everything`

**Restore a single agent**

1. `state.backup.restore(backup_id='bkp-20260325T140000', agent='my-agent')`

### Errors

**`BackupNotFound`** — No backup exists with the given ID.

- **list_backups**: Use state.backup.list to see available backups.

**`DecryptionFailed`** — Wrong key or corrupted backup file.

- **check_vault**: Ensure the vault is unlocked with the correct master passphrase.

**`VaultLocked`** — The vault must be unlocked to decrypt backups.

- **unlock_vault**: Unlock the vault with the master passphrase.

**Tags:** `state` `backup` `write`

---

## `state.backup.export`

**Permission:** 🟡 Approval Required · **Version:** 1.0

> Exports an encrypted backup to an external file path.

### When to use

Use state.backup.export to copy an encrypted backup to an external location
for off-site storage or transfer. The exported file is encrypted — the
recipient needs the vault master passphrase to import it.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `backup_id` | `String` | Yes | — | The backup ID to export. |
| `path` | `String` | Yes | — | Destination path. If a directory, the backup filename is preserved. If a file path, the backup is written to that exact path. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `exported_path` | `String` | The full path where the backup was written. |
| `size_bytes` | `Integer` | Size of the exported file in bytes. |

### Side effects

- Copies the encrypted backup file to the specified path. *(not reversible)*

### Common patterns

**Export for off-site storage**

1. `state.backup.create() to create a fresh backup`
2. `state.backup.export(backup_id='...', path='/mnt/external/')`

### Errors

**`BackupNotFound`** — No backup exists with the given ID.

- **list_backups**: Use state.backup.list to see available backups.

**`IoError`** — Failed to write to the destination path.

- **check_path**: Verify the destination path exists and is writable.

**Tags:** `state` `backup` `write`

---
