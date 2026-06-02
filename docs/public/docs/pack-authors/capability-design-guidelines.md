# Capability Design Guidelines (v0.0.2)

How to write capabilities that AI agents can use efficiently and reliably.

## Why this matters

A KruxOS capability is a typed tool an AI agent calls during task execution. The agent reads your capability definition to decide **whether and how to use it**, then reads the **output** to decide what to do next.

Every token spent on a capability's metadata is a token spent in the agent's context window. Every parse step the agent does on your output is a step that could fail. The goal of these guidelines is to make capabilities **maximally efficient for AI agents to use correctly on the first try**.

The canonical reference is the **rss.fetch** worked example — shipped both as a template in the [Pack SDK](https://github.com/altvale/kruxos-pack-sdk) and as a community pack in the [pack registry](https://github.com/altvale/kruxos/tree/main/packs). Read its `definitions/rss.fetch.yaml` alongside this document — every rule below is exercised there.

---

## The seven rules

### 1. Domain-specific, not generic

**GOOD:** `rss.fetch` (one type of operation, clear use case, fast for the agent to pattern-match)
**BAD:** `data.process` (vague — what does it do? when do I use it?)

If your capability is "http.fetch with parameter X", consider whether the agent should just call http.fetch directly. The bar is:

> Would an agent reach for THIS specifically over the generic primitive?

If the answer is no, you don't need a capability — you need a documentation patch on the generic one.

**Why it matters:** Dedicated tools beat generic primitives because (a) the agent's prompt is shorter, (b) failure surface is smaller, (c) lints/tests can be tighter. The dedicated `Read` tool beats `Bash("cat /path")` because Read is purpose-built (line numbers, image support, page handling). Your capability should beat its generic alternative the same way.

### 2. Validate at the boundary

**GOOD:** `rss.fetch` raises `InvalidURL` synchronously before any network work — agent gets feedback in milliseconds
**BAD:** Returning `{"error": "bad url"}` after a partial network call — agent paid latency for a recoverable error

The agent should not have to pre-validate inputs. Your capability's signature **is the contract**.

```yaml
inputs:
  - name: url
    type: String
    required: true
    description: "Public HTTP or HTTPS URL — file:// and other schemes
      are rejected at the input boundary."
```

**Why it matters:** Fast-fail errors are cheap; speculative-execution-then-error is expensive. The `Edit` tool errors INSTANTLY on `old_string` ambiguity — the LLM gets fast feedback without speculative reasoning.

### 3. Pre-parse the output

**GOOD:** `rss.fetch` returns `entries[].title` as String fields, ready to use
**BAD:** Returning raw XML bytes for the agent to parse

The agent should **never** need to write parser code in response to your output. If you return raw bytes/text, you've created a generic primitive — see Rule 1.

**Why it matters:** Pre-parsed output uses fewer agent tokens AND has fewer failure modes. `Read` returns `cat -n`-formatted content — I can reference line 47 immediately. If Read returned raw bytes, I'd have to count lines first, and the count would sometimes be wrong.

### 4. Tell the agent when to use AND when NOT to use

**GOOD:**
```yaml
when_to_use: |
  Use rss.fetch when the agent needs structured data from a feed URL.
  Typical missions: "summarise the last 5 entries from <feed>".

  Do NOT use rss.fetch for arbitrary HTML scraping — use http.fetch
  for raw HTML and parse downstream.

  Do NOT use rss.fetch on auth-required feeds — use http.fetch with
  the appropriate headers.
```

**BAD:** `when_to_use: "Call when needed"`

Pack the doc with the decision the agent has to make. If the answer is in the doc, the agent doesn't have to ask the user OR pick wrong.

**Why it matters:** The `Bash` tool docs say "Avoid using this tool to run `cat`, `head`... use the appropriate dedicated tool". That single line steers many agent decisions away from the wrong tool. Your `Do NOT` pairs do the same.

The pack-sdk lint requires `when_to_use` to mention `instead` or `not` — enforcing this rule.

### 5. Typed errors with concrete recovery actions

**GOOD:**
```yaml
errors:
  - type: InvalidURL
    recovery:
      - action: abort
        description: "URL is structurally wrong. Do not retry. Treat the source as suspect."
  - type: FetchFailed
    recovery:
      - action: retry
        description: "Single retry after 5-30s. If second attempt also fails, treat as permanent."
  - type: ParseFailed
    recovery:
      - action: abort
        description: "URL does not serve a feed. Try http.fetch instead to get raw bytes."
```

**BAD:** Returning `{"success": false, "error": "something failed"}`

The agent reads `errors[].recovery[].action` to decide what to do. Every error should tell the agent **what to do next** — including pointing at a different capability when applicable.

**Why it matters:** The `Edit` tool's errors literally say "File has not been read yet" or "old_string not unique" — instant fix, no reasoning required. Your errors should be the same shape.

### 6. Defaults that work, ceilings that prevent abuse

**GOOD:**
```yaml
inputs:
  - name: limit
    type: Integer
    required: false
    description: "Default 20, max 100. Out-of-range values are clamped silently."
  - name: timeout
    type: Integer
    required: false
    description: "Default 15s, max 60s. Lower for monitoring loops, higher for slow feeds."
```

**BAD:** Making every input required, with no defaults, no ceilings.

If a sensible default exists, declare it. If a value can be abused, clamp it.

**Why it matters:** `Read` defaults to 2000 lines. I don't think about it unless I have a reason. `Bash` defaults to 2-minute timeout; longer requires explicit intent. Forced-thinking on every input wastes agent tokens.

### 7. Output convenience aggregates

**GOOD:**
```yaml
outputs:
  - name: entries
    type: Array
  - name: entry_count
    type: Integer   # agent doesn't len()
  - name: truncated
    type: Boolean   # agent doesn't compare to limit
  - name: cache_hint_minutes
    type: Integer   # agent doesn't parse RSS <ttl>
```

**BAD:** Returning only `entries` and forcing the agent to derive everything else.

If the agent would derive a value from your output, return it yourself. You know what the agent's next question will be — pre-answer it.

**Why it matters:** `Read` returns content WITH line numbers. I don't have to enumerate. `gh pr view --json` returns structured fields, not just JSON; the structure IS the answer. Convenience aggregates save the agent from reasoning steps that don't change the outcome.

---

## The permission tier ladder

| Tier | When | Examples |
|---|---|---|
| `autonomous` | Pure reads, idempotent, no observable effect outside the agent's view | `time.now`, `hash.sha256`, `markdown.render` |
| `notify` | Network egress, observable but reversible, low-risk | `rss.fetch`, `http.fetch_json`, `weather.forecast` |
| `approval` | File writes outside workspace, payments, irreversible, secret access | `gmail.send`, `stripe.charge`, `filesystem.delete` |

**Choose conservatively.** Operators can promote tiers in their policy YAML (`autonomous → notify`); they cannot demote a permission gate that the capability declared. If in doubt, pick the stricter tier — operators can relax it but they cannot tighten what you didn't gate.

---

## Anti-patterns (DON'T)

- **Wrap http.fetch with a vanity name** (`my_company.fetch_data`). The agent should just call http.fetch — and operators won't trust the wrapper.
- **Return stringified JSON in a String field.** Declare the structure.
- **Hide errors in `result["error"]` instead of raising.** Typed errors are first-class — see Rule 5.
- **Make every input required.** Sensible defaults exist — see Rule 6.
- **Forget to declare side effects.** Operators audit packs against the declared `side_effects[]` list; an undeclared side effect is a trust violation.
- **Use generic verbs.** `rss.fetch` is good. `rss.do` is bad. `rss.process_feed_and_filter` is overcompounded — split it.

---

## Map the rules to your YAML

| Rule | Schema field |
|---|---|
| 1. Domain-specific | `name`, `purpose`, `tags` |
| 2. Validate at boundary | `inputs[].required`, `errors[]` (typed and synchronous) |
| 3. Pre-parse output | `outputs[]` with concrete types, not stringified blobs |
| 4. When to use + NOT use | `when_to_use` |
| 5. Typed errors + recovery | `errors[].type`, `errors[].recovery[]` |
| 6. Defaults + ceilings | `inputs[].description` (state default + max), function-side clamping |
| 7. Convenience aggregates | `outputs[]` includes derived/computed fields |

---

## Out of scope for v0.0.2 (planned for v0.0.3)

These conventions aren't yet enforceable by the SDK but will be:

- **Idempotency declaration** (`idempotent: true/false/conditional`) — critical for retry policies
- **Expected-latency declaration** (`p50_latency_ms`, `p99_latency_ms`) — agent decides whether to parallelize
- **Nested output type schemas** (`Array<FeedEntry>` with `FeedEntry` as a named struct) — sub-type rigour
- **Token-cost estimation** (`estimated_input_tokens`, `estimated_output_tokens`) — budget-aware agents
- **Explicit anti-pattern field** (`not_for: [...]`) — moves out of the `when_to_use` prose

When you encounter these gaps:
- Document inline in `purpose` or `when_to_use` (until the field exists)
- File a v0.0.3 issue suggesting the schema addition

---

## See also

- `rss-fetch` v1.1 — the canonical exercise of all seven rules
- `kruxos-pack lint` — enforces a growing subset of these statically
- `kruxos-pack test` — validates against this guideline structure
- The pack runtime strips gateway-internal `_context` kwargs (e.g. `_user_id`) before invoking your function — write your capability signature using only your own declared inputs
