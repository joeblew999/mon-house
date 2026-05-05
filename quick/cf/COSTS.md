# `cf/` — costs, model knobs, blast-radius

Living document. Edit when prices change, when we add a model, or when we
turn a knob. Don't let this drift — read it before any change that could
affect the AI bill.

---

## TL;DR

- **Plan:** Cloudflare Workers Paid — **$5/month**.
- **Free Workers AI budget:** **10,000 Neurons / day**, then **$0.011 / 1,000 Neurons**.
- **Two surfaces burn budget:** `POST /translate` and the `/agents/*` chat WebSocket.
- **Both are public** at https://quick-worker.gedw99.workers.dev/. **Anyone on the internet can hit them.**
- **Rate limit:** 60 req/min/IP via `RATE_LIMIT` binding (`wrangler.toml`). Caps damage from unauthenticated abuse.
- **Auth: not yet.** Tracked as a follow-up — see "Open: auth gate" below.

---

## The two cost knobs

### 1. Translate model (`QUICK_CF_MODEL_TRANSLATE`)

Used by:
- `cf/src/backends/workers-ai.ts` — what the Rust CLI hits via `/translate` for cache misses.

Defaults to **`@cf/aisingapore/gemma-sea-lion-v4-27b-it`** because Sea-Lion is the only model we've validated on Thai construction-spec markdown without quality regressions.

| Model | $/M input | $/M output | Notes |
|---|---|---|---|
| `@cf/aisingapore/gemma-sea-lion-v4-27b-it` ← default | ~$0.35 | ~$0.56 | Thai-aware, 128K context |
| `@cf/meta/llama-3.3-70b-instruct-fp8-fast` | ~$0.29 | ~$2.25 | smarter but **4× output cost** |
| `@cf/meta/m2m100-1.2b` | ~$0.34 | ~$0.34 | dedicated translation, lower quality on long markdown |

### 2. Chat model (`QUICK_CF_MODEL_CHAT`)

Used by:
- `cf/src/chat.ts::ChatAgent.onChatMessage` — the Spec Copilot.
- **Per-message override**: the SPA can pass `body.model` and the chat agent honours it. Lets users opt into a smarter (more expensive) model for one turn without redeploying.

Defaults to **`@cf/google/gemma-3-12b-it`** — roughly half the per-turn cost of Sea-Lion at similar conversational quality.

| Model | $/M input | $/M output | Notes |
|---|---|---|---|
| `@cf/google/gemma-3-12b-it` ← default | ~$0.10 | ~$0.20 | cheap, fluent |
| `@cf/google/gemma-3-27b-it` | ~$0.20 | ~$0.40 | bigger Gemma, still cheap |
| `@cf/aisingapore/gemma-sea-lion-v4-27b-it` | ~$0.35 | ~$0.56 | when the user is editing Thai |
| `@cf/meta/llama-4-scout-17b-16e-instruct` | ~$0.20 | ~$0.85 | newer mid-tier |
| `@cf/meta/llama-3.3-70b-instruct-fp8-fast` | ~$0.29 | ~$2.25 | smart but pricey |
| `@cf/deepseek-ai/deepseek-r1` | ~$0.55 | ~$2.20 | reasoning, expensive output |
| `@cf/openai/gpt-oss-120b` | ~$5 | ~$25 | OpenAI-quality, costs 50× the default |

**Pricing rule of thumb:** the *output* column is what kills you. A long
streaming reply on a 70B+ model is the most expensive shape. Bigger model
+ longer reply + many users = budget gone.

---

## Sanity-check arithmetic

Per-turn cost on the **default** chat model (gemma-3-12b-it):

- ~300 input tokens (system prompt + history) × $0.10/M = $0.00003
- ~250 output tokens × $0.20/M = $0.00005
- **≈ $0.00008 / turn** (≈ 1.5 Neurons)

10K free Neurons/day = **~6,000 free chat turns/day**. Mon and Joe will
not get close. A botnet absolutely will — that's why the rate limit exists.

Per chunk on the **default** translate model (sea-lion 27b):

- ~500 input × $0.35/M = $0.000175
- ~700 output × $0.56/M = $0.000392
- **≈ $0.00057 / chunk** (≈ 3 Neurons)

10K free Neurons/day = **~3,000 free chunks/day**. A full repo
re-translation (~70 chunks) consumes 2-3% of the daily budget. Even doing
it 30× a day fits in free tier.

---

## Rate limit (defence in depth)

Configured in `wrangler.toml`:

```toml
[[unsafe.bindings]]
name         = "RATE_LIMIT"
type         = "ratelimit"
namespace_id = "1001"
simple       = { limit = 60, period = 60 }   # 60 req / minute / key
```

Applied in `cf/src/index.ts` on both `/translate` and `/agents/*` via
`checkRateLimit(c)`, keyed on the `CF-Connecting-IP` header. Returns
HTTP 429 when exceeded. **The DO and Workers AI never run if the gate
returns false** — that's the point.

Tuning:
- Tighten `limit` to lower abuse ceiling.
- Raise `limit` if legitimate users hit it.
- Bump `namespace_id` only when intentionally creating a fresh counter
  pool (e.g. policy reset).

---

## Things that would blow the budget

In rough order of "scariest to fix":

1. **Switch the default chat model to Llama 70B or DeepSeek-R1.** The
   per-turn cost jumps 10-30×. A small group of curious users would burn
   the daily free budget in an hour.
2. **Forget the rate limit.** Without it, a script doing 60 chat turns/sec
   at 1.5 Neurons/turn = 5,400 Neurons/min = full daily budget in <2 min.
3. **Long streaming replies on big models.** Output tokens dominate; a
   single 32K-output response on Llama 70B costs $0.07. Multiply by
   "everyone who pasted a spec asking for a rewrite."
4. **Loops.** A buggy retry storm (e.g. the rate-limiter rejects but the
   client keeps trying; a malformed response makes the agent regenerate
   forever). Always cap retries (Rust translate has 3 attempts; the
   chat agent inherits the SDK's bounded behaviour).

---

## Open: auth gate

Today the chat is open to the internet. Mitigations:

- Rate limit (in place — soft cap).
- Future: **CF Access** in front of the Worker, gating `/agents/*` and
  optionally `/translate` to GitHub-authenticated users only. CF Access
  is free for the first 50 users and integrates with the existing
  shared-mise-tasks `cf:access-setup` script we already pull in.
- Or: **server-issued bearer tokens** stored in user keychain. More
  custom but no CF Access dependency.

Auth is tracked as the next ops follow-up. Until it's in place: monitor
`wrangler tail` and the CF dashboard's Workers AI charts.

---

## When you change a model

1. Edit the env var in `wrangler.toml` (`QUICK_CF_MODEL_TRANSLATE` or
   `QUICK_CF_MODEL_CHAT`).
2. Update the row in the table above with the new $/M numbers from
   https://developers.cloudflare.com/workers-ai/models/.
3. Deploy: `mise run 10-deploy`.
4. Note the change in the commit message — "switched default chat
   model to X because Y" — so future-you can grep the history.

## When you change the rate limit

1. Edit the `simple = { limit, period }` line in `wrangler.toml`.
2. Update the "Rate limit" section above.
3. Deploy.

## When CF changes pricing

The Workers AI catalog page is the truth. If a price moves more than ±20%
or a model is deprecated, edit this file, update the default if needed,
and commit with a "pricing-update YYYY-MM" tag in the message body so
the change is searchable.
