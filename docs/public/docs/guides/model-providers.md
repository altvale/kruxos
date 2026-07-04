# Model Providers

KruxOS supports 10+ AI model providers. This page is the complete reference for choosing, configuring, and using each provider.

## Quick Start

```bash
# Anthropic (recommended)
kruxos model add anthropic --auth api-key

# OpenAI
kruxos model add openai --auth api-key

# DeepSeek (OpenAI-compatible via base_url)
kruxos model add openai --auth api-key --name deepseek \
  --endpoint https://api.deepseek.com/v1 --model deepseek-chat

# Or just uncomment the provider in /data/kruxos/models.yaml and add your key
```

## Provider Reference

### Anthropic (Claude)

| | |
|---|---|
| **Base URL** | `https://api.anthropic.com` (built-in, no config needed) |
| **Auth** | API Key |
| **Thinking** | Yes — adaptive effort (low/medium/high/max) on 4.6 models, budget_tokens on older |
| **Prompt Caching** | Yes — requires explicit `cache_control`, auto-managed by KruxOS |
| **Context Compaction** | Yes — native server-side via `context_management.edits` |
| **Batch Mode** | Yes — 50% discount, processed within 24 hours |
| **Token Counting** | Yes — `/messages/count-tokens` pre-flight endpoint |

**Available Models:**

| Model | Tier | Input / Output per 1M tokens |
|-------|------|------------------------------|
| `claude-sonnet-4-6` | Recommended — best value | $3 / $15 |
| `claude-opus-4-6` | Flagship | $15 / $75 |
| `claude-haiku-4-5-20251001` | Fast/cheap | $0.80 / $4 |
| `claude-sonnet-4-5-20250929` | Previous gen | $3 / $15 |
| `claude-opus-4-5-20251101` | Previous gen | $15 / $75 |
| `claude-opus-4-1-20250805` | Previous gen | $15 / $75 |
| `claude-sonnet-4-20250514` | Previous gen | $3 / $15 |
| `claude-opus-4-20250522` | Previous gen | $15 / $75 |

```yaml
# models.yaml
providers:
  claude-api:
    type: anthropic
    auth: api_key
    model: claude-sonnet-4-6
    label: Claude Sonnet 4.6
```

---

### OpenAI (GPT)

| | |
|---|---|
| **Base URL** | `https://api.openai.com/v1` (built-in, no config needed) |
| **Auth** | API Key |
| **Thinking** | Yes — `reasoning.effort` (none/low/medium/high), `xhigh` for max |
| **Prompt Caching** | Automatic — no config needed. `prompt_cache_retention: "extended"` for autonomous agents |
| **Context Compaction** | Yes — native server-side via Responses API `compact_threshold` |
| **Batch Mode** | Yes — similar discount to Anthropic |
| **Token Counting** | No pre-flight endpoint |

**Available Models:**

| Model | Tier | Input / Output per 1M tokens |
|-------|------|------------------------------|
| `gpt-5.4` | Recommended | ~$5 / $15 |
| `gpt-5.4-mini` | Balanced | ~$1.50 / $6 |
| `gpt-5.4-nano` | Fast/cheap | ~$0.50 / $2 |
| `gpt-5.2` | Previous gen | ~$5 / $15 |
| `gpt-4o` | Legacy | ~$2.50 / $10 |

```yaml
providers:
  openai-api:
    type: openai
    auth: api_key
    model: gpt-5.4
    label: GPT-5.4
```

---

### OpenAI Codex (Subscription)

| | |
|---|---|
| **Base URL** | ChatGPT backend API (built-in) |
| **Auth** | OAuth (device code flow) — sign in with your ChatGPT account |
| **Thinking** | Same as OpenAI |
| **Prompt Caching** | Automatic |
| **Context Compaction** | Same as OpenAI |
| **Batch Mode** | No (flat rate already) |

Uses your ChatGPT subscription ($20/mo flat rate) instead of per-token billing. OpenAI explicitly permits subscription OAuth for third-party tools.

!!! note "Codex as a model vs Codex as an MCP client"
    This section covers Codex as a *model provider* — KruxOS calls the
    ChatGPT backend to back chat and autonomous agents. The reverse
    direction — running the **Codex CLI** and having *it* call KruxOS
    tools over MCP — is a different integration. See
    [Running Codex CLI on KruxOS](codex.md).

