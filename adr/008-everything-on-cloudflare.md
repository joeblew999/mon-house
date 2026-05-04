# ADR 008: Everything-on-Cloudflare migration — the north-star architecture

**Date**: 2026-05-04
**Status**: Proposed (north-star vision)
**Context**: We have a working bilingual-spec pipeline today. Translation runs on Cloudflare Workers AI; PDF compile runs in CI; storage is local + GitHub; reasoning is local Claude Code. This ADR captures the **end-state architecture** — everything in the browser and on Cloudflare, no local installs — and the **phased migration plan** to get there. It exists to keep day-to-day implementation decisions aligned with a coherent destination instead of accumulating layers that point in different directions.

---

## End-state vision

> A user (Joe, the builder, anyone) opens any modern browser. Edits a spec. Translates it. Previews the Thai PDF. Commits it. Asks an AI agent for help refining the spec. Submits a builder quote. **All of this happens in the browser and on Cloudflare. Zero local installs. Zero VSCode dependency. Zero CI dependency for the iteration loop.**

The repo on GitHub stays the canonical source of truth. Everything else is a client.

```
                              ┌──────── Browser SPA ─────────┐
                              │                              │
                              │  Editor (Monaco/CodeMirror)  │
                              │  ↕ FS Access API (Mac+Chrome)│
                              │     OR GitHub API (mobile etc)│
                              │     + OPFS (cache)           │
                              │                              │
                              │  Reasoning Agent UI (chat)   │
                              │  ↕ WebSocket → CF Agent      │
                              │                              │
                              │  Translate (Thai)            │
                              │  ↕ POST /translate → Worker  │
                              │                              │
                              │  PDF Preview                 │
                              │  ↕ typst-WASM (in-browser)   │
                              │                              │
                              │  Quote Form (per spec)       │
                              │  ↕ POST /quote → Worker      │
                              │                              │
                              │  Quote Dashboard             │
                              │  ↕ git/quotes/* via GitHub API│
                              │                              │
                              └──────────────┬───────────────┘
                                             │
                                             ▼
                              ┌─── Cloudflare Worker ────────┐
                              │  (quick-worker.gedw99…)      │
                              │                              │
                              │  /translate  → Workers AI    │
                              │  /quote/:s   → GitHub API    │
                              │  /agents/*   → PipelineAgent │
                              │                              │
                              └──────────────┬───────────────┘
                                             │
                                             ▼
                              ┌─── Cloudflare Workers AI ────┐
                              │                              │
                              │  Translation: SEA-LION 27B   │
                              │  Reasoning: Llama 3.3 70B    │
                              │              (model swap     │
                              │              one-line)       │
                              │                              │
                              └──────────────┬───────────────┘
                                             │
                                             ▼
                              ┌────────── GitHub repo ────────┐
                              │  joeblew999/mon-house          │
                              │  ├── quick/specs/  (markdown) │
                              │  └── quick/quotes/ (per spec) │
                              └────────────────────────────────┘
```

**Key property: every layer is replaceable.** Storage is dual-mode (FS or GitHub). Translation model is configurable (`QUICK_CF_MODEL`). Reasoning model is configurable. PDF compiler is the same WASM whether browser or workers-rs hosts it. The architecture is decomposed enough that no single piece is locked in.

---

## Layers and their current status

| Layer | Today | Target | Gap |
|---|---|---|---|
| **Translation** | CF Worker `/translate` → Workers AI (SEA-LION) | Same | ✅ Done |
| **PDF compile** | typst CLI in CI (mise + GHA) | typst-WASM in browser (Track A); typst-WASM on Worker (Track B) as fallback | 🟡 ADR 007 — not built; `quick/crates/quick-compiler/` workers-rs scaffold already exists |
| **Storage (read/write specs)** | Local clone + git via VSCode | Browser ↔ disk via File System Access API (Chromium); browser ↔ GitHub via OAuth (Safari/Firefox/mobile); OPFS as cache | 🟡 Not built; the immediate next deliverable |
| **Quote intake** | LINE chat + manual transcription | Per-spec form → CF Worker → git commit; capability-URL identity | 🟡 ADR 006 — not built |
| **Reasoning** | Claude Code locally (Anthropic API) | Workers AI in-browser agent (PipelineAgent extended with tools) | 🟡 Not built; reasoning quality still trails frontier (see below) |
| **Editor UI** | VSCode | Browser-based editor (Monaco/CodeMirror) in the SPA | 🟡 Not built |
| **Build/deploy** | GitHub Actions builds canonical PDFs into `specs-latest` release | Same — keeps the publishing-pipeline role even after browser preview lands | ✅ Done; complementary to browser-side preview |
| **Auth (Joe)** | n/a (he owns the repo, accesses via VSCode/git) | GitHub OAuth in the SPA when off the FS-Access path | 🟡 Not built |
| **Auth (builder)** | n/a (LINE chat) | Capability URL token (no sign-in) per ADR 006 | 🟡 Not built |
| **Identity for AI commits** | Joe's GitHub PAT (CI auto-translate-back) | Same; upgrade to GitHub App later if needed | ✅ Done |

