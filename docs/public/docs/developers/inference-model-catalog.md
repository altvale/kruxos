# Inference Model Catalog

By the end of this page, you'll know how the community inference-model catalog
works and how to contribute a new local model to it.

KruxOS ships a small, curated set of **built-in** GGUF models that every
appliance can pull and run on-device (see
[On-Appliance Inference](../guides/on-appliance-inference.md)). The **community
catalog** is a separate, contributor-maintained list that lets the community
offer *additional* models in the appliance's model picker — without shipping a
new appliance image.

## What the catalog is

The catalog is a single JSON file published at:

```
https://docs.kruxos.com/inference/models.json
```

which is served from this documentation repository at
`docs/public/docs/inference/models.json`.

Every appliance fetches this file and merges it with its built-in list, so a
model you add here shows up in the **Settings → Inference** catalog on
appliances everywhere. Two properties make this safe to open to contributions:

- **The appliance verifies every download.** Each entry carries a `sha256`.
  The appliance hashes the file it downloads and refuses to install it unless
  the hash matches exactly. A wrong, truncated, or tampered download is
  rejected — never run.
- **The catalog can only *add* models.** A community entry can introduce a new
  model, but it can never modify or replace a built-in model. The appliance
  enforces this: on any id collision, the built-in always wins, and reserved
  id prefixes (`hf-`, `byom-`) are refused outright.

## The file format

`models.json` is an object with a `schema_version` and a `models` array:

```json
{
  "schema_version": 1,
  "models": [
    {
      "id": "tinyllama-1.1b-chat-v1.0-q4_k_m",
      "label": "TinyLlama 1.1B Chat v1.0 (Q4_K_M)",
      "params": "1.1B",
      "license_tag": "Apache-2.0",
      "url": "https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/52e7645ba7c309695bec7ac98f4f005b139cf465/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf",
      "sha256": "9fecc3b3cd76bba89d504f29b616eedf7da85b96540e490ca5824d3f7d2776a0",
      "size_bytes": 668788096,
      "description": "Compact 1.1B-parameter Llama chat model, 4-bit Q4_K_M GGUF."
    }
  ]
}
```

### Field reference

| Field | Required | Rules |
|-------|----------|-------|
| `id` | **Yes** | A safe slug: characters `A–Z a–z 0–9 . _ -` only, 1–128 chars, no `..`, no leading `.` or `-`. Must be **unique**, must **not** collide with a built-in id, and must **not** start with `hf-` or `byom-` (those prefixes are reserved for operator-pulled models). |
| `url` | **Yes** | A direct `https://` download URL to the `.gguf` file. Only `https` is accepted — `http`, `file`, and other schemes are refused. |
| `sha256` | **Yes** | The file's SHA-256, exactly **64 lowercase hex** characters. This is the integrity anchor the appliance checks before use. |
| `size_bytes` | **Yes** | The file's exact size in bytes, a nonzero integer. Also used as a download cap. |
| `label` | No | Human-friendly display name (e.g. `TinyLlama 1.1B Chat`). |
| `params` | No | Parameter count string (e.g. `1.1B`). |
| `license_tag` | No | Short SPDX-style license tag (e.g. `Apache-2.0`, `MIT`). |
| `description` | No | One or two sentences shown in the picker. |

!!! warning "Do not add these keys"
    Do **not** include a `source` field (the appliance sets it) or a
    `default_model_id` key (only the built-in list may steer the default). The
    whole file must also stay well under 1 MiB.

## How to add a model

1. **Pick a suitable model.** It must be:
   - a **GGUF** file that llama.cpp can run (a quantized instruct model such as
     `Q4_K_M` is a good fit for small appliances);
   - **permissively licensed** — MIT, Apache-2.0, or a similarly open license.
     Models with restrictive or bespoke terms-of-use should be flagged clearly
     in the PR and may be declined;
   - a **new** model — not one already in the built-in set.

2. **Get the `sha256` and `size_bytes` without downloading the weights.**
   Hugging Face stores large files with Git LFS, and for an LFS file the object
   id *is* the file's SHA-256. You can read both values from the LFS metadata:

   ```bash
   # x-linked-etag is the sha256; x-linked-size is size_bytes
   curl -sI 'https://huggingface.co/<org>/<repo>/resolve/<revision>/<file>.gguf' \
     | grep -iE 'x-linked-etag|x-linked-size'
   ```

   Cross-check the same two values from the repository tree API:

   ```bash
   # prints: <path> <lfs.oid = sha256> <lfs.size = size_bytes>
   curl -s 'https://huggingface.co/api/models/<org>/<repo>/tree/<revision>' \
     | python3 -c "import sys,json; [print(f['path'], f['lfs']['oid'], f['lfs']['size']) for f in json.load(sys.stdin) if f['path'].endswith('.gguf')]"
   ```

   The `x-linked-etag` (minus its quotes) must equal the tree API's `lfs.oid`,
   and `x-linked-size` must equal `lfs.size`. Use those exact values.

    !!! tip "Pin the URL to a revision"
        Use a specific commit hash in the `url` (`…/resolve/<commit>/…`) rather
        than `…/resolve/main/…`, so the bytes behind the URL can never change
        after review. If the file is ever re-uploaded, the pinned URL keeps
        pointing at the reviewed bytes — and even if it didn't, the `sha256`
        check would reject a mismatched download.

