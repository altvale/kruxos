# Error Reference

Every KruxOS error is a **StructuredError** containing a machine-readable code, a human description, an agent-facing description, recovery suggestions, and a retryable flag. This page documents every error type your agent or SDK may encounter.

## How to read this catalogue

Each entry lists:

| Field | Meaning |
|-------|---------|
| **Error type** | The `error_type` string returned in the StructuredError JSON |
| **Retryable** | Whether the SDK will auto-retry or you should retry manually |
| **Retry after** | Suggested wait time before retrying (if applicable) |
| **Recovery actions** | Machine-readable action names returned in `recovery_actions` |

### StructuredError JSON shape

```json
{
  "error_type": "PolicyDenied",
  "description": "Policy rule 'deny-network' denied execution of 'network.http'",
  "agent_description": "Your request to execute 'network.http' was blocked by policy rule 'deny-network'...",
  "recovery_actions": [
    {
      "action": "check_policy",
      "description": "Review your agent's policy group and permissions.",
      "capability": "agent.policy",
      "inputs": null
    }
  ],
  "retryable": false,
  "retry_after": null,
  "context": {
    "rule": "deny-network",
    "capability": "network.http"
  }
}
```

### Python SDK exception mapping

Every `error_type` maps to a Python exception class in `kruxos.errors`. All exceptions inherit from `KruxOSError` and expose `.structured`, `.retryable`, and `.error_type` properties.

```python
from kruxos.errors import PolicyDeniedError, RateLimitedError

try:
    result = await os.call_async("filesystem.write", path="/etc/passwd", content="x")
except PolicyDeniedError as e:
    print(e.structured.recovery_actions)  # [RecoveryAction(...)]
    print(e.retryable)                    # False
except RateLimitedError as e:
    print(e.retry_after)                  # 30.0
```

---

## Authentication errors

These errors occur during the WebSocket handshake before a session is established.

### `auth.invalid_credentials`

| | |
|---|---|
| **Message** | Invalid agent name or API key. |
| **Cause** | The agent name does not exist, or the API key does not match. |
| **Fix** | Verify the agent name with `kruxos agent list`. Re-check the API key for copy-paste errors (trailing whitespace, truncation). If the key was rotated, use the new key. |
| **Retryable** | No |
| **SDK exception** | `AuthenticationError` |
| **Recovery actions** | `check_credentials` |

### `auth.agent_revoked`

| | |
|---|---|
| **Message** | Agent 'my-agent' has been revoked. |
| **Cause** | An administrator revoked the agent via `kruxos agent revoke`. |
| **Fix** | Contact the administrator to reinstate the agent, or create a new one with `kruxos agent create`. |
| **Retryable** | No |
| **SDK exception** | `AuthenticationError` |
| **Recovery actions** | `contact_admin`, `create_new_agent` |

### `auth.rate_limited`

| | |
|---|---|
| **Message** | Too many authentication attempts. Max N per minute. |
| **Cause** | Too many failed authentication attempts from the same IP address. |
| **Fix** | Wait 60 seconds before retrying. If you are scripting connections, add a delay between retries. |
| **Retryable** | Yes |
| **SDK exception** | `RateLimitedError` |
| **Recovery actions** | `wait_and_retry` |

---

## Session errors

### `NotAuthenticated`

| | |
|---|---|
| **Message** | Not authenticated: {reason} |
| **Cause** | The request could not be authenticated. This can happen if credentials are missing from the connection handshake. |
| **Fix** | Reconnect with valid credentials (agent name + API key). |
| **Retryable** | No |
| **SDK exception** | `AuthenticationError` |
| **Recovery actions** | `reconnect` |

### `SessionExpired`

| | |
|---|---|
| **Message** | Session '{session_id}' has expired |
| **Cause** | The session timed out due to inactivity, or the Gateway restarted and the session was not recovered from the checkpoint. |
| **Fix** | Reconnect to establish a new session. The SDK reconnects automatically with exponential backoff. |
| **Retryable** | No |
| **SDK exception** | `SessionExpiredError` |
| **Recovery actions** | `reconnect` |
| **Context** | `session_id` |

---

## Policy errors

### `PolicyDenied`

| | |
|---|---|
| **Message** | Policy rule '{rule}' denied execution of '{capability}' |
| **Cause** | The policy engine blocked this capability invocation. The agent's permission tier does not allow the requested operation. |
| **Fix** | Check which policy rule is blocking with `kruxos config check-policy --agent <name> --capability <cap>`. Edit the policy YAML to adjust the tier, or ask a supervisor to grant permission. |
| **Retryable** | No |
| **SDK exception** | `PolicyDeniedError` |
| **Recovery actions** | `check_policy` (calls `agent.policy`), `request_escalation` |
| **Context** | `rule`, `capability` |

### `ApprovalPending`

