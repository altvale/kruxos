# Migration from OpenClaw

By the end of this page, you'll have migrated your OpenClaw skills to KruxOS capabilities with full type safety, permission controls, and audit logging.

!!! note "Welcome, OpenClaw users"
    We built KruxOS because we believe AI agents deserve a real operating system — not a collection of shell scripts. If you've been running OpenClaw, you already understand why agents need structured tools. KruxOS takes that idea further with typed APIs, deterministic governance, and a safety model designed for production.

    Your existing skills will keep working. This guide helps you migrate at your own pace.

## What changes (and what doesn't)

| Aspect | OpenClaw | KruxOS | Migration effort |
|--------|----------|---------|-----------------|
| Skill definitions | JSON/YAML (loose types) | YAML (semantic types) | **Automatic** via importer |
| Permission model | All-or-nothing | 4-tier (autonomous → blocked) | **Automatic** tier assignment |
| Agent protocol | OpenClaw WebSocket | MCP / JSON-RPC | **Zero** (bridge on port 7702) |
| Skill execution | Node.js processes | Sandboxed Node.js subprocesses | **Zero** (executor handles it) |
| Audit trail | None | Hash-chained append-only logs | **Automatic** |
| Secret management | Environment variables | Use-not-read vault | Requires re-mapping |

## Migration paths

### Path 1: Bridge mode (zero changes)

Run your existing OpenClaw agents through the compatibility bridge. No code changes needed.

```bash
# Install the OpenClaw compatibility pack
kruxos pack install openclaw-bridge

# Start the bridge (listens on port 7702)
kruxos openclaw bridge start
```

Your OpenClaw agents connect to port 7702 instead of their previous endpoint. The bridge translates:

- OpenClaw authentication → KruxOS agent auth
- `invoke {skill, params}` → `capabilities.call {capability, inputs}`
- `list_skills` → `capabilities.list`
- Responses back to OpenClaw format

All calls pass through KruxOS's policy engine, sandboxing, and audit system.

### Path 2: Import skills (recommended)

Convert your OpenClaw skill definitions to native KruxOS capabilities:

```bash
openclaw-import /path/to/your/skills --output /data/kruxos/packs/imported/
```

Expected output:

```
OpenClaw → KruxOS Migration
━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Scanning /path/to/your/skills...
Found 42 skills in 8 categories

Translating...
  ✓ 35 skills translated successfully
  ⚠  5 skills need manual review (flagged as potentially dangerous)
  ✗  2 skills failed (missing required fields)

Permission tiers assigned:
  Autonomous:         18 (read-only skills)
  Notify:             12 (modifying/external access)
  Approval required:   5 (flagged for review)
  Skipped:             2 (dangerous keywords detected)

Output written to:
  Capabilities: /data/kruxos/packs/imported/definitions/
  Policy:       /data/kruxos/packs/imported/policy.yaml
  Report:       /data/kruxos/packs/imported/migration-report.txt

Review the migration report, especially skills marked 'needs_review'.
```

### Path 3: Gradual migration

Use both paths simultaneously. Run the bridge for existing skills while migrating high-value skills to native capabilities:

