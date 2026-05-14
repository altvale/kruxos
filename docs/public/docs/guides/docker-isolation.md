# Docker Agent Isolation

KruxOS supports two Docker deployment patterns. Choose the one that fits your use case.

## Single-Container (Development / Single Agent)

The default `docker-compose.yml` runs everything in one container — gateway, dashboard, and agents share the same process space. This is the fastest way to get started.

```bash
docker compose up -d
```

**When to use:** local development, evaluation, single-agent workflows.

## Multi-Container (Production / Multi-Agent)

`docker-compose.agent.yml` runs each agent in its own isolated container. The gateway runs in the main KruxOS container, and agent containers connect to it over a private Docker network.

Each agent gets:

- **Isolated filesystem** — a dedicated Docker volume mounted at `/workspace`
- **Resource limits** — configurable memory and CPU caps per agent
- **Network isolation** — agents can reach the gateway but cannot communicate with each other directly

### Setup

**1. Start the gateway:**

```bash
docker compose up -d
```

Wait for the health check to pass (`docker compose ps` should show `healthy`).

**2. Create agent credentials:**

```bash
# Create credentials for each agent
docker exec kruxos /opt/kruxos/bin/kruxos agent create \
    --name "agent-1" --purpose "Data analysis agent"

docker exec kruxos /opt/kruxos/bin/kruxos agent create \
    --name "agent-2" --purpose "Code generation agent"
```

Save the API keys from the output.

**3. Start agent containers:**

```bash
# Pass API keys as environment variables
AGENT_1_API_KEY=<64-char hex> AGENT_2_API_KEY=<64-char hex> \
    docker compose -f docker-compose.yml -f docker-compose.agent.yml up -d
```

### Adding More Agents

Copy an agent service block in `docker-compose.agent.yml`, changing:

- Service name (e.g., `agent-3`)
- `container_name`
- Volume name
- Network name (create a new `agent-3-net` in the `networks:` section)
- `AGENT_NAME` and `AGENT_API_KEY` environment variables

Then add the new volume and network to their respective sections, and add the new network to the gateway's `networks:` list with the `gateway` alias.

### Verifying Isolation

**Filesystem isolation** — files created in one agent's `/workspace` are invisible to other agents:

```bash
# Write a file in agent-1
docker exec kruxos-agent-1 touch /workspace/secret.txt

# Verify agent-2 cannot see it
docker exec kruxos-agent-2 ls /workspace/secret.txt
# ls: cannot access '/workspace/secret.txt': No such file or directory
```

**Network isolation** — agents cannot reach each other directly:

```bash
# From agent-1, try to reach agent-2 (should fail — name won't even resolve)
docker exec kruxos-agent-1 bash -c 'timeout 2 bash -c "cat < /dev/null > /dev/tcp/kruxos-agent-2/80" 2>&1 || echo "Name resolution failed (expected)"'

# From agent-1, verify gateway is reachable
docker exec kruxos-agent-1 bash -c 'timeout 2 bash -c "cat < /dev/null > /dev/tcp/gateway/7700" 2>&1 && echo "Gateway reachable"'
```

### Network Architecture

```
┌───────────────────────────────────────────────┐
│              Docker Host                      │
│                                               │
│  ┌─────────────┐      ports 7700/7701/7800    │
│  │   KruxOS    │◄──── host network           │
│  │  (gateway +  │                             │
│  │  dashboard)  │                             │
│  └──┬───────┬───┘                             │
│     │       │                                 │
│  agent-1-net│  agent-2-net                    │
│  (bridge)   │  (bridge)                       │
│     │       │                                 │
│  ┌──┴──┐ ┌──┴──┐                             │
│  │ A1  │ │ A2  │  (separate networks —        │
│  └─────┘ └─────┘   cannot resolve each other) │
└───────────────────────────────────────────────┘
```

Each agent has its own Docker bridge network. The gateway joins all agent networks (with the `gateway` alias), but agents only join their own — they cannot resolve or reach other agents at all.

Networks are marked `internal: true`, which also prevents outbound internet access from agent containers. Remove `internal: true` from the agent's network in `docker-compose.agent.yml` if that agent needs to call external APIs.

### Resource Limits

Default per-agent limits in `docker-compose.agent.yml`:

| Resource | Limit |
|----------|-------|
| Memory   | 512 MB |
| CPUs     | 1.0   |

Adjust in each agent's `deploy.resources.limits` section. The gateway container has its own limits (2 GB / 2 CPUs) set in `docker-compose.yml`.
