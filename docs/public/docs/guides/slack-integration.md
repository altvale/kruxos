# Slack Integration

Connect your KruxOS instance to Slack so agents can search messages, post to channels, and react to messages — all through the Service Proxy's safety layer.

This page covers **what agents can do with Slack once it's connected**. For the
one-time connect step — creating the Slack app and pasting its token — see
**[Connecting Services](connecting-services.md)**.

## Connecting Slack

Slack connects by pasting a **Bot User OAuth Token** (`xoxb-…`) — there's no
browser OAuth dance, because the bot token *is* the bearer the Service Proxy
uses. You create a Slack app from the provided manifest, install it to your
workspace, and hand KruxOS the token, either from the dashboard **Service
Proxy** page (`/proxy`) Connect tile or with:

```bash
kruxos connect slack
```

The full walkthrough — the app manifest (which requests exactly the scopes the
proxy needs), the install steps, and where to find the `xoxb-` token — lives in
[Connecting Services → Slack](connecting-services.md#slack). Run
`kruxos connect status` to confirm the connection. Once connected, the
capabilities below become available to agents.

## What Agents Can Do

### Read Operations (Autonomous)

These run against the local read-replica — zero Slack API calls, no rate limits.

| Capability | Description |
|-----------|-------------|
| `slack.search` | Search messages by channel, user, text, or date range |
| `slack.read` | Read a specific message and its thread replies |
| `slack.channels` | List all available channels |

### Write Operations

| Capability | Tier | Buffer | Description |
|-----------|------|--------|-------------|
| `slack.send` | notify | 30 seconds | Post a message to a channel |
| `slack.reply` | notify | 30 seconds | Post a threaded reply |
| `slack.react` | autonomous | immediate | Add a reaction emoji |
| `slack.remove_react` | autonomous | immediate | Remove a reaction emoji |

**Buffer window**: `slack.send` and `slack.reply` are held for 30 seconds before executing. During this window, you (or the agent) can cancel with the returned `write_id`. Reactions execute immediately since they're low-risk and easily reversible.

### Batch Protection

Agents are limited to **10 messages per hour per channel** by default. Exceeding this threshold escalates the operation to `approval_required`. Configure thresholds in your policy file.

## Example Agent Usage (Python SDK)

```python
from kruxos import KruxOS

async with KruxOS.connect() as agent:
    # Find channels
    channels = await agent.call("slack.channels")
    
    # Search for messages
    results = await agent.call("slack.search", {
        "channel": "engineering",
        "query": "deployment",
        "limit": 10
    })
    
    # Read a specific message and thread
    message = await agent.call("slack.read", {
        "message_id": "C01234567/1700000001.000001"
    })
    
    # Post a message (buffered 30 seconds)
    result = await agent.call("slack.send", {
        "channel": "C01234567",
        "text": "Deployment complete!"
    })
    # result["write_id"] can be used to cancel
    
    # React to a message (immediate)
    await agent.call("slack.react", {
        "channel": "C01234567",
        "ts": "1700000001.000001",
        "reaction": "white_check_mark"
    })
```

## Cancelling and approving writes

`slack.send` and `slack.reply` are held in the write buffer for 30 seconds
before they execute. During that window you can act on them from the dashboard
**Service Proxy** page (`/proxy`), which lists buffered writes by `write_id`
and offers **cancel**, **retry**, and **discard** actions. Reactions execute
immediately and aren't buffered.

When batch protection escalates an operation to `approval_required`, it lands
in the approval queue:

```bash
kruxos approve list            # pending approval requests
kruxos approve accept <id>     # let it through
kruxos approve reject <id> --reason "…"
```

## Sync Configuration

The Slack read-replica syncs automatically:

| Setting | Default | Description |
|---------|---------|-------------|
| Sync interval | 60 seconds | How often new messages are fetched |
| Initial sync | Last 100 messages per channel | First sync pulls recent history |
| Incremental sync | New messages only | Subsequent syncs use `oldest` parameter |

Messages in the local replica are typically less than 60 seconds behind live Slack.

## Disconnecting

To revoke access, **uninstall the app from your Slack workspace** (Slack app
settings → *Install App* → *Revoke*). That invalidates the `xoxb-` bot token,
so the Service Proxy can no longer reach Slack. To swap in a different token,
use the **Reconnect** action on the dashboard Service Proxy tile (or re-run
`kruxos connect slack`), which replaces the stored token.

## Troubleshooting

**"Slack service is not connected"**: No Slack token is stored yet. Connect the
service first — see [Connecting Services → Slack](connecting-services.md#slack).
Read operations also require the sync to have run at least once.

**"Slack write operation failed"**: Check that the bot has been invited to the target channel. Slack bots can only post to channels they've been added to.

**"Batch protection triggered"**: The agent has exceeded the per-channel message limit. Either wait for the hourly window to reset, or approve the write via `kruxos approve`.

**Stale search results**: The local replica syncs every 60 seconds. Very recent messages may not appear immediately.
