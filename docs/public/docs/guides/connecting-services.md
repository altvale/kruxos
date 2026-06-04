# Connecting Services (Gmail & Slack)

KruxOS never hands an agent a raw Gmail or Slack token. External services are
reached through the **Service Proxy**, which syncs to a local read-replica,
buffers writes, and escalates risky batches to the approval queue. Before any
of that can happen, an operator has to connect the account once. This guide
covers that one-time connect step.

You bring your own app — a Google OAuth client for Gmail, a Slack app for
Slack. KruxOS never ships shared credentials, so the account, its scopes, and
its tokens are entirely yours. Tokens are stored in the appliance vault; the
connect flow itself runs as the **User principal** over the loopback User API.

There are two equivalent ways to connect: the **dashboard** (point-and-click)
or the **`kruxos connect`** CLI. Both drive the same gateway endpoints — pick
whichever fits your workflow.

!!! note "Prerequisites"
    - The appliance is running and the **vault is unlocked** (connect stores
      tokens in the vault).
    - You have a **User token** (`krx_user_*`) — the CLI reads it from the
      vault automatically; the dashboard uses your session.

---

## Gmail

Gmail uses a full OAuth2 authorization-code flow (Google doesn't offer a
device-code flow for `gmail.*` scopes), so you'll click through a Google
consent screen in your browser. KruxOS requests three scopes:
`gmail.readonly`, `gmail.send`, and `gmail.modify`.

### Step 1 — Create a Google OAuth client