| | |
|---|---|
| **Message** | Approval pending for '{capability}' (request: {request_id}) |
| **Cause** | The capability's policy tier requires human approval before execution. The request is queued and waiting for a supervisor. |
| **Fix** | Wait for a supervisor to approve via `kruxos approve accept <request_id>` or the dashboard. Poll for the decision using the request ID. |
| **Retryable** | No |
| **SDK exception** | `ApprovalRequiredError` (has `.request_id` property) |
| **Recovery actions** | `wait_for_approval`, `check_status` |
| **Context** | `request_id`, `capability` |

### `ApprovalRejected`

| | |
|---|---|
| **Message** | Approval rejected for request {request_id}: {reason} |
| **Cause** | A supervisor reviewed and rejected the capability request. |
| **Fix** | Read the rejection reason. Modify the operation parameters (e.g., use a less sensitive path, reduce scope) and submit a new request. Contact the supervisor for guidance. |
| **Retryable** | No |
| **SDK exception** | `ApprovalRejectedError` |
| **Recovery actions** | `modify_and_retry`, `contact_supervisor` |
| **Context** | `request_id`, `reason` |

---

## Input and path errors

### `InvalidInput`

| | |
|---|---|
| **Message** | Invalid input: field '{field}' -- {reason} |
| **Cause** | A capability input parameter failed validation (wrong type, missing required field, out of range). |
| **Fix** | Read the `reason` and correct the `field` value. To inspect a capability's schema, call MCP `tools/list` (or JSON-RPC `capabilities.list`) and find the entry by name — v0.0.1 does not ship a `kruxos cap` CLI subcommand. |
| **Retryable** | No |
| **SDK exception** | `InvalidInputError` |
| **Recovery actions** | `fix_input` |
| **Context** | `field`, `reason` |

### `PathOutOfScope`

| | |
|---|---|
| **Message** | Path '{path}' is outside the agent's allowed scope |
| **Cause** | The file path is outside the agent's workspace directory. Agents are sandboxed to their assigned workspace. |
| **Fix** | Use a path within the workspace (typically `/data/kruxos/workspace/<agent>/`). Check workspace boundaries with the `agent.session` capability. |
| **Retryable** | No |
| **SDK exception** | `PathOutOfScopeError` |
| **Recovery actions** | `check_scope` (calls `agent.session`), `use_workspace_path` |
| **Context** | `path` |

### `FileNotFound`

| | |
|---|---|
| **Message** | File not found: {path} |
| **Cause** | The specified file does not exist at the given path. |
| **Fix** | Verify the path spelling and case sensitivity. List the parent directory to find the correct filename. |
| **Retryable** | No |
| **SDK exception** | `FileNotFoundError` |
| **Recovery actions** | `list_directory` (calls `filesystem.list` on parent dir), `check_path` |
| **Context** | `path` |

### `PermissionDenied`

| | |
|---|---|
| **Message** | Permission denied: cannot {operation} '{path}' |
| **Cause** | The OS denied the operation on the file. The file exists, but the sandbox does not grant the required access. |
| **Fix** | Check file permissions with `filesystem.stat`. If the file is owned by root or another user, ask a supervisor to adjust sandbox permissions. |
| **Retryable** | No |
| **SDK exception** | `PermissionDeniedError` |
| **Recovery actions** | `check_permissions` (calls `filesystem.stat`), `request_access` |
| **Context** | `path`, `operation` |

---

## Resource and rate limit errors

### `ResourceExhausted`

| | |
|---|---|
| **Message** | Resource exhausted: {resource} at {current} (limit: {limit}) |
| **Cause** | A quota or system resource limit was reached (disk space, state quota, memory, open files). |
| **Fix** | Check current usage with `system.resources`. Clean up unused data (delete old state keys, remove temporary files). To increase quotas, edit `/etc/kruxos/gateway.yaml`. |
| **Retryable** | No |
| **SDK exception** | `ResourceExhaustedError` |
| **Recovery actions** | `check_usage` (calls `system.resources`), `cleanup` |
| **Context** | `resource`, `limit`, `current` |

### `RateLimited`

| | |
|---|---|
| **Message** | Rate limited: {limit} requests per {window_seconds}s exceeded |
| **Cause** | The agent exceeded its capability invocation rate limit. |
| **Fix** | Wait the specified `retry_after` seconds, then retry. To adjust rate limits, modify the policy YAML. |
| **Retryable** | Yes |
| **Retry after** | `retry_after_seconds` (from context) |
| **SDK exception** | `RateLimitedError` (has `.retry_after` property) |
| **Recovery actions** | `wait_and_retry` |
| **Context** | `limit`, `window_seconds`, `retry_after_seconds` |

### `Timeout`

| | |
|---|---|
| **Message** | Operation '{operation}' timed out after {timeout_seconds}s |
| **Cause** | The capability did not complete within the allowed time. The operation may still be running in the background. |
| **Fix** | Retry the operation. If it consistently times out, try with a longer timeout value or break the work into smaller operations. |
| **Retryable** | Yes |
| **SDK exception** | `TimeoutError` |
| **Recovery actions** | `retry`, `increase_timeout` |
| **Context** | `operation`, `timeout_seconds` |

---

## Concurrency and service errors

### `ConflictError`