1. Start with bridge mode for everything
2. Identify your most-used skills
3. Import those skills natively (they'll be faster and get full type safety)
4. Gradually move more skills over
5. Shut down the bridge when all skills are migrated

## Step-by-step: Import your skills

### Step 1: Locate your skills directory

OpenClaw skills are usually in one of these locations:

```bash
# Check OpenClaw config
cat ~/.openclaw/config.yaml | grep skills_dir

# Common locations
ls ~/openclaw-skills/
ls /opt/openclaw/skills/
```

### Step 2: Dry run

Preview the migration without writing anything:

```bash
openclaw-import /path/to/skills --dry-run
```

Review the output carefully. Pay attention to:

- **Skills marked "needs_review"** — these have dangerous keywords (sudo, shell, exec) and won't be auto-imported
- **Type mapping** — check that parameter types were correctly inferred
- **Permission tiers** — verify the automatic tier assignment makes sense

### Step 3: Import

```bash
openclaw-import /path/to/skills --output /data/kruxos/packs/imported/
```

### Step 4: Review flagged skills

Open the migration report:

```bash
cat /data/kruxos/packs/imported/migration-report.txt
```

For each "needs_review" skill, decide:

- **Safe to import**: Edit the generated YAML to set the appropriate tier
- **Too dangerous**: Leave it out. KruxOS may have a built-in capability that's safer (e.g., use `process.run` instead of a shell-exec skill)

### Step 5: Load the imported pack

```bash
kruxos pack install /data/kruxos/packs/imported/
```

### Step 6: Test

```python
import asyncio
from kruxos import KruxOS

async def main():
    os = await KruxOS.connect_async(
        endpoint="ws://localhost:7700",
        agent_name="my-agent",
        api_key="<64-char hex>",
        purpose="Migration test",
    )

    try:
        # List imported capabilities
        caps = await os.capabilities.list_async(category="openclaw")
        print(f"Imported {len(caps)} OpenClaw capabilities")

        # Test one
        result = await os.call_async("openclaw.utils.word_count", text="Hello world")
        print(f"Result: {result.data}")
    finally:
        await os.close_async()

asyncio.run(main())
```

## Common gotchas

### 1. Skills that shell out

OpenClaw skills that use `child_process.exec()` or `subprocess` are flagged as dangerous. In KruxOS, use the built-in `process.run` capability instead — it's sandboxed and audit-logged.

**Before (OpenClaw):**
```javascript
const { exec } = require('child_process');
exec('ls -la /workspace', (err, stdout) => { ... });
```

**After (KruxOS):**
```python
result = await os.call_async("filesystem.list", path="/workspace")
```

### 2. Skills that read environment variables for secrets

OpenClaw skills often read API keys from environment variables. In KruxOS, secrets are in the vault — agents never see raw values.

**Before (OpenClaw):**
```javascript
const apiKey = process.env.MY_API_KEY;
```

**After (KruxOS):**
```python
# The capability handler accesses the vault internally
result = await os.call_async("my_service.query", prompt="Hello")
# The API key was injected by the vault — the agent never sees it
```

### 3. Skills with filesystem access outside /workspace

OpenClaw skills can access any path. KruxOS sandboxes all filesystem access to the agent's workspace. If a skill needs access to a specific path, configure it in the agent's policy.

### 4. Network-dependent skills during migration

Skills that make HTTP requests will work through the bridge, but they bypass the Service Proxy. For production use, consider wrapping external service calls in a Service Proxy adapter.

## What you gain by migrating

| Feature | OpenClaw | KruxOS |
|---------|----------|---------|
| Type safety | Loose string types | Semantic types (FilesystemPath, URL, etc.) |
| Permission control | None | 4-tier policy with rate limits |
| Audit trail | None | Hash-chained, append-only, queryable |
| Error handling | Unstructured | Typed errors with recovery suggestions |
| Secret management | Environment variables | Encrypted vault, use-not-read |
| Sandbox isolation | None | Namespaces + cgroups + seccomp + Landlock |
| Multi-agent | Single agent | Session isolation, agent-to-agent comms |
| Tool discovery | Manual listing | Auto-discovery with schema + docs |

## Getting help

If you run into issues migrating:

- Check [Troubleshooting](troubleshooting.md) for common problems
- File an issue on [GitHub](https://github.com/altvale/kruxos/issues) with the `migration` label
- The migration report (`migration-report.txt`) contains detailed information about any failures

## Next steps

- [Managing Agents](managing-agents.md) — set up agents for your migrated skills
- [Policies](policies.md) — fine-tune permissions for imported capabilities
- [Monitoring](monitoring.md) — watch your migrated skills in action
