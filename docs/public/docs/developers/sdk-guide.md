# Python SDK Guide

The KruxOS Python SDK (`kruxos`) provides a typed, async client for connecting agents to the KruxOS Gateway over the Gateway's MCP WebSocket.

## Installation

The SDK (`kruxos`, version `0.0.2`) ships **bundled inside the appliance** at `/opt/kruxos/sdk/python/`, importable from interactive shells via `/etc/profile.d/kruxos-sdk.sh`. From an autonomous agent task or an in-appliance Python shell, `import kruxos` just works.

For host-side use, copy `/opt/kruxos/sdk/python/` off the appliance into your project. A published `pip install kruxos` package on PyPI is planned for a future release — it is **not** the install path today.

Requires Python 3.11+.

## Quick start

```python
import asyncio
from kruxos import KruxOS

async def main():
    async with await KruxOS.connect_async(
        endpoint="ws://localhost:7700",
        agent_name="my-agent",
        api_key="<64-char hex key from kruxos agent create>",
    ) as agent:
        # Discover capabilities
        caps = await agent.capabilities.list_async()
        print(f"Available: {len(caps)} capabilities")

        # Read a file
        result = await agent.call_async(
            "filesystem.read",
            path="/workspace/README.md",
        )
        print(result.data["content"])

asyncio.run(main())
```

`connect_async()` returns a connected client, and the client is itself an async context manager — hence `async with await KruxOS.connect_async(...)`, which closes the connection automatically on exit. Capabilities are invoked with `agent.call_async(name, **inputs)` (async) or `agent.call(name, **inputs)` (sync).

## Connection

### Async connection (recommended)

```python
from kruxos import KruxOS

async with await KruxOS.connect_async(
    endpoint="ws://localhost:7700",
    agent_name="my-agent",
    api_key="<64-char hex key from kruxos agent create>",
) as agent:
    # agent is connected and authenticated
    pass
# Connection closed automatically
```

Prefer to manage the connection yourself? Await the factory and close explicitly:

```python
agent = await KruxOS.connect_async(
    endpoint="ws://localhost:7700",
    agent_name="my-agent",
    api_key="<64-char hex key from kruxos agent create>",
)
try:
    ...
finally:
    await agent.close_async()
```

### Sync connection

```python
from kruxos import KruxOS

agent = KruxOS.connect(
    endpoint="ws://localhost:7700",
    agent_name="my-agent",
    api_key="<64-char hex key from kruxos agent create>",
)
result = agent.call("filesystem.list", path="/workspace")
agent.close()
```

The sync wrappers (`connect`, `call`, `state.get`, `briefing`, …) each run on
their own short-lived event loop, so a sync client re-authenticates per call and
**cannot** be used from inside a running event loop (a Jupyter kernel, an async
web framework, an MCP-bridge process). Called from a running loop, they raise a
`RuntimeError` that points you at the async API. In any async host, use
`connect_async` and the `*_async` methods directly.

### Connection details from the environment

The SDK does not read connection settings implicitly — pass them in yourself. A common pattern is to source them from the environment:

```python
import os
from kruxos import KruxOS

agent = await KruxOS.connect_async(
    endpoint=os.environ.get("KRUXOS_ENDPOINT", "ws://localhost:7700"),
    agent_name=os.environ["KRUXOS_AGENT_NAME"],
    api_key=os.environ["KRUXOS_API_KEY"],
)
```

These are the same variable names the bundled Claude bridge reads (`KRUXOS_ENDPOINT`, `KRUXOS_AGENT_NAME`, `KRUXOS_API_KEY`).

## Capability discovery

### List all capabilities

```python
caps = await agent.capabilities.list_async()
for cap in caps:
    print(f"{cap.name}: {cap.description}")
```

### Filter by category

```python
fs_caps = await agent.capabilities.list_async(category="filesystem")
```

### Describe a capability

