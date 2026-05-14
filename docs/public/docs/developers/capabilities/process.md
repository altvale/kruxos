# Process Capabilities

Execute commands, monitor background processes, and retrieve output.

## Overview

| Capability | Permission | Purpose |
|------------|:----------:|---------|
| [`process.run`](#processrun) | 🔵 Notify | Executes a command within the agent's sandbox and returns the exit code, captured output, duration, and resource usage. |
| [`process.monitor`](#processmonitor) | 🟢 Autonomous | Returns the current status, resource usage, and runtime of a previously started background process. |
| [`process.kill`](#processkill) | 🔵 Notify | Sends a signal to terminate a running process started by this agent. |
| [`process.list`](#processlist) | 🟢 Autonomous | Lists all processes owned by this agent, including running, completed, and recently killed processes. |
| [`process.logs`](#processlogs) | 🟢 Autonomous | Retrieves stdout and/or stderr output from a running or completed process. |

## `process.run`

**Permission:** 🔵 Notify · **Version:** 1.1

> Executes a command within the agent's sandbox and returns the exit code, captured output, duration, and resource usage. On timeout, returns the partial output captured before the kill so diagnostic signal is preserved.

### When to use

Use process.run to execute shell commands, scripts, or binaries within your sandbox.
Do NOT use process.run for filesystem operations — use filesystem.* capabilities instead.

**Timeout behaviour:**

- Default timeout is 300 seconds (5 minutes).
- Maximum timeout is 3600 seconds (1 hour) — pass an explicit `timeout` for any command expected to exceed 5 minutes (test suites, builds, slow integration tests).
- On timeout the call **returns Ok** with `state="timed_out"` and `exit_code=null`; `stdout_summary` / `stderr_summary` contain whatever the process wrote before the kill. Check `state` (not an error type) to detect timeouts.

**Background mode for long-running operations:**

- For commands that may exceed the 1-hour foreground cap (long test suites, model training, large builds), pass `background: true`. The call returns immediately with a `process_id`; the foreground timeout does not apply.
- Use `process.monitor` to check status, `process.logs` to retrieve output, and `process.kill` to terminate a background process.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `command` | `String` | Yes | — | The command to execute. Passed to the shell as a single string. Use array-style quoting for arguments with spaces. |
| `args` | `Array` | No | — | Arguments to pass to the command. Each element is a separate argument. Preferred over embedding arguments in the command string. |
| `working_directory` | `FilesystemPath` | No | — | Directory to execute the command in. Must be within the agent's workspace. Defaults to the workspace root. |
| `environment` | `Object` | No | — | Additional environment variables as key-value pairs. Merged with the sandbox's base environment. Cannot override system-protected variables. |
| `timeout` | `Integer` | No | `300` | Maximum **foreground** execution time in seconds. Default 300 (5 minutes). Maximum 3600 (1 hour). Pass an explicit value for commands expected to exceed 5 minutes. On timeout returns Ok with `state="timed_out"`, `exit_code=null`, and partial output. For commands that may exceed 1 hour, use `background=true` instead. |
| `background` | `Boolean` | No | `False` | If true, start the process in the background and return immediately with a process_id. The foreground timeout does not apply to background processes — use this for multi-hour operations. Use `process.monitor` / `process.logs` / `process.kill` to manage. |
| `stdin` | `String` | No | — | Input to feed to the process via stdin. Ignored if background=true. |
| `cpu_limit` | `Float` | No | — | CPU limit as a fraction of one core (e.g. 0.5 = half a core). Cannot exceed the agent's allocated CPU. Defaults to the agent's cgroup limit. |
| `memory_limit` | `Integer` | No | — | Memory limit in bytes. Cannot exceed the agent's allocated memory. Defaults to the agent's cgroup limit. |
| `stdout_lines` | `Integer` | No | `100` | Number of trailing stdout lines to include in stdout_summary. Full output available via process.logs. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `exit_code` | `Integer` | Process exit code. 0 typically indicates success. Null if process was killed or timed out. |
| `stdout_summary` | `String` | Last N lines of stdout (controlled by stdout_lines input). Use process.logs for full output. |
| `stderr_summary` | `String` | Last N lines of stderr. Use process.logs for full stderr. |
| `duration_ms` | `Integer` | Wall-clock execution time in milliseconds. |
| `process_id` | `String` | Unique identifier for this process execution. Always returned. Use with process.monitor, process.logs, and process.kill for background processes. |
| `state` | `String` | Process state: `completed` (process exited on its own), `running` (background), `killed` (explicitly killed), or `timed_out` (exceeded foreground timeout — partial output preserved, exit_code=null). |
| `resource_usage` | `Object` | Resource consumption: {cpu_ms, memory_peak_bytes, io_read_bytes, io_write_bytes}. |

### Side effects

- Executes a process in the agent's sandbox which may modify files, consume resources, or produce network traffic within sandbox limits. *(not reversible)*
- Background processes persist until they complete, are killed, or the session ends. *(reversible)*

### Common patterns

**Run a command and check output**

1. `process.run(command='python3', args=['script.py'])` to execute
2. Check `exit_code == 0` for success
3. Parse `stdout_summary` for results

**Long test suite (5–60 minutes) — extend the foreground timeout**

1. `process.run(command='pytest', args=['-q', 'tests/'], timeout=1800)` to allow up to 30 minutes
2. If `state` is `timed_out`, inspect `stdout_summary` for the partial test output and rerun with a higher timeout or `background=true`
3. The default 5-minute timeout will fire on any test suite slower than that — pass an explicit `timeout` up to the 1-hour cap

**Multi-hour operation (model training, large build) — use background mode**

1. `process.run(command='python3', args=['train.py'], background=true)` to start; foreground timeout does not apply
2. `process.monitor(process_id=...)` periodically to check state
3. `process.logs(process_id=..., stream='stdout')` for streaming output
4. `process.kill(process_id=...)` if cancellation needed

**Background task with resource limits**

1. `process.run(command='make', args=['build'], cpu_limit=0.5, memory_limit=536870912, timeout=600)`
2. Check `resource_usage` in response to verify consumption

### Errors

**`CommandNotFound`** — The specified command does not exist in the sandbox PATH.

- **check_path**: Use process.run(command='which', args=['command_name']) to check if the command is installed.
- **use_full_path**: Specify the full absolute path to the executable.

> **Note: `Timeout` is not an error type.** The call returns Ok with `state="timed_out"` and `exit_code=null` so the caller can inspect partial output. To recover: rerun with a higher `timeout` (up to 3600), or pass `background=true` if the operation may exceed the 1-hour foreground cap.

**`ResourceLimitExceeded`** — Process was killed because it exceeded its memory or CPU limit.

- **increase_limits**: Retry with higher cpu_limit or memory_limit if within agent allocation.
- **optimize**: Modify the command or script to use fewer resources.

**`PathOutOfScope`** — The working_directory is outside the agent's workspace.

- **check_scope**: Call agent.session to see accessible directories.

**`SpawnFailed`** — Failed to start the process due to OS-level error.

- **check_permissions**: Verify the command is executable and the sandbox has required permissions.
- **retry**: Retry the operation.

**Tags:** `process` `execute` `sandbox`

---

## `process.monitor`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Returns the current status, resource usage, and runtime of a previously started background process.

### When to use

Use process.monitor to check whether a background process is still running, has completed, or was killed.
Use process.logs to retrieve the actual output of the process.
Use process.kill to terminate the process if it needs to be stopped.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `process_id` | `String` | Yes | — | The process identifier returned by process.run. Use process.list to find active process IDs. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `process_id` | `String` | The process identifier being monitored. |
| `command` | `String` | The command that was executed. |
| `state` | `String` | Current state: 'running', 'completed', 'killed', or 'timed_out'. |
| `exit_code` | `Integer` | Exit code if the process has completed. Null if still running or was killed. |
| `duration_ms` | `Integer` | Elapsed time in milliseconds since the process started. |
| `resource_usage` | `Object` | Current resource consumption: {cpu_ms, memory_peak_bytes, io_read_bytes, io_write_bytes}. |
| `started_at` | `DateTime` | ISO 8601 timestamp when the process was started. |

### Common patterns

**Poll a background process until completion**

1. `process.monitor(process_id=...) to check state`
2. `If state is 'running', wait and poll again`
3. `When state is 'completed', use process.logs to get output`

### Errors

**`ProcessNotFound`** — No process with the given ID exists or it has been cleaned up.

- **list_processes**: Use process.list to see all tracked processes.

**Tags:** `process` `monitor` `status`

---

## `process.kill`

**Permission:** 🔵 Notify · **Version:** 1.0

> Sends a signal to terminate a running process started by this agent.

### When to use

Use process.kill to stop a background process that is no longer needed or is misbehaving.
Use process.monitor first to check if the process is still running.
Use process.list to find process IDs if you don't have one.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `process_id` | `String` | Yes | — | The process identifier returned by process.run. |
| `signal` | `Signal` | No | `SIGTERM` | Signal to send. Use SIGTERM for graceful shutdown, SIGKILL for forced termination. Default is SIGTERM. |
| `force_after` | `Integer` | No | `10` | Seconds to wait after SIGTERM before sending SIGKILL. Only applies when signal is SIGTERM. Set to 0 for immediate SIGTERM without escalation. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `killed` | `Boolean` | True if the process was successfully terminated. |
| `signal_sent` | `String` | The signal that was sent (e.g. 'SIGTERM', 'SIGKILL'). |
| `process_id` | `String` | The process identifier that was targeted. |
| `state` | `String` | Final process state after kill: 'killed' or 'completed' (if it exited before the signal arrived). |

### Side effects

- Terminates the running process and frees its resources. *(not reversible)*

### Common patterns

**Graceful shutdown with fallback to force kill**

1. `process.kill(process_id=..., signal='SIGTERM', force_after=5)`
2. `If killed=false, the process had already exited`

**Immediate forced termination**

1. `process.kill(process_id=..., signal='SIGKILL')`

### Errors

**`ProcessNotFound`** — No process with the given ID exists or it has already been cleaned up.

- **list_processes**: Use process.list to see all tracked processes.

**`ProcessAlreadyExited`** — The process has already completed or been killed.

- **check_status**: Use process.monitor to see the final state and exit code.

**`KillFailed`** — The OS refused to deliver the signal.

- **force_kill**: Retry with signal='SIGKILL'.

**Tags:** `process` `kill` `terminate`

---

## `process.list`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Lists all processes owned by this agent, including running, completed, and recently killed processes.

### When to use

Use process.list to discover what processes are currently running or have recently completed.
Use process.monitor to get detailed status of a specific process.
Use process.logs to retrieve output from a listed process.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `state_filter` | `String` | No | `all` | Filter by state: 'running', 'completed', 'killed', 'timed_out', or 'all'. Default is 'all'. |
| `limit` | `Integer` | No | `50` | Maximum number of processes to return. Most recent first. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `processes` | `Array` | Array of process entries, each with: {process_id, command, state, exit_code, started_at, duration_ms, resource_usage}. |
| `total` | `Integer` | Total number of processes matching the filter (may exceed limit). |

### Common patterns

**Find all running background tasks**

1. `process.list(state_filter='running') to see active processes`
2. `process.monitor(process_id=...) for detailed status on each`

**Review recent completed processes**

1. `process.list(state_filter='completed', limit=10)`
2. `Check exit_code of each to identify failures`

### Errors

**`InvalidFilter`** — The state_filter value is not one of the valid options.

- **use_valid_filter**: Use one of: 'running', 'completed', 'killed', 'timed_out', 'all'.

**Tags:** `process` `list` `status`

---

## `process.logs`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Retrieves stdout and/or stderr output from a running or completed process.

### When to use

Use process.logs to get full output from a process, especially when stdout_summary from process.run was truncated.
Use process.monitor to check if the process is still running before retrieving logs.
For real-time output of long-running processes, call process.logs periodically with increasing offset.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `process_id` | `String` | Yes | — | The process identifier returned by process.run. |
| `stream` | `String` | No | `both` | Which output stream: 'stdout', 'stderr', or 'both'. Default is 'both'. |
| `offset` | `Integer` | No | — | Byte offset to start reading from. Use for paginated reads of large output. |
| `limit` | `Integer` | No | — | Maximum bytes to return. Default returns all available output. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `stdout` | `String` | Stdout content (if stream is 'stdout' or 'both'). Empty string if no output. |
| `stderr` | `String` | Stderr content (if stream is 'stderr' or 'both'). Empty string if no output. |
| `stdout_size` | `Integer` | Total size of stdout in bytes. |
| `stderr_size` | `Integer` | Total size of stderr in bytes. |
| `truncated` | `Boolean` | True if output was truncated due to limit parameter. |
| `process_id` | `String` | The process identifier. |
| `state` | `String` | Current process state when logs were retrieved. |

### Common patterns

**Get full output after process completes**

1. `process.monitor(process_id=...) to confirm state is 'completed'`
2. `process.logs(process_id=..., stream='both') for all output`

**Stream output from a running process**

1. `process.logs(process_id=..., stream='stdout', offset=0, limit=4096) for first chunk`
2. `Use stdout_size as next offset for subsequent calls`
3. `Repeat until process completes`

### Errors

**`ProcessNotFound`** — No process with the given ID exists or it has been cleaned up.

- **list_processes**: Use process.list to see all tracked processes.

**`InvalidStream`** — The stream value is not one of 'stdout', 'stderr', or 'both'.

- **use_valid_stream**: Use one of: 'stdout', 'stderr', 'both'.

**Tags:** `process` `logs` `output`

---
