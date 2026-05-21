# Dashboard Chat

The Chat page is your primary interface for interacting with agents through the KruxOS dashboard. It provides a four-column desktop layout — plus a search overlay — for managing conversations across all your registered agents.

## Four-Column Layout

The chat interface is organized into four columns plus a search overlay:

- **Agents (left)** — Active agents only. Status dots map green (active) · amber (paused) · red (disconnected or revoked). The `auto` tag flags autonomous agents.
- **Conversations (left-center)** — Scoped to the selected agent. **Pinned** conversations appear at the top; **Recent** conversations follow. Inline rename · pin · delete actions surface on hover.
- **Messages (center)** — Streaming responses, tool-call cards with policy-tier colouring, inline approval actions, per-message Model + Thinking dropdowns above the composer, and a context-usage badge.
- **Knowledge (right, collapsible)** — Toggle via the bookmark button in the Messages header. Inline add · edit · delete for the selected agent's persistent state entries.
- **Search (overlay, `⌘K` / `Ctrl+K`)** — Substring filter on conversation title and preview. Press Enter to jump to the first match.

On screens narrower than 768 px the panels collapse to a 3-state mobile navigation (**agents → conversations → messages**) with back buttons to return up the hierarchy.

## Creating and Managing Conversations

- Click **+ New** in the conversation panel to start a fresh conversation with the selected agent.
- **Rename**: hover over a conversation and click the pencil icon, or double-click the title.
- **Pin / Unpin**: click the star icon to pin important conversations to the top of the list.
- **Delete**: hover and click the X icon. A confirmation modal prevents accidental deletion.

Each conversation maintains its own message history. Your agent's knowledge persists automatically across all conversations.

## Switching Between Agents

Click any agent in the left column to switch. The conversation list updates to show that agent's conversations. Your place in other agents' conversations is preserved — switch back anytime.

## Per-Message Model and Thinking Overrides

Two dropdowns sit above the composer, both scoped to the current conversation:

- **Model** — pick the provider + model for the next message. The selection persists for the active conversation (per browser session) so you can mix models within one thread when needed. For OpenRouter providers, choose **Custom model…** to enter any catalog ID by hand.
- **Thinking** — toggle extended-thinking modes when the selected model supports them.

The two selections ride with the next message you send; you can change them again before the next turn.

## Tool Call Visualization

When the model invokes a KruxOS capability during a conversation, the call appears as a collapsible card in the timeline. The card colour and icon map to the policy tier:

- **Lightning icon** — autonomous or notify-tier capability (executes immediately)
- **Hourglass icon** — `approval_required` (waits for your decision)
- **Warning icon** — blocked or skipped

Each card shows the capability name (e.g. `filesystem.read`), the policy-tier badge, a short result summary, and an execution duration. Click the card to expand and inspect the full arguments and result JSON.

## Inline Approval Workflow

When the agent calls a capability gated as `approval_required`:

1. An hourglass card appears in the chat with the capability name and the policy reason.
2. Two buttons appear: **Cancel** (reject) and **Send Now** (approve).
3. Click **Send Now** to approve — the capability executes and the conversation continues.
4. Click **Cancel** to reject — the agent receives a rejection message and stops the current chain.
5. If no decision is made within the configured hold window, the request times out automatically.

Browser notifications alert you when approval is needed (grant notification permission when prompted). The same request also appears in the dedicated **Approvals** page; deciding it in either surface settles the other.

## Knowledge Panel

Click the bookmark button in the Messages header to open the **Knowledge** panel on the right. This shows everything the selected agent knows (persistent state entries).

- **View**: all key-value entries with their current values
- **Add**: enter a key and value, click **Add**
- **Edit**: hover over an entry and click the pencil icon
- **Delete**: hover and click the X icon

The panel auto-refreshes when an agent's tool call writes to `state.persistent.*`, so changes the agent makes mid-conversation surface here in real time.

### Save to Knowledge

On any assistant message, click **Save to knowledge** to capture that response as a knowledge entry. The button shows a saved state once captured, so you can tell at a glance which messages have already been promoted into state.

## Search

Press `⌘K` (macOS) or `Ctrl+K` (Windows/Linux) anywhere on the page to open the Search overlay. It substring-matches against conversation title and message preview; Enter jumps to the first match. Press `Esc` or click the backdrop to dismiss.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Send message |
| `Shift+Enter` | New line in message |
| `⌘K` / `Ctrl+K` | Open the Search overlay |
| `Esc` | Close the Search overlay or any open modal |

## Tips

- **Keep conversations focused** — start a new conversation when switching tasks. Your agent's knowledge persists automatically, so fresh conversations inherit everything the agent knows.
- **Use the knowledge panel** to see what your agent knows before asking questions. Add context manually if needed.
- **Pin important conversations** for quick access to ongoing work.
- **Watch tool calls** to understand what your agent is doing. Expand the cards to see full arguments and results.
- **Mix models when it helps** — use the per-message Model dropdown for one-off shifts (e.g. a stronger model on a hard question) without leaving the conversation.
