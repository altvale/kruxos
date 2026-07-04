# Python SDK Guide

The KruxOS Python SDK (`kruxos`) provides a typed, async client for connecting agents to the KruxOS Gateway over the supervision WebSocket.

## Installation

In v0.0.1 the SDK ships **bundled inside the appliance** at `/opt/kruxos/sdk/python/`, importable from interactive shells via `/etc/profile.d/kruxos-sdk.sh`. From an autonomous agent task or an in-appliance Python shell, `import kruxos` just works.

For host-side use, copy `/opt/kruxos/sdk/python/` off the appliance into your project — the external `pip install kruxos` distribution to PyPI lands in **v0.0.3** alongside the license-server cycle.

Requires Python 3.11+.

## Quick start

```python
import asyncio
from kruxos import KruxOS

async def main():
    async with KruxOS.connect_async("localhost:7700", api_key="<64-char hex>") as agent:
        # Discover capabilities
        caps = await agent.capabilities.list()
        print(f"Available: {len(caps)} capabilities")

        # Read a file
        result = await agent.capabilities.invoke(
            "filesystem.read",
            path="/workspace/README.md"
        )
        print(result.data["content"])

asyncio.run(main())
```

## Connection

### Async connection (recommended)

```python
from kruxos import KruxOS

async with KruxOS.connect_async(
    host="localhost:7700",
    api_key="7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c",
) as agent:
    # agent is connected and authenticated
    pass
# Connection closed automatically
```

### Sync connection

```python
from kruxos import KruxOS

agent = KruxOS.connect("localhost:7700", api_key="<64-char hex>")
result = agent.capabilities.invoke("filesystem.list", path="/workspace")
agent.disconnect()
```

### Connection from environment

```python
from kruxos import KruxOS

# Reads KRUXOS_HOST and KRUXOS_API_KEY from environment
async with KruxOS.connect_async() as agent:
    pass
```

## Capability discovery

### List all capabilities

```python
caps = await agent.capabilities.list()
for cap in caps:
    print(f"{cap.name} ({cap.permission_tier})")
```

### Filter by category

```python
fs_caps = await agent.capabilities.list(category="filesystem")
```

### Describe a capability

```python
cap = await agent.capabilities.describe("filesystem.read")
print(cap.purpose)
print(cap.when_to_use)
for inp in cap.inputs:
    print(f"  {inp.name}: {inp.type} {'(required)' if inp.required else '(optional)'}")
```

### Search by tag

```python
safe_caps = await agent.capabilities.list(tag="safe")
```

## Invocation

### Basic invocation

```python
result = await agent.capabilities.invoke(
    "filesystem.read",
    path="/workspace/data.csv"
)

if result.success:
    print(result.data["content"])
else:
    print(f"Error: {result.error.type} — {result.error.message}")
```

### With error handling

```python
from kruxos.errors import CapabilityError, PolicyDenied, ApprovalRequired

try:
    result = await agent.capabilities.invoke(
        "process.run",
        command="ls -la /workspace"
    )
    print(result.data["stdout"])
except PolicyDenied as e:
    print(f"Policy blocked: {e.message}")
    print(f"Rule: {e.rule}")
except ApprovalRequired as e:
    print(f"Waiting for approval: {e.approval_id}")
    # Wait for approval (see Approval section below)
    result = await agent.approvals.wait(e.approval_id, timeout=300)
except CapabilityError as e:
    print(f"Error: {e.type}")
    for recovery in e.recovery:
        print(f"  Try: {recovery.action} — {recovery.description}")
```

## Approvals

### Blocking wait

```python
from kruxos.errors import ApprovalRequired

try:
    result = await agent.capabilities.invoke(
        "process.run",
        command="kubectl apply -f deploy.yaml"
    )
except ApprovalRequired as e:
    print(f"Approval needed: {e.approval_id}")
    # Blocks until approved/rejected (or timeout)
    result = await agent.approvals.wait(e.approval_id, timeout=600)
    if result.approved:
        print(f"Approved! Result: {result.data}")
    else:
        print(f"Rejected: {result.reason}")
```

### Non-blocking (poll)

```python
from kruxos.errors import ApprovalRequired
import asyncio

try:
    result = await agent.capabilities.invoke("process.run", command="deploy.sh")
except ApprovalRequired as e:
    approval_id = e.approval_id
    # Do other work while waiting
    while True:
        status = await agent.approvals.check(approval_id)
        if status.resolved:
            break
        await asyncio.sleep(5)
```

## State management

### Session state (in-memory, current session only)