1. In the [Google Cloud console](https://console.cloud.google.com/apis/credentials),
   create a project and an **OAuth 2.0 Client ID** of type **Web
   application**.
2. Add the **redirect URI** KruxOS shows you to the client's *Authorized
   redirect URIs*. It is your dashboard origin plus `/api/oauth/gmail/callback`
   — for example `https://kruxos.local:7800/api/oauth/gmail/callback`. The
   dashboard connect form and the CLI both print the exact value to register.

!!! warning "Two things that trip everyone up"
    **1. Set the OAuth app's publishing status to "In production."** Then,
    when you authorize, **click through the "unverified app" warning**
    ("Google hasn't verified this app" → *Advanced* → *Go to … (unsafe)*). You
    do **not** need to submit the app for Google verification — it's your own
    account. Leaving the app in *Testing* mode instead works, but Google
    expires *Testing*-mode refresh tokens after 7 days, so the connection
    silently dies a week later. Production + click-through avoids that.

    **2. The dashboard must be reached over HTTPS or `http://localhost`.**
    Google rejects plain-`http` LAN redirect URIs (e.g.
    `http://192.168.1.50:7800/...`) for `gmail.*` scopes. Use HTTPS on the
    dashboard, or reach it via `http://localhost` (e.g. an SSH tunnel) while
    connecting.

### Step 2 — Connect

=== "Dashboard"

    1. Open the dashboard and go to the **Service Proxy** page (`/proxy`).
    2. Click **Connect** on the **Gmail** tile. The modal shows the exact
       redirect URI to register (with a copy button) and the two tips above.
    3. Paste your **Client ID** and **Client secret**, then start the flow.
    4. A Google consent screen opens. Approve access; the dashboard callback
       completes the connection and the Gmail tile flips to *connected*.

=== "CLI"

    ```bash
    kruxos connect gmail \
      --client-id <YOUR_CLIENT_ID> \
      --client-secret <YOUR_CLIENT_SECRET> \
      --dashboard-url https://kruxos.local:7800
    ```

    `--dashboard-url` is your dashboard origin; KruxOS appends
    `/api/oauth/gmail/callback` to form the redirect URI (pass `--redirect-uri`
    instead if you registered a different one). The command prints an
    authorization URL, tries to open your browser, and then polls until the
    dashboard callback completes the flow (default timeout 300s, tune with
    `--timeout-secs`).

    To keep the client secret out of your shell history, set it in the
    environment instead of `--client-secret`:

    ```bash
    export KRUXOS_GMAIL_CLIENT_SECRET=<YOUR_CLIENT_SECRET>
    kruxos connect gmail --client-id <YOUR_CLIENT_ID> --dashboard-url https://kruxos.local:7800
    ```

    Run `kruxos connect gmail` with no flags to print the guided setup (the
    redirect URI to register and the tips above) without starting a flow.

On success you'll see the connected account email and the granted scopes.

---

## Slack

Slack needs no OAuth dance: a **Bot User OAuth Token** (`xoxb-…`) *is* the
bearer the Service Proxy uses. You create a Slack app, install it to your
workspace, and paste the token. KruxOS validates it with Slack's `auth.test`
(which also returns the workspace name) before storing it.

### Step 1 — Create the Slack app from the manifest

1. Go to [api.slack.com/apps](https://api.slack.com/apps) → **Create New App**
   → **From a manifest**, and paste the manifest below. KruxOS prints this
   exact manifest in the connect form / CLI so the bot requests precisely the
   scopes the proxy needs.
2. **Install** the app to your workspace.
3. Copy the **Bot User OAuth Token** (it starts with `xoxb-`).

```json
{
  "display_information": { "name": "KruxOS Connector" },
  "features": {
    "bot_user": { "display_name": "kruxos", "always_online": true }
  },
  "oauth_config": {
    "scopes": {
      "bot": [
        "channels:read",
        "channels:history",
        "groups:read",
        "groups:history",
        "chat:write",
        "reactions:write",
        "users:read"
      ]
    }
  },
  "settings": {
    "org_deploy_enabled": false,
    "socket_mode_enabled": false,
    "token_rotation_enabled": false
  }
}
```

### Step 2 — Connect

=== "Dashboard"

    1. On the **Service Proxy** page (`/proxy`), click **Connect** on the
       **Slack** tile.
    2. The modal shows the manifest above and a link to the Slack apps
       console. Create + install the app, then paste the **Bot User OAuth
       Token** (`xoxb-…`) and connect.

=== "CLI"

    ```bash
    kruxos connect slack
    ```

    With no `--token`, the command prints the guided setup (manifest + steps)
    and then prompts you to paste the `xoxb-…` token on stdin — the least-
    exposed option. You can also pass `--token xoxb-…` (visible in shell
    history / `ps`) or set `KRUXOS_SLACK_TOKEN` in the environment.

KruxOS rejects anything that isn't a `xoxb-` bot token, and surfaces Slack's
own error if the token is invalid. On success it prints the workspace name and
granted scopes. (Bot tokens don't expire by default.)

---

## Checking status

```bash
kruxos connect status
```

Shows whether Gmail and Slack are connected, the connected identity (account
email / workspace), token expiry, and whether a connection needs attention
(reconnect recommended). The dashboard's Service Proxy page shows the same
status live on the Connect tiles.

Once a service is connected, the matching `email.*` / Slack capabilities
become available to agents through the Service Proxy's safety chain — see the
[Slack Integration](slack-integration.md) and [Connect Gmail](../quickstart/gmail.md)
pages for how agents use them.

## Troubleshooting

- **"redirect_uri must be an absolute URL"** — register and pass the full
  origin + `/api/oauth/gmail/callback`, not just a path.
- **Gmail authorization succeeds but the connection drops after ~7 days** —
  the OAuth app is still in *Testing* mode. Switch it to *In production* (see
  the warning above) and reconnect.
- **Google refuses the redirect URI** — the dashboard isn't on HTTPS or
  `http://localhost`. Reach it over HTTPS or a localhost tunnel while
  connecting.
- **"expected a Bot User OAuth Token (starts with xoxb-)"** — you pasted a
  user token (`xoxp-`) or an app-level token (`xapp-`); copy the **Bot User**
  OAuth token from the app's *OAuth & Permissions* page.
- **"vault is locked"** — unlock the vault (dashboard or `kruxos vault
  unlock`) before connecting; tokens can't be stored while it's locked.

## Next steps

- [Approval Workflow](approval-workflow.md) — how buffered writes and risky
  batches reach the approval queue
- [Policies](policies.md) — restrict which agents may use email / Slack
  capabilities
- [Monitoring](monitoring.md) — watch sync health and the write buffer