---

## Phased migration plan

Each phase is a coherent deliverable that ships value standalone. No phase strands the project mid-state.

### Phase 0 — DONE (today)
- CF Worker live with `/translate` (Workers AI / SEA-LION).
- Rust CLI uses `QUICK_TRANSLATE_URL` to hit it.
- CI auto-commits regenerated `.th.md` translations.
- All builder-facing PDFs in `specs-latest` release.
- **Workflow:** edit in VSCode → push → CI builds Thai PDF → send via LINE.

### Phase 1 — Browser editor (dual-FS storage layer) [NEXT]
- SPA has an editor pane (Monaco or CodeMirror).
- **On Chromium browsers**, file picker uses File System Access API → user points at local `mon-house/quick/` clone → SPA reads/writes the same files VSCode is editing. **Two interfaces, one disk.**
- **On Safari/Firefox/mobile**, fall back to GitHub OAuth + GitHub REST API for read/write. OPFS caches loaded content for fast re-load.
- Translate uses the existing `/translate` endpoint — **the SPA inherits it for free**.
- **Workflow after Phase 1:** edit in VSCode *or* SPA; both see the same disk content (Mac+Chrome) or commit through GitHub (mobile). Translation triggers automatically. PDF still requires CI (same as today, until Phase 2).
- **Effort:** ~2-3 days. Lowest risk of the SPA phases — just browser APIs + OAuth, no novel architecture.

### Phase 2 — In-browser PDF preview (Track A, ADR 007)
- Stripped typst-WASM artifact loaded once, cached in OPFS.
- SPA "Preview" button compiles PDF locally in <2s.
- Same WASM artifact reusable on the Worker (Track B) via `quick/crates/quick-compiler/` if device-perf becomes a bottleneck — fallback path exists.
- **Workflow after Phase 2:** edit → translate → preview, all in browser, no CI in the loop. CI keeps publishing canonical PDFs to `specs-latest` for archival / LINE handoff.
- **Effort:** ~1-2 day spike (validate bundle size) + ~1 day SPA integration. Bundle-size risk is the only unknown; spike resolves it.

### Phase 3 — Builder quote system (ADR 006)
- Capability-URL form per spec, builders quote prices line-by-line on phone.
- Quotes commit to git as JSON files alongside specs, version-locked to spec rev.
- Joe's dashboard surfaces incoming quotes + spec-rev mismatch warnings.
- With Phase 2 in place, builder also gets instant rendered cost-summary PDF preview after submit.
- **Workflow after Phase 3:** zero PDF round-trips. Joe sends URL on LINE → builder fills in numbers → quote in git → CI rebuilds → side-by-side comparison in dashboard.
- **Effort:** ~2-3 days.

