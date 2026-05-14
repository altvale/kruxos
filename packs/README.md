# KruxOS Capability Packs

This directory will host community capability packs starting in **v0.0.2**.

## What's a pack?

A capability pack adds new capabilities to KruxOS. The pack-SDK supports three pack types:

- **Python** — direct Python implementation of capability handlers
- **Node.js** — direct Node.js implementation
- **Service-proxy** — wraps an external service (e.g. Gmail, Slack) with read-replica + write-buffering semantics

## Status

v0.0.1 ships local-path pack install only — `kruxos pack install <local-path>` works on a running appliance. There is no remote registry yet and no operator-facing path to fetch packs from this repository, so checking source packs in here for v0.0.1 would be theatre.

The community registry (`kruxos pack install <name>`), pack-SDK CLI distribution (`@kruxos/pack-sdk` on npm), GitHub-PR-based publishing flow, and the initial seed packs all land together in v0.0.2.

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for the contribution flow once packs ships in v0.0.2.

## License

Capability packs in this directory are licensed under [Apache 2.0](LICENSE).
