# Connect Gmail

By the end of this page, your agents will be able to search, read, and send emails through the Service Proxy safety chain.

!!! warning "Operator-facing connection flow ships in v0.0.2"
    The v0.0.1 appliance bundles the Gmail / Slack OAuth **adapters** — the read-replica sync, write buffer, and batch-protection chain all work, and the vault stores OAuth tokens with auto-refresh. What v0.0.1 does **not** yet ship is the operator-facing connection UX (`kruxos connect gmail` CLI subcommand, dashboard Gmail-OAuth flow). This page describes the runtime behaviour that's in place today; the wiring will arrive in **v0.0.2**.

## How the Service Proxy works

KruxOS never gives agents direct access to Gmail. Instead, the Service Proxy provides three layers of safety:

1. **Read-replica** — emails are synced to a local SQLite database. Search and read operations hit the local copy, not Gmail. Zero risk of accidental deletion or modification during reads.
2. **Write buffer** — sends, deletes, and label changes are buffered for a configurable delay (default: 30 seconds). During this window, writes can be cancelled.
3. **Batch protection** — if an agent tries to send more than 5 emails in a short window, the operation escalates to the KruxOS approval queue.

## Email capabilities in v0.0.1

Once the OAuth handshake is in place (manual today, dashboard-driven in v0.0.2), seven typed `email.*` capabilities are exposed to agents:

| Capability | Description | Safety |
|-----------|-------------|--------|
| `email.search` | Search messages by query | Reads local replica |
| `email.read` | Read full message content | Reads local replica |
| `email.send` | Send a new email | Buffered with delay |
| `email.reply` | Reply to a message | Buffered with delay |
| `email.forward` | Forward a message | Buffered with delay |
| `email.delete` | Delete a message | Soft-delete, 24 h recovery via the trash subsystem |
| `email.label` | Add/remove labels | Buffered with delay |

These show up under `tools/list` annotated with their policy tier; agents see them once the OAuth handshake has stored a refreshable token in the vault.

## Verify email capabilities from the SDK

Once a Gmail account is connected (today: by an operator wiring OAuth tokens into the vault manually; v0.0.2: via the dashboard), agents call email capabilities like any other typed capability:

```python
import asyncio
from kruxos import KruxOS

async def main():
    os = await KruxOS.connect_async(
        endpoint="ws://localhost:7700",
        agent_name="my-agent",
        api_key="7f3a8c1d2e9b5a4f8e6c1d3b7a9f2e5c8d1b4a7f3c9e6d8b1a4c7f2e5d9b8a3c",
        purpose="Gmail test",
    )

    try:
        # Search emails (hits local replica — zero Gmail API calls)
        result = await os.call_async(
            "email.search",
            query="is:unread",
            max_results=5,
        )
        print(f"Found {len(result.data['messages'])} unread messages")

        # Read a specific email
        if result.data["messages"]:
            msg_id = result.data["messages"][0]["id"]
            email = await os.call_async("email.read", message_id=msg_id)
            print(f"Subject: {email.data['subject']}")
            print(f"From: {email.data['sender']}")
    finally:
        await os.close_async()

asyncio.run(main())
```

Expected output:

```
Found 3 unread messages
Subject: Weekly status update
From: team@example.com
```

## Sending email (with the write buffer)

```python
result = await os.call_async(
    "email.send",
    to="colleague@example.com",
    subject="Report ready",
    body="The daily report has been generated and is attached.",
)

print(f"Status: {result.data['status']}")
print(f"Buffer ID: {result.data['buffer_id']}")
print(f"Send at: {result.data['scheduled_send']}")
print(f"Cancel before: {result.data['cancel_deadline']}")
```

The email is held in the write buffer for 30 seconds. During this window:

- The agent can cancel with `email.cancel(buffer_id="buf_abc123")`
- A human can cancel via the dashboard or CLI
- After the deadline, the email is sent to Gmail

## What lands in v0.0.2

- **Operator-facing connection path** — dashboard Gmail-OAuth flow + a CLI subcommand so operators can bind a Google account to the appliance without hand-editing the vault.
- **Slack adapter parity** — Slack uses the same Service Proxy primitives; the operator UX lands at the same time.

The release-notes file `docs/release-notes/v0.0.1.md` records this as a known limitation under "Known limitations → Health-driven A/B rollback automation … and the operator-facing Gmail/Slack OAuth flow ship in v0.0.2."

## Next steps

- [Connecting Services](../guides/connecting-services.md) — the operator-facing flow to connect a Gmail account (dashboard or `kruxos connect gmail`)
- [Managing Agents](../guides/managing-agents.md) — control which agents can access email
- [Policies](../guides/policies.md) — restrict email capabilities per agent
- [Monitoring](../guides/monitoring.md) — monitor sync health and write buffers