```python
cap = await agent.capabilities.describe_async("filesystem.read")
print(cap.description)

# input_schema is a JSON Schema object describing the capability's inputs
properties = cap.input_schema.get("properties", {})
required = set(cap.input_schema.get("required", []))
for name, spec in properties.items():
    kind = spec.get("type", "any")
    flag = "(required)" if name in required else "(optional)"
    print(f"  {name}: {kind} {flag}")
```

### Search by name

```python
read_caps = await agent.capabilities.list_async(name_contains="read")
```

## Invocation

### Basic invocation

```python
result = await agent.call_async(
    "filesystem.read",
    path="/workspace/data.csv",
)

# A failed call raises a typed exception (see "With error handling" below), so a
# returned response is always a success: `result.status` is
# `ResponseStatus.SUCCESS` and the payload is in `result.data`. On the default
# MCP protocol `result.error` is not populated — never branch on it.
print(result.data["content"])
```

A capability that returns a plain-text (non-JSON) body surfaces it at
`result.data["text"]`; structured capabilities return their fields directly in
`result.data`.

### Timeouts

Two independent timeouts apply to a call, and they are deliberately separate:

- A capability's **own** `timeout` input — capabilities such as `process.run`
  and the `network.*` family accept a `timeout`. Pass it like any other input:

  ```python
  await agent.call_async("process.run", command="sleep 5", timeout=30)
  ```