| | |
|---|---|
| **Message** | Conflict: {resource} was modified concurrently |
| **Cause** | Another agent or operation modified the resource since it was last read (optimistic locking failure). |
| **Fix** | Re-read the current version of the resource, then retry the update with the new version number. |
| **Retryable** | Yes |
| **SDK exception** | `ConflictError` |
| **Recovery actions** | `re_read` |
| **Context** | `resource`, `expected_version`, `actual_version` |

### `ServiceUnavailable`

| | |
|---|---|
| **Message** | Service '{service}' is unavailable: {reason} |
| **Cause** | An external service (Gmail, Slack, etc.) or internal subsystem is temporarily unavailable. |
| **Fix** | Wait and retry. The service proxy automatically retries queued operations. Check service status with `system.health`. |
| **Retryable** | Yes |
| **Retry after** | 60 seconds |
| **SDK exception** | `ServiceUnavailableError` |
| **Recovery actions** | `retry_later`, `check_service_status` (calls `system.health`) |
| **Context** | `service`, `reason` |

---

## Service Proxy errors

These errors are specific to the Service Proxy framework and external service integrations.

### `proxy.service_error`

| | |
|---|---|
| **Message** | External service {service} returned an error: {message} |
| **Cause** | The external service (e.g., Gmail API) returned an error response. |
| **Fix** | Wait 60 seconds and retry. If the error persists, check the service status and your OAuth token validity. |
| **Retryable** | Yes |
| **Retry after** | 60 seconds |
| **Recovery actions** | `retry_later` |

### `proxy.batch_protection`

| | |
|---|---|
| **Message** | Batch protection triggered: {count} {operation} writes exceeds threshold ({threshold}) |
| **Cause** | The agent attempted more write operations than the safety threshold allows. This is a safeguard against accidentally sending hundreds of emails or making bulk destructive changes. |
| **Fix** | Request supervisor approval for the batch operation. Reduce the batch size, or ask an admin to adjust the threshold. |
| **Retryable** | No |
| **Recovery actions** | `request_approval` |
| **Context** | `operation`, `count`, `threshold` |

---

## Error type quick reference

| Error type | Retryable | SDK exception | Category |
|---|---|---|---|
| `auth.invalid_credentials` | No | `AuthenticationError` | Auth |
| `auth.agent_revoked` | No | `AuthenticationError` | Auth |
| `auth.rate_limited` | Yes | `RateLimitedError` | Auth |
| `NotAuthenticated` | No | `AuthenticationError` | Session |
| `SessionExpired` | No | `SessionExpiredError` | Session |
| `PolicyDenied` | No | `PolicyDeniedError` | Policy |
| `ApprovalPending` | No | `ApprovalRequiredError` | Policy |
| `ApprovalRejected` | No | `ApprovalRejectedError` | Policy |
| `InvalidInput` | No | `InvalidInputError` | Input |
| `PathOutOfScope` | No | `PathOutOfScopeError` | Input |
| `FileNotFound` | No | `FileNotFoundError` | Input |
| `PermissionDenied` | No | `PermissionDeniedError` | Input |
| `ResourceExhausted` | No | `ResourceExhaustedError` | Resource |
| `RateLimited` | Yes | `RateLimitedError` | Resource |
| `Timeout` | Yes | `TimeoutError` | Resource |
| `ConflictError` | Yes | `ConflictError` | Concurrency |
| `ServiceUnavailable` | Yes | `ServiceUnavailableError` | Service |
| `proxy.service_error` | Yes | `ServiceUnavailableError` | Proxy |
| `proxy.batch_protection` | No | `CapabilityError` | Proxy |

---

## Handling errors in the SDK

### Catch specific errors

```python
from kruxos.errors import (
    PolicyDeniedError,
    ApprovalRequiredError,
    RateLimitedError,
    TimeoutError,
    ConflictError,
)

try:
    result = await os.call_async("filesystem.write", path="output.txt", content=data)
except PolicyDeniedError as e:
    # Not allowed -- check policy
    print(f"Blocked by rule: {e.structured.context['rule']}")
except ApprovalRequiredError as e:
    # Wait for human approval
    print(f"Waiting for approval: {e.request_id}")
except RateLimitedError as e:
    # Back off and retry
    await asyncio.sleep(e.retry_after)
except TimeoutError:
    # Retry with longer timeout
    pass
except ConflictError:
    # Re-read and retry
    pass
```

### Check retryability generically

```python
from kruxos.errors import KruxOSError

try:
    result = await os.call_async("process.run", command=["make", "build"])
except KruxOSError as e:
    if e.retryable:
        # Safe to retry after a delay
        await asyncio.sleep(e.structured.retry_after or 5)
    else:
        # Inspect recovery_actions for next steps
        for action in e.structured.recovery_actions:
            print(f"  Try: {action.description}")
```

### Use recovery actions programmatically

Recovery actions may include a `capability` field suggesting which capability to call:

```python
except FileNotFoundError as e:
    for action in e.structured.recovery_actions:
        if action.capability:
            # Call the suggested capability
            result = await os.call_async(action.capability, **(action.inputs or {}))
```
