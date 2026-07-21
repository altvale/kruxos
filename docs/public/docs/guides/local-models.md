# Local Models (Hugging Face)

KruxOS ships an **on-box inference engine** — a self-hosted model that runs on
the appliance itself, so a conversation or an agent can use a local model with
nothing leaving your hardware and no cloud key. This guide covers the
**Settings › Local Models** tab (and its `kruxos inference` CLI mirror): how to
find and pull a GGUF model — from the built-in catalog or straight from Hugging
Face — and how to enable, disable, and remove the models you have installed.

!!! note "Prerequisites"
    - The appliance is running and you can reach the **dashboard** (or the
      **appliance console** for the CLI / advanced steps).
    - Enough **free storage** for the model weights. Local models are large —
      a few hundred MB for a tiny model, tens of GB for a big one. The tab shows
      your free model storage and warns before a download that won't fit.

!!! info "This is the on-box engine, not a model *provider*"
    A local model *provider* points KruxOS at an external server you run
    yourself (Ollama, vLLM, LM Studio) — see
    [Model Providers](model-providers.md) and
    [Connect Local Models](../quickstart/connect-local.md). This page is about
    the engine **built into the appliance**, whose models you install right here
    in the dashboard.

---

## The Local Models tab at a glance

Open the dashboard and go to **Settings › Local Models**. The tab is organised
into four sections:

| Section | What it shows |
|---------|---------------|
| **Status** | Engine state and the **active model**, plus (when available) your **system memory** and **free model storage**. |
| **Available models** | Everything installed on this appliance, each with an **Active** badge and **Enable** / **Disable** / **Remove** controls. |
| **Get models** | The curated catalog to pull from, plus the **Search Hugging Face** panel. |
| **Advanced: custom model (BYOM)** | A collapsed section for pulling a model from a direct URL + checksum. |

The suggested model carries a **Recommended** badge, and when nothing is
installed yet it is pre-selected so you have an obvious first choice.

---

## Pull a model from the catalog

The simplest path is the built-in catalog under **Get models**:

1. Find a model in the list. The **Recommended** one is a good default. Each
   entry shows its license: conditioned licenses carry their attribution copy
   automatically (for example *"Built with Llama"*), and an amber
   **License: review** badge flags a non-commercial, restricted, or
   unrecognized license, with the full note in its tooltip. The badges are
   advisory only — they never block pulling or enabling a model.
2. Click **Pull**.
3. A **Confirm download** dialog states the exact size against your free model
   storage, an estimated hardware-fit grade (**Fits** / **Tight** / **May not
   fit**), and whether the model will be **enabled now** or **staged for
   later**. An unusually large model needs an extra explicit acknowledgement
   before the **Pull** button unlocks — so a huge download is never one click
   away.
4. Confirm. The download shows its own progress with a **Cancel** button and no
   longer freezes the rest of the tab. When it finishes, the result is marked
   right on the model you pulled (**Staged**, **Enabled**, or **Failed**).

### Does pulling switch my active model?

Not by default. **The first model you pull activates automatically** so you have
something running. After that, a pull while a model is already active is
**staged** — it lands in **Available models** without switching your running
model. You choose when to enable it.

---

## Search and pull from Hugging Face

Under **Get models**, the **Search Hugging Face** panel lets you pull any
(non-gated) GGUF model from the Hub:

1. Type a few letters — e.g. `qwen`, `llama`, `mistral`. Results appear as you
   type, each card showing **downloads**, **likes**, **parameter count**,
   **context length**, and **licence** — with the same advisory license badges
   as the catalog (attribution copy for conditioned licenses, amber
   **License: review** for non-commercial, restricted, or unrecognized ones).
   Sort by most downloads, most likes, or trending.
2. Click **View downloads** on a model. Its GGUF files are grouped **Small** /
   **Balanced** / **Large**, with one option marked **Recommended** (the widely
   used `Q4_K_M` quantisation) — no wall of cryptic filenames.
3. Each option shows its exact size and an estimated **Fits** / **Tight** /
   **May not fit** tip for your appliance's memory and storage. A multi-part
   (sharded) model appears as a single **"N parts"** row with the total size.
4. Click **Pull** on the option you want. The same pre-download confirmation as
   the catalog appears; confirm to start.

You never hand-type a checksum — the size and `sha256` for the file (or the
whole shard set) are resolved for you automatically.

!!! note "Gated models"
    Models that require a Hugging Face account (gated repositories, such as some
    Llama-family repos) are shown for reference but **cannot be pulled in this
    release**. They carry a **Gated** badge and a link to view the model on
    Hugging Face instead of a Pull button.

---

## Enable, disable, and remove installed models

Every installed model lives under **Available models** with three controls:

