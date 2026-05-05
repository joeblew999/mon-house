# ADR 007: Typst-WASM in Browser — client-side PDF preview

**Date**: 2026-05-04
**Status**: Proposed
**Phase of**: ADR 008 — Phase 2 of the everything-on-Cloudflare migration.
**Context**: We need PDF preview in the browser without a CI round-trip. This unlocks: (1) builder sees their quote rendered into the cost-summary PDF instantly after submit, (2) Joe edits a spec and previews the PDF immediately, (3) the SPA becomes genuinely useful offline / on a phone / without GitHub Actions in the loop.

---

## Problem

Today the only way to produce a PDF from a spec is the GitHub Actions Build Specs workflow:

1. Edit `quick/specs/BATHROOM.md` locally (VSCode) → push.
2. CI runs `mise run all` → translates (via Worker) → builds PDF with the typst CLI → uploads to `specs-latest` release.
3. Total round-trip: ~60-90 seconds from push to PDF available.

Pain points this creates:

- **No instant preview** when iterating on a spec. You change a sentence, push, wait, refresh, repeat.
- **Builder cannot preview** their submitted quote. The quote-form (ADR 006) writes a JSON file to git; CI rebuilds the cost-summary PDF and that's where the builder finally sees their numbers laid out — minutes later, on the release page.
- **The SPA depends on CI for any PDF.** Even though translation already runs on the Worker (ADR 005 → Workers AI), PDF compile still goes through the typst CLI in CI. The SPA is half-online.

We've already done the cheaper half of removing the CI dependency:

- **Translation**: now a CF Worker call — `POST /translate` → SEA-LION 27B. Works in <2s, no CI involvement.

What's left is **PDF compile**. The typst CLI runs on the GHA runner because typst is a Rust binary. To remove the CI round-trip, typst must run somewhere closer to the user.

Three deployment targets for typst-the-compiler:

| Target | Status | Verdict |
|---|---|---|
| **CF Containers** running typst CLI | Disabled (commit `8abd89f`) | Rejected — see [project_typst_cf_containers.md](../memory/project_typst_cf_containers.md). |
| **CF Worker** via workers-rs + typst-as-lib WASM | Future direction; bundle size unknown vs the 10 MB Worker limit | Possible. Track B in `quick/CLAUDE.md`. |
| **Browser** via typst-as-lib compiled to WASM | Not started | **This ADR.** Track A in `quick/CLAUDE.md`. |

---

## Decision

**Compile typst client-side in the browser via WASM.** The browser downloads the typst-WASM artifact once, caches it in OPFS, and uses it to render any spec markdown into a PDF locally. Translation still runs on the Worker; PDF compile runs on the user's device.

### Architecture

```
                       Browser
        ┌─────────────────────────────────────────────┐
        │                                             │
        │  SPA UI                                     │
        │   │                                         │
        │   ├─ markdown (from File System Access      │
        │   │             API or GitHub API)          │
        │   │                                         │
        │   ├─ POST /translate ──────► CF Worker ─────┼──► Workers AI (SEA-LION)
        │   │  (returns Thai .th.md)                  │
        │   │                                         │
        │   ▼                                         │
        │  typst-WASM (cached in OPFS)                │
        │   │                                         │
        │   ├─ load Thai fonts from /fonts/ (CF       │
        │   │   Worker static assets)                 │
        │   │                                         │
        │   ▼                                         │
        │  PDF bytes ─► <iframe src=blob://> preview  │
        │                                             │
        └─────────────────────────────────────────────┘
```

### Reference implementation

