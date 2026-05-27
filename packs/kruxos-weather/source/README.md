# kruxos-weather

Weather lookup capabilities for KruxOS agents. Wraps the
[open-meteo.com](https://open-meteo.com) public API — no API key
required.

| Capability | Purpose |
|------------|---------|
| `weather.current_conditions` | Real-time temperature / wind / weather code at a lat/lon |
| `weather.forecast` | 1-7 day daily forecast at a lat/lon |

## Install

```bash
kruxos pack install kruxos-weather
```

## Attribution

Both capabilities return an `attribution` field — surface this in
user-visible output per open-meteo.com's data licence (CC-BY 4.0):

> Weather data by Open-Meteo.com (CC-BY 4.0)

## Security

- **No secrets required** — open-meteo.com is unauthenticated.
- **HTTPS only** — `api.open-meteo.com` is the only outbound host
  contacted; HTTP is not used anywhere.
- **Bounded inputs** — coordinate range, day count, and temperature
  unit are validated before any network call.

## License

MIT
