# Python SDK Guide

The KruxOS Python SDK (`kruxos`) provides a typed, async client for connecting agents to the KruxOS Gateway over the Gateway's MCP WebSocket.

## Installation

In v0.0.1 the SDK ships **bundled inside the appliance** at `/opt/kruxos/sdk/python/`, importable from interactive shells via `/etc/profile.d/kruxos-sdk.sh`. From an autonomous agent task or an in-appliance Python shell, `import kruxos` just works.

For host-side use, copy `/opt/kruxos/sdk/python/` off the appliance into your project — the external `pip install kruxos` distribution to PyPI lands in **v0.0.3** alongside the license-server cycle.

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
from kruxos import ResponseStatus

result = await agent.call_async(
    "filesystem.read",
    path="/workspace/data.csv",
)

if result.status == ResponseStatus.SUCCESS:
    print(result.data["content"])
else:
    # On the default MCP protocol a failure comes back as a non-SUCCESS status
    # with the message in result.data; structured `result.error` objects are not
    # populated here. Most failures instead raise a typed exception — see
    # "With error handling" below.
    message = result.data.get("text") if result.data else None
    print(f"Failed: {message or 'unknown error'}")
```

### With error handling

Denials, missing approvals, and other failures are raised as typed exceptions — catch the specific subclass you care about, falling back to `CapabilityError`:

```python
from kruxos.errors import CapabilityError, PolicyDeniedError, ApprovalRequiredError

try:
    result = await agent.call_async(
        "process.run",
        command="ls -la /workspace",
    )
    print(result.data["stdout"])
except PolicyDeniedError as e:
    print(f"Policy blocked: {e}")
except ApprovalRequiredError as e:
    print(f"Waiting for approval: {e.request_id}")
    # Wait for approval (see Approvals section below)
    result = await agent.wait_for_approval_async(e.request_id, timeout=300)
except CapabilityError as e:
    print(f"Error: {e.error_type}")
    if e.structured:
        for recovery in e.structured.recovery_actions:
            print(f"  Try: {recovery.action} — {recovery.description}")
```

## Approvals

When a call needs human approval, `call_async` raises `ApprovalRequiredError`
carrying the `request_id`. Resolve it with `wait_for_approval_async`.

### Blocking wait

```python
from kruxos import ResponseStatus
from kruxos.errors import ApprovalRequiredError

try:
    result = await agent.call_async(
        "process.run",
        command="kubectl apply -f deploy.yaml",
    )
except ApprovalRequiredError as e:
    print(f"Approval needed: {e.request_id}")
    # Blocks until approved/rejected (or timeout)
    result = await agent.wait_for_approval_async(e.request_id, timeout=600)
    if result.status == ResponseStatus.SUCCESS:
        print(f"Approved! Result: {result.data}")
    else:
        # Rejected/failed: on the default MCP protocol the reason is in result.data
        detail = result.data.get("text") if result.data else None
        print(f"Rejected: {detail or 'no reason given'}")
```

### Non-blocking (poll)

```python
import asyncio
from kruxos.errors import ApprovalRequiredError

try:
    result = await agent.call_async("process.run", command="deploy.sh")
except ApprovalRequiredError as e:
    # Resolve the approval in the background while doing other work
    approval = asyncio.create_task(
        agent.wait_for_approval_async(e.request_id, timeout=600)
    )
    while not approval.done():
        # ... do other work ...
        await asyncio.sleep(5)
    result = await approval
```

## State management

The `agent.state` API stores values across three tiers — `session` (in-memory,
current session only), `persistent` (per-agent, survives sessions), and `shared`
(cross-agent). The tier is a parameter; `persistent` is the default.

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

### Shared state (cross-agent)

```python
from kruxos.errors import KruxOSError

# Read a value visible to all agents
value = await agent.state.get_async("counter", tier="shared")

# Update it. Shared writes are last-write-wins through the SDK — there is no
# expected-version / optimistic-locking argument, so concurrent writers do not
# raise a conflict; the most recent write simply prevails.
try:
    await agent.state.set_async("counter", (value or 0) + 1, tier="shared")
except KruxOSError:
    # Surfaces transport or gateway errors (base class for all SDK errors)
    pass
```

## Transactions

```python
async with agent.transaction() as tx:
    await tx.call_async("filesystem.write", path="/workspace/a.txt", content="hello")
    await tx.call_async("filesystem.write", path="/workspace/b.txt", content="world")
    # Commits on clean exit; if any call raises, all are rolled back
```

## Context briefings

```python
# Get a structured summary of what changed since the last connection
briefing = await agent.briefing_async()
print(f"Filesystem changes: {len(briefing.filesystem_changes)}")
print(f"Pending approvals:  {len(briefing.pending_approvals)}")
print(f"State changes:      {len(briefing.state_changes)}")
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
        print(f"{len(briefing.state_changes)} state changes since last run")

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
