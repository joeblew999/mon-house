# ADR 006: Builder Quote System — capability URLs + quotes-in-git

**Date**: 2026-05-04
**Status**: Proposed
**Phase of**: ADR 008 — Phase 3 of the everything-on-Cloudflare migration.
**Context**: Sending PDFs to builders via LINE and getting prices/comments back manually is the day-to-day pain point. We want a frictionless way for multiple builders to quote each spec and for those quotes to be first-class artifacts in the repo, version-locked to the spec rev they were quoted against.

---

## Problem

The current renovation workflow is:

1. Joe edits `quick/specs/BATHROOM.md` (or any spec)
2. CI builds `BATHROOM.th.pdf` and publishes to the `specs-latest` GitHub release
3. Joe sends the PDF to a builder over LINE
4. Builder reads the PDF, types prices and comments back into LINE chat
5. Joe **manually transcribes** prices into the cost-summary table or compares to other builders by hand
6. Repeat for every spec change / every builder

Pain points:

- **Manual transcription is error-prone** — "5,500 บาท" in chat → 4500 in the spec via typo, no audit
- **One spec at rev 5, builder quoted rev 3** — no mechanical way to detect the mismatch
- **Comparing two builders' quotes** = side-by-side reading of LINE chats; no structured comparison
- **No history** — builder revises a quote, the old number is buried in chat scrollback
- **Builder identity is implicit** — "Khun Jaeb" said it was 4500 last week, but on a phone screenshot it could be anyone

We have spent considerable effort on the **1:1 quote rule** (one spec file = one quote line item; see banner in every builder-facing spec). The current LINE-based workflow undermines this — it's structurally easy for a builder to bundle line items in chat, and Joe has to re-impose the structure manually.

---

## Decision

Build a **per-spec quote form** at `quick-worker.gedw99.workers.dev/quote/<SPEC>/b/<TOKEN>` that:

1. Renders a Thai-localised, mobile-friendly form for the spec's cost-summary line items.
2. Identifies the builder via an opaque **capability URL token** — no sign-in, no OAuth, no PIN.
3. On submission, **commits the quote as a JSON file in git**, alongside the spec and version-locked to the spec rev.
4. Provides Joe a dashboard that lists incoming quotes per spec and surfaces stale-rev warnings.

### Data layout

Quotes live in the repo:

```
quick/
├── specs/
│   ├── BATHROOM.md                rev: "5"
│   ├── BATHROOM.th.md
│   └── BATHROOM.th.md.hash
└── quotes/
    ├── _builders.json             ← capability-token registry
    └── BATHROOM/
        ├── jaeb-2026-05-04.json   ← quoted rev 5
        ├── mansoo-2026-05-05.json ← quoted rev 5
        └── jaeb-2026-05-12.json   ← re-quoted rev 6
```

### Quote file shape

```json
{
  "spec": "BATHROOM",
  "spec_rev": "5",
  "builder": { "id": "jaeb-7x9k2m" },
  "submitted_at": "2026-05-04T13:00:00Z",
  "lines": [
    {
      "id": "demolish-wall",
      "label": "Demolish existing entrance wall",
      "amount_thb": 4500,
      "comment": ""
    },
    {
      "id": "build-new-wall",
      "label": "Build new entrance wall",
      "amount_thb": 8000,
      "comment": "ต้องดูหน้างานก่อน"
    }
  ],
  "total_thb": 87500,
  "notes": "เริ่มงานได้ 15 พ.ค."
}
```

### Builder registry shape

`quick/quotes/_builders.json`:

```json
[
  {
    "id": "jaeb-7x9k2m",
    "name_en": "Jaeb",
    "name_th": "ขุนแจ้ บ.ก่อสร้างแจ้",
    "line_id": "@jaeb-builder",
    "phone": "+66 81 234 5678",
    "added": "2026-05-04",
    "added_by": "joe"
  }
]
```

### Line-ID schema

Cost-summary rows in spec markdown carry stable IDs as HTML comments:

```markdown
| Demolish existing entrance wall <!-- id:demolish-wall --> | TBC |
| Build new entrance wall <!-- id:build-new-wall --> | TBC |
```

The Worker parses the markdown to derive the form schema. **No sidecar quote-schema file** — one source of truth.