**Available Models:**

| Model | Notes |
|-------|-------|
| `gpt-5.4` | Recommended |
| `gpt-5.4-mini` | Balanced |

```yaml
providers:
  codex-subscription:
    type: openai-codex
    model: gpt-5.4
    auth: oauth
    label: GPT-5.4 (Subscription)
```

---

### Google Gemini

| | |
|---|---|
| **Base URL** | `https://generativelanguage.googleapis.com/v1beta` (built-in) |
| **Auth** | API Key (Google bans subscription OAuth for third-party tools) |
| **Thinking** | Partial — `reasoning_effort` parameter (low/medium/high). No "none" or "max". |
| **Prompt Caching** | Automatic — no config needed |
| **Context Compaction** | No native support — uses client-side fallback |
| **Batch Mode** | No |

**Available Models:**

| Model | Tier | Input / Output per 1M tokens |
|-------|------|------------------------------|
| `gemini-2.5-flash` | Recommended — fast, cheap | ~$0.15 / $0.60 |
| `gemini-2.5-pro` | Premium | ~$1.25 / $5 |

```yaml
providers:
  gemini-api:
    type: gemini
    auth: api_key
    model: gemini-2.5-flash
    label: Gemini 2.5 Flash
```

---

### DeepSeek

| | |
|---|---|
| **Base URL** | `https://api.deepseek.com/v1` |
| **Auth** | API Key |
| **Thinking** | Always-on reasoning — no control parameter. `thinking_effort` is ignored. |
| **Prompt Caching** | Automatic — no config needed |
| **Context Compaction** | No native support — uses client-side fallback |
| **Batch Mode** | No |

**Available Models:**

| Model | Notes | Input / Output per 1M tokens |
|-------|-------|------------------------------|
| `deepseek-chat` | Recommended | ~$0.27 / $1.10 |

```yaml
providers:
  deepseek:
    type: openai
    base_url: https://api.deepseek.com/v1
    model: deepseek-chat
    auth: api_key
    label: DeepSeek V3
```

---

### GLM (Z.ai)

| | |
|---|---|
| **Base URL** | `https://api.z.ai/v1` |
| **Auth** | API Key |
| **Thinking** | Binary — `thinking: true/false`. Maps: none/low → false, medium/high/max → true. |
| **Prompt Caching** | Automatic — no config needed |
| **Context Compaction** | No native support — uses client-side fallback |
| **Batch Mode** | No |

**Available Models:**

| Model | Tier |
|-------|------|
| `glm-5` | Recommended |
| `glm-5-turbo` | Fast |
| `glm-4.7` | Budget |

```yaml
providers:
  glm-5:
    type: openai
    base_url: https://api.z.ai/v1
    model: glm-5
    auth: api_key
    label: GLM-5
```

---

### Grok (xAI)

| | |
|---|---|
| **Base URL** | `https://api.x.ai/v1` |
| **Auth** | API Key |
| **Thinking** | Limited — only low and high. Maps: none/low → low, medium/high/max → high. |
| **Prompt Caching** | Automatic — no config needed |
| **Context Compaction** | No native support — uses client-side fallback |
| **Batch Mode** | No |

**Available Models:**

| Model | Notes |
|-------|-------|
| `grok-4` | Only model available |

```yaml
providers:
  grok:
    type: openai
    base_url: https://api.x.ai/v1
    model: grok-4
    auth: api_key
    label: Grok 4
```

---

### Mistral

| | |
|---|---|
| **Base URL** | `https://api.mistral.ai/v1` |
| **Auth** | API Key |
| **Thinking** | No reasoning control — `thinking_effort` is ignored |
| **Prompt Caching** | Automatic — no config needed |
| **Context Compaction** | No native support — uses client-side fallback |
| **Batch Mode** | No |

**Available Models:**

| Model | Notes |
|-------|-------|
| `mistral-large-latest` | Flagship |

```yaml
providers:
  mistral:
    type: openai
    base_url: https://api.mistral.ai/v1
    model: mistral-large-latest
    auth: api_key
    label: Mistral Large
```

---