Start from [`automataIA/wasm-typst-studio-rs`](https://github.com/automataIA/wasm-typst-studio-rs):

- Rust crate using `typst-as-lib` compiled to `wasm32-unknown-unknown`.
- Public deploy ships **11.3 MB gzipped** WASM, but that includes Leptos + `web-sys` framework overhead which a stripped build wouldn't carry.
- Confirmed to handle multi-page typst docs in-browser.

### What we strip from the reference

- **Leptos / SSR scaffolding** — we just need the typst compile call, not their browser SPA.
- **Editor UI** (Monaco, syntax highlighting) — handled separately by the SPA.
- **Server-side compile fallback** — irrelevant; we're browser-only.

What we keep:

- `typst-as-lib` core compiler.
- `typst-pdf` for PDF rendering.
- Minimal JS bridge: `compile(markdown: Uint8Array, fonts: Map<string, Uint8Array>) -> Uint8Array`.

### Bundle-size budget

| Variant | Estimate | Verdict |
|---|---|---|
| `wasm-typst-studio-rs` as-shipped (Leptos included) | 11.3 MB gzipped | Acceptable for an SPA but heavy. |
| Stripped (typst + bridge only) | **TBD — first measurement is the spike** | Target: **&lt;5 MB gzipped**. |
| In-browser one-time download, cached in OPFS for life | One-time cost on first load; instant on subsequent loads | Acceptable. |

**OPFS caching is the key UX move.** First visit: ~5 MB download (a few seconds on broadband, ~10s on mobile). Every subsequent visit: zero network cost, WASM loads from OPFS in &lt;100 ms.

### Font handling

Thai PDFs need Thai fonts (Noto Sans Thai or similar). We **already** download these for CI in `mise run fonts` and serve them from the CF Worker as static assets at `/fonts/...`.

Two options for getting fonts into typst-WASM:

| Approach | Pros | Cons |
|---|---|---|
| **Bundle fonts into the WASM** | Single artifact, no font-loading code | Bigger bundle, harder to update fonts |
| **Lazy-load fonts at runtime** from `/fonts/`, cache in OPFS | Smaller WASM, fonts updatable independently | Two HTTP requests on first load |

**Decision: lazy-load + OPFS cache.** Same caching story as the WASM itself. First load: WASM + 3-4 font files (Noto Sans, Noto Sans Thai, etc.) ≈ 6-8 MB total over a few requests. Cached forever after.

### Where the compile happens in the SPA

In the existing React SPA at `cf/dist/client/`:

```ts
import { TypstCompiler } from "./typst-wasm-shim";

const compiler = await TypstCompiler.load({
  wasmUrl: "/wasm/typst.wasm",
  fontsBaseUrl: "/fonts/",
  cache: "opfs",
});

// On any "preview" click:
const pdfBytes = await compiler.compile({
  source: bathroomMd,
  fontStack: ["Noto Sans Thai", "Noto Sans"],
});
const blobUrl = URL.createObjectURL(new Blob([pdfBytes], { type: "application/pdf" }));
iframe.src = blobUrl;
```

A thin TypeScript wrapper around the WASM. The existing PipelineAgent (which only does translate today, returns "PDF compile not available" per `cf/src/agent.ts:88-97`) gets retired or repurposed — PDF compile no longer needs to be a server concern.

---

## Why this beats the alternatives

### Track B: typst-WASM on the Worker (workers-rs)
Deploy the same WASM artifact to a Rust Worker via workers-rs.

- **Pros:** Zero device requirements (no spec phone needs to be powerful enough to run typst). Server-side fallback for slow client devices. Single deploy unit.
- **Cons:** **Bundle-size risk.** CF Workers Paid plan caps at 10 MB compressed bundle. A stripped typst-WASM might fit, but no certainty until measured. **Compute cost.** Every PDF preview burns Worker CPU; renovation traffic is low so this is fine, but it does shift cost from "user's phone" to "Joe's CF account."
- **Verdict: Track A first, Track B as fallback.** A working browser PDF preview removes the urgency for Track B. Once Track A is in production, measure Worker bundle feasibility and add Track B as a fallback only if devices are too slow to compile in-browser (unlikely for renovation specs — they're 10-20 pages of mostly text).

### Disabled: CF Containers running typst CLI
Already rejected (commit `8abd89f`, see `project_typst_cf_containers.md`). Summary: cold start cost is high; the Container path was a spike that didn't justify its complexity.

### Status quo: CI-only PDF compile
Works but blocks the SPA from being "instant." Specifically blocks:

- The quote-form preview UX (ADR 006) — builder cannot see their cost-summary PDF immediately after submit.
- Joe's edit-and-preview loop — every PDF requires a push.
- Offline / mobile-first scenarios — SPA is useless without internet because PDFs require GitHub Actions.

---

## Consequences

### Positive

- **Zero CI round-trip for PDF preview.** Edit a spec, click preview, see the PDF in &lt;2 seconds.
- **Quote-form preview becomes possible.** ADR 006's quote form can show the builder their submitted numbers rendered into the cost-summary PDF immediately.
- **SPA works on a phone**, even on a flaky connection (after first WASM load).
- **CI's PDF generation becomes optional** — kept for canonical artifacts published to `specs-latest` release, but no longer the only path.
- **Composable.** Same architecture works whether files come from File System Access API (Mac Chrome), GitHub API (cross-browser), or pasted markdown (one-shot).

### Negative

- **Bundle size cost.** First visit downloads ~5-8 MB (WASM + fonts). Mitigated by aggressive OPFS caching.
- **Device performance variation.** A 6-year-old Android may compile a 20-page Thai PDF in 5-10 seconds; a current iPhone in &lt;1 second. Acceptable for renovation specs.
- **Two PDF render paths to keep aligned** — typst CLI on CI vs typst-WASM in browser. They use the same typst version + same theme.typ + same fonts, so output should be byte-identical. **Worth a CI assertion** that EN+Thai PDFs from CI match the WASM output on a sample spec.

### Risks accepted

- **First-load time on slow connections.** ~10s on 3G. Mitigation: progress UI ("Loading PDF compiler... 3.2 MB / 5 MB").
- **OPFS quota.** Browser might evict OPFS data under disk pressure. Mitigation: graceful re-download.
- **Browser API support.** OPFS works in all modern browsers (Chrome, Safari, Firefox, mobile). WASM is universal. No fragmentation risk.

---

## Implementation plan

Roughly **2-3 days for a working spike**, then **1-2 days to integrate into the SPA**:

### Spike (validate feasibility)

| Step | Effort |
|---|---|
| 1. Clone `wasm-typst-studio-rs` and confirm it renders one of our specs (e.g. `BATHROOM.th.md`) end-to-end with Thai fonts in a stock browser | half day |
| 2. Strip Leptos / framework code; keep `typst-as-lib` + `typst-pdf` only; expose `compile(source, fonts) -> bytes` via `wasm-bindgen` | 1 day |
| 3. Measure stripped bundle size (gzipped). Confirm &lt;5 MB. If &gt;5 MB, identify what's pulling in size and strip further | half day |
| 4. Validate font loading from URL works (Thai test) | half day |

**Spike exit criteria:** a single HTML page that loads the stripped WASM, fetches Thai fonts, compiles `BATHROOM.th.md` into a valid PDF, and displays it in an iframe. End-to-end browser-only.

### Integration

| Step | Effort |
|---|---|
| 5. CF Worker: serve WASM at `/wasm/typst.wasm` (static asset under `cf/public/wasm/` or similar) with proper Cache-Control headers | 1h |
| 6. SPA: TypeScript wrapper module (`typst-wasm-shim.ts`) handling load + OPFS cache + compile API | half day |
| 7. SPA UI: "Preview PDF" button next to spec content; renders into `<iframe>` | 3-4h |
| 8. Hook into ADR 006's quote-form: builder sees rendered cost-summary PDF after submit | 2-3h |
| 9. CI assertion: WASM output for a sample spec matches CLI output (byte-identical or visually identical) | half day |

### Out of scope for this ADR

- **Track B (Worker-side PDF compile via workers-rs)** — separate ADR if/when device perf becomes a real constraint. Note: a workers-rs scaffold already exists at `quick/crates/quick-compiler/` (Cargo crate using `worker` + `typst` + `typst-pdf` + `typst-as-lib`), suggesting both Tracks A and B should share the same WASM artifact. This ADR's "build a stripped typst-WASM" step delivers usable Track-A output; reusing it on the Worker is a follow-on without re-doing the compile work.
- **OPFS workspace + GitHub OAuth editor** — separate ADR. Independent of how PDFs are compiled.
- **Theme switching from the SPA** — current themes registry is in the typst source; building a UI to switch is a UX feature, not a compile-target decision.

### How this composes with the dual-mode storage layer (preview)

The SPA's storage layer will be dual-mode (separate ADR):

- **Chromium browsers** (Chrome/Edge/Brave/Arc) on a desktop → **File System Access API** points at the local clone of `mon-house/quick/`. SPA reads `specs/*.md` directly from disk, same files VSCode is editing. Two interfaces, one disk — fully synchronous (last write wins, same as two VSCode windows).
- **Safari / Firefox / mobile** → **GitHub OAuth + GitHub REST API** for read/write, with **OPFS** as a local cache for fast reload + offline tolerance.

This ADR's WASM compile path is **storage-agnostic**: it accepts markdown bytes, returns PDF bytes. Whether the markdown came from the local filesystem (Mac+Chrome+VSCode same machine) or from the GitHub API (phone Safari) is not the WASM's concern. **Track A delivers preview value regardless of which storage path is in use.**

---

## References

- ADR 005: Headless AI Translation (translation already off the CI critical path; this ADR does the same for PDF compile)
- ADR 006: Builder Quote System (consumes Track A for quote-preview UX)
- `quick/CLAUDE.md`: "TODO — research: typst-as-WASM, on client and on Worker" — the project plan that originates Tracks A and B
- `automataIA/wasm-typst-studio-rs`: reference implementation
- `quick/cf/src/agent.ts:88-97`: current placeholder where `runCompile` errors out — gets retired or repurposed once Track A lands
- `project_typst_cf_containers.md` (memory): why CF Containers approach was abandoned

---

## Worker home: `cf/` (not a new worker)

Track A lands in the **existing** `cf/` Worker — same React SPA at `cf/src/client.tsx`, same deploy pipeline (`mise run 10-deploy`), same domain (`quick-worker.gedw99.workers.dev`). The integration steps above (serve WASM at `/wasm/typst.wasm`, mount the React `<Preview>` component, hook ADR 006's quote form) are all inside `cf/`.

A bare `wrangler init` scaffold at `quick/deckfs/` was created speculatively before this ADR was written and was **deleted** when this ADR's decision pinned the work to `cf/`. The TypeScript wrapper (`typst-wasm-shim.ts`) is still written without `cf/`-specific assumptions so it stays portable, but adding a second Worker just for this is unnecessary overhead.