### Submission flow

```
Builder phone           CF Worker                           GitHub repo
─────────────           ──────────                          ────────────
                                                            (current main: BATHROOM.md rev 5)
[opens URL]    GET ──→ /quote/BATHROOM/b/jaeb-7x9k2m
                       reads BATHROOM.md from main
                       parses line IDs from markdown
                       reads _builders.json, looks up token
                       renders Thai form, name pre-filled
[fills + submits]      POST /quote/BATHROOM/b/jaeb-7x9k2m
                       validates payload
                       builds quote JSON, stamps spec_rev
                       commits via GitHub PAT ──→ new commit:
                                                  quotes/BATHROOM/jaeb-2026-05-04.json
                       returns "submitted" + edit-link
[success]
```

### Authentication model: capability URLs

Each builder has a unique URL token (e.g. `jaeb-7x9k2m` — random base32, ≥10 chars, ≥80 bits entropy). The token IS the identity. No password, no sign-in.

| Property | How |
|---|---|
| Add a builder | Joe types name + LINE ID + phone in dashboard → Worker generates token, commits to `_builders.json`, returns the URL → Joe shares on LINE |
| Revoke a builder | Edit `_builders.json` to change the token → old URL becomes 404 → re-share new URL |
| Audit trail | Git log of `_builders.json` shows when each builder was added, by whom, and any token rotations |

### Worker → git auth

