# Local Models

By the end of this page, you'll know how to run a model **on the appliance itself** — search Hugging Face, pull a GGUF model in one click, and manage which one is active — all from **Settings › Local Models**, without leaving the dashboard.

This is KruxOS's **on-box engine**: the appliance downloads a model's weights to its own disk and runs them locally. Nothing you send to an on-box model leaves the machine, and it works with no API key and no external account.

!!! note "Three ways to use a local model — this is the on-box one"
    KruxOS has three distinct "local model" paths; this page covers the third:

    - **SDK connector** — *your* script runs the model (via Ollama, vLLM, …) and calls KruxOS for tools. See [Connect Local Models](../quickstart/connect-local.md).
    - **Ollama / OpenAI-compatible provider** — KruxOS runs the agent and calls a model server **you host elsewhere** (your laptop, a LAN box). See [Model Providers → Ollama](model-providers.md#ollama-local).
    - **On-box engine (this page)** — KruxOS runs the model **on the appliance itself**. You don't run or host anything; you just pick a model and pull it.

## Why you'd want this

- **Fully private** — prompts and responses never leave the appliance.
- **No account, no key, no bill** — the model runs on hardware you already own.
- **Air-gapped friendly** — once a model is pulled, it runs with no network at all.

!!! info "Hardware matters"
    On-box inference uses your appliance's CPU (and GPU, if you've set one up on the Hardware page). A small appliance runs small models comfortably; larger models need more memory. Every model in the picker shows an **estimated** hardware-fit tip so you can choose before you download — see [Hardware-fit tips](#hardware-fit-tips) below.

## Open the tab

In the dashboard, go to **Settings › Local Models**. The tab has four sections:

| Section | What it's for |
|---------|---------------|
| **Status** | Engine state, the active model, and — when available — your total memory and free model storage. |
| **Available models** | Everything already installed on this appliance, each with **Enable**, **Disable**, and **Remove**. The running one carries an **Active** badge. |
| **Get models** | The curated catalog to pull from, plus the **Search Hugging Face** panel. |
| **Advanced: custom model (BYOM)** | A collapsed form to pull any `.gguf` by direct URL + checksum. |

## Search Hugging Face and pull a model

In **Get models**, open the **Search Hugging Face** panel and type a few letters — for example `qwen` or `llama`. Results list GGUF repositories with their downloads, likes, parameter count, context length, and licence, so you can compare at a glance.

Pick a model to open its **download picker**. Instead of a wall of cryptic filenames, the available downloads are grouped by size:

| Group | Meaning |
|-------|---------|
| **Small — fastest, lowest quality** | Smallest files, quickest to run. |
| **Balanced — recommended for most** | The best all-round choice; the consensus default (`Q4_K_M`) is marked **Recommended**. |
| **Large — highest quality, most memory** | Best output, needs the most memory. |

Each option shows its exact size and an estimated hardware-fit tip. Click **Pull** on the one you want. A **pre-download confirmation** states the download size against your free disk before anything is fetched; an unusually large model asks for one extra confirmation, so a 90 GB surprise is never a single click away.

### Hardware-fit tips

Each download is graded against your appliance's memory, so you can judge fit **before** downloading:

- **Fits** — comfortable headroom.
- **Tight** — it will load, but it's close to the limit.
- **May not fit** — larger than the memory budget. This is informational only — it never blocks the download, since you know your hardware best.

The grade is an **estimate** from a simple size-versus-memory rule, not a live test. Free disk space is checked for real before a download starts.

### Download progress, cancel, and resume

A pull runs in the background with its own progress bar, and the rest of the tab stays usable. You can **Cancel** a running download — the part already fetched is **kept on disk**, so pulling the same model again **resumes** from where it left off rather than starting over.

### Multi-part (sharded) models

Very large models are published as several files ("shards"). KruxOS shows a sharded model as a **single "N parts" row** with the summed size, downloads and verifies every part, and treats the whole set as one model — you pull, enable, and remove it as a unit.

### Gated models

Some repositories (the Llama family, for example) require you to be signed in to a Hugging Face account. These appear in search with a note:

> **requires a Hugging Face account token — not yet supported in KruxOS**

Gated models are shown for reference but **cannot be pulled in this release**. A link to the model page is provided for reading only. (Gated results are de-emphasised by default; a toggle reveals them.)

## Manage your models: Enable, Disable, Remove

**Pulling a model no longer always switches which one is active.** The behaviour is:

- The **first** model you pull activates automatically — the appliance goes from "no local model" to "ready" with zero extra clicks.
- After that, new pulls are **staged**: they're added to **Available models** but the model you're already running keeps serving. You choose when to switch.

Each row in **Available models** has three controls:

- **Enable** — make this model active (or switch to it). Switching **restarts the engine and aborts any in-flight chat completions**, so KruxOS asks you to confirm first. If the new model fails to load, KruxOS automatically **restores the previous one**.
- **Disable** — stop the engine and clear the active model. The **weights stay on disk**; enable it again any time.
- **Remove** — delete the model's weights to reclaim disk space. The confirmation shows how much space you'll get back. You can't remove the model that's currently active — disable it first.

## Advanced: custom model (BYOM)

If a `.gguf` you want isn't on Hugging Face, expand **Advanced: custom model (BYOM)** and provide a direct **URL**, its **sha256** checksum, and its **size in bytes**. KruxOS verifies the checksum after download, exactly as it does for a Hugging Face pull. This is the same form as before — the Hugging Face panel just fills those details in for you automatically.

## From the command line

Everything above is also available over the CLI (the dashboard is the primary surface; the CLI mirrors it for scripting and headless setups):

```bash
kruxos inference catalog                 # list the curated on-box catalog
kruxos inference status                  # engine state + active model

kruxos inference pull qwen2.5-3b         # pull a catalog model by id
kruxos inference pull --url https://example.com/model.gguf \
  --sha256 <64-hex> --size <bytes>       # BYOM: pull a custom .gguf

kruxos inference activate <id>           # switch to an installed model (restarts the engine)
kruxos inference disable                 # stop the engine, keep the weights
kruxos inference remove <id>             # delete an installed model's weights
```

As in the dashboard, `pull` activates the **first** model automatically and **stages** later ones — it tells you which happened — and `remove` is refused while a model is active.

!!! note "Hugging Face search is dashboard-only"
    The `kruxos inference` CLI pulls by catalog id or by URL (BYOM). Browsing and searching Hugging Face is done from the dashboard's **Search Hugging Face** panel.

## Next steps

- [Dashboard Chat](dashboard-chat.md) — try your on-box model in a conversation.
- [Model Providers](model-providers.md) — mix on-box models with hosted providers and set per-agent defaults.
- [Managing Agents](managing-agents.md) — point an agent at your local model.
