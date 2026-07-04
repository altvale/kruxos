# Contributing to KruxOS

KruxOS community-extensible directories — `packs/`, `plugins/`, `themes/`, and `docs/public/` are licensed under [Apache 2.0](https://github.com/altvale/kruxos/blob/main/packs/LICENSE).

For the streamlined version of this page, see the repo root's [CONTRIBUTING.md](https://github.com/altvale/kruxos/blob/main/CONTRIBUTING.md).

## Ways to contribute

| Contribution | Where | Available |
|---|---|---|
| Bug reports + feature requests | [GitHub Issues](https://github.com/altvale/kruxos/issues) | now |
| Documentation improvements (`docs/public/`) | Pull requests to the public repo | now |
| Capability packs (`packs/`) | Pack development flow ships with v0.0.2 | v0.0.2 |
| Dashboard themes (`themes/`) | Theme system ships v0.0.3 | v0.0.3 |
| Extension plugins (`plugins/`) | Plugin runtime ships v0.0.3 | v0.0.3 |
| Community discussion | [KruxOS Discord](https://discord.gg/VXvQKNv6Jn) | now |

For v0.0.1 specifically, the only live contribution surface is `docs/public/` — typo fixes, clarifications, additional examples. The other surfaces are placeholder-only until their release cycles light them up.

## Working on docs

The public docs are built with [mkdocs Material](https://squidfunk.github.io/mkdocs-material/). Preview your changes locally before opening a PR:

```bash
git clone https://github.com/altvale/kruxos.git
cd kruxos/docs/public
pip install mkdocs mkdocs-material
mkdocs serve
# browse http://localhost:8000
```

The live site at [docs.kruxos.com](https://docs.kruxos.com) is built from the same source with the same theme — what you see locally is what readers will see.

### Style

- Concrete, literal, AI-readable prose. Capability descriptions especially should avoid metaphors — agents read these descriptions to decide which tool to call.
- Match the existing cadence of nearby pages rather than introducing new section structures.

## Working on capability packs (v0.0.2+)

The pack development flow — pack-SDK CLI, registry-based publishing, scaffolded examples — ships with v0.0.2. Until then, `packs/` accepts placeholder material only; substantive pack PRs should wait for the v0.0.2 docs that land alongside the SDK.

In v0.0.1 today, you can develop packs **locally** against a running appliance via the local-path install:

```bash
# On a v0.0.1 appliance, after building your pack directory:
kruxos pack install /path/to/my-pack
```

The pack-development documentation lives under [developers/packs/](packs/quickstart.md) and describes the runtime contract for handlers and the YAML capability definitions, even though the SDK distribution path arrives in v0.0.2.

## Pull request flow

1. Fork the [public repo](https://github.com/altvale/kruxos) and clone your fork.
2. Branch off `main`: `git checkout -b docs/typo-fix-in-quickstart`.
3. Make your changes (follow the prose style above for docs).
4. Open a PR against `main` with a clear description of what changed and why.

### PR guidelines

- One change per PR — typo fixes, clarification PRs, additional examples should each be their own PR.
- For docs PRs: render with `mkdocs serve` and click through the affected pages before opening the PR.
- For pack PRs (v0.0.2+): include the YAML capability definition, the handler implementation, and at least one example invocation in the PR description.

## Reporting security issues

To report a security vulnerability, email **security@altvale.com**. Do **not** file a public issue. See [SECURITY.md](https://github.com/altvale/kruxos/blob/main/SECURITY.md) for the disclosure timeline.

## License

By contributing to `packs/`, `plugins/`, `themes/`, or `docs/public/`, you agree that your contribution is licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) — the same license as the surrounding directory.
