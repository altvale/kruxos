# Troubleshooting

Common issues and their solutions.

## Connection issues

### Agent can't connect to Gateway

**Symptom:** `ConnectionRefusedError` or timeout when connecting.

**Check:**

```bash
# Is the Gateway running?
kruxos status

# Can you reach port 7700?
curl -v ws://localhost:7700
```

**Solutions:**

- **Gateway not running:** `systemctl start kruxos-gateway`
- **Wrong endpoint:** Verify the endpoint URL matches your setup (Docker: `ws://localhost:7700`, VM: `ws://<vm-ip>:7700`)
- **Firewall:** Ensure ports 7700 and 7800 are open
- **Docker networking:** If running inside another container, use `host.docker.internal` instead of `localhost`

### Authentication failed

**Symptom:** `AuthenticationError: Invalid credentials`

**Check:**

```bash
# Is the agent registered?
kruxos agent list

# Is the agent revoked?
kruxos agent show my-agent
```

**Solutions:**

- **Wrong token:** Agent tokens are 64-character hex strings in v0.0.1; User tokens start with `krx_user_`. Check for copy-paste errors (trailing whitespace, truncation). Agent connections use the bare-hex token via the MCP handshake on port 7700; the loopback User API and dashboard use `krx_user_*`.
- **Agent revoked:** Create a new agent with `kruxos agent create`
- **Key rotated:** If you rotated the key, update your agent configuration with the new key

### WebSocket disconnects frequently

**Symptom:** Agent connects but disconnects after a few seconds.

**Solutions:**

- **Network instability:** The SDK auto-reconnects with exponential backoff (1s to 60s). This is usually transient.
- **Session limit:** Check if another instance of the same agent is connected. Only one session per agent is allowed.
- **Resource pressure:** Check `kruxos alerts` for high CPU or memory usage.

## Policy issues

### Capability denied unexpectedly

**Symptom:** `PolicyDeniedError` for a capability you expect to work.

**Debug:**

```bash
# Check what tier the capability gets for this agent
kruxos config check-policy --agent my-agent --capability filesystem.write
```

**Solutions:**

- **Wrong policy tier:** Edit the policy YAML to change the tier
- **Hierarchy override:** A system or org policy may be more restrictive than the agent policy. The hierarchy display shows which layer is blocking.
- **Rate limited:** Check if you've exceeded a rate limit with `kruxos audit query --agent my-agent --last 1h`

### Approval never completes

**Symptom:** Agent waiting for approval that never comes.

**Check:**

```bash
# Is the approval in the queue?
kruxos approve list
```

**Solutions:**

- **No one watching:** Set up `kruxos approve watch` or check the dashboard regularly
- **Approval expired on agent side:** The agent may have timed out. The approval request is still in the queue — the agent can call `wait_for_approval_async` again with the same request ID.
- **Wrong approval queue:** If using per-agent policies, approvals are scoped to the agent's policy. Check the correct queue.

## Service Proxy issues

### Gmail not syncing

**Symptom:** `email.search` returns no results or stale data.

**Check:**

```bash
kruxos status
```

Look for the proxy section:

```
Proxy:      error (Gmail: sync failed 5m ago)
```

**Solutions:**

- **OAuth token expired:** v0.0.1 stores Gmail OAuth tokens with auto-refresh in the vault; if a manual reconnect is needed, the operator-facing dashboard Gmail-OAuth flow / `kruxos connect gmail` CLI subcommand ships in **v0.0.2**. Until then, re-seed the vault entry manually (see [Service Proxy Adapters](../developers/services.md)).
- **Google API quota:** Gmail API has rate limits. Wait and retry.
- **Network issue:** Check internet connectivity from the KruxOS host.

### Email stuck in write buffer

**Symptom:** Email sent but never delivered.

**Check:**

```bash
# Check the dashboard Service Proxy page, or:
kruxos status
```

**Solutions:**

- **Buffer delay:** Emails are held for 30 seconds by default. Wait for the buffer to flush.
- **Batch protection:** If the agent sent too many emails, the operation may have escalated to approval. Check `kruxos approve list`.
- **Gmail API error:** Check the audit log for error details: `kruxos audit query --capability email.send --last 1h`

## State issues

### State quota exceeded

**Symptom:** `QuotaExceededError` when setting state.

**Check:**

```bash
kruxos state quota my-agent
```

**Solutions:**

- **Clean up old state:** Delete unused keys with `kruxos state delete my-agent old_key`
- **Increase quota:** Edit `/etc/kruxos/config.yaml` and increase `state.persistent.quota_mb`
- **Use shared state sparingly:** Shared state has a separate quota (default 500 MB total)

### State lost after restart

**Symptom:** Agent state is empty after Gateway restart.

**Explanation:** **Session state** is in-memory and is lost on restart. **Persistent state** survives restarts. Check which tier the agent is using:

```python
# Session state (lost on restart)
await os.state.set_async("key", value, tier="session")

# Persistent state (survives restart)
await os.state.set_async("key", value, tier="persistent")  # or just omit tier
```

Session state is checkpointed to disk every 30 seconds, so brief crashes may be recovered. But for important data, always use persistent state.