- **Enable** — make this model the **active** one. Enabling **restarts the
  engine** and aborts any in-flight chat completions, so the dashboard asks you
  to confirm first. If the newly enabled model fails to load, KruxOS
  automatically restores the previous one. (This is the same action the CLI
  calls `activate`.)
- **Disable** — **unload the model from memory but keep it on disk.** This frees
  the engine's RAM and stops it; you can **re-enable it later without
  re-downloading**. Use this to reclaim memory without losing the weights.
- **Remove** — **delete the model's files** from the appliance. Removing
  confirms the disk space it will reclaim before deleting, and cannot be undone.
  Remove is disabled on the active model — disable it first.

!!! tip "Disable vs. Remove"
    **Disable** keeps the weights (fast to bring back); **Remove** deletes them
    (you'd have to pull again). Disable to save memory; remove to save disk.

If a model's files ever go missing on disk, it stays listed with a clear note
and a **Remove** to tidy it up — **Enable** stays disabled until you pull it
again.

---

## Custom model (BYOM)

To pull a model that isn't in the catalog or on the Hub — for example one you
host yourself — expand **Advanced: custom model (BYOM)** under Get models and
provide three things:

- a direct **`https`** URL to a `.gguf` file,
- its **`sha256`** checksum (64 hex characters),
- its **size in bytes**.

The download is verified against the checksum before the model is activated.

---

## Manage local models from the CLI

Everything above is mirrored by `kruxos inference` on the appliance console:

```bash
kruxos inference catalog          # list the on-appliance model catalog
kruxos inference status           # engine status + the active model
kruxos inference pull <id>        # pull a catalog model (prints a job id)
kruxos inference activate <id>    # make an installed model active (restarts the engine)
kruxos inference disable          # stop the engine, keep the weights
kruxos inference remove <id>      # delete an installed model's weights
kruxos inference cancel <job-id>  # cancel an in-flight pull (partial kept for resume)
```

A few notes:

- **`activate`** is the CLI name for the dashboard's **Enable**.
- **`pull`** tells you whether the model became active or was staged for later,
  and prints a **job id**. The download keeps running on the appliance even if
  you close the terminal — note the job id so you can `cancel` it if needed. A
  cancelled or interrupted download is kept and **resumes** where it left off on
  the next pull.
- Pull a custom (BYOM) model with `--url`, `--sha256`, and `--size`:

    ```bash
    kruxos inference pull --url https://…/model.gguf --sha256 <64-hex> --size <bytes>
    ```

---

## Set the inference context size

The engine's **context size** — how many tokens of prompt + history a request
can use — is capped to keep memory in check. The KV cache is preallocated for
the *whole* context window when a model loads, so this setting directly bounds
the engine's idle RAM. The default is **8192 tokens**, chosen to be safe on an
8 GB appliance.

Change it with the **`KRUXOS_INFERENCE_CTX`** key in
`/data/kruxos/inference.env` on the appliance:

```bash
# On the appliance console:
cp /opt/kruxos/inference/inference.env.example /data/kruxos/inference.env
vi /data/kruxos/inference.env      # uncomment and set KRUXOS_INFERENCE_CTX
systemctl restart kruxos-inference
```

Guidance:

- **Raise it** (e.g. `16384`, `32768`) for longer prompts or conversation
  history — but only when you have RAM headroom, since KV cache memory scales
  roughly linearly with the context size.
- **Lower it** (e.g. `4096`, `2048`) if a model runs out of memory at load and
  won't start.
- Keep it **at or below the model's trained context length** — you can't set a
  larger window than the model supports.

The same file (`inference.env`) documents the other engine levers; every key is
optional and falls back to a safe baked default.

---

## Troubleshooting

- **A model "May not fit" / fails to load** — it needs more memory than the
  appliance has. Pick a smaller model or a smaller quantisation (Small tier), or
  lower `KRUXOS_INFERENCE_CTX`.
- **Hugging Face search says it's unavailable** — the panel degrades gracefully
  and never breaks the tab; the curated catalog and BYOM stay usable. Try the
  search again in a moment.
- **A model I want is Gated** — it requires a Hugging Face account token, which
  isn't supported in this release. Choose a non-gated GGUF repository.
- **A download seems stuck** — cancel it (dashboard **Cancel** or
  `kruxos inference cancel <job-id>`); the partial file is kept and the next
  pull resumes it.
- **Enabling a model interrupted a chat** — expected: switching the active model
  restarts the engine and aborts in-flight completions. Enable during a quiet
  moment.

---

## Next steps

- [Model Providers](model-providers.md) — use cloud models, or point KruxOS at a
  local model server you run yourself.
- [Connect Local Models](../quickstart/connect-local.md) — the quickstart for
  local inference.
- [Dashboard Chat](dashboard-chat.md) — talk to your active local model from the
  dashboard.