### Phase 4 — Reasoning agent in the SPA
- Extend the existing `PipelineAgent` Durable Object: add chat endpoint, tool definitions (read_spec, write_spec, list_specs, translate, compile, search_specs, etc.).
- Workers AI as the reasoning model (Llama 3.3 70B today; swap to whatever's strongest as the Workers AI catalog evolves).
- SPA gets a chat panel beside the editor.
- **Initially runs in parallel with Claude Code** — not a replacement, an addition. Use the SPA agent for day-to-day spec editing; keep Claude Code for ADR-grade architectural sessions.
- **Migrate fully when Workers AI catches up to frontier reasoning** for the kind of multi-file architectural work this repo occasionally needs. Until then, hybrid is fine.
- **Effort:** ~3-4 days for an MVP agent + chat UI + ~5 useful tools. Reasoning quality is the multi-month gap, not the implementation.

### Phase 5 — Decommission local install (optional)
- Once Phases 1-4 stabilise, the local Rust CLI becomes optional.
- VSCode + git + Claude Code remain available for users who prefer them, but no longer required.
- Onboarding becomes: "open the URL, sign in with GitHub, start working."
- **Effort:** zero new code; just documentation.

---

## Honest assessment of the reasoning-quality gap

**The single biggest unknown in this migration is whether Workers AI reasoning can replace Claude Code for this project's actual workload.**

| Workload | Claude Code (today) | Best Workers AI today | Gap |
|---|---|---|---|
| "Add a stainless bidet sprayer to BATHROOM" | Trivial | Trivial | None |
| Translate EN→TH | Same backend either way | Identical | None |
| Mechanical refactor across 3 files (e.g. rename `wall-exterior` → `envelope-vertical`) | Easy | Easy with good tool definitions | None |
| Spot bugs in markdown structure | Easy | Easy | None |
| "Review the Phase 02 specs and tell me which line items the builder will most likely overlook" | Strong | Probably-OK | Small |
| Multi-file architectural reasoning, ADR-quality output (this session's work) | Strong | Trails | **Real gap, closing** |
| Following nuanced multi-turn instructions ("hold off on X until Y; if Z, then W") | Strong | Trails | **Real gap, closing** |

**Verdict:** for **80-90% of day-to-day spec editing**, Workers AI Llama 3.3 70B is good enough today. For **architectural sessions like this one**, frontier models still have the edge.

**Therefore Phase 4 introduces the agent in *parallel*, not as a replacement.** Joe gets to use whichever is right for the task. The migration finishes itself when Workers AI quality crosses the threshold — which it will, given the rate the catalog is growing.

---

## Why this north-star is right

### Decomposed, not monolithic
Each layer (storage / translate / compile / reasoning / quotes) is independently replaceable. If Workers AI catalog regresses, swap models. If GitHub gets too expensive, swap to S3-backed storage. If typst-WASM stays too big, push to workers-rs Track B. **No single component is irreplaceable.**

### Reuses what's already built
- `/translate` endpoint — already done, Phase 0.
- `PipelineAgent` DO — already exists, Phase 4 just extends.
- `quick/crates/quick-compiler/` — already a workers-rs scaffold, ready for Track B.
- GitHub PAT + auto-commit-translations CI — already wired, ADR 006 reuses.

### Matches GitHub-as-database
Quotes in git, `_builders.json` in git, specs in git, history in git. **No separate database to back up, migrate, or worry about.** Repo handoff = full history with zero extras.

### Cross-device by design
Mac + Chrome → FS Access API direct-to-disk workflow. Phone Safari → GitHub OAuth + OPFS. Browser anywhere → it works. **Joe's iPhone becomes a viable spec-editing device** during a Phase 2 walkthrough at the build site.

### Vendor risk acknowledged but bounded
Cloudflare is the primary host (Worker, AI, R2, D1 if needed). If we ever needed to leave, the architecture decomposes cleanly:
- Translate → swap to OpenRouter / Anthropic / etc. (Hono routes are model-agnostic; backends already abstracted in `cf/src/backends/`).
- PDF compile → already runs in CI separately; could also run on any other Rust runtime.
- Reasoning → same backend abstraction.
- Storage → already GitHub.

The only *deep* CF lock-in is the Agents SDK + DO state. That's worth it for Phase 4 — but if the project ever needed to leave, that's the chunk to rewrite.

---

## Consequences

### Positive

- **One mental model.** Browser is the UI. CF is the backend. GitHub is the storage. Done. Every implementation decision has a clear home.
- **Cross-device.** Joe edits on his Mac, his phone at the building site, anywhere. Builder quotes from his phone in LINE.
- **Cheap operationally.** Workers AI free tier is 10k neurons/day — way above renovation traffic. CF Workers free tier handles the routes. GitHub Free covers the repo. Total infra cost: ~$0.
- **Open hand-off.** Anyone with the repo + CF account replication can run their own copy. No proprietary install.

### Negative

- **Reasoning quality is the gating factor for full migration.** Phase 4 ships in parallel mode for a reason. Frontier reasoning still better today.
- **CF outage = SPA partially degraded.** Translate + compile (if Track B) + agent all depend on CF being up. If CF has a 99.9% SLA, that's ~9 hours of partial-outage per year. Acceptable for renovation; would be unacceptable for production SaaS.
- **Workers AI bundle size for typst-WASM (Track B) is unmeasured.** ADR 007 covers this — the spike resolves it.
- **OAuth complexity for the GitHub-API path.** GitHub App + installation token vs PAT vs user OAuth — adds setup. Mitigated by starting with PAT (already done) and upgrading later.

### Risks accepted

- **Vendor lock to CF.** The architecture decomposes well enough that re-host is feasible if ever needed. Not "lock-in" in the harmful sense.
- **Workers AI catalog evolution.** Models that dominate today may be deprecated in 12 months. The `QUICK_CF_MODEL` knob in `wrangler.toml` makes the swap trivial.
- **OPFS quota in browsers.** Worst-case eviction = re-download (slow first-load again). Not a correctness risk.

---

## Alternatives considered

### A. Stay desktop-first, never go browser
Keep VSCode + Claude Code + local Rust + git as canonical. **Rejected.** Builder collaboration (ADR 006) and phone-anywhere editing genuinely require a browser path. Going there in pieces is no different from going in one decision.

### B. Multi-cloud (CF + AWS + Anthropic + …)
Spread the layers across providers. **Rejected.** Renovation-scale doesn't justify the operational complexity. Single cloud (CF) for everything except code-host (GitHub) keeps the architecture simple. Decompose-on-demand is feasible (see "Vendor risk" above).

### C. Self-host everything
Run the SPA, the AI, the compile, the storage on Joe's infra. **Rejected.** Operational overhead never justified by renovation scale. CF free tier + GitHub free tier covers the actual traffic.

### D. Skip the SPA, just do quote-form + LINE
Keep VSCode + Claude Code on Joe's side, just add the quote-form for builders. **Considered as a halt-point.** ADR 006 alone, without the SPA, still solves the day-to-day PITA. If Phase 1 + 2 + 4 are deferred indefinitely and only Phase 3 (quote form) lands, the renovation is still well-served. The SPA work is incremental upside, not an all-or-nothing migration.

### E. Use typst-WASM but skip the dual-FS storage layer (paste-only SPA)
Keep VSCode for editing; SPA is paste-and-preview demo. **Rejected as a long-term answer** — the dual-FS layer is what makes the SPA actually useful. But **acceptable as a temporary state** if Phase 2 (WASM) is built before Phase 1 (storage). The order of Phases 1 and 2 is reversible if dictated by external concerns.

---

## How this ties prior ADRs together

- **ADR 005** (superseded — Anthropic Headless Translation) — early experiment that taught us the translation pipeline shape.
- **ADR 006** (Builder Quote System) — Phase 3 of this migration. Introduces capability URLs + quotes-in-git.
- **ADR 007** (Typst-WASM in Browser) — Phase 2. Removes the CI round-trip from PDF preview.
- **ADR 008** (this) — the meta-architecture. Phases 1-5 above. Every other ADR is a step toward this end-state.

Future ADRs likely needed:
- **ADR 009** — Dual-mode storage layer (specific FS Access API + GitHub OAuth + OPFS implementation). Currently absorbed into Phase 1 of this ADR; will be promoted to its own ADR when implementation work begins.
- **ADR 010** — Reasoning agent / PipelineAgent extension. Phase 4 specifics. Capture when implementation begins.

---

## Implementation cadence

Recommended order, each phase ~2-4 days of work:

1. **Phase 1** (dual-FS storage) — next concrete deliverable
2. **Phase 2** (typst-WASM)
3. **Phase 3** (quote form)
4. **Phase 4** (reasoning agent in parallel)
5. **Phase 5** (decommission, doc-only)

Phases 1-2 unblock the SPA. Phase 3 directly kills the day-to-day PITA. Phase 4 is the long-tail migration. **Phase 4 is the only one with a meaningful "wait" attached to it** — for Workers AI to catch up to frontier reasoning. Phases 1-3 can land in 1-2 weeks of focused work.

This ADR is the **north star** referred to by every implementation decision until the migration completes.
