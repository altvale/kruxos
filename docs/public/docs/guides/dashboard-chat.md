# Dashboard Chat

The Chat page is your primary interface for interacting with agents through the KruxOS dashboard. It provides a three-panel layout for managing conversations across all your registered agents.

## Three-Panel Layout

The chat interface is organized into three panels:

- **Left panel: Agent list** -- Shows all registered agents with status indicators (green = active, grey = idle). Click an agent to see their conversations.
- **Middle panel: Conversation list** -- Shows conversations for the selected agent, sorted by most recent activity. Pinned conversations appear first.
- **Right panel: Messages** -- The chat area for the selected conversation, with streaming responses, tool call visualization, and inline approval actions.

On mobile devices, panels collapse to a single view with back navigation buttons.

## Creating and Managing Conversations

- Click **+ New** in the conversation panel to start a fresh conversation with the selected agent.
- **Rename**: hover over a conversation and click the pencil icon, or double-click the title.
- **Pin/Unpin**: click the star icon to pin important conversations to the top of the list.
- **Delete**: hover and click the X icon. A confirmation dialog prevents accidental deletion.

Each conversation maintains its own message history. Your agent's knowledge persists automatically across all conversations.

## Switching Between Agents

Click any agent in the left panel to switch. The conversation list updates to show that agent's conversations. Your place in other agents' conversations is preserved -- switch back anytime.

## Tool Call Visualization

When the AI model invokes KruxOS capabilities during a conversation, tool calls appear as collapsible blocks:

- **Lightning icon** -- Autonomous or notify-tier capability (executes immediately)
- **Hourglass icon** -- Approval-required capability (waits for your decision)
- **Warning icon** -- Blocked or skipped capability

Each tool call block shows:

- The capability name (e.g., `filesystem.read`)
- The policy tier badge (autonomous, notify, approval_required)
- A result summary and execution time
- Click to expand and see full arguments and results

## Inline Approval Workflow

When an agent requests a capability that requires approval:

1. An hourglass block appears in the chat with the capability name and reason.
2. Two buttons appear: **Cancel** (reject) and **Send Now** (approve).
3. Click **Send Now** to approve -- the capability executes and the conversation continues.
4. Click **Cancel** to reject -- the agent receives a rejection message and stops further actions.
5. If no decision is made within 5 minutes, the request times out automatically.

Browser notifications alert you when approval is needed (grant notification permission when prompted).

## Knowledge Panel

Click the **Knowledge** button in the message panel header to open the knowledge sidebar. This shows everything the selected agent knows (persistent state entries).

- **View**: all key-value entries with their current values
- **Add**: enter a key and value, click Add
- **Edit**: hover over an entry and click the pencil icon
- **Delete**: hover and click the X icon

### Save to Knowledge

On any assistant message, click **Save to knowledge** to capture that response as a knowledge entry. This is useful for manually curating what your agent remembers from conversations.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Enter | Send message |
| Shift+Enter | New line in message |
| Cmd/Ctrl+K | Search conversations |

## Tips

- **Keep conversations focused** -- start a new conversation when switching tasks. Your agent's knowledge persists automatically, so fresh conversations inherit everything the agent knows.
- **Use the knowledge panel** to see what your agent knows before asking questions. Add context manually if needed.
- **Pin important conversations** for quick access to ongoing work.
- **Watch tool calls** to understand what your agent is doing. Expand the blocks to see the full details.