### Groq

| | |
|---|---|
| **Base URL** | `https://api.groq.com/openai/v1` |
| **Auth** | API Key |
| **Thinking** | No reasoning control — `thinking_effort` is ignored |
| **Prompt Caching** | Automatic — no config needed |
| **Context Compaction** | No native support — uses client-side fallback |
| **Batch Mode** | No |

Groq runs open-source models on custom LPU hardware for extremely fast inference.

**Available Models:**

| Model | Tier |
|-------|------|
| `llama-3.3-70b-versatile` | Recommended |
| `llama-3.3-8b-instant` | Fast |

```yaml
providers:
  groq:
    type: openai
    base_url: https://api.groq.com/openai/v1
    model: llama-3.3-70b-versatile
    auth: api_key
    label: Groq Llama 3.3 70B
```

---

### OpenRouter

| | |
|---|---|
| **Endpoint** | `https://openrouter.ai/api/v1` (configurable via `base_url`) |
| **Auth** | API Key — get one at [openrouter.ai/keys](https://openrouter.ai/keys) |
| **Thinking** | Pass-through (`reasoning_effort: low/medium/high`) — honored by upstream models that support it |
| **Prompt Caching** | Depends on the routed model (Anthropic/OpenAI cache automatically; others vary) |
| **Context Compaction** | Client-side fallback only |
| **Batch Mode** | Not applicable |

OpenRouter is a model aggregator that exposes 200+ models from multiple providers — Llama, Mistral, Claude, GPT, Gemini, DeepSeek, Command R, Gemma, Qwen, and many more — behind a single API key. KruxOS treats it as its own provider type so the right headers (`HTTP-Referer`, `X-Title`) and tool-name sanitization are applied; routing decisions and per-model billing happen on OpenRouter's side.

**Why use OpenRouter:**

- Access open-source and frontier models without managing per-vendor accounts.
- Try multiple models behind one key — swap `meta-llama/llama-3.1-405b-instruct` for `anthropic/claude-3.5-sonnet` by editing one field.
- Built-in failover: OpenRouter can route around upstream outages automatically.
- Pay-as-you-go pricing per model (see [openrouter.ai/models](https://openrouter.ai/models)) — no monthly subscription.

**Model identifiers** use `provider/model` format. Browse the catalog at [openrouter.ai/models](https://openrouter.ai/models). Examples:

- `meta-llama/llama-3.1-405b-instruct` — Llama 3.1 405B (open weights flagship)
- `anthropic/claude-3.5-sonnet` — Claude 3.5 Sonnet via OpenRouter
- `openai/gpt-4o` — GPT-4o via OpenRouter
- `google/gemini-pro-1.5` — Gemini Pro 1.5
- `mistralai/mistral-large` — Mistral Large
- `deepseek/deepseek-chat` — DeepSeek V3
- `cohere/command-r-plus` — Command R+

**Recommended models by use case:**

| Use case | Suggested model id |
|---|---|
| Coding (open weights) | `meta-llama/llama-3.1-405b-instruct` or `qwen/qwen-2.5-coder-32b-instruct` |
| Research / long context | `anthropic/claude-3.5-sonnet` or `google/gemini-pro-1.5` |
| Cheap chat | `meta-llama/llama-3.1-8b-instruct` or `mistralai/mistral-7b-instruct` |
| Reasoning | `deepseek/deepseek-r1` |
| RAG / retrieval | `cohere/command-r-plus` |

**Cost:** per-token, set per upstream model. The current price for any model is shown at [openrouter.ai/models](https://openrouter.ai/models). OpenRouter passes upstream costs through with a small markup; there is no monthly subscription.

```yaml
providers:
  openrouter:
    type: openrouter
    auth: api_key
    default_model: meta-llama/llama-3.1-405b-instruct
    label: OpenRouter
    # base_url override is optional — defaults to https://openrouter.ai/api/v1
```

---

### Ollama (Local)

| | |
|---|---|
| **Type** | `ollama` |
| **Endpoint** | `http://localhost:11434` (configurable) — the bare server root, **no `/v1`** |
| **Auth** | None — runs locally |
| **Thinking** | No reasoning control |
| **Prompt Caching** | Not applicable |
| **Context Compaction** | No native support — uses client-side fallback |
| **Batch Mode** | Not applicable |

Free, private, no data leaves your machine. Requires [Ollama](https://ollama.com) installed locally.

KruxOS talks to Ollama on its **native API** (`/api/chat`, `/api/tags`), so the
endpoint is just the server root — do **not** add a `/v1` path. (Servers that
speak the OpenAI format instead — vLLM, LM Studio, llama.cpp — are configured as
an [OpenAI-compatible provider](#openai-compatible-providers-base_url) with a
`base_url` ending in `/v1`, not as `ollama`.)

**Available Models:** Any model you pull with `ollama pull`. Enter the model name as free text (e.g., `llama3.3:8b`, `mistral:latest`, `codellama:34b`).

```yaml
providers:
  local-default:
    type: ollama          # `local` is still accepted (it maps to `ollama`)
    auth: none
    endpoint: http://localhost:11434
    model: llama3.3:8b
    label: Local Llama
```

From the CLI:

```bash
kruxos model add ollama --name local-default \
  --endpoint http://localhost:11434 --model llama3.3:8b
# (`--auth none` is the default for ollama)
```

!!! note "Running Ollama on another machine"
    From a KruxOS appliance, `localhost` is the **appliance**, not your laptop or
    server. To reach an Ollama box on your LAN, use that machine's LAN IP and make
    sure Ollama listens on it — e.g. start Ollama with `OLLAMA_HOST=0.0.0.0` and set
    the endpoint to `http://192.168.1.50:11434`.

---

## Feature Comparison Matrix

| Provider | Auth | Thinking | Caching | Compaction | Batch | Token Pre-flight |
|----------|------|----------|---------|------------|-------|-----------------|
| **Anthropic** | API Key | Adaptive effort | Explicit (auto-managed) | Native server-side | 50% discount | Yes |
| **OpenAI** | API Key | Reasoning effort | Automatic | Native (Responses API) | Yes | No |
| **OpenAI Codex** | OAuth | Same as OpenAI | Automatic | Same as OpenAI | No (flat rate) | No |
| **Gemini** | API Key | Partial (low/med/high) | Automatic | Client-side fallback | No | No |
| **DeepSeek** | API Key | Always-on (no control) | Automatic | Client-side fallback | No | No |
| **GLM** | API Key | Binary (on/off) | Automatic | Client-side fallback | No | No |
| **Grok** | API Key | Limited (low/high) | Automatic | Client-side fallback | No | No |
| **Mistral** | API Key | None | Automatic | Client-side fallback | No | No |
| **Groq** | API Key | None | Automatic | Client-side fallback | No | No |
| **OpenRouter** | API Key | Pass-through | Per upstream model | Client-side fallback | No | No |
| **Ollama** | None | None | N/A | Client-side fallback | N/A | No |

## OpenAI-Compatible Providers (base_url)

Any API that implements the OpenAI Chat Completions format can be used by setting `type: openai` with a `base_url`. The base URL should include the version path (e.g., `/v1`). KruxOS appends `/chat/completions` automatically.

```yaml
providers:
  my-custom-provider:
    type: openai
    base_url: https://my-api.example.com/v1
    model: my-model
    auth: api_key
    label: My Custom Provider
```

This works with any provider that accepts the OpenAI request/response format: Together AI, Fireworks, Anyscale, vLLM, LiteLLM, and others.

## Which Provider Should I Use?

| Use Case | Recommendation | Why |
|----------|---------------|-----|
| **Best quality reasoning** | Anthropic `claude-opus-4-6` | Flagship model, best at complex tasks |
| **Best value for daily use** | Anthropic `claude-sonnet-4-6` | Strong quality at 1/5 the Opus cost |
| **Always-on agents (flat rate)** | OpenAI Codex (subscription) | $20/mo, no per-token surprise bills |
| **Fastest inference** | Groq `llama-3.3-70b-versatile` | Custom LPU hardware, extremely low latency |
| **Cheapest per token** | Gemini `gemini-2.5-flash` | ~$0.15/M input tokens |
| **Testing and development** | Anthropic `claude-haiku-4-5-20251001` | $0.80/M input — cheapest current Claude model |
| **Privacy-sensitive** | Ollama (local) | Data never leaves your machine |
| **Budget reasoning** | DeepSeek `deepseek-chat` | Strong reasoning at very low cost |
| **Code generation** | Anthropic Sonnet 4.6 or OpenAI GPT-5.4 | Both excellent at code |

## Adding Providers

### Via CLI

```bash
# Standard providers
kruxos model add anthropic --auth api-key
kruxos model add openai --auth api-key
kruxos model add gemini --auth api-key
kruxos model add ollama --endpoint http://localhost:11434   # local model; --auth none is the default
kruxos model add openrouter --auth api-key

# OpenAI-compatible providers (endpoint with /v1 auto-detected as base_url)
kruxos model add openai --auth api-key --name deepseek \
  --endpoint https://api.deepseek.com/v1 --model deepseek-chat
kruxos model add openai --auth api-key --name groq \
  --endpoint https://api.groq.com/openai/v1 --model llama-3.3-70b-versatile
```

### Via Dashboard

Open the **Settings** page at `https://localhost:7800/settings`. The page surfaces a System Defaults card at the top (the current chat / autonomous / fallback choices) and one card per configured provider below it. Click **+ Add Provider** to register a new one.

The Add Provider form supports six provider types, with auth-conditional fields per type:

| Type | Auth | Notes |
|------|------|-------|
| **Anthropic** | API key | Base URL is built-in |
| **OpenAI** | API key | Built-in base URL; switch to "OpenAI" + custom base URL for any OpenAI-compatible provider (DeepSeek / Grok / GLM / Mistral / Groq / your own) |
| **OpenAI Codex** | OAuth device code | "Sign in" launches the ChatGPT subscription device-code flow — KruxOS shows a verification URL and copy-to-clipboard code, then polls until you approve in the browser |
| **Gemini** | API key | Built-in base URL |
| **OpenRouter** | API key | Inline info banner with a link to `openrouter.ai/keys` |
| **Local** | None | Runtime preset dropdown (Ollama / vLLM / LM Studio / llama.cpp) auto-fills the endpoint |

Each provider card on the page shows a credentials-status dot (configured vs missing), the default model selector, a Base URL field, the agent assignments referencing this provider, and three action buttons: **Test** (probes the upstream and renders the result inline), **Set Default** (for chat / autonomous / fallback), and **Remove** (confirm-modal — also wipes the vault-stored credentials).

If the vault is locked when you open the page, the cards are gated behind a banner prompting you to unlock the vault first.

### Via models.yaml

The default `models.yaml` includes commented-out examples for all providers. Uncomment and add your API key via `kruxos vault add model-provider:<name> api-key`:

```bash
# After uncommenting a provider in models.yaml:
kruxos vault add model-provider:deepseek api-key
# Enter API key when prompted
```

## Managing Providers

```bash
kruxos model list                          # List all providers
kruxos model test claude-api               # Test connectivity
kruxos model remove openai-codex           # Remove provider + credentials
kruxos model default chat claude-api       # Set default for chat
kruxos model default autonomous deepseek   # Set default for autonomous agents
kruxos model default fallback local-default # Last resort fallback
```

## Per-Agent Model Assignment

```bash
kruxos agent create --name code-bot --model claude-api
kruxos agent create --name research --model deepseek
kruxos agent create --name helper --model local-default
```

**Fallback resolution order:**

1. Agent's assigned provider (if set and enabled)
2. System default for the role (`chat`, `autonomous`, or `fallback`)
3. Fallback provider
4. Error — no provider available

## Troubleshooting

### Provider shows "no credentials configured"

The API key wasn't stored in the vault. Re-add:
```bash
kruxos model remove my-provider
kruxos model add openai --auth api-key --name my-provider
```

### Rate limit errors

Set a fallback provider: `kruxos model default fallback local-default`

### OpenAI-compatible provider returns 404

Check that your `base_url` includes `/v1`. Example: `https://api.deepseek.com/v1`, not `https://api.deepseek.com`.

### Changes to models.yaml not reflected

The gateway watches the file and hot-reloads automatically. Check:
- File is at `/data/kruxos/models.yaml`
- Gateway is running: `kruxos verify`
- Check gateway logs for reload messages
