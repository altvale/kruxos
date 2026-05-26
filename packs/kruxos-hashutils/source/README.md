# kruxos-hashutils

Example KruxOS capability pack bundling three pure-compute utilities:
`hash.sha256` and `hash.md5` (text → hex digest) and `hash.uuid4` (random
UUID v4 generator). No network, no secrets, stdlib only. Exercises the
multi-capability shape where a single definitions YAML lists several
capabilities and a single implementations module exposes their handlers.
