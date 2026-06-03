# Getting started

Build a KruxOS capability pack with the Pack SDK.

## Install

```bash
npm install -g @kruxos/pack-sdk
```

Requires Node.js >= 18. Python packs also need `python3` (3.10+).

## Create → test → publish

```bash
kruxos-pack create --type python my-pack
cd my-pack
# implement your capability in src/capabilities.py + definitions/*.yaml
kruxos-pack lint      # check the capability definitions
kruxos-pack test      # schema checks + your tests
kruxos-pack docs      # generate the pack README
kruxos-pack publish   # build the tarball + show how to distribute it
```

## Distributing a pack

`kruxos-pack publish` builds `dist/<pack>.tar.gz` and prints your options:

- **Community registry** — open a PR adding your pack under `packs/`; a maintainer reviews and merges. Operators then browse and install it from their dashboard.
- **Share the tarball** — hand anyone `dist/<pack>.tar.gz`; they install it from the KruxOS dashboard (`/packs → upload`). No hosting needed — the tarball is self-contained.
- **Your own registry** — host the tarball plus an `index.json` and point operators' `KRUXOS_PACK_REGISTRY_URL` at it.

## Next

- [Capability Design Guidelines](./capability-design-guidelines.md) — how to write capabilities AI agents use efficiently.
- Source and issues: [github.com/altvale/kruxos-pack-sdk](https://github.com/altvale/kruxos-pack-sdk).
