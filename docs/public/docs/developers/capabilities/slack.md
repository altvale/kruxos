# Slack Capabilities

Search, read, send, reply, react, and list Slack channels via the Service Proxy.

## Overview

| Capability | Permission | Purpose |
|------------|:----------:|---------|
| [`slack.search`](#slacksearch) | 🟢 Autonomous | Searches Slack messages in the local read-replica using channel, user, text, and date filters. |
| [`slack.read`](#slackread) | 🟢 Autonomous | Reads a specific Slack message or thread by ID from the local read-replica. |
| [`slack.channels`](#slackchannels) | 🟢 Autonomous | Lists all Slack channels available in the local read-replica. |
| [`slack.send`](#slacksend) | 🔵 Notify | Posts a message to a Slack channel. Buffered for 30 seconds. |
| [`slack.reply`](#slackreply) | 🔵 Notify | Posts a threaded reply to an existing Slack message. Buffered for 30 seconds. |
| [`slack.react`](#slackreact) | 🟢 Autonomous | Adds a reaction emoji to a Slack message. Executed immediately. |
| [`slack.remove_react`](#slackremove_react) | 🟢 Autonomous | Removes a reaction emoji from a Slack message. Executed immediately. |

## `slack.search`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Searches Slack messages in the local read-replica using channel, user, text, and date filters. Executes entirely against the local SyncStore — zero Slack API calls.

### When to use

Use slack.search to find messages matching criteria before reading or acting on them.
Use slack.read to get a specific message or thread found via search.
Results are from the local replica, which syncs every 60 seconds by default.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `channel` | `String` | No | — | Filter by channel name or ID (e.g., 'general' or 'C01234567'). Matches either the channel name or ID. |
| `user` | `String` | No | — | Filter by user name or ID. Matches user_name (substring) or user_id (exact). |
| `query` | `String` | No | — | Free-text search across message text. Case-insensitive substring match. |
| `from_date` | `String` | No | — | Only return messages with ts >= this value. RFC 3339 format or Slack timestamp. |
| `to_date` | `String` | No | — | Only return messages with ts <= this value. RFC 3339 format or Slack timestamp. |
| `limit` | `Integer` | No | `20` | Maximum number of messages to return per page. Max 100. |
| `offset` | `Integer` | No | `0` | Number of results to skip for pagination. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `messages` | `Array` | List of matching message objects with fields: id, channel_id, user_id, user_name, text, ts, thread_ts, reactions, synced_at. |
| `total` | `Integer` | Total number of matching messages (may exceed messages array length when paginated). |

### Common patterns

**Find messages from a specific user in a channel**

1. `slack.search(channel='engineering', user='Alice')`

**Search for messages mentioning a topic**

1. `slack.search(query='deployment', limit=10)`

### Errors

**`DatabaseError`** — Local replica database is corrupted or unavailable.

- **retry_later**: Wait for the next sync cycle to repair the replica.

**Tags:** `slack` `messaging` `read` `search` `safe`

---

## `slack.read`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Reads a specific Slack message or thread by ID from the local read-replica. If the message is a thread parent, thread replies are included.

### When to use

Use slack.read after slack.search to get a specific message and its thread replies.
Use slack.search to find message IDs before calling slack.read.
The message ID format is 'channel_id/ts' (e.g., 'C01234567/1700000001.000001').

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `message_id` | `String` | Yes | — | Slack message ID in 'channel_id/ts' format. Obtain from slack.search results. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `message` | `Object` | Message object with fields: id, channel_id, user_id, user_name, text, ts, thread_ts, reactions, synced_at. |
| `thread_replies` | `Array` | Thread replies if the message is a thread parent, sorted by timestamp ascending. Empty array if not a thread parent. |

### Common patterns

**Search and read a specific message**

1. `slack.search(query='deployment') to find matching messages`
2. `slack.read(message_id=<id from search>) to get full message and thread`

### Errors

**`MessageNotFound`** — No message with this ID exists in the local replica.

- **search**: Use slack.search to find valid message IDs.
- **wait_for_sync**: The message may appear after the next sync cycle.

**Tags:** `slack` `messaging` `read` `safe`

---

## `slack.channels`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Lists all Slack channels available in the local read-replica. Returns channel metadata including name, topic, member count, and privacy status.

### When to use

Use slack.channels to discover available channels before searching for messages.
Use this to find the channel ID needed for slack.search or slack.send.

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `channels` | `Array` | List of channel objects with fields: id, name, topic, member_count, is_private, synced_at. Sorted alphabetically by name. |

### Common patterns

**List channels then search a specific one**

1. `slack.channels() to get the list of channels`
2. `slack.search(channel='engineering') to find messages in that channel`

### Errors

**`DatabaseError`** — Local replica database is corrupted or unavailable.

- **retry_later**: Wait for the next sync cycle to repair the replica.

**Tags:** `slack` `messaging` `read` `safe`

---

## `slack.send`

**Permission:** 🔵 Notify · **Version:** 1.0

> Posts a message to a Slack channel. The message is buffered for 30 seconds before sending, allowing cancellation.

### When to use

Use slack.send to post a message to a channel.
The message is buffered for 30 seconds — use the returned write_id to cancel if needed.
Use slack.channels to find the correct channel ID first.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `channel` | `String` | Yes | — | Channel ID to post to (e.g., 'C01234567'). Use slack.channels to find IDs. |
| `text` | `String` | Yes | — | Message text to post. Supports Slack's mrkdwn formatting. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `write_id` | `String` | Unique ID for the buffered write. Use this to cancel within the buffer window. |
| `buffer_until` | `String` | ISO 8601 timestamp when the buffer expires and the message will be sent. |
| `status` | `String` | Always 'buffered' on success. |

### Side effects

- A message will be posted to the Slack channel after the 30-second buffer expires. *(reversible)*

### Common patterns

**Post a message to a channel**

1. `slack.channels() to find the channel ID`
2. `slack.send(channel='C01234567', text='Hello team!')`

### Errors

**`WriteFailed`** — Failed to buffer the write operation.

- **retry_later**: Wait and retry the operation.

**`BatchProtection`** — Too many messages sent to this channel recently.

- **request_approval**: Request supervisor approval for additional messages.

**Tags:** `slack` `messaging` `write` `buffered`

---

## `slack.reply`

**Permission:** 🔵 Notify · **Version:** 1.0

> Posts a threaded reply to an existing Slack message. The reply is buffered for 30 seconds before sending.

### When to use

Use slack.reply to respond in a thread rather than posting to the main channel.
Requires the parent message's ts (Slack timestamp). Find this via slack.search.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `channel` | `String` | Yes | — | Channel ID containing the parent message. |
| `thread_ts` | `String` | Yes | — | Timestamp of the parent message to reply to. Obtain from slack.search or slack.read. |
| `text` | `String` | Yes | — | Reply text. Supports Slack's mrkdwn formatting. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `write_id` | `String` | Unique ID for the buffered write. |
| `buffer_until` | `String` | ISO 8601 timestamp when the buffer expires. |
| `status` | `String` | Always 'buffered' on success. |

### Side effects

- A threaded reply will be posted after the 30-second buffer expires. *(reversible)*

### Common patterns

**Reply to a message in a thread**

1. `slack.search(query='deployment') to find the message`
2. `slack.reply(channel='C01234567', thread_ts='1700000001.000001', text='Looks good!')`

### Errors

**`WriteFailed`** — Failed to buffer the write operation.

- **retry_later**: Wait and retry the operation.

**Tags:** `slack` `messaging` `write` `buffered`

---

## `slack.react`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Adds a reaction emoji to a Slack message. Executed immediately (no buffer) — reactions are low-risk.

### When to use

Use slack.react to add an emoji reaction to a message.
This is immediate (no buffer delay) because reactions are low-risk and easily reversible.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `channel` | `String` | Yes | — | Channel ID containing the message. |
| `ts` | `String` | Yes | — | Timestamp of the message to react to. |
| `reaction` | `String` | Yes | — | Reaction emoji name without colons (e.g., 'thumbsup', 'eyes', 'white_check_mark'). |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `status` | `String` | 'ok' on success. |

### Side effects

- A reaction is added to the message immediately. *(reversible)*

### Common patterns

**React to a message**

1. `slack.react(channel='C01234567', ts='1700000001.000001', reaction='thumbsup')`

### Errors

**`WriteFailed`** — Reaction could not be added (e.g., already reacted, message not found).

- **retry_later**: Wait and retry.

**Tags:** `slack` `messaging` `write` `immediate`

---

## `slack.remove_react`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Removes a reaction emoji from a Slack message. Executed immediately (no buffer).

### When to use

Use slack.remove_react to remove a previously-added emoji reaction from a message.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `channel` | `String` | Yes | — | Channel ID containing the message. |
| `ts` | `String` | Yes | — | Timestamp of the message. |
| `reaction` | `String` | Yes | — | Reaction emoji name to remove (e.g., 'thumbsup'). |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `status` | `String` | 'ok' on success. |

### Side effects

- A reaction is removed from the message immediately. *(reversible)*

### Common patterns

**Remove a reaction**

1. `slack.remove_react(channel='C01234567', ts='1700000001.000001', reaction='thumbsup')`

### Errors

**`WriteFailed`** — Reaction could not be removed (e.g., not reacted, message not found).

- **retry_later**: Wait and retry.

**Tags:** `slack` `messaging` `write` `immediate`

---
