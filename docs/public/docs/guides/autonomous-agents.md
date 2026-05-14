# Autonomous Agents

Autonomous agents run server-side without an active chat session. They wake on schedules, incoming messages, or manual triggers, execute their instructions using AI, and go back to sleep. Humans remain the oversight layer — you configure what they can do, monitor what they did, and intervene when needed.

## How Autonomous Agents Differ from Interactive Chat

| | Interactive (chat) | Autonomous (server-side) |
|---|---|---|
| **Runs when** | You open a conversation | On a schedule, trigger, or message |
| **Model calls** | You see each response live | Happens in the background |
| **Tool calls** | Shown for approval in real time | Policy-controlled, logged to audit |
| **Session lifetime** | Until you close the chat | One task execution (seconds to minutes) |
| **State** | Conversation history | Persistent knowledge + task history |

An interactive agent is a meeting. An autonomous agent is an employee who follows standing instructions.

## Creating Your First Autonomous Agent

### Step 1: Create the agent

```bash
kruxos agent create bookkeeper \
  --purpose "Process invoices from email and update the ledger" \
  --autonomous \
  --schedule "0 */2 * * *" \
  --instructions "Check for new invoice emails. Extract amounts and dates. Update the shared ledger state. Flag anything over $10,000 for human review." \
  --triggers comms
```

### Step 2: Assign a model

```bash
kruxos agent set-model bookkeeper anthropic
```

### Step 3: Verify it's running

```bash
kruxos agent show bookkeeper
```

You'll see the agent's status, schedule, next run time, and recent task history.

### Step 4: Monitor from the dashboard

Open the dashboard and navigate to **Agents > bookkeeper**. The detail page shows:

- Current status (active, paused, revoked)
- Schedule and next run time
- Recent task executions with duration, tool calls, and status
- Persistent state keys

## Schedule Expressions

Schedules use standard cron syntax with five fields:

```
┌───────── minute (0-59)
│ ┌─────── hour (0-23)
│ │ ┌───── day of month (1-31)
│ │ │ ┌─── month (1-12)
│ │ │ │ ┌─ day of week (0-6, 0=Sunday)
│ │ │ │ │
* * * * *
```

### Common patterns

| Schedule | Expression | Use case |
|---|---|---|
| Every hour | `0 * * * *` | Check for new emails |
| Every 15 minutes | `*/15 * * * *` | Monitor a service |
| Daily at 9 AM | `0 9 * * *` | Morning summary |
| Weekdays at 6 PM | `0 18 * * 1-5` | End-of-day report |
| Every Monday at 8 AM | `0 8 * * 1` | Weekly review |
| First of month | `0 0 1 * *` | Monthly reconciliation |

## Writing Effective Instructions

Instructions are the standing orders your agent follows every time it wakes up. Good instructions are specific, bounded, and include failure modes.

### Be specific about triggers and responses

```
# Bad
Handle emails.

# Good
Check for new emails with subject containing "invoice" or "receipt".
For each match, extract the sender, amount, date, and line items.
Store results in persistent state under "invoices:{date}:{sender}".
```

### Define what "urgent" means

```
If an invoice is over $10,000, send a comms message to the "alerts"
channel with priority "high". Do NOT process it automatically.
```

### Specify escalation rules

```
If you encounter an email you cannot parse, store it under
"invoices:unparsed:{message_id}" and continue to the next one.
Never skip more than 3 emails without flagging for review.
```

### Include negative instructions

```
Never delete emails.
Never send replies without approval.
Never modify invoices that have already been marked as "reconciled".
```

### Keep instructions focused

Each agent should have one clear job. If you find yourself writing instructions that cover multiple unrelated responsibilities, create separate agents.

## When to Create Agents vs. Conversations

### Create a new agent when you need:

- **Different trust levels** — a code agent shouldn't access your email
- **Different models** — use Claude for complex reasoning, Ollama for simple tasks
- **Data isolation** — each agent gets its own state, sandbox, and workspace
- **Different schedules** — the email checker runs every hour, the report runs daily
- **A distinct role** — "bookkeeper" and "code-assistant" are different jobs

### Start a new conversation when you need:

- **A fresh context** — the agent is the same, you just want a clean slate
- **A different topic** — same agent, different project discussion
- **To resume later** — conversations persist and can be continued

**Mental model:** Agents are employees with roles. Conversations are meetings.

## Real-World Setup Examples

### The Developer

Three agents, each sandboxed to its own project:

```bash
kruxos agent create frontend-dev \
  --purpose "Maintain the React dashboard" \
  --autonomous --schedule "0 9 * * 1-5" \
  --instructions "Review open PRs, run tests, flag failures."

kruxos agent create backend-dev \
  --purpose "Maintain the API server" \
  --autonomous --schedule "0 9 * * 1-5" \
  --instructions "Check CI status, review dependency updates."

kruxos agent create devops \
  --purpose "Monitor infrastructure" \
  --autonomous --schedule "*/30 * * * *" \
  --instructions "Check service health endpoints. Alert on failures."
```

Each agent has its own workspace, its own sandbox, and cannot access the others' files.

### The Business Owner

Three agents with different models based on task complexity:

```bash
# Complex reasoning tasks — use Claude
kruxos agent create assistant \
  --purpose "Personal assistant for scheduling and communication"
kruxos agent set-model assistant anthropic

# Repetitive financial tasks — use a cheaper model
kruxos agent create bookkeeper \
  --purpose "Invoice processing and reconciliation" \
  --autonomous --schedule "0 */4 * * *"
kruxos agent set-model bookkeeper openai-gpt4o-mini

# Organization tasks — use local model (free)
kruxos agent create organizer \
  --purpose "File organization and tagging" \
  --autonomous --schedule "0 22 * * *"
kruxos agent set-model organizer local-default
```

