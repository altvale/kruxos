# kruxos-code

Surgical, literal-string code editing for AI agents — packaged as an in-process
WASM pack on the KruxOS pack runtime.

| Capability | Purpose |
|------------|---------|
| `code.edit` | Make one exact `old_string` → `new_string` change to an existing text file |
| `code.multi_edit` | Apply several edits to one file atomically (all-or-nothing) |

## Why a dedicated edit capability

An agent has two reflexes for changing a file: rewrite the whole thing
(`filesystem.write`) or shell out to a stream editor (`sed`, `awk`, `perl -i`,
PowerShell `(Get-Content)-replace`) through `process.run`. Both are worse than a
purpose-built edit tool:

- **`code.edit` costs tokens proportional to the change, not the file.** You emit
  only the text you are replacing and its replacement — `O(change)`, not
  `O(file)` — and it physically cannot drop or alter the lines you did not touch
  (the classic whole-file-rewrite failure).
- **`code.edit` is a literal string, identical on every OS.** `old_string` is
  matched verbatim — no regex metacharacters to escape, no delimiter or quoting
  hazards, no GNU-vs-BSD `sed -i` differences, no PowerShell BOM/CRLF mangling.
- **`code.edit` gives a structured signal.** Instead of `sed` silently making
  zero substitutions (or rewriting every matching line), an ambiguous or missing
  match returns a typed `ambiguous_match` / `no_match` outcome with concrete,
  content-free recovery the agent can act on.

## Outcomes

Every call returns a JSON object with a top-level `outcome` discriminator:

- `applied` — the change was made. Carries `replacements`, `match_tier`,
  `bytes_before` / `bytes_after`, `lines_added` / `lines_removed`, a post-edit
  `checksum`, and the written `path` — so the agent confirms the write without
  re-reading.
- `ambiguous_match` — `old_string` matched more than one site. Carries
  `match_count` and `match_lines` (line numbers only). Nothing is written.
- `no_match` — `old_string` matched nothing. Nothing is written.
- `parse_error` — the request was malformed (empty `old_string`, `old == new`, a
  bad `edits[]` element, over the edit ceiling, or a too-generic `replace_all`).
- `stale_file` — an `expected_sha` was supplied and the file no longer matches.

Only `applied` changes the file, and only `applied` carries a `path`. Recovery
strings never include file content — only counts and line numbers.

## Matching

1. **Exact** — `old_string` is matched byte-for-byte and must be unique (set
   `replace_all: true` to change every occurrence). `match_tier: "exact"`.
2. **Whitespace-normalized** — if the exact match fails, each line is re-matched
   with its leading/trailing whitespace trimmed (internal whitespace and blank
   lines are never collapsed). A normalization that yields more than one match is
   reported as `ambiguous_match`, never silently resolved. `match_tier:
   "whitespace"`.

Line endings are handled deterministically: a uniformly-CRLF file has the model's
`\n` translated to `\r\n` before matching, and untouched bytes stay byte-exact.
Fuzzy / edit-distance matching is intentionally not supported — a "close enough"
match is the wrong-edit corruption mode.

## `code.multi_edit`

`code.multi_edit` takes an `edits[]` array (up to 50) applied sequentially over
one evolving buffer and committed with a single atomic write. It is
all-or-nothing: if any edit fails, the whole batch is rejected, zero bytes are
written, and the result names the failing edit index so you can fix it and
resubmit the complete array.

## Security

- **No secrets required.**
- **No network egress.** The module imports only the capability-gated host
  filesystem interface; every read and write is confined to the agent's mounted
  workspace and re-checked against the secrets denylist and write-protect rules
  host-side.
- **Bounded inputs.** Files up to 16 MiB; `old_string` / `new_string` up to 1 MiB
  each; at most 50 edits per `code.multi_edit`; `replace_all` requires an
  `old_string` of at least 3 characters and changes at most 1000 sites.
- **Permission tier `autonomous`** — edits are surgical, uniqueness-checked, and
  confined in-workspace, with the prior content snapshotted before each write.

## Install

On a KruxOS appliance, `kruxos-code` ships **bundled** — it is seeded into the
pack registry on first boot, so no install step is needed there. The command
below is for boxes where it is not bundled (a dev checkout or a non-appliance
run); on an appliance where it is already present it is an idempotent no-op.

```bash
kruxos pack install kruxos-code
```

## License

Apache-2.0
