# Secrets (Vault)

When an agent needs to call a third-party API that requires a key — Stripe,
GitHub, an internal service — you don't hand the agent the key. You store it once
in KruxOS's **encrypted vault**, scope it to exactly what may use it and where it
may be sent, and the gateway attaches it **at the network boundary**. The agent
references the secret **by name** and never sees the value.

This guide covers **Settings › Secrets** (and its `kruxos vault` CLI mirror): how
to add a scoped, host-bound key, how agents use it, and the deny-by-default rules
that keep an unscoped key harmless.

!!! note "Prerequisites"
    - The appliance is running and the **vault is unlocked** (secrets are stored
      in the vault). Unlock it from the dashboard or with `kruxos vault unlock`.
    - You have the API key or token you want to store.

---

## How it works

A stored secret is bound to **two** things, and an agent can only use it when
**both** are set:

1. **A capability** — which capability may attach it. For outbound API calls that
   is `network.credentialed_request`.
2. **Allowed hosts** — which destination host(s) it may be sent to.

This is **deny-by-default**. A secret with no scope is stored but **no agent can
use it**. A secret with no host binding is likewise unusable for a credentialed
request. Neither is ever silently widened — binding a key to **every** host (a
`*` wildcard) is always a deliberate choice you make explicitly, never a default.
In the dashboard that choice is guarded by a confirmation checkbox before the
form will accept it.

The value itself is **write-only**: once you save it, it is encrypted in the
vault and **never shown or read back** through the dashboard or CLI. To replace
it, rotate it.

When an agent makes a credentialed request, it names the secret; the gateway
looks it up, injects it into the outbound request (as an `Authorization: Bearer`
header by default), sends the request, and returns a response **that never
contains the credential**. The agent never handles the secret material.

---

## Add a secret from the dashboard

Open **Settings › Secrets** and click **+ Add Secret**:

1. **Name** — a short identifier, e.g. `stripe-api-key`. (A few prefixes are
   reserved for KruxOS-managed credentials like model-provider keys; the form
   tells you if you pick one.)
2. **Type** — **API key** or **OAuth token**.
3. **Value** — the key or token. This is a password field; it is stored
   encrypted and **never shown again**.
4. **Capability scope** — tick **"Attach to outbound API calls"**
   (`network.credentialed_request`) so agents may use the key on network
   requests. You can add further capability patterns in the custom field.
   *Leaving this empty is safe — the key is stored but agents are **denied** use
   of it until you scope it.*
5. **Allowed hosts** — the host(s) this key may be sent to, comma- or
   space-separated, exact (`api.stripe.com`) or a suffix wildcard
   (`*.githubusercontent.com`). *Until you bind at least one host, the key is not
   yet usable for credentialed requests.*
6. Click **Add Secret**.

!!! warning "Binding to every host"
    Entering `*` as an allowed host lets an agent send the key to **any** host
    (exfiltration-to-anywhere). The form reveals a required confirmation
    checkbox before it will accept a `*` binding. Prefer specific hosts.

---

## Add a secret from the CLI

`kruxos vault add` mirrors the dashboard. The value is read from **stdin** (never
a flag) so it stays out of your shell history:

```bash
kruxos vault add \
  --name stripe-api-key \
  --type api_key \
  --allowed-capabilities network.credentialed_request \
  --allowed-hosts api.stripe.com
# then type the secret value at the prompt
```

- `--allowed-capabilities` and `--allowed-hosts` each take a **comma- or
  space-separated** list. Both **default to empty** — so, exactly as in the
  dashboard, an omitted scope means agents are denied and an omitted host binding
  means the key is not yet usable. Neither is ever widened to `*` for you.
- Unlike the dashboard, the CLI has no interactive confirm step: passing
  `--allowed-hosts '*'` is **accepted with a warning** (no prompt). Bind specific
  hosts unless you genuinely intend the key to reach any host.
- `--type` is `api_key` or `oauth_token`.

The command tells you if you left the scope or host binding empty, so you know
the key isn't usable yet.

---

## List and revoke secrets

The Secrets tab lists every stored secret with its **name, type, capability
scope, and allowed hosts** — and clear flags when a key is **unscoped /
not-yet-usable** or bound to every host. **It never shows the value.** Each row
has a **Revoke** control.

From the CLI:

```bash
kruxos vault list             # name, type, capabilities, allowed hosts (never values)
kruxos vault revoke <name>    # revoke a secret (kept for audit, unusable)
kruxos vault rotate <name>    # replace a secret's value (read from stdin)
```

A revoked secret remains in the vault for audit but can no longer be used.

---

## How an agent uses a stored secret

Agents reach a stored key through the **`network.credentialed_request`**
capability. The agent supplies the request URL and the **name** of the secret —
never its value:

- The **host** in the URL must be in **both** the agent's egress allowlist **and**
  the secret's allowed hosts.
- The capability must be in the secret's scope.
- By default the credential is injected as an `Authorization: Bearer <value>`
  header. It can instead be placed in a different header or a query parameter,
  and for an OAuth token the access token is injected (not the raw JSON).

If either binding is missing, the call fails closed with a clear error telling
the agent to ask you to scope or host-bind the secret — it never silently
proceeds. For calls that need no credential, agents use
`network.http_request` instead.

---

## Troubleshooting

- **The agent gets "unscoped" / scope-violation** — the secret isn't scoped to
  `network.credentialed_request`. Re-add or re-scope it with that capability.
- **The agent gets "host not allowed"** — the target host isn't in the secret's
  allowed hosts (or not in the agent's egress allowlist). Add the host to the
  secret.
- **The secret shows "unscoped — agents denied" / "none bound"** — expected for a
  freshly added key with no scope or host yet. Add a capability scope and at
  least one host to make it usable.
- **"Vault is locked"** — unlock the vault (dashboard or `kruxos vault unlock`)
  before adding or using secrets.
- **I need to see the value again** — you can't; values are write-only by design.
  Rotate the secret to set a new value.

---

## Next steps

- [Policies](policies.md) — restrict which agents may use the
  `network.credentialed_request` capability.
- [Connecting Services](connecting-services.md) — connect Gmail and Slack, which
  use their own managed tokens in the vault.
- [Approval Workflow](approval-workflow.md) — how sensitive operations reach the
  approval queue.