```python
# Store
await agent.state.session.set("task.current", {"step": 3, "total": 10})

# Read
result = await agent.state.session.get("task.current")
if result.found:
    print(result.value)  # {"step": 3, "total": 10}

# List keys
keys = await agent.state.session.list(prefix="task.")

# Clean up
await agent.state.session.delete("task.current")
```

### Persistent state (survives across sessions)

```python
# Store (versioned — each write creates a new version)
await agent.state.persistent.set("config.threshold", 0.85)

# Read
result = await agent.state.persistent.get("config.threshold")
print(f"Value: {result.value}, version: {result.version}")

# Read specific version
old = await agent.state.persistent.get("config.threshold", version=1)

# Version history
history = await agent.state.persistent.history("config.threshold", limit=5)
for entry in history.versions:
    print(f"v{entry.version}: {entry.value} ({entry.updated_at})")
```

### Shared state (cross-agent, optimistic locking)

```python
# Read (returns version for optimistic locking)
entry = await agent.state.shared.get("counter")

# Write (must pass expected_version)
try:
    await agent.state.shared.set(
        "counter",
        entry.value + 1,
        expected_version=entry.version
    )
except VersionConflict:
    # Another agent updated — re-read and retry
    pass

# Watch for changes
await agent.state.shared.watch(prefix="config.", callback=on_config_change)
```

## Transactions

```python
from kruxos import transaction

async with transaction(agent) as tx:
    await tx.invoke("filesystem.write", path="/workspace/a.txt", content="hello")
    await tx.invoke("filesystem.write", path="/workspace/b.txt", content="world")
    # If any invocation fails, all are rolled back
```

## Context briefings

```python
# Get a summary of what changed since last session
briefing = await agent.capabilities.invoke("state.briefing.generate")
print(briefing.data["briefing"]["summary"])
```

## Service Proxy (email example)

```python
# Search emails (reads from local replica — no API calls)
result = await agent.capabilities.invoke(
    "email.search",
    query="invoice",
    is_read=False
)

for msg in result.data["messages"]:
    print(f"{msg['from']}: {msg['subject']}")

# Send email (buffered — 5 min cancellation window)
result = await agent.capabilities.invoke(
    "email.send",
    to="alice@example.com",
    subject="Report",
    body="Please find the report attached."
)
write_id = result.data["write_id"]
print(f"Buffered until {result.data['buffer_until']}")

# Cancel if needed
await agent.capabilities.invoke("proxy.cancel_write", write_id=write_id)
```

## MCP configuration (for Claude)

```python
from kruxos import KruxOS

agent = KruxOS.connect("localhost:7700", api_key="7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c")
config = agent.as_mcp_config()
print(config)
```

Output:

```json
{
  "mcpServers": {
    "kruxos": {
      "url": "ws://localhost:7700/mcp",
      "headers": {
        "Authorization": "Bearer 7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c"
      }
    }
  }
}
```

Add this to Claude Desktop's `claude_desktop_config.json` or pass it programmatically.

## Pydantic models

All SDK responses are typed Pydantic models:

```python
from kruxos.models import (
    CapabilityDefinition,
    InvocationResult,
    ErrorResponse,
    RecoveryAction,
    StateEntry,
    ApprovalStatus,
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
    async with KruxOS.connect_async() as agent:
        # Check what happened while we were offline
        briefing = await agent.capabilities.invoke("state.briefing.generate")
        print(briefing.data["briefing"]["summary"])

        # Resume from checkpoint
        checkpoint = await agent.state.persistent.get("research.checkpoint")
        start_from = checkpoint.value["page"] if checkpoint.found else 0

        # Do work
        for page in range(start_from, 100):
            try:
                result = await agent.capabilities.invoke(
                    "network.http_request",
                    url=f"https://api.example.com/data?page={page}",
                    method="GET"
                )
                data = result.data["body"]

                # Save to workspace
                await agent.capabilities.invoke(
                    "filesystem.write",
                    path=f"/workspace/data/page_{page}.json",
                    content=data
                )

                # Checkpoint progress
                await agent.state.persistent.set(
                    "research.checkpoint",
                    {"page": page + 1, "status": "in_progress"}
                )
            except CapabilityError as e:
                # Send alert to supervisor
                await agent.capabilities.invoke(
                    "alerts.send",
                    severity="warning",
                    title=f"Research failed on page {page}",
                    message=str(e)
                )
                break

        # Final status
        await agent.state.persistent.set(
            "research.checkpoint",
            {"page": page, "status": "completed"}
        )

asyncio.run(research_agent())
```
