# Desktop Dashboard

By the end of this page, you'll know what the KruxOS **desktop** is, the
surfaces it gives you, and how to switch between the Desktop and Classic
dashboard layouts.

!!! note "Feature preview — and the default in v0.0.3"
    The desktop is a **preview** and the **default** layout in v0.0.3: with no
    saved preference, signing in — and the dashboard home — lands you on the
    desktop. Every classic page stays reachable; see
    [Switching layouts](#switching-layouts).

## What the desktop is

The desktop is an OS-style view over the same KruxOS dashboard you already know.
Instead of moving page to page, you supervise your appliance from a single
"desktop": open the surfaces you care about as free-form windows, keep an eye on
health and pending approvals from a menu bar and widgets, and act on approval
requests as they arrive. Every windowed app shows the **same live data** as its
classic page — it's the same backend, just arranged differently.

## The main surfaces

- **Menu bar** — a top strip showing the pending-approvals count, overall health
  status, and the clock, plus the menu you use to lock your session.
- **Windows** — open surfaces as free-form windows you can drag, resize, and
  arrange however you like.
- **Dock** — your running agents, always one click away.
- **Command palette (`⌘K`)** — press `⌘K` (or `Ctrl+K`) to jump to a surface or
  action without reaching for the mouse.
- **Mission Control** — a zoomed-out overview of everything you have open.
- **Desktop widgets** — at-a-glance tiles for health, pending approvals, and
  live activity.
- **Notification center** — where approval requests arrive as actionable
  interrupts (see below).

## Approving from the desktop

Approval requests surface in the **notification center** as actionable
interrupts with **Approve** and **Reject** buttons. This is the *same* approval
queue as the classic [Approvals](approval-workflow.md) page — deciding a request
in one place settles it everywhere — and every decision runs through the same
deterministic policy engine. Nothing about how approvals are evaluated changes;
the desktop is simply another surface to act on them.

## Switching layouts

The layout is a **per-browser** preference (stored locally, like your theme), so
each browser you sign in from remembers its own choice.

To switch, open **Settings › System** and use the **Dashboard layout** card —
it's present on *both* layouts:

- Choose **Classic** to return to the page-by-page dashboard.
- Choose **Desktop** to go back to the OS-style view.

Either way, **every classic page stays reachable by its direct URL** in either
mode, so nothing is hidden by the layout you pick.

## Locking your session

Use the menu bar to open the **lock screen** when you step away. Unlock by
signing back in with your dashboard passphrase.

## Related

- [Dashboard Chat](dashboard-chat.md) — the multi-panel chat interface
- [Approval Workflow](approval-workflow.md) — how approvals work across every surface
- [Monitoring](monitoring.md) — the Health and Activity surfaces
