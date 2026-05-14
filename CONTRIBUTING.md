# Contributing to KruxOS

This repository hosts KruxOS's community-contributable extension surfaces — capability packs, plugins, and themes — plus the appliance release artefacts on the [Releases](../../releases) page.

## Where contributions go

| Surface | Available | Notes |
|---|---|---|
| `packs/` — capability packs (Python / Node.js / service-proxy) | v0.0.2 | pack-SDK CLI + remote registry land in v0.0.2 |
| `plugins/` — extension plugins | v0.0.3 | plugin runtime ships v0.0.3 |
| `themes/` — dashboard themes | v0.0.3 | theme system ships v0.0.3 |
| `docs/public/` — public documentation | now | direct PRs welcome |

For v0.0.1, the only contribution surface that's currently live is `docs/public/` — typo fixes, clarification PRs, additional examples. The other surfaces are placeholder-only until their respective release cycles light them up.

## Reporting issues

- **Bug reports / feature requests** — GitHub Issues. Include reproduction steps, expected vs. actual behaviour, and the version (`kruxos version`).
- **Security disclosures** — see [SECURITY.md](SECURITY.md). Do **not** file public issues for security vulnerabilities.

## Pull request flow

1. Fork the repo and clone your fork.
2. Branch off `main`.
3. Make your changes (follow the per-surface conventions linked above once they ship).
4. Open a PR against `main` with a clear description of what changed and why.

For documentation PRs, please follow the existing prose style — concrete, literal, AI-readable. Capability descriptions especially should avoid metaphors.

## Testing docs locally

For `docs/public/` contributions, preview your changes before opening a PR:

```bash
cd docs/public
pip install mkdocs mkdocs-material
mkdocs serve
# browse http://localhost:8000
```

The live site at [docs.kruxos.com](https://docs.kruxos.com) is built from the same source with the same theme — what you see locally is what readers will see.

## License

By contributing to `packs/`, `plugins/`, `themes/`, or `docs/public/`, you agree that your contribution will be licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) — the same license as the surrounding directory. See each subdirectory's `LICENSE` file for the full text.

The KruxOS appliance is governed by the [End User License Agreement](https://altvale.com/legal/kruxos-eula); the open-source content of this repository is not subject to the EULA.

## Code of Conduct

All contributors are expected to follow our [Code of Conduct](CODE_OF_CONDUCT.md).
