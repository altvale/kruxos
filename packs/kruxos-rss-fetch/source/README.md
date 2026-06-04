# kruxos-rss-fetch

The canonical worked example for the
[Capability Design Guidelines](https://github.com/altvale/kruxos/blob/main/docs/public/docs/pack-authors/capability-design-guidelines.md)
— a single capability that exercises all seven rules. Also shipped as a
template in the [Pack SDK](https://github.com/altvale/kruxos-pack-sdk) under
`examples/rss-fetch/`.

| Capability | Purpose |
|------------|---------|
| `rss.fetch` | Fetch an RSS or Atom feed and return its entries as ready-to-use string fields |

## Install

```bash
kruxos pack install kruxos-rss-fetch
```

## Output

`rss.fetch` returns pre-parsed, ready-to-use fields — the agent never writes a
line of XML parsing:

- `entries[]` — newest-first, each `{ title, link, summary, published, id }` as strings
- `entry_count` — number returned (no `len()` needed)
- `truncated` — were there more entries than `limit`?
- `cache_hint_minutes` — re-poll interval from RSS `<ttl>` or the syndication module
- `feed_url` — final URL after redirects
- `feed_title` — the feed's title

## Security

- **No secrets required.**
- **Network egress** — issues one HTTP/HTTPS GET to the feed URL. Permission
  tier is `notify`.
- **Bounded inputs** — `url` is validated at the boundary (`InvalidURL` raised
  synchronously, before any network work; `file://` and non-http(s) schemes are
  rejected); `limit` (max 100) and `timeout` (max 60s) are clamped.

## Versioning

The pack is at its first packaging, `1.0.0`. The `rss.fetch` capability
contract is documented as `version: "1.1"` to match the guidelines doc's "See
also" reference ("rss-fetch v1.1 — the canonical exercise"). Pack version and
capability version are intentionally independent.

## License

MIT
