# Service Proxy Adapters

The Service Proxy framework provides safe, buffered access to external services. Each service adapter implements three composable traits to integrate with the framework.

## Architecture

```
Agent
  |
  v
Capability Handler          (validates inputs, creates BufferedWrite)
  |
  v
WriteBuffer                 (SQLite queue, configurable delay per operation)
  |
  v
EnhancedWriteExecutor       (creates rollback point, executes, handles DLQ)
  |
  v
WriteExecutor (adapter)     (service-specific API call)
  |
  v
External Service API        (Slack, Gmail, etc.)
```

Read operations bypass the write pipeline entirely and query the local SyncStore (SQLite read-replica), which syncs periodically from the external service.

## Framework Traits

Every service adapter implements up to three traits from `crates/proxy/src/framework.rs`:

### `SyncAdapter`

Syncs data from the external service into a local SQLite read-replica.

```rust
#[async_trait]
pub trait SyncAdapter: Send + Sync {
    async fn full_sync(&self, store: &SyncStore) -> Result<SyncResult, ProxyError>;
    async fn incremental_sync(
        &self,
        store: &SyncStore,
        last_sync: DateTime<Utc>,
    ) -> Result<SyncResult, ProxyError>;
}
```

### `WriteExecutor`

Executes buffered write operations against the external service API.

```rust
#[async_trait]
pub trait WriteExecutor: Send + Sync {
    async fn execute_write(
        &self,
        write: &BufferedWrite,
    ) -> Result<serde_json::Value, ProxyError>;
}
```

### `RollbackCapable`

Provides undo capability for write operations.

```rust
#[async_trait]
pub trait RollbackCapable: Send + Sync {
    fn supports_rollback(&self, operation: &str) -> bool;
    async fn create_rollback_point(
        &self,
        write: &BufferedWrite,
    ) -> Result<RollbackPoint, ProxyError>;
    async fn rollback(&self, point: &RollbackPoint) -> Result<(), ProxyError>;
}
```

## Available Adapters

### Gmail (`crates/proxy/src/adapters/gmail.rs`)

| Capability | Type | Buffer Delay | Rollback |
|-----------|------|-------------|----------|
| `email.search` | read | — | N/A |
| `email.read` | read | — | N/A |
| `email.send` | write | 5 minutes | No |
| `email.delete` | write | 24 hours | Yes (untrash) |
| `email.move` | write | 1 hour | Yes (reverse labels) |
| `email.draft` | write | immediate | No |
| `email.bulk_delete` | write | — | Blocked by default |

**OAuth callback port**: 8080

!!! info "Operator-facing connection flow ships in v0.0.2"
    The Gmail adapter is wired end-to-end in v0.0.1 — read-replica, write buffer, batch protection, and vault-backed token storage with auto-refresh all work. What v0.0.1 doesn't ship is the dashboard Gmail-OAuth flow / `kruxos connect gmail` CLI subcommand; that operator UX lands in v0.0.2.

### Slack (`crates/proxy/src/adapters/slack.rs`)

| Capability | Type | Buffer Delay | Rollback |
|-----------|------|-------------|----------|
| `slack.search` | read | — | N/A |
| `slack.read` | read | — | N/A |
| `slack.channels` | read | — | N/A |
| `slack.send` | write | 30 seconds | Yes (chat.delete) |
| `slack.reply` | write | 30 seconds | Yes (chat.delete) |
| `slack.react` | write | immediate | Yes (reactions.remove) |
| `slack.remove_react` | write | immediate | No |

**OAuth callback port**: 8081
**Batch protection**: 10 messages/hour per channel

!!! info "Slack operator UX lands in v0.0.2"
    Same status as Gmail: the Slack adapter is fully wired in v0.0.1, but the operator-facing connection flow (dashboard or CLI) ships in v0.0.2.

## Key Concepts

### Write Buffering

All write operations pass through the `WriteBuffer`, which stores them in SQLite with a configurable delay. During the buffer window:

- The agent receives a `write_id` immediately
- The supervisor can view pending writes via `kruxos approve list`
- The agent or supervisor can cancel with the `write_id`
- After the delay expires, the `EnhancedWriteExecutor` picks up and executes the write

### Batch Protection

Each adapter configures a `WriteProxyConfig` with batch and hourly thresholds. When an agent exceeds the threshold, additional writes are rejected with a batch protection error, preventing message flooding.

### Rollback Points

Before executing a write, the framework calls `create_rollback_point()` to capture state needed for an undo. Rollback points expire after a service-specific retention period (24h for Slack, 72h for Gmail). Expired points are cleaned up automatically.

### Dead Letter Queue

If a write execution fails (API error, network issue), the write is moved to the Dead Letter Queue (DLQ) for manual retry or investigation. The DLQ uses exponential backoff for automatic retries.

## Building a New Adapter

To add a new service adapter:

1. **Create the adapter module**: `crates/proxy/src/adapters/<service>.rs`
2. **Define an API client trait**: Abstract the external API for testability
3. **Implement `SyncAdapter`**: Define full and incremental sync logic
4. **Implement `WriteExecutor`**: Map operations to API calls
5. **Implement `RollbackCapable`**: Define undo logic where possible
6. **Create capability definitions**: `definitions/<service>.yaml`
7. **Create capability handlers**: `crates/capabilities/src/<service>.rs`
8. **Create default policies**: `policies/<service>.yaml`
9. **Register the module**: Add `pub mod <service>;` to `crates/proxy/src/adapters/mod.rs`

Follow the Gmail and Slack adapters as reference implementations.
