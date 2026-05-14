# Secrets Capabilities

List available secrets and request rotation. Values are never exposed.

## Overview

| Capability | Permission | Purpose |
|------------|:----------:|---------|
| [`secrets.list`](#secretslist) | 🟢 Autonomous | Lists the names and metadata of secrets the agent can access. |
| [`secrets.use`](#secretsuse) | 🔵 Notify | Marker capability — agents do not call this directly. |
| [`secrets.rotate_request`](#secretsrotate_request) | 🔵 Notify | Requests rotation of an expiring or compromised secret. |

## `secrets.list`

**Permission:** 🟢 Autonomous · **Version:** 1.0

> Lists the names and metadata of secrets the agent can access. Never returns secret values.

### When to use

Use secrets.list to discover what secrets are available and their types (OAuth token, API key, etc.).
Secret values are never exposed to agents — they are injected into capability execution automatically.
Use secrets.rotate_request if a secret is expiring.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `include_revoked` | `Boolean` | No | `False` | If true, include revoked secrets in the listing. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `secrets` | `Array` | Array of secret metadata objects, each with: name, type ('oauth_token', 'api_key', 'agent_auth_token', 'encryption_key'), allowed_capabilities (array of capability patterns), created_at, revoked (boolean). |
| `total` | `Integer` | Total number of secrets listed. |

### Common patterns

**Check available credentials before connecting to a service**

1. `secrets.list() to see available secrets`
2. `Check allowed_capabilities to see which capabilities can use each secret`

**Check for expiring secrets**

1. `secrets.list() to see all secrets`
2. `If a secret shows needs_attention, use secrets.rotate_request`

### Errors

**`VaultLocked`** — The secrets vault is locked. A supervisor must unlock it.

- **alert_supervisor**: Use alerts.send to notify the supervisor that the vault needs to be unlocked.

**`VaultError`** — Internal vault error.

- **retry**: Retry the operation.

**Tags:** `secrets` `vault` `safe` `read`

---

## `secrets.use`

**Permission:** 🔵 Notify · **Version:** 1.0

> Marker capability — agents do not call this directly. Secrets are injected into capabilities that need them. This definition exists for policy and audit purposes.

### When to use

You do not call secrets.use directly. When you invoke a capability that needs a secret
(e.g. network.http_request with an API key), the system automatically injects the secret
into the execution environment. This capability exists so that policy rules can control
which agents can use secrets and for audit logging.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `secret_name` | `String` | Yes | — | Name of the secret being used (for audit purposes). |
| `capability` | `String` | Yes | — | The capability that is consuming the secret. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `used` | `Boolean` | Always true if the operation succeeds. |

### Side effects

- Accesses the secret value internally. The value is never exposed to the agent. *(not reversible)*

### Common patterns

**Implicit usage**

1. `This capability is invoked automatically by the system, not by agents`

### Errors

**`SecretNotFound`** — The named secret does not exist.

- **check_secrets**: Use secrets.list to see available secrets.

**`ScopeViolation`** — The calling capability is not in the secret's allowed_capabilities list.

- **check_scope**: Use secrets.list to see which capabilities can use each secret.

**`VaultLocked`** — The secrets vault is locked.

- **alert_supervisor**: Use alerts.send to notify the supervisor.

**Tags:** `secrets` `vault` `internal`

---

## `secrets.rotate_request`

**Permission:** 🔵 Notify · **Version:** 1.0

> Requests rotation of an expiring or compromised secret. Creates a supervisor alert to handle the rotation.

### When to use

Use secrets.rotate_request when you detect that a secret is about to expire, has failed
authentication, or may be compromised. The supervisor will receive an alert and handle
the actual rotation.

### Inputs

| Parameter | Type | Required | Default | Description |
|-----------|------|:--------:|---------|-------------|
| `secret_name` | `String` | Yes | — | Name of the secret to rotate. |
| `reason` | `String` | No | `expiring` | Reason for the rotation request: 'expiring', 'compromised', 'auth_failure'. |

### Outputs

| Field | Type | Description |
|-------|------|-------------|
| `request_id` | `String` | Unique identifier for the rotation request. |
| `secret_name` | `String` | The secret that rotation was requested for (echo of input). |
| `status` | `String` | Status of the request: 'submitted' (supervisor notified). |

### Side effects

- Creates an alert visible to the supervisor requesting secret rotation. *(not reversible)*

### Common patterns

**Request rotation of an expiring OAuth token**

1. `secrets.rotate_request(secret_name='gmail_oauth', reason='expiring')`
2. `The supervisor will handle the actual token refresh`

### Errors

**`SecretNotFound`** — The named secret does not exist.

- **check_secrets**: Use secrets.list to see available secrets.

**`VaultLocked`** — The secrets vault is locked.

- **alert_supervisor**: Use alerts.send to notify the supervisor.

**Tags:** `secrets` `vault` `rotation`

---
