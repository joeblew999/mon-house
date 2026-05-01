# Cloudflare Workers — `quick/`

Status of `quick/`'s Cloudflare deploys, mirroring the at-a-glance shape of
[`joeblew999/authz-core`'s CLOUDFLARE.md](https://github.com/joeblew999/authz-core/blob/cloudflare/CLOUDFLARE.md).

> Working notes (architecture, conventions, gotchas) live in
> [`CLAUDE.md`](CLAUDE.md). This file is the externally-readable summary.

## What's deployed

| Component | Where | Notes |
|---|---|---|
| `quick-worker` (TypeScript) | https://quick-worker.gedw99.workers.dev | React SPA + `/health` + `PipelineAgent` DO + R2 + Workers AI translate. **Live, free tier.** |
| `quick-compiler` (Rust + workers-rs) | _pending CF infra resolution_ | typst → PDF Worker. Builds + bundle fits 10 MiB; CF rejected last upload with 502. |

## What works

- ✅ **TypeScript Worker live.** SPA renders, `/health` returns 200, translate endpoint functional.
- ✅ **Rust + workers-rs builds for wasm32-unknown-unknown.** typst v0.14 + typst-pdf v0.14 + typst-as-lib v0.15 compile clean (no C deps, no `tokio`, no `libc`, no `getrandom` issues — getrandom uses the `js` feature on wasm32).
- ✅ **`worker-build` produces a deployable WASM bundle.** Pattern lifted from `joeblew999/authz-core`'s `authz-worker` crate: workspace member at `crates/quick-compiler/`, `crate-type = ["cdylib"]`, `worker = "0.8"`.
- ✅ **Bundle fits CF Paid limit (10 MiB compressed).** Out-of-the-box `worker-build` produced 10.5 MB gzipped — over the limit. Aggressive `wasm-opt -Oz --converge --vacuum` (with `--disable-gc --disable-reference-types --disable-typed-function-references --disable-custom-descriptors` to keep the output WASM-MVP-compatible with V8 in Workers) lands at **9.98 MiB gzipped — 20 KiB under the limit**.
- ✅ **Pipeline orchestrated via mise + shared task library.** `mise run worker:build` does the full optimised build; `mise run worker:deploy` ships it.

## What doesn't work yet

| Issue | Symptom | Plan |
|---|---|---|
| **CF deploy blocked by `invalid heap type 'exact'` (10021)** | rustc 1.93 (LLVM 19) emits WASM with **custom-descriptors / exact-heap-types** that CF's V8 rejects. `wasm-opt --mvp-features --converge` does *not* strip these — they're encoded in the bytecode at compile time, not at link time. authz-core deploys cleanly with the same Rust toolchain because typst's transitive crates trigger this codegen path; authz's don't. | Three real fixes: (1) pin Rust to <1.85 where LLVM didn't emit these types, (2) wait for CF to roll out `--experimental-wasm-custom-descriptors` in V8 (they're tracking it), (3) try a custom wasm-opt build that does strip them. Picking it up next session. |
| **No package resolver** | Specs that `#import "@preview/cmarker:0.1.8"` don't compile on the Worker yet — only flat `.typ` source works. | Add a `FileResolver` impl that fetches packages from the typst CDN and caches in R2 / `caches.default`. Same pattern any other typst-WASM consumer uses. |
| **Fonts embedded via `include_bytes!`** | All 8 `.ttf`s are baked into the WASM (~320 KB raw, ~150 KB compressed). Wastes Worker memory; also couples deploys to font changes. | Move to R2 binding; lazy-load on first request, hold in module-level static. Saves bundle size and lets fonts evolve independently. |

## Why this exists

The TypeScript Worker handles translation just fine. Adding PDF compilation
to the Worker is what required the Rust path:

- **typst is Rust** — running it in JS would mean running its WASM build from a
  JS host. That's possible but it's "Rust running inside JS running on V8",
  versus "Rust running on V8 directly" via workers-rs. The latter is one fewer
  layer of glue.
- **Rust everywhere lets us version-lock CF + browser + Tauri.** Same typst
  artefact runs in all three contexts; no risk of the SPA having a different
  typst version than the Worker than the desktop wrap. See `CLAUDE.md`'s
  "TODO — research" section.
- **Reference impl proven at home.** `joeblew999/authz-core`'s `cloudflare`
  branch demonstrates a non-trivial Rust engine deployed via workers-rs +
  D1 + service bindings. The pattern transfers cleanly.

## Build chain

```bash
mise install                 # installs typst + node + wrangler + worker-build + fnox
mise run worker:check        # cargo check --target wasm32-unknown-unknown
mise run worker:build        # worker-build --release + aggressive wasm-opt -Oz
mise run worker:deploy       # validates CF token via fnox, then wrangler deploy
mise run worker:tail         # follow live logs
mise run worker:health       # curl /health on the deployed worker
```

`wrangler.toml` deliberately has **no `[build]` block**: wrangler's default
invocation re-runs `worker-build` and re-optimises the WASM with milder
settings, which undoes our size shave. Always run `mise run worker:build`
before `mise run worker:deploy`.

## Pinned versions

| Tool | Version | Why pinned |
|---|---|---|
| `worker` (Rust crate) | `0.8` | Latest as of 2026-04-17 — required for `Router` + `event(fetch)` API used here |
| `cargo:worker-build` (mise) | `0.8.1` | Matches the workers-rs runtime |
| `npm:wrangler` (mise) | `4.87.0` | Older versions had a `fetch failed` bug on this upload size |
| `typst` / `typst-pdf` | `0.14` | typst 0.14.2 was verified to build clean for wasm32 |
| `typst-as-lib` | `0.15` | Targets typst 0.14; minimal wrapper |
| `getrandom` | `0.2` with `features = ["js"]` | Required for wasm32 random sources via JS |

## Observability

`wrangler.toml` enables CF Workers Observability with 100% sampling.
Inspect via the Cloudflare Dashboard → Workers → quick-compiler → Logs,
or via the [Cloudflare Observability MCP](https://github.com/cloudflare/mcp-server-cloudflare).

## Rust-everywhere roadmap (future)

The same typst-WASM artefact will eventually run in:

- **The browser** — Leptos SPA replacing the current React UI. Reference:
  [`automataIA/wasm-typst-studio-rs`](https://github.com/automataIA/wasm-typst-studio-rs)
  and [`cloudflare/workers-rs/templates/leptos`](https://github.com/cloudflare/workers-rs/tree/main/templates/leptos).
- **CF Worker** — this crate (`crates/quick-compiler/`), once the deploy
  infra hiccup is resolved.
- **Desktop** — Tauri wrap of the Leptos SPA, sharing the same WASM blob.

The architectural win is single-source-of-truth versioning: any `.typ`
written by the user renders the same way on all three.
