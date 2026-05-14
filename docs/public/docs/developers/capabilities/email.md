# Email (Gmail) Capabilities

Search, read, send, delete, move, and draft emails via the Service Proxy.

## Overview

| Capability | Permission | Purpose |
|------------|:----------:|---------|
| [`email.search`](#emailsearch) | 🟢 Autonomous | Searches emails in the local read-replica using text, label, and read-status filters. |
| [`email.read`](#emailread) | 🟢 Autonomous | Reads a single email message by ID, returning full metadata and body content. |
| [`email.send`](#emailsend) | 🔵 Notify | Sends an email message. |
| [`email.delete`](#emaildelete) | 🔵 Notify | Soft-deletes an email by moving it to Gmail's Trash. |
| [`email.move`](#emailmove) | 🟢 Autonomous | Moves an email by modifying its Gmail labels (adding and/or removing labels). |
| [`email.draft`](#emaildraft) | 🟢 Autonomous | Creates a draft email in Gmail. |
| [`email.bulk_delete`](#emailbulk_delete) | 🟡 Approval Required | Deletes multiple emails in a single operation. |

## `email.search`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Searches emails in the local read-replica using text, label, and read-status filters. Executes entirely against the local SyncStore — zero Gmail API calls.

### When to use

Use email.search to find emails matching criteria before reading or acting on them.
Use email.read to get the full body of a specific message found via search.
Results are from the local replica, which syncs every 5 minutes by default.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `query` | `String` | No | — | Free-text search across from, to, subject, and snippet fields. Case-insensitive substring match. |
| `label` | `String` | No | — | Filter by Gmail label (e.g., 'INBOX', 'SENT', 'IMPORTANT'). Exact match against the message's label list. |
| `is_read` | `Boolean` | No | — | Filter by read status. true = read messages only, false = unread only, omit for all. |
| `limit` | `Integer` | No | `20` | Maximum number of messages to return per page. Max 100. |
| `offset` | `Integer` | No | `0` | Number of results to skip for pagination. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `messages` | `Array` | List of matching message objects with fields: id, thread_id, from, to, subject, snippet, labels, date, is_read, has_attachments. |
| `total` | `Integer` | Total number of matching messages (may exceed messages array length when paginated). |

### Common patterns

**Find unread emails from a specific sender**

1. `email.search(query='alice@example.com', is_read=false)`

**Browse inbox with pagination**

1. `email.search(label='INBOX', limit=10, offset=0) for page 1`
2. `email.search(label='INBOX', limit=10, offset=10) for page 2`

### Errors

**`DatabaseError`** — Local replica database is corrupted or unavailable.

- **retry_later**: Wait for the next sync cycle to repair the replica.

**Tags:** `email` `gmail` `read` `search` `safe`

---

## `email.read`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Reads a single email message by ID, returning full metadata and body content. Metadata comes from the local replica; the full body is fetched from Gmail on first access and cached locally.

### When to use

Use email.read after email.search to get the full body of a specific message.
The first read of a message body makes one Gmail API call; subsequent reads are local.
Use email.search to find message IDs before calling email.read.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `message_id` | `String` | Yes | — | Gmail message ID. Obtain from email.search results. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `message` | `Object` | Message metadata: id, thread_id, from, to, subject, snippet, labels, date, is_read, has_attachments. |
| `body` | `Object` | Message body with fields: message_id, text_plain (plain text content), text_html (HTML content), cached_at (when the body was fetched). |

### Side effects

- First read triggers a Gmail API call to fetch the full body (cached for future reads). *(not reversible)*

### Common patterns

**Search and read a specific email**

1. `email.search(query='invoice') to find matching messages`
2. `email.read(message_id=<id from search>) to get full body`

### Errors

**`MessageNotFound`** — No message with this ID exists in the local replica.

- **search**: Use email.search to find valid message IDs.
- **wait_for_sync**: The message may appear after the next sync cycle.

**`BodyFetchFailed`** — Failed to fetch the full body from Gmail API.

- **retry_later**: Wait and retry — the Gmail API may be temporarily unavailable.

**Tags:** `email` `gmail` `read` `safe`

---

## `email.send`

**Permission:** 🔵 Notify · **Version:** 1.0

> Sends an email message. The send is buffered in the Write Proxy with a 5-minute delay, during which it can be cancelled. The supervisor is notified.

### When to use

Use email.send to compose and send an email. The message is not sent immediately —
it is held in a 5-minute buffer. During this time, the supervisor or agent can cancel it.
Use email.draft instead if you want to save without sending.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `to` | `String` | Yes | — | Recipient email address. For multiple recipients, use comma-separated addresses. |
| `subject` | `String` | No | `(no subject)` | Email subject line. |
| `body` | `String` | No | `` | Plain text email body content. |
| `from` | `String` | No | `me` | Sender address. Defaults to the authenticated Gmail account. |
| `cc` | `String` | No | — | Carbon copy recipients (comma-separated). |
| `bcc` | `String` | No | — | Blind carbon copy recipients (comma-separated). |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `write_id` | `String` | ID of the buffered write. Use this to cancel the send during the buffer period. |
| `buffer_until` | `DateTime` | When the buffer expires and the email will actually be sent. |
| `status` | `String` | Write status — 'buffered' while waiting, 'completed' after sending. |

### Side effects

- Creates a buffered write in the Write Proxy (5-minute delay). *(reversible)*
- Supervisor receives a notification about the pending send. *(not reversible)*
- After buffer expires, sends the email via Gmail API. *(not reversible)*

### Common patterns

**Send an email with cancellation option**

1. `email.send(to='bob@example.com', subject='Hello', body='Hi Bob!') — returns write_id`
2. `To cancel before it sends: proxy.cancel_write(write_id=<id>)`

**Send and verify delivery**

1. `email.send(to='bob@example.com', subject='Report', body='...')`
2. `Wait for buffer period, then email.search(label='SENT', query='Report') to confirm`

### Errors

**`MissingParameter`** — Required parameter 'to' is missing.

- **provide_recipient**: Include the 'to' parameter with a valid email address.

**`BatchProtection`** — More than 20 sends per hour — operation requires supervisor approval.

- **request_approval**: Request supervisor approval for high-volume sending.
- **wait**: Wait for the hourly window to reset.

**`WriteFailed`** — Gmail API rejected the message (invalid recipient, auth error, etc.).

- **check_parameters**: Verify the recipient address and message content.
- **retry_later**: Wait and retry if the error is transient.

**Tags:** `email` `gmail` `write` `buffered` `cancellable`

---

## `email.delete`

**Permission:** 🔵 Notify · **Version:** 1.0

> Soft-deletes an email by moving it to Gmail's Trash. The deletion is buffered for 24 hours, during which it can be cancelled. Supervisor is notified.

### When to use

Use email.delete to remove an email. The message is moved to Trash (not permanently deleted),
and the operation is buffered for 24 hours — plenty of time for a supervisor to intervene.
For bulk deletions (>5 in a session), supervisor approval is required automatically.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `message_id` | `String` | Yes | — | Gmail message ID to delete. Obtain from email.search results. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `write_id` | `String` | ID of the buffered write. Use this to cancel the deletion during the 24h buffer. |
| `buffer_until` | `DateTime` | When the buffer expires and the message will be trashed. |
| `status` | `String` | Write status — 'buffered' while waiting. |

### Side effects

- Creates a buffered write in the Write Proxy (24-hour delay). *(reversible)*
- Supervisor receives a notification about the pending deletion. *(not reversible)*
- After buffer expires, moves the message to Gmail Trash. *(reversible)*
- Rollback point is created with the message's content and labels for potential undo. *(not reversible)*

### Common patterns

**Delete a specific email with undo window**

1. `email.search(query='spam offer') to find the message`
2. `email.delete(message_id=<id>) — 24h buffer, cancellable`

**Cancel a pending deletion**

1. `email.delete(message_id=<id>) — returns write_id`
2. `proxy.cancel_write(write_id=<id>) — cancels before execution`

### Errors

**`MissingParameter`** — Required parameter 'message_id' is missing.

- **search_first**: Use email.search to find the message ID.

**`BatchProtection`** — More than 5 deletions in this session — operation requires supervisor approval.

- **request_approval**: Request supervisor approval for batch deletions.

**`MessageNotFound`** — No message with this ID exists.

- **search**: Use email.search to find valid message IDs.

**Tags:** `email` `gmail` `write` `buffered` `soft-delete` `cancellable`

---

## `email.move`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Moves an email by modifying its Gmail labels (adding and/or removing labels). Buffered for 1 hour before execution.

### When to use

Use email.move to reorganize emails by changing their labels (e.g., move from Inbox to a folder).
Gmail uses labels instead of folders — 'moving' means adding the destination label
and optionally removing the source label.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `message_id` | `String` | Yes | — | Gmail message ID to move. Obtain from email.search results. |
| `add_labels` | `Array` | No | — | Labels to add to the message (e.g., ['IMPORTANT', 'STARRED']). |
| `remove_labels` | `Array` | No | — | Labels to remove from the message (e.g., ['INBOX']). |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `write_id` | `String` | ID of the buffered write. Use this to cancel during the 1h buffer. |
| `buffer_until` | `DateTime` | When the buffer expires and the label change will be applied. |
| `status` | `String` | Write status — 'buffered' while waiting. |

### Side effects

- Creates a buffered write in the Write Proxy (1-hour delay). *(reversible)*
- After buffer expires, modifies labels on the message via Gmail API. *(reversible)*
- Rollback point records the original labels for potential undo. *(not reversible)*

### Common patterns

**Archive an email (remove from Inbox)**

1. `email.move(message_id=<id>, remove_labels=['INBOX'])`

**Star and mark important**

1. `email.move(message_id=<id>, add_labels=['STARRED', 'IMPORTANT'])`

### Errors

**`MissingParameter`** — Required parameter 'message_id' is missing.

- **search_first**: Use email.search to find the message ID.

**`NoLabelsProvided`** — Neither add_labels nor remove_labels was provided.

- **specify_labels**: Provide at least one of add_labels or remove_labels.

**Tags:** `email` `gmail` `write` `buffered` `labels`

---

## `email.draft`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Creates a draft email in Gmail. The draft is pushed immediately (no buffer delay) and can be edited or sent later.

### When to use

Use email.draft to compose an email without sending it. The draft is saved to Gmail
and can be edited in the Gmail UI or sent later.
Use email.send instead if you want to send the message (with the 5-minute safety buffer).

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `to` | `String` | Yes | — | Intended recipient email address. |
| `subject` | `String` | No | `(no subject)` | Draft subject line. |
| `body` | `String` | No | `` | Plain text draft body content. |
| `from` | `String` | No | `me` | Sender address. Defaults to the authenticated Gmail account. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `draft_id` | `String` | Gmail draft ID. Can be used to reference the draft later. |
| `to` | `String` | The recipient address the draft is addressed to. |
| `subject` | `String` | The subject line of the draft. |

### Side effects

- Creates a draft in the user's Gmail Drafts folder via the Gmail API. *(reversible)*

### Common patterns

**Create a draft for later review**

1. `email.draft(to='boss@company.com', subject='Weekly Report', body='...')`
2. `Notify supervisor that a draft is ready for review`

### Errors

**`MissingParameter`** — Required parameter 'to' is missing.

- **provide_recipient**: Include the 'to' parameter with a valid email address.

**`WriteFailed`** — Gmail API rejected the draft creation.

- **retry_later**: Wait and retry — the Gmail API may be temporarily unavailable.

**Tags:** `email` `gmail` `write` `draft` `safe`

---

## `email.bulk_delete`

**Permission:** 🟡 Approval Required · **Version:** 1.0

> Deletes multiple emails in a single operation. BLOCKED BY DEFAULT — requires explicit policy enablement by a supervisor.

### When to use

DO NOT use this operation unless it has been explicitly enabled by policy.
By default, email.bulk_delete is blocked to prevent accidental mass deletion.
Use email.delete for individual messages instead (with its 24h safety buffer).

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `message_ids` | `Array` | Yes | — | List of Gmail message IDs to delete. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `error` | `String` | Error message explaining that the operation is blocked. |

### Common patterns

**Use individual deletes instead**

1. `email.search(query='old newsletter') to find messages`
2. `email.delete(message_id=<id>) for each message individually`

### Errors

**`OperationBlocked`** — This operation is blocked by default. Contact your supervisor to enable it via policy.

- **use_individual_delete**: Use email.delete for individual messages instead.
- **request_policy_change**: Ask a supervisor to enable email.bulk_delete in the policy configuration.

**Tags:** `email` `gmail` `write` `destructive` `blocked`

---
