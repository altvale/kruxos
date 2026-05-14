# Communications Capabilities

Agent-to-agent messaging, broadcast, pub/sub channels, and inbox polling.

## Overview

| Capability | Permission | Purpose |
|------------|:----------:|---------|
| [`comms.send`](#commssend) | 🔵 Notify | Sends a message to a specific agent by name. |
| [`comms.broadcast`](#commsbroadcast) | 🔵 Notify | Sends a message to all agents matching an optional filter. |
| [`comms.subscribe`](#commssubscribe) | 🟢 Autonomous | Subscribes to a named pub/sub channel. |
| [`comms.receive`](#commsreceive) | 🟢 Autonomous | Polls for pending messages in the agent's inbox, including direct messages and channel subscriptions. |

## `comms.send`

**Permission:** 🔵 Notify · **Version:** 1.0

> Sends a message to a specific agent by name. The message is queued if the recipient is not currently connected.

### When to use

Use comms.send to communicate with another agent directly. Messages are delivered
immediately if the recipient is connected, or queued for delivery when they reconnect.
Use comms.broadcast to send to multiple agents. Use comms.receive to check for incoming messages.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `to` | `AgentId` | Yes | — | Name of the recipient agent. |
| `message` | `String` | Yes | — | Message content (plain text or JSON string). |
| `priority` | `String` | No | `normal` | Message priority: 'low', 'normal', 'high'. High-priority messages appear first in the recipient's queue. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `message_id` | `String` | Unique identifier for the sent message. |
| `status` | `String` | Delivery status: 'delivered' (recipient is connected) or 'queued' (recipient offline, will receive on reconnect). |
| `to` | `AgentId` | The recipient agent name (echo of input). |

### Side effects

- Delivers a message to another agent's message queue. *(not reversible)*

### Common patterns

**Request another agent to perform a task**

1. `comms.send(to='data-processor', message='{"action": "process", "file": "/workspace/data.csv"}')`
2. `comms.receive() later to check for the response`

**Notify another agent of completion**

1. `comms.send(to='orchestrator', message='Task XYZ completed successfully')`

### Errors

**`RecipientNotFound`** — No agent exists with the specified name.

- **check_agents**: Verify the agent name is correct.

**`CommsError`** — Failed to send the message.

- **retry**: Retry the operation.

**Tags:** `comms` `messaging` `agent-to-agent`

---

## `comms.broadcast`

**Permission:** 🔵 Notify · **Version:** 1.0

> Sends a message to all agents matching an optional filter. Messages are queued for offline agents.

### When to use

Use comms.broadcast to send a message to multiple agents at once.
Use comms.send for targeted messages to a specific agent.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `message` | `String` | Yes | — | Message content (plain text or JSON string). |
| `filter` | `Object` | No | — | Optional filter to limit recipients. Fields: policy_group (string), name_pattern (glob pattern). Omit to broadcast to all agents. |
| `priority` | `String` | No | `normal` | Message priority: 'low', 'normal', 'high'. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `message_id` | `String` | Unique identifier for the broadcast message. |
| `delivery_count` | `Integer` | Number of agents the message was delivered or queued to. |
| `delivered` | `Integer` | Number of agents that received the message immediately (connected). |
| `queued` | `Integer` | Number of agents where the message was queued (offline). |

### Side effects

- Delivers a message to multiple agents' message queues. *(not reversible)*

### Common patterns

**Announce a system event to all agents**

1. `comms.broadcast(message='System maintenance starting in 10 minutes', priority='high')`

### Errors

**`CommsError`** — Failed to broadcast the message.

- **retry**: Retry the operation.

**Tags:** `comms` `messaging` `broadcast`

---

## `comms.subscribe`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Subscribes to a named pub/sub channel. Messages published to the channel will be delivered to this agent.

### When to use

Use comms.subscribe to listen for messages on a topic channel (e.g. 'build-events', 'alerts').
Use comms.receive to poll for messages. Use comms.send for direct agent-to-agent messaging.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `channel` | `String` | Yes | — | Name of the channel to subscribe to. Channels are created on first subscription. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `subscription_id` | `String` | Unique identifier for this subscription. Use to unsubscribe later. |
| `channel` | `String` | The channel name (echo of input). |

### Side effects

- Registers the agent as a subscriber on the named channel. Future messages on this channel will be delivered. *(reversible)*

### Common patterns

**Subscribe to build events**

1. `comms.subscribe(channel='build-events')`
2. `comms.receive() periodically to check for new events`

### Errors

**`CommsError`** — Failed to subscribe to the channel.

- **retry**: Retry the operation.

**Tags:** `comms` `pubsub` `subscribe`

---

## `comms.receive`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Polls for pending messages in the agent's inbox, including direct messages and channel subscriptions.

### When to use

Use comms.receive to check for incoming messages from other agents or channel subscriptions.
Call periodically to process incoming requests or notifications.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `limit` | `Integer` | No | `10` | Maximum number of messages to return. |
| `channel` | `String` | No | — | If specified, only return messages from this channel. Omit to receive all pending messages. |
| `acknowledge` | `Boolean` | No | `True` | If true, mark returned messages as received (they will not appear again). If false, messages remain in queue. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `messages` | `Array` | Array of message objects, each with: message_id, from (sender agent or channel name), message (content), priority, sent_at (ISO 8601), channel (null for direct messages). |
| `remaining` | `Integer` | Number of messages remaining in the queue after this fetch. |

### Side effects

- If acknowledge=true (default), marks returned messages as received, removing them from the queue. *(not reversible)*

### Common patterns

**Process all pending messages**

1. `comms.receive(limit=50) to fetch pending messages`
2. `Process each message`
3. `Repeat if remaining > 0`

**Peek at messages without consuming**

1. `comms.receive(acknowledge=false) to see messages`
2. `Decide which to process, then receive with acknowledge=true`

### Errors

**`CommsError`** — Failed to receive messages.

- **retry**: Retry the operation.

**Tags:** `comms` `messaging` `receive` `read`

---
