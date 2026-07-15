# First-Boot Wizard

By the end of this page, you'll know what the first-boot wizard sets up and how to walk through each step.

On a fresh KruxOS install, the dashboard opens into a guided wizard. It configures the four things every appliance needs: **secrets**, **identity**, **host CLIs**, and **policy**.

```mermaid
flowchart LR
    W1[Welcome] --> W2[Vault]
    W2 --> W3[Workspace]
    W3 --> W4[AdminAgent]
    W4 --> W5[License]
    W5 --> W6[User token]
    W6 --> W7[CLI tools]
    W7 --> W8[SSH key]
    W8 --> W9[Remote access]
    W9 --> W10[Done]
```

## When the wizard appears

- First visit to `https://<appliance>:7800` after a fresh install
- After a factory reset (if `/data/kruxos/wizard.done` is removed)
- Re-run manually: `kruxos setup --reconfigure`

The progress rail at the top lets you click back to any completed step.

## Step-by-step

### 1. Welcome

Orientation card explaining what the wizard configures. Click **Get started**.

### 2. Vault passphrase

Set the passphrase that encrypts secrets in the vault. A live strength meter scores your choice before you submit. **Choose a strong passphrase** — there is no recovery if you forget it.

### 3. Workspace

Pick the AdminAgent's home directory. The default `/data/kruxos/users/admin` is auto-created. Use the **directory browser** to click through folders, or toggle **Type path instead** for a free-text input.

### 4. AdminAgent (Identity)

Name your first agent and optionally configure a model provider:

| Provider | What you enter |
|----------|----------------|
| Anthropic | API key |
| OpenAI | API key (or base URL for compatible providers) |
| OpenAI Codex | OAuth device-code flow |
| OpenRouter | API key |
| Local | Endpoint preset |
| Skip | Defer to Settings later |

Credentials and the agent record are saved atomically.

### 5. License

Paste a JWT license key or skip. KruxOS is free for personal use.

### 6. User token

Generates a `krx_user_*` bearer token shown **once**. Copy it — you'll need it for host CLIs and the User API. Acknowledge with the checkbox to continue.

### 7. Install CLI Tools

Optional. Installs Claude Code and/or Codex CLI with seed configs. Both can be installed later from **Integrations**.

### 8. SSH key (optional)

Paste an SSH public key if you plan to use [SSH access](ssh-access.md) later. You can skip and add keys from **Settings → System**.

### 9. Remote access (optional)

Optional Tailscale setup. Enable remote dashboard access from your tailnet. See [Remote Access](remote-access.md#built-in-tailscale-recommended) for details. Skip if you'll configure this later.

### 10. Done

Confirmation screen with a link to the main dashboard. Your appliance is ready.

## After the wizard

| What was created | Where to manage it later |
|------------------|-------------------------|
| Vault passphrase | Cannot be changed easily — plan ahead |
| AdminAgent | **Agents** page |
| User token | **Identities** page |
| Model provider | **Settings → Models** |
| CLI seed configs | **Integrations** page |
| SSH key | **Settings → System** |
| Tailscale | **Settings → System → Remote access** |

## Re-run the wizard

```bash
kruxos setup --reconfigure
```

Or use the dashboard if a reconfigure entry point is available. Re-running does not destroy existing agents or data — it walks through configuration again.

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| Wizard loops back to step 1 | Check `/data/kruxos/wizard.done` exists after completion |
| Provider registration fails | Verify API key; agent is not created if provider fails |
| Lost User token | Create a new one on **Identities** — the wizard token still works if you saved it |
| Gmail OAuth fails during wizard | Gmail connect is not part of the wizard — use [Connecting Services](connecting-services.md) after |

## Next steps

- [Getting Started](../getting-started.md) — connect your first AI model
- [Web Dashboard](../quickstart/dashboard.md) — tour every page
- [Host CLI Integrations](host-cli-integrations.md) — wire up Claude Code or Codex