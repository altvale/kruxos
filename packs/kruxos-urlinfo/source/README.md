# kruxos-urlinfo

URL introspection capabilities for KruxOS agents.

| Capability | Purpose | Network? |
|------------|---------|----------|
| `urlinfo.parse_host` | Parse a URL into scheme / host / port / path | No |
| `urlinfo.fetch_headers` | HTTPS HEAD probe — status + headers, no body | Yes |
| `urlinfo.robots_txt_check` | Fetch + apply robots.txt rules for a URL/agent | Yes |

## Install

```bash
kruxos pack install kruxos-urlinfo
```

## Security

- **HTTPS only** — `http://` inputs are rejected by `fetch_headers` and
  `robots_txt_check`. v0.0.2's KruxOS threat model treats plain-text
  HTTP traffic as out-of-scope.
- **No secrets required** — no API keys, no tokens.
- **`parse_host` is pure** — no network side effects.

## License

MIT
