# Slack Integration

Connect your KruxOS instance to Slack so agents can search messages, post to channels, and react to messages — all through the Service Proxy's safety layer.

!!! warning "Operator-facing connection flow ships in v0.0.2"
    The Slack adapter (search, read, channels, send, reply, react, remove_react) is **wired end-to-end in v0.0.1** — the read-replica sync, write buffer, batch protection, and vault-backed token storage with auto-refresh all work. What v0.0.1 does **not** yet ship is the operator-facing OAuth connection UX — the `kruxos connect slack` CLI subcommand and the dashboard Slack-OAuth flow both land in **v0.0.2**. This page describes the runtime behaviour that's in place today; the wiring described under "Connecting Slack" is the v0.0.2 surface.

## Prerequisites

- A running KruxOS instance (Docker via `altvale/kruxos`, or the VM image)
- A Slack workspace where you have permission to install apps
- A Slack App with a Bot Token (see [Creating a Slack App](#creating-a-slack-app))

## Creating a Slack App

1. Go to [api.slack.com/apps](https://api.slack.com/apps) and click **Create New App**
2. Choose **From scratch**, give it a name (e.g. "KruxOS"), and select your workspace
3. Under **OAuth & Permissions**, add these **Bot Token Scopes**:
   - `channels:read` — list channels and metadata
   - `channels:history` — read message history
   - `chat:write` — post messages and replies
   - `reactions:write` — add and remove reactions
   - `users:read` — resolve user names
4. Under **OAuth & Permissions**, set the **Redirect URL** to `http://127.0.0.1:8081`
5. Note your **Client ID** from the **Basic Information** page

## Connecting Slack (v0.0.2)

Run the connect command:

```bash
kruxos connect slack --client-id <YOUR_CLIENT_ID>
```

In v0.0.2 this will:

1. Open a browser window for Slack OAuth authorization
2. Start a local callback server on port 8081
3. Exchange the authorization code for access tokens (PKCE flow)
4. Store the encrypted tokens in the KruxOS vault
5. Initialize the local read-replica database
6. Start background sync (every 60 seconds by default)

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

## Rollback

Sent messages and replies can be rolled back (deleted) for up to **24 hours** after sending. Reactions can also be rolled back (removed). Use the rollback system through the Service Proxy dashboard or CLI:

```bash
kruxos approve list --service slack    # view recent writes
kruxos approve rollback <write_id>     # undo a write
```

## Sync Configuration

The Slack read-replica syncs automatically:

| Setting | Default | Description |
|---------|---------|-------------|
| Sync interval | 60 seconds | How often new messages are fetched |
| Initial sync | Last 100 messages per channel | First sync pulls recent history |
| Incremental sync | New messages only | Subsequent syncs use `oldest` parameter |

Messages in the local replica are typically less than 60 seconds behind live Slack.

## Disconnecting (v0.0.2)

```bash
kruxos disconnect slack
```

In v0.0.2 this will:

1. Revoke the OAuth tokens
2. Stop background sync
3. Cancel any buffered (unsent) writes
4. Delete the local read-replica database
5. Remove tokens from the vault

## Troubleshooting

**"Slack service is not connected"**: The operator-facing `kruxos connect slack` subcommand and the dashboard Slack-OAuth flow ship in v0.0.2. Until then, seed the vault entry manually — see [Service Proxy Adapters](../developers/services.md) for the layout. Read operations require the sync to be active.

**"Slack write operation failed"**: Check that the bot has been invited to the target channel. Slack bots can only post to channels they've been added to.

**"Batch protection triggered"**: The agent has exceeded the per-channel message limit. Either wait for the hourly window to reset, or approve the write via `kruxos approve`.

**Stale search results**: The local replica syncs every 60 seconds. Very recent messages may not appear immediately.
