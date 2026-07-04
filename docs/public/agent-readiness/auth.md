# KruxOS auth.md

> Agent authentication and credential provisioning for KruxOS appliances.

## Two principals

| Principal | Port | Credential | Format |
|-----------|------|------------|--------|
| **Agent** | 7700 (MCP WebSocket) | API key | 64-char hex |
| **User** (operator/CLI) | 7703 (HTTP, loopback) | Bearer token | `krx_user_<base64url>` |
| **Operator** (dashboard) | 7800 (HTTPS) | Passphrase | bcrypt hash from vault step |

## Agent authentication (MCP :7700)

Agents **must** authenticate. No anonymous MCP access.

```
Endpoint:  ws://<host>:7700/mcp
Transport: WebSocket
Auth:      Authorization: Bearer <64-char-hex>
           or ws://<host>:7700/mcp?token=<64-char-hex>
```

Provision via `kruxos agent create --name <name>` or the AdminAgent wizard step.

## User API (:7703)

```http
Authorization: Bearer krx_user_...
```

Provision via wizard step 6, `kruxos user-token create --label <label>`, or Dashboard → Settings → Tokens.

Environment variable for bridges: `KRUXOS_USER_TOKEN`.

## Supervision WebSocket (:7701)

Requires `krx_user_*` bearer or active operator dashboard session.

## Discovery

| Resource | URL |
|----------|-----|
| MCP gateway docs | https://docs.kruxos.com/developers/api/gateway-mcp/ |
| REST / User API | https://docs.kruxos.com/developers/api/rest-api/ |
| OAuth metadata | https://docs.kruxos.com/.well-known/oauth-authorization-server |
| MCP server card | https://docs.kruxos.com/.well-known/mcp/server-card.json |

## Further reading

- [Install guide](https://docs.kruxos.com/quickstart/install/)
- [Dashboard wizard](https://docs.kruxos.com/quickstart/dashboard/)
- [Managing agents](https://docs.kruxos.com/guides/managing-agents/)
- [CLI reference](https://docs.kruxos.com/quickstart/cli/)