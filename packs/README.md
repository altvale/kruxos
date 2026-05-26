# KruxOS Capability Packs

This directory is the source of the KruxOS public capability pack registry,
hosted at **https://docs.kruxos.com/packs/**. Cloudflare Pages copies this
directory into the published docs site on every merge to `main` (and on
preview builds for each PR).

## What's a pack?

A capability pack adds new capabilities to KruxOS. The pack SDK supports
three pack types:

- **Python** — direct Python implementation of capability handlers
- **Node.js** — direct Node.js implementation
- **Service-proxy** — wraps an external service (e.g. Gmail, Slack) with
  read-replica + write-buffering semantics

## Registry layout

```
packs/
├── index.json                           # this registry's manifest (schema below)
├── README.md                            # this file
├── LICENSE                              # Apache 2.0 for KruxOS-published packs
└── <pack-name>/
    └── <version>/
        └── <pack-name>-<version>.tar.gz # the tarball clients fetch
```

`index.json` is the operator-facing entrypoint:
`https://docs.kruxos.com/packs/index.json`.

## `index.json` schema (`registry_version: 1`)

The index is a wrapped envelope — a JSON object with a `packs[]` array.

```json
{
  "registry_version": "1",
  "updated_at": "2026-05-26T00:00:00Z",
  "packs": [
    {
      "name": "hello-world",
      "version": "1.0.0",
      "description": "Minimal example pack",
      "author": "KruxOS Team",
      "license": "Apache-2.0",
      "pack_type": "python",
      "minimum_kruxos_version": ">=0.0.2",
      "tarball_url": "https://docs.kruxos.com/packs/hello-world/1.0.0/hello-world-1.0.0.tar.gz",
      "checksum_sha256": "<64-char-hex>",
      "capabilities": ["hello_world.echo"],
      "tags": ["example"],
      "security_review_required": false,
      "published_at": "2026-05-26T00:00:00Z",
      "homepage": "https://example.com/hello-world"
    }
  ]
}
```

### Envelope fields

- `registry_version` — schema version. Required. Currently `"1"`.
- `updated_at` — ISO-8601 timestamp of the most recent write. Recommended;
  clients use it as a cache freshness hint.
- `packs[]` — pack entries.

### Pack entry fields

**Required:** `name`, `version`, `description`, `license`, `tarball_url`,
`checksum_sha256`.

**Conventional (seed packs always carry these):** `author`, `pack_type`,
`minimum_kruxos_version`, `capabilities`, `tags`, `security_review_required`,
`published_at`.

**Optional:** `homepage`.

### Validation rules

- `name`: lowercase, hyphens, must start with a letter (`^[a-z][a-z0-9-]*$`)
- `version`: semver (`^\d+\.\d+\.\d+$`)
- `checksum_sha256`: 64-character hex SHA-256
- `tarball_url`: must use HTTPS
- `pack_type`: one of `python`, `nodejs`, `proxy`
- No duplicate `name` within `packs[]`

## Status — v0.0.2

The registry shipped initially empty. Seed packs (`hello-world`,
`filesystem-utils`, `email-tools`) land alongside the v0.0.2 tag once
the wider Pack SDK work completes. Until then, this `index.json`
intentionally carries an empty `packs[]` array — clients see the
schema, parse cleanly, and find nothing to install.

## Local-path install still works

The `kruxos pack install <local-path>` flow is unchanged from v0.0.1 —
operators with packs already on the appliance can install without
touching this registry. See the [pack docs](../docs/) for the local
install procedure.

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md). The pack SDK CLI
(`@kruxos/pack-sdk` on npm; bundled in the appliance as
`/opt/kruxos/bin/kruxos-pack`) handles tarball + entry generation:
`kruxos-pack publish` produces a `dist/<name>.json` ready to PR
into `packs/` here.

## License

Capability packs published by the KruxOS team in this directory are
licensed under [Apache 2.0](LICENSE). Community-submitted packs carry
their own SPDX `license` field — verify per-pack before reuse.