## Vault issues

### Vault locked

**Symptom:** `ServiceUnavailableError` mentioning vault.

**Solution:**

```bash
kruxos vault unlock
```

Enter the admin passphrase. The vault auto-locks after a configurable timeout (default: never for single-node).

### Lost admin passphrase

**Symptom:** Cannot unlock the vault.

**Solution:** There is no password recovery for the vault — this is by design for security. You'll need to:

1. Back up any unencrypted data you can access
2. Reinitialize: `kruxos setup --reconfigure`
3. Re-create agents and re-add secrets

!!! warning
    Keep your admin passphrase in a secure password manager. There is no recovery mechanism.

## Audit issues

### Audit hash chain broken

**Symptom:** `kruxos audit stats` reports chain verification failure.

**Explanation:** This indicates the audit log may have been tampered with or corrupted.

**Solutions:**

- **Disk corruption:** Check disk health. The audit system is append-only — corruption usually means hardware failure.
- **Manual file editing:** If someone edited the raw CBOR files, the chain is broken. The audit log is append-only and should never be edited directly.
- **Recovery:** The system continues logging with a new chain from the break point. A note is added to the audit log indicating the chain break.

## Performance issues

### High latency on capability calls

**Check:**

```bash
kruxos audit stats --last 1h
```

Look at average latency. Normal is under 50ms for most capabilities.

**Solutions:**

- **Disk I/O:** Check disk utilisation. SQLite writes (state, audit) can slow down on busy disks.
- **Too many agents:** Check active agent count. Each agent consumes memory for its session.
- **Large state:** Agents with very large state databases may see slower state operations. Consider cleaning up old keys.

### Gateway using too much memory

**Check:**

```bash
kruxos alerts
```

**Solutions:**

- **Audit buffer:** If disk is full, the audit system buffers in memory (up to 10K entries). Free disk space.
- **Many concurrent agents:** Each session uses memory. Reduce active agent count or increase available RAM.
- **State cache:** Session state is in-memory. Agents storing large amounts of session state will increase memory usage.

## Claude Desktop / Claude Code issues

### Claude doesn't see any KruxOS tools

**Symptom:** Claude Desktop or Claude Code shows no tools from KruxOS.

**Check:**

- **Claude Desktop:** Is `claude_desktop_config.json` in the right location?
    - macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
    - Windows: `%APPDATA%\Claude\claude_desktop_config.json`
    - Linux: `~/.config/Claude/claude_desktop_config.json`
- **Claude Code:** Is `~/.claude/settings.json` populated? Try `kruxos cli-config generate --write` on the appliance to (re-)write it.
- **mcp-bridge binary:** `ls -l /opt/kruxos/bin/mcp-bridge` — must exist and be executable on the host launching Claude.
- **Did you restart?** Claude Desktop must be restarted after config changes

**Manual test:**

```bash
KRUXOS_ENDPOINT=wss://localhost:7700 \
KRUXOS_AGENT_NAME=default-agent \
KRUXOS_AGENT_TOKEN=<64-char hex> \
/opt/kruxos/bin/mcp-bridge
```

Structured exit codes (10 = auth, 11 = network) point at the cause. If the bridge prints an error to stderr, that's your issue (auth failure, unreachable endpoint, missing binary).

### Tools appear but calls fail

**Symptom:** Claude sees the tools but calls return errors.

**Solutions:**

- **Check the Gateway is running:** `curl -s http://localhost:7800` or `docker logs <container>`
- **Check the API key:** `kruxos agent list` — agent should show `active`
- **Workspace path:** Filesystem operations need full workspace paths (e.g., `/data/kruxos/workspace/{agent-name}/file.txt`), not relative paths
- **Policy blocking:** Check if the capability's policy tier allows the agent to call it

### agent.whoami returns "unknown"

**Symptom:** Asking "who are you?" returns an agent name of "unknown".

**Solution:** Update to the latest gateway. The Gateway injects the authenticated agent name into `agent.*` capability calls at the protocol handler level.

## Docker-specific issues

### Container won't start

```bash
docker logs kruxos
```

Common causes:

- **Port conflict:** Another service is using port 7700 or 7800. Change the port mapping: `-p 7710:7700`
- **Volume permissions:** If using a bind mount, ensure the directory is writable
- **Out of disk:** Docker needs space for the image and runtime data

### Data lost after container recreation

Always use a named volume:

```bash
docker run -v kruxos-data:/data/kruxos ...
```

Without `-v`, data is stored in the container's writable layer and lost when the container is removed.

## Error reference

For a complete catalogue of every error code, its cause, fix, and SDK exception mapping, see the [Error Reference](../troubleshooting/errors.md).

## Getting help

If your issue isn't covered here:

1. Check the [audit log](../quickstart/cli.md#audit-log) for detailed error information
2. Review the [health endpoint](monitoring.md#health-checks) for service status
3. File an issue on [GitHub](https://github.com/altvale/kruxos/issues) with:
    - KruxOS version (`kruxos version`)
    - Relevant audit log entries
    - Steps to reproduce