3. **Edit `docs/public/docs/inference/models.json`** — add your object to the
   `models` array. Keep the existing entries intact.

4. **Validate the shape locally** before opening the PR — a plain JSON syntax
   check is *not* enough (see [Validate before you open a PR](#validate-before-you-open-a-pr)
   below). Run the shape check and fix anything it flags.

5. **Open a pull request** describing the model, its license, and how you
   obtained the `sha256`/`size_bytes` (paste the commands above and their
   output). Flag anything unusual about the license.

## Validate before you open a PR

!!! danger "A structural mistake breaks the catalog for *every* appliance"
    The appliance deserializes the whole file in one pass, so a **missing or
    misspelled required key**, or a **wrong field type** — for example
    `"size_bytes": "123"` (a string) instead of `123` (an integer) — fails the
    *entire* file and takes every appliance's catalog offline until it's fixed.
    These mistakes are still valid JSON, so a plain syntax check (`json.tool`)
    passes them. Per-entry dropping only rescues entries that *parse* but fail a
    value rule (bad id, non-https url, malformed `sha256`, zero size) — it does
    **not** rescue a shape error.

Validate the *shape*, not just the JSON syntax:

```bash
python3 - <<'PY'
import json, re
d = json.load(open("docs/public/docs/inference/models.json"))
assert d.get("schema_version") == 1, "schema_version must be 1"
slug = re.compile(r"^[A-Za-z0-9._-]{1,128}$")
for m in d["models"]:
    for k in ("id", "url", "sha256", "size_bytes"):
        assert k in m, f"missing required key: {k}"
    i = m["id"]
    assert slug.match(i) and ".." not in i and i[0] not in ".-", f"bad id: {i}"
    assert not i.startswith(("hf-", "byom-")), f"reserved id prefix: {i}"
    assert m["url"].startswith("https://"), f"url must be https: {i}"
    assert re.fullmatch(r"[0-9a-f]{64}", m["sha256"]), f"sha256 must be 64 lowercase hex: {i}"
    assert isinstance(m["size_bytes"], int) and not isinstance(m["size_bytes"], bool) \
        and m["size_bytes"] > 0, f"size_bytes must be an integer > 0: {i}"
print("OK —", len(d["models"]), "entries valid")
PY
```

## Review and trust

**Maintainer review is the trust gate.** Every entry lands only via a
pull request that a maintainer reviews and merges — there is no automatic
ingestion. Reviewers check that the license is genuinely permissive, that the
`sha256`/`size_bytes` are real and verifiable by the method above, and that the
`id` is sane and additive.

Be precise about what the `sha256` proves and what it doesn't. It guarantees
every appliance receives **exactly the bytes the reviewer approved** — it does
*not* prove the model is benign. A malicious model that matches its own declared
hash still installs and runs; the hash only rules out substitution and
corruption, not intent. That is why **maintainer PR review is the trust gate**
for what a model actually is.

Two things bound the blast radius even so:

- The catalog is **additive-only** — an entry can never shadow, downgrade, or
  replace a built-in model.
- The inference engine runs models in a **sandboxed, unprivileged service** —
  no network beyond a local socket, a read-only model store, and no access to
  your vault or agent state — so a hostile model's worst case is bad *outputs*,
  not a compromised appliance.

## What happens on the appliance

When an appliance loads the catalog, it:

- **Fetches over HTTPS with a size cap** — the download is bounded, so a
  hostile or misconfigured host cannot flood the appliance.
- **Validates each entry** — an entry that *parses* but fails a value rule (bad
  id, non-https url, malformed `sha256`, zero size) is dropped on its own and
  the rest of the catalog still loads. A **structural** error is different — a
  missing or misspelled required key, or a wrong field type fails the
  whole-file parse (see [Validate before you open a PR](#validate-before-you-open-a-pr)),
  so keep the shape valid.
- **Merges built-ins first** — built-in models always win an id collision, and
  reserved-prefix entries are refused.
- **Caches the result** and **degrades gracefully** — if the catalog is
  unreachable, the appliance simply uses its built-in models, so the model
  picker never breaks.

## Next steps

- [On-Appliance Inference](../guides/on-appliance-inference.md) — pull and run
  local GGUF models on the appliance
- [Local Models connector](connectors/local.md) — connect an external Ollama or
  vLLM server instead