Use a **GitHub Personal Access Token** (Joe's PAT) stored as a CF Worker secret:

```bash
wrangler secret put GITHUB_TOKEN
```

The Worker uses the PAT to commit quote JSON files via the GitHub REST API. Quote commits attribute to "joeblew999" — fine for renovation scale. Upgrade to a GitHub App later if/when commits need to attribute to a "quick-quote-bot" identity.

---

## Why this architecture

### Quotes-in-git wins

- **Single source of truth** — no separate database to back up, migrate, or sync. Repo handoff = full history with zero extras.
- **Spec-rev locking is mechanical** — every quote stamps the rev it was quoted against. UI can render "you quoted rev 5; current is rev 6" warnings deterministically.
- **Comparison is git-native** — `quotes/BATHROOM/*.json` lists side-by-side quotes; CI can build a "quotes summary PDF" per spec for free.
- **Audit trail is the git log** — full history of negotiations, who quoted what when, no extra tooling needed.
- **Open-source / handoff friendly** — anyone with the repo can replay the entire renovation negotiation.

### Capability URLs win for this scale

- **Zero auth ceremony for the builder** — they tap a link, the form opens, they fill in numbers. No sign-up, no password, no LINE account required.
- **Same model as Calendly, Google Doc share-links, Typeform pre-fills, Stripe payment links** — boring, proven, well-understood.
- **Revocable in 5 seconds** — edit one JSON file in the registry.
- **Right blast radius for renovation scale** — 3-5 builders, low frequency. Token leakage = limited damage, easy to rotate. Production-grade auth is overkill.

### Markdown-embedded line IDs win

- **One source of truth** — adding/removing a cost line in the spec automatically updates the form. No schema-drift between the spec and a sidecar JSON.
- **Invisible to readers** — HTML comments don't render in the EN or Thai PDFs.
- **Translatable** — the builder sees the **Thai label** of each line because the Worker reads `BATHROOM.th.md` for label rendering and uses the ID from `BATHROOM.md` as the stable key.

---

## Consequences

### Positive

- **Solves the PDF-PITA loop directly.** Builder quotes via web form on phone; quotes land in git; Joe sees them in dashboard. No transcription.
- **Mechanically enforces the 1:1 quote rule.** Builder cannot bundle line items because the form has fixed slots per line.
- **Multi-builder is free.** Add a builder → unique URL → quotes file under their ID. Compare side-by-side becomes trivial.
- **CI gets new powers.** Per-spec "quotes summary PDF" showing all builders' numbers becomes a free byproduct of the existing build workflow.
- **Architecture composes with future plans.** Track A (typst-WASM in browser, ADR TBD) and OPFS+OAuth editor (ADR TBD) operate against the same git source of truth.

### Negative

- **Builder URL leakage = identity leakage.** Mitigated by trivial token rotation, but not zero. For renovation scale, acceptable.
- **Worker must hold a GitHub PAT.** Wrangler secret. If the secret leaks, attacker has Joe's repo write permissions. Mitigated by using a fine-grained PAT scoped only to `joeblew999/mon-house`. Upgrade to GitHub App later if scale demands.
- **Each quote submission = a git commit.** For low-frequency renovation use this is fine. At higher frequencies, batch commits or move to a dedicated quotes branch with periodic merges to main.
- **No real builder accountability** — if a malicious actor with the URL submits fake numbers, only Joe's dashboard catches it. Fine at scale; revisit if it ever bites.

### Risks explicitly accepted

- **Builder URL forwarded to a 3rd party** — same blast radius as a leaked Calendly link. Joe rotates the token if it happens.
- **Race conditions on simultaneous submits** — extremely unlikely at renovation scale; if hit, GitHub API rejects non-fast-forward and the Worker can retry.
- **GitHub API rate limits** — well below renovation-scale traffic.

---

## Alternatives considered

### A. LINE Login (OAuth)
Builder signs in with their LINE account. Pros: real verified Thai identity, no URL-leakage risk. Cons: OAuth flow setup work, builder learning curve, more code. **Deferred** — capability URLs are sufficient for this renovation; LINE Login is the natural upgrade path if scale grows.

### B. GitHub Discussions, one per spec
Builder gets a GitHub account and comments on a Discussion. Pros: zero infrastructure, leverages GitHub UI. Cons: GitHub UI in English, builder doesn't have or want a GitHub account, friction. **Rejected** for builder-facing UX.

### C. Quotes in Cloudflare D1 (database, not git)
Faster writes, no Git API round-trip per submission. **Rejected** because:
- Loses the "single source of truth + audit log" property of git.
- Adds a separate backup/migration story.
- Repo handoff would be incomplete.
- Renovation-scale write volume is trivially handled by GitHub API.

### D. Quotes as markdown sections embedded in the spec
Each spec gets a `## Builder quotes` section that builders edit. **Rejected** because:
- Conflicts when multiple builders edit simultaneously.
- Loses structured data — quote summaries become text-mining.
- Mixes spec authorship with quote submission.

### E. PIN per builder (shared secret)
Builder enters name + PIN. **Rejected** because URL token is the same security model with less typing.

### F. Magic-link via SMS or email
Builder enters phone/email → receives one-time code → enters it. **Deferred** — adds infra for marginal security gain at this scale. Capability URLs first; magic-link if rotation ever becomes a daily pain.

---

## Implementation plan

Roughly **2-3 days of work** for a working MVP:

| Step | Effort |
|---|---|
| 1. Add line-ID HTML comments to existing spec cost-summary rows | 1h |
| 2. Markdown-table parser → form schema (TS, in Worker) | 2h |
| 3. CF Worker `GET /quote/:spec/b/:token` route — render Thai form | half day |
| 4. CF Worker `POST /quote/:spec/b/:token` route — validate, build JSON, commit via GitHub PAT | 3-4h |
| 5. Builder confirmation page + edit-quote URL | 2h |
| 6. Joe's dashboard view (read `quotes/` via GitHub API or `_builders.json`) | half day |
| 7. CI: per-spec "quotes summary PDF" generation step | 2-3h |
| 8. ADR-driven testing + polish | half day |

**Dependencies:**

- The `POST /translate` endpoint and the `QUICK_TRANSLATE_URL` plumbing (already done).
- The auto-commit-translations CI step (already done).
- A GitHub PAT with `contents:write` scope on `joeblew999/mon-house`, set as a wrangler secret named `GITHUB_TOKEN`.

**Out of scope for this ADR:**

- Track A (typst-WASM in browser) — separate ADR.
- OPFS workspace + GitHub OAuth editor — separate ADR.
- LINE Login upgrade path — separate ADR if/when needed.

---

## References

- ADR 005: Headless AI Translation Architecture (what made the translation pipeline autonomous in the first place)
- `quick/CLAUDE.md` — translation backend resolution, BLAKE3 idempotency
- `quick/cf/src/index.ts` — current Worker routes (`/health`, `/translate`, `/agents/*`)
- `quick/cf/src/agent.ts` — PipelineAgent (existing WebSocket-based translate; complementary, not replaced)
