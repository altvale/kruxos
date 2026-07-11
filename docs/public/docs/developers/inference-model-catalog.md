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

4. **Validate locally** before opening the PR:

   ```bash
   python3 -m json.tool docs/public/docs/inference/models.json > /dev/null && echo OK
   ```

5. **Open a pull request** describing the model, its license, and how you
   obtained the `sha256`/`size_bytes` (paste the commands above and their
   output). Flag anything unusual about the license.

## Review and trust

**Maintainer review is the trust gate.** Every entry lands only via a
pull request that a maintainer reviews and merges — there is no automatic
ingestion. Reviewers check that the license is genuinely permissive, that the
`sha256`/`size_bytes` are real and verifiable by the method above, and that the
`id` is sane and additive.

Because the catalog is additive-only and hash-verified, the worst a bad entry
can do is fail to install (a hash mismatch is refused on the appliance). It can
never shadow, downgrade, or replace a built-in model.

## What happens on the appliance

When an appliance loads the catalog, it:

- **Fetches over HTTPS with a size cap** — the download is bounded, so a
  hostile or misconfigured host cannot flood the appliance.
- **Validates the schema** — malformed entries (bad id, non-https url, wrong
  `sha256` format, zero size) are dropped individually; one bad entry never
  breaks the rest of the catalog.
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