- The SDK's **per-request** timeout, `request_timeout` (keyword-only) — how long
  the client waits for the Gateway to respond before giving up. It defaults to
  30 seconds. Raise it to wait through a server-side approval hold (see
  [Approvals](#approvals)):

  ```python
  await agent.call_async("process.run", command="deploy.sh", request_timeout=3600)
  ```

`request_timeout` is named distinctly so it never shadows a capability's own
`timeout` input.

### With error handling

Denials, missing approvals, and other failures are raised as typed exceptions on both the MCP and JSON-RPC transports — catch the specific subclass you care about, falling back to `CapabilityError`:

```python
from kruxos.errors import (
    CapabilityError,
    PolicyDeniedError,
    ApprovalRequiredError,
    ConflictError,
)

try:
    result = await agent.call_async(
        "process.run",
        command="ls -la /workspace",
    )
    print(result.data["stdout"])
except PolicyDeniedError as e:
    # Blocked by policy — e.reason / e.rule_reference carry the policy detail
    print(f"Policy blocked: {e.reason or e}")
except ApprovalRequiredError as e:
    # Needs human approval — resolve it (see Approvals below)
    print(f"Waiting for approval: {e.request_id}")
except ConflictError as e:
    # Optimistic-lock failure on a shared-state write — re-read and retry
    print(f"Conflict: {e}")
except CapabilityError as e:
    # Base class for every capability failure; e.error_type is the wire type
    print(f"Error ({e.error_type}): {e}")
    if e.structured:
        for recovery in e.structured.recovery_actions:
            print(f"  Try: {recovery.action} — {recovery.description}")
```

## Approvals

Approval-gated capabilities are **held server-side** by the Gateway until the
operator decides — up to 24 hours. The dashboard queue is the approval surface;
the SDK never prompts. There are two ways to handle an approval-gated call.

### Ride the hold (recommended)

Pass a generous `request_timeout` so the client waits through the operator's
decision. When approved, the call returns its result directly — the same
`CapabilityResponse` a non-gated call would return. A rejection or an expired
hold raises a typed exception:

```python
from kruxos.errors import ApprovalRejectedError, ApprovalRequiredError

try:
    # Wait up to an hour for the operator to decide in the dashboard queue.
    result = await agent.call_async(
        "process.run",
        command="kubectl apply -f deploy.yaml",
        request_timeout=3600,   # client-side wait; NOT a capability `timeout` input
    )
    print(f"Approved and executed: {result.data}")
except ApprovalRejectedError as e:
    print(f"Rejected: {e.request_id}")
except ApprovalRequiredError as e:
    print(f"Still pending after the hold: {e.request_id}")
```

### Catch and poll for status

If you would rather not hold the call, catch `ApprovalRequiredError` and poll
`wait_for_approval_async`. It returns a terminal **status string** — one of
`"approved"`, `"rejected"`, `"expired"`, `"timed_out"` — and does **not**
re-execute the capability. On `"approved"`, re-invoke the capability yourself;
the caller decides what to run:

```python
from kruxos.errors import ApprovalRequiredError

try:
    result = await agent.call_async("process.run", command="deploy.sh")
    print(result.data)
except ApprovalRequiredError as e:
    # Polls kruxos/checkApproval until a terminal status (MCP transport only).
    status = await agent.wait_for_approval_async(e.request_id, timeout=600)
    if status == "approved":
        # The SDK does not auto-re-execute — run the capability again yourself.
        result = await agent.call_async("process.run", command="deploy.sh")
        print(result.data)
    else:
        print(f"Not approved: {status}")
```

`wait_for_approval_async` raises `TimeoutError` if no terminal status is reached
within its own `timeout`; the request stays in the queue, so a supervisor can
still act and you can poll again with the same `request_id`. You can also run the
poll as a background task while doing unrelated work:

```python
import asyncio
from kruxos.errors import ApprovalRequiredError

try:
    result = await agent.call_async("process.run", command="deploy.sh")
except ApprovalRequiredError as e:
    approval = asyncio.create_task(
        agent.wait_for_approval_async(e.request_id, timeout=600)
    )
    while not approval.done():
        # ... do other, unrelated work ...
        await asyncio.sleep(5)
    status = await approval          # a status string, e.g. "approved"
```

A call held for approval occupies the connection until it resolves; the SDK does
not run a second capability in parallel with a held call.

## State management

The `agent.state` API stores values across three tiers — `session` (in-memory,
current session only), `persistent` (per-agent, survives sessions), and `shared`
(cross-agent, optimistic-locked). The tier is a parameter; `persistent` is the
default. `get_async` returns a value (or `None`); `set_async` returns nothing;
`delete_async` returns a bool; `list_async` returns a list of key names.

### Session state (in-memory, current session only)

```python
# Store
await agent.state.set_async("task.current", {"step": 3, "total": 10}, tier="session")

# Read (returns the stored value, or None if not set)
value = await agent.state.get_async("task.current", tier="session")
if value is not None:
    print(value)  # {"step": 3, "total": 10}

# List keys
keys = await agent.state.list_async(tier="session", prefix="task.")

# Clean up
await agent.state.delete_async("task.current", tier="session")
```

### Persistent state (survives across sessions)

```python
# Store (persistent is the default tier)
await agent.state.set_async("config.threshold", 0.85)

# Read
value = await agent.state.get_async("config.threshold")
print(f"Value: {value}")

# List keys under a prefix
keys = await agent.state.list_async(prefix="config.")
```

### Shared state (cross-agent, optimistic locking)

Shared-tier writes are guarded by **optimistic locking**: `set_async` requires an
`expected_version`, and the write fails with `ConflictError` if another agent has
written the key since you read it. Read the current version with
`get_entry_async` (a plain `get_async` returns the value only and discards the
version), then thread that version into the write. Use `expected_version=0` to
create a brand-new key. Omitting `expected_version` on the shared tier raises
`ValueError` before anything is sent.

```python
from kruxos.errors import ConflictError

# Read the entry WITH its optimistic-locking version.
entry = await agent.state.get_entry_async("counter", tier="shared")
# entry.found / entry.value / entry.version / entry.owner_agent
current = entry.value or 0

# Write back under the version we read (0 when the key does not exist yet).
try:
    await agent.state.set_async(
        "counter",
        current + 1,
        tier="shared",
        expected_version=entry.version or 0,
    )
except ConflictError:
    # Another agent wrote between our read and write — re-read and retry.
    ...
```

## Context briefings

`briefing_async()` returns a `ContextBriefing` summarising what changed since the
agent's last activity. The counts (`pending_approvals`, `unread_messages`) are
**integers**; `summary` is a ready-to-read template string; and the subsystem
reports (`filesystem_changes`, `process_events`, `alerts`, `health`) are dicts.

```python
briefing = await agent.briefing_async()

# `summary` is a human/LLM-readable string; the counts are plain integers.
print(briefing.summary)
print(f"Pending approvals: {briefing.pending_approvals}")
print(f"Unread messages:   {briefing.unread_messages}")

# The subsystem reports are dicts — inspect the fields you need.
print(f"Filesystem report keys: {list(briefing.filesystem_changes)}")

# Ask for more detail (default detail_level is "summary").
detailed = await agent.briefing_async(detail_level="detailed")
```

## Service Proxy (email example)

```python
# Search emails (reads from local replica — no API calls)
result = await agent.call_async(
    "email.search",
    query="invoice",
    is_read=False,
)

for msg in result.data["messages"]:
    print(f"{msg['from']}: {msg['subject']}")

# Send email (buffered — 5 min cancellation window)
result = await agent.call_async(
    "email.send",
    to="alice@example.com",
    subject="Report",
    body="Please find the report attached.",
)
write_id = result.data["write_id"]
print(f"Buffered until {result.data['buffer_until']}")

# Cancel if needed
await agent.call_async("proxy.cancel_write", write_id=write_id)
```

## MCP configuration (for Claude)

Claude Desktop connects to KruxOS through a small stdio bridge. Generate the
config entry with `generate_claude_desktop_config`:

```python
import json
from kruxos.connectors import generate_claude_desktop_config

config = generate_claude_desktop_config(
    endpoint="ws://localhost:7700",
    agent_name="my-agent",
    api_key="<64-char hex key from kruxos agent create>",
)
print(json.dumps(config, indent=2))
```

Output:

```json
{
  "kruxos": {
    "command": "python3",
    "args": [
      "-m",
      "kruxos.connectors.claude_bridge"
    ],
    "env": {
      "KRUXOS_ENDPOINT": "ws://localhost:7700",
      "KRUXOS_AGENT_NAME": "my-agent",
      "KRUXOS_API_KEY": "<64-char hex key from kruxos agent create>"
    }
  }
}
```

Merge this under `mcpServers` in Claude Desktop's `claude_desktop_config.json`, then restart Claude Desktop.

For a connected client, `as_mcp_config(agent)` from `kruxos.connectors` returns the equivalent transport/auth descriptor programmatically.

## Pydantic models

All SDK responses are typed Pydantic models:

```python
from kruxos.models import (
    CapabilityDef,
    CapabilityResponse,
    StateEntry,
    StructuredError,
    RecoveryAction,
    ContextBriefing,
    ResponseStatus,
)
```

## Logging

The SDK uses Python's standard `logging` module:

```python
import logging
logging.getLogger("kruxos").setLevel(logging.DEBUG)
```

## Complete example

```python
import asyncio
from kruxos import KruxOS
from kruxos.errors import CapabilityError

async def research_agent():
    async with await KruxOS.connect_async(
        endpoint="ws://localhost:7700",
        agent_name="research-agent",
        api_key="<64-char hex key from kruxos agent create>",
    ) as agent:
        # Check what happened while we were offline
        briefing = await agent.briefing_async()
        print(briefing.summary)

        # Resume from checkpoint
        checkpoint = await agent.state.get_async("research.checkpoint")
        start_from = checkpoint["page"] if checkpoint else 0

        # Do work
        page = start_from
        for page in range(start_from, 100):
            try:
                result = await agent.call_async(
                    "network.http_request",
                    url=f"https://api.example.com/data?page={page}",
                    method="GET",
                )
                data = result.data["body"]

                # Save to workspace
                await agent.call_async(
                    "filesystem.write",
                    path=f"/workspace/data/page_{page}.json",
                    content=data,
                )

                # Checkpoint progress
                await agent.state.set_async(
                    "research.checkpoint",
                    {"page": page + 1, "status": "in_progress"},
                )
            except CapabilityError as e:
                # Send alert to supervisor
                await agent.call_async(
                    "alerts.send",
                    severity="warning",
                    message=f"Research failed on page {page}: {e}",
                    context={"page": page},
                )
                break

        # Final status
        await agent.state.set_async(
            "research.checkpoint",
            {"page": page, "status": "completed"},
        )

asyncio.run(research_agent())
```