### The Privacy-Conscious User

Everything runs locally with Ollama — zero cloud API calls:

```bash
kruxos agent create jarvis --purpose "General assistant"
kruxos agent set-model jarvis local-default

kruxos agent create researcher \
  --purpose "Research and summarize documents" \
  --autonomous --schedule "0 8 * * *"
kruxos agent set-model researcher local-default
```

## Agent Separation for Security

### Why one agent per project matters

Each agent runs in its own Linux sandbox with:

- **Filesystem isolation** — agents can only access their own workspace
- **Network isolation** — agents can only reach allowed endpoints
- **Process isolation** — agents cannot see or signal each other's processes
- **State isolation** — each agent has its own persistent state database

If agent A is compromised or makes a mistake, it cannot affect agent B's workspace, read agent B's secrets, or modify agent B's state.

### Shared state as controlled handoff

When agents need to coordinate, they use **shared state** — a dedicated cross-agent key-value store with optimistic locking. This is a controlled handoff point, not shared access to each other's internals.

```
# Agent A writes
state.shared.set(key="report:2024-03", value={...}, expected_version=0)

# Agent B reads
state.shared.get(key="report:2024-03")
```

The shared state is visible to all agents, versioned, and audit-logged. It's the equivalent of a shared drive — not shared root access.

## Multi-Agent Coordination

### Task delegation via comms

Agents can send messages to each other using `comms.send`. When an autonomous agent has `comms` in its triggers list, incoming messages wake it up automatically.

**Example: Invoice processing pipeline**

```
1. email-bot detects invoice email
   → comms.send(to="bookkeeper", message='{"action":"new_invoice","email_id":"abc123"}')

2. bookkeeper triggers automatically, processes invoice
   → comms.send(to="email-bot", message='{"action":"task_complete","status":"reconciled"}')

3. email-bot triggers, sends confirmation reply to sender
```

To set this up:

```bash
# Create email-bot with comms trigger
kruxos agent create email-bot \
  --purpose "Monitor and respond to emails" \
  --autonomous --schedule "*/15 * * * *" \
  --triggers schedule,comms

# Create bookkeeper with comms trigger (no schedule — purely reactive)
kruxos agent create bookkeeper \
  --purpose "Process invoices" \
  --autonomous \
  --triggers comms
```

### Shared state for cross-agent data

Agents can write to the shared state for data that multiple agents need. Shared state uses dedicated capability names (`state.shared.*`), not a scope parameter:

```
# bookkeeper writes (expected_version=0 creates a new key)
state.shared.set(key="invoices:march:status", value="complete", expected_version=0)

# assistant reads
state.shared.get(key="invoices:march:status")
# → returns { value: "complete", version: 1, owner_agent: "bookkeeper", updated_at: "..." }
```

Optimistic locking prevents lost updates — if two agents try to update the same key simultaneously, one will get a `VersionConflict` error and can retry with the current version.

### Event-driven workflows

Combine schedules and comms for complex workflows:

1. **Scheduled agent** runs on a cron schedule and produces results
2. **Reactive agent** listens for comms messages and processes them
3. **Supervisor agent** monitors shared state and escalates issues

## Cost Management

Different tasks need different models. Assign models per agent based on complexity:

| Task type | Recommended model | Why |
|---|---|---|
| Complex reasoning | Claude Sonnet/Opus | Needs strong reasoning |
| Simple classification | GPT-4o-mini | Fast, cheap, good enough |
| Privacy-sensitive | Ollama (local) | No data leaves your machine |
| High-volume monitoring | Ollama (local) | No per-token cost |

```bash
kruxos agent set-model complex-agent anthropic
kruxos agent set-model simple-agent openai-gpt4o-mini
kruxos agent set-model private-agent local-default
```

## Monitoring

### Reading agent logs

```bash
# Show recent task executions
kruxos agent logs bookkeeper

# Show last 5 tasks
kruxos agent logs bookkeeper --limit 5

# Stream live output (follows new tasks)
kruxos agent logs bookkeeper --follow
```

### Task history in the dashboard

Navigate to **Agents > (agent name)** to see:

- Task execution timeline with status indicators
- Tool calls per task (what the agent did)
- Token usage and duration
- Error details for failed tasks

### Interpreting failures

| Status | Meaning | Action |
|---|---|---|
| `completed` | Task finished normally | None needed |
| `max_iterations` | Hit the safety limit | Increase `max_iterations` or simplify instructions |
| `timeout` | Exceeded wall-clock time | Task may be too complex for one run |
| `token_budget` | Used too many tokens | Reduce scope or increase budget |
| `model_error` | Model API failed | Check API key, model availability |
| `blocked` | All actions blocked by policy | Review policy configuration |

## Pausing, Resuming, and Editing

### Pause an agent

```bash
kruxos agent pause bookkeeper
```

The agent stops executing scheduled tasks but keeps its state and configuration. You can also pause from the dashboard.

### Resume an agent

```bash
kruxos agent resume bookkeeper
```

The agent resumes its schedule from the next due time.

### Edit agent configuration

```bash
kruxos agent edit bookkeeper \
  --schedule "0 */4 * * *" \
  --instructions "Updated instructions here" \
  --max-iterations 30
```

### Trigger a manual run

```bash
kruxos agent run bookkeeper "Process the invoice from ACME Corp"
```

This triggers the agent immediately with the given instruction, regardless of its schedule.
