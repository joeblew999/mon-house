# Platform Architecture

> **Status:** Draft, working answer. Decisions captured here are the design contract — not "how it currently runs," which is closer to "Phase 0" of the plan below. TBDs are flagged inline.

## 1. Goal & scope

Build a reusable platform for **bilingual construction-document authoring with AI assistance**, deployable across dozens of construction projects. The current Laem Chabang renovation is the first project (Phase 0) and the proving ground for the platform shape.

Each "project" is one construction job — a renovation, a new build, a commercial fitout — and serves multiple users (owner, contractor, architect, builder). The platform provides the editor, the AI assistant, the dep-graph engine, the bilingual translation pipeline, the typst-based PDF rendering, and the shared catalog of materials/products. Projects are isolated tenants on shared infrastructure.

---

## 2. Deployment surfaces

| Surface | Built from | Used by |
|---|---|---|
| **Native CLI** | Rust binary, links core directly | Platform team only — provisioning, dev work, scripting |
| **Browser** | Web frontend, loads `engine.wasm` | End users — no install required |
| **Tauri desktop** | Same web frontend, native Rust backend via Tauri IPC | End users on macOS/Windows/Linux |
| **Tauri mobile** | Same web frontend, native Rust backend via Tauri IPC | End users on iOS/Android |
| **CF Worker** | TypeScript Agent + lean WASM subset | Coordinator: WebSocket fan-out, persistence, headless jobs, AI chat surface |

The web frontend is **one implementation** that runs everywhere users are. The native CLI is the only surface that doesn't share that frontend — it's terminal-only and platform-team-only.

---

## 3. Compute model

**Compute travels to where users are; state and coordination stay on CF.**

When a user is connected via browser/Tauri, the engine runs on their device — typst compilation, dep-graph queries, includes expansion, image processing all happen client-side via the Rust core (WASM in browser, native in Tauri). The CF Worker is reserved for things only it can do: persistence, fan-out, auth, AI chat, headless work when no user is online.

This shifts cost off the platform and onto users' devices, where compute is effectively free. CF only pays for what it uniquely provides (durability, real-time coordination, AI-as-a-service).

---

## 4. Engine architecture

The Rust core library at [`cli/src/`](../cli/src/) is the engine. Built as **lib + bin**: the binary is one consumer, alongside the WASM build for browser/Worker and the Tauri-native link.

Existing portable abstractions stay in their lanes:

- `vfs` — all file I/O. Pluggable backends (see §5).
- `http` — all outbound HTTP. `ureq` natively, `fetch()` in browser/Worker.
- `config` — all configurable paths via `QUICK_*` env vars.
- `idempotency` — hash + skip logic (`sha256_hex`, `needs_build_in`).
- `translate` — section-granular translation with cache.
- `includes` — partial expansion via `<!-- include: ... -->` directive.

New for the platform:

- `dep_graph` — extracts dependency edges from spec content, returns dependents on query. Edge types include `partial-include`, `image-ref`, `spec-link`, `font-ref` (see §13). Initially recompute-on-demand from `vfs::glob`; SQLite-backed incremental graph deferred.
- `git` — peer of `vfs`. History, commit, push/pull. Each backend implements its own variant (system git, isomorphic-git, GitHub API). See §5.

The engine is consumed via **coarse-grained API** — a few dozen exported functions that take and return JSON-serializable data. No chatty per-byte calls. WASM/JS marshaling is a real cost but cheap when calls are batched.

```rust
// Representative API surface
fn compute_graph(specs_root: &Path) -> GraphSnapshot;
fn dependents_of(graph: &GraphSnapshot, path: &Path) -> Vec<PathBuf>;
fn expand_includes(content: &str, base: &Path) -> Result<String>;
fn translate_chunks(spec_path: &Path, chunks: Vec<Chunk>) -> Result<Vec<Chunk>>;
fn build_spec(spec_path: &Path, theme: &Path) -> Result<Pdf>;
fn git_log(path: &Path) -> Result<Vec<Commit>>;
fn git_commit(message: &str, author: &Author) -> Result<CommitId>;
```

All `path`s here are abstract — their resolution depends on the mounted FS backend (§5).

---

## 5. FS backends

The `vfs` and `git` abstractions support **three concrete backends**, selected at runtime:

| Backend | Storage | Git implementation |
|---|---|---|
| **Local** | POSIX filesystem (assumed to be a git working tree) | System `git` binary |
| **Workspace** | `@cloudflare/shell` SQLite + R2 (auto-spillover at 1.5 MB) | `isomorphic-git` on virtual FS |
| **GitHubFS** | GitHub itself via Contents API | Direct API — every write is a commit |

The same engine code runs against any backend. The engine never branches on "which backend am I on" — it asks "what's the history?" or "expand includes" and the backend handles the rest.

**R2 auto-spillover** ([`shell/src/filesystem.ts:105`](../.src/cloudflare-agents/packages/shell/src/filesystem.ts#L105)) is what lets image storage and PDF outputs work without custom plumbing — small files in SQLite, large files in R2 with per-project key prefix, all transparent to callers.

Each backend has different sync semantics:
- **Local**: single-user, syncs to other backends via `git push`/`pull`.
- **Workspace**: multi-user real-time. Browser/Tauri/Local-CLI all read/write the same Workspace via WebSocket and HTTP API.
- **GitHubFS**: every save is a public commit. No Workspace involved. Lighter setup; loses real-time collaboration.

Cloudflare ships only a git **client** in `@cloudflare/shell`, not a server. The git server is GitHub (or any compatible host). Each backend is a git client; they synchronize via a shared remote.

---

## 6. State model

Two storage tiers per Project Agent:

**Shared Workspace** — the project's files (specs, images, build outputs, theme, partials). Single canonical copy. All users read/write the same Workspace. R2 auto-spillover handles large files. Git layer provides history.

**Per-user rows in the DO's SQLite** — chat history with the AI, pending notification acknowledgments, user preferences (theme, layout, default AI model), draft messages. Keyed by user identity. Never shared.

This avoids the distributed-editing problem (per-user storage) while still giving each user their own private slice. Real-time collaboration is event-driven — Agent broadcasts "file changed" → connected clients refresh — not CRDT-based. CRDTs (Yjs/Automerge) can be added later if simultaneous-typing UX becomes a requirement; the foundation doesn't preclude it.

---

## 7. Tenancy model

**One Project Agent Durable Object instance per construction project.** DO ID derives from the project ID. Multiple users connect to the same DO; their permissions are checked against the project's ACL on every operation.

ACL roles (initial set):

- `owner` — full read/write, can manage ACL
- `collaborator` — read/write project files, read everyone's chat-with-AI history
- `viewer` — read-only access to project files
- `service` — internal accounts for headless tasks (CI builds, scheduled rebuilds)

**Authentication:** [better-auth](https://better-auth.com) (TypeScript-first, multi-provider). Migration target from the current Cloudflare Access + email allowlist. Better-auth gives OAuth (GitHub/Google), magic links, and clean session handling; integrates with Hono and the Agents SDK.

**Identity is cross-project.** One user account ↔ many projects, same person. Each Project Agent's ACL references the user's identity (email or stable user ID); identity itself lives in a platform-wide auth store.

A separate **Platform Agent** DO holds cross-project state (the global user list, project list, platform catalog repo references, billing eventually). Phase 1 may inline this into a config file; Phase 2 it becomes a proper Agent.

---

## 8. UI strategy

**Layout:** the chat surface is a fixed side dock — Cloudflare's Agents GUI, their components (`@cloudflare/ai-chat` hooks, Kumo design tokens, agents-starter layout primitives). We don't fork or customize the chat. The rest of the page is owned by **a Rust GUI compiled to WASM**, structured as a multi-pane editor environment: tabs, splits, side-by-side spec/PDF preview, file tree, dep-graph view, image gallery — IDE-shaped.

**Framework:** [Leptos](https://leptos.dev) for the layout/DOM/reactivity. [`wgpu`](https://wgpu.rs) for panes that need GPU rendering or compute (3D site views, image processing, GPU-accelerated PDF preview, future on-device ML). Both target browser via WASM and native via Tauri — same Rust source, no fork. Bundle cost of `wgpu` is small (~1–2 MB compressed); composes inside Leptos as `<canvas>` panes.

**Communication between panes:** the chat side-pane and Rust main-pane talk via a small JS event bridge (low-level UI state) and the Project Agent's WebSocket (high-level events that other clients also need to see).

**Tauri shells** host the same web frontend in a webview. Native IPC adds OS integration (filesystem, notifications, file dialogs). Tauri's Rust backend can also link the engine *natively* — bypassing WASM — for best performance.

**The reference clones at [`.src/`](../.src/) are the source of truth** for the chat surface's API shape. Re-pin them on every `agents` SDK bump (per the bump-discipline rule in [`quick/CLAUDE.md`](../CLAUDE.md)). Look at:
- `cloudflare-agents/examples/ai-chat/` — canonical chat API
- `cloudflare-agents/examples/playground/` — chat + side panels layout
- `cloudflare-agents/examples/workspace-chat/` — chat + filesystem (closest analog)
- `cloudflare-agents/examples/multi-ai-chat/` — multiple users + multiple AIs on one Agent

---

## 9. Catalog versioning

Construction quotes are effectively contracts. Once a builder prices ROOF.md based on today's PAINT catalog, rebuilding next month must produce the same numbers. **Live links are explicitly rejected** — they would silently mutate historical PDFs.

Catalogs are git repos:

- **Project-local partials** live in the project's own Workspace under `_partials/`. Plain relative-path includes: `<!-- include: _partials/paint-metal.md -->`. Updates are immediate within the project.
- **Platform-shared catalogs** live in a separate platform git repo. Projects pull from it via `isomorphic-git`, pinned to a specific tag for build determinism. Updates are opt-in: bumping the tag is a git operation, visible in history.

The exact include syntax for platform refs is implementation detail. Under the hood it resolves to "look up the platform catalog repo at the pinned ref, read this file." UI can prompt "newer catalog version available — review changes?" without re-resolving the content.

This piggybacks on git's existing pinning model (tags, commit SHAs) — no custom version system to invent.

---

## 10. Sync model

**The Workspace is canonical for each project.** Browser/Tauri/Local-CLI clients all read and write the same Workspace via the Project Agent. This is the live-collaboration path.

**GitHub is a backup and publishing target**, not a sync hub. The Project Agent's git client pushes to GitHub for:

- External backup (durability beyond a single CF account)
- Public mirrors (contractor downloads via GitHub release — same `specs-latest` pattern as today)
- Cross-project import (catalog repos)
- Disaster recovery (spin up a new Project Agent from a GitHub clone)

**Local CLI** talks to the Workspace via HTTP/WebSocket as the primary sync mechanism. `git push`/`pull` against GitHub is the secondary path for offline editing or platform-bypass scenarios. Both work; the first is faster and supports real-time multi-user collaboration.

---

## 11. PDF generation

**Three rendering paths, picked per surface:**

| Path | Used for |
|---|---|
| typst-WASM in browser | Live preview while editing in a browser tab |
| typst native in Tauri | Live preview in installed apps; faster than WASM |
| typst CLI in CF Containers | Headless rebuilds: scheduled jobs, CI, no-user-online |

The engine doesn't pick — the deployment surface does. All three coexist. They produce identical output (typst is deterministic given the same input).

Reference for the WASM path: [`automataIA/wasm-typst-studio-rs`](https://github.com/automataIA/wasm-typst-studio-rs). Stripped down to just the typst bindings (no Leptos SPA scaffolding), it should fit comfortably in the browser. Sizing the stripped build is a Phase 1 task.

---

## 12. Translation

**Cloudflare Workers AI** for now — real-time requirement during editing means the round-trip needs to stay on-platform. Existing setup at [`cf/`](.) handles this; per-section caching ensures unchanged content is free.

The `translate.rs` module already has a pluggable backend interface (`TranslateBackend`). Future backends (Anthropic API for higher quality on long-form, on-device Llama for offline) can be added without engine changes. Cost knobs documented in [`cf/COSTS.md`](COSTS.md).

---

## 13. Dep-graph engine

The dep-graph is the engine's awareness of "which files depend on which." Edge types tracked in the initial taxonomy:

| Edge type | Detection regex / convention | Use case |
|---|---|---|
| `partial-include` | `<!-- include: _partials/X.md -->` | Rename/delete safety, content drift notifications |
| `image-ref` | `![alt](resources/images/X.jpg)` | Image rename/delete safety |
| `spec-link` | `[X](Y.md)` between specs | Cross-spec rename safety |
| `font-ref` | Font names in `theme.typ` | Font rename safety |
| `catalog-ref` (TBD) | Pinned catalog version refs | Platform-catalog version drift notification |

**Watch is the primary feedback layer** — events fire as files save, surfacing dependents in seconds. Pre-commit and CI are backstops, not the primary signal. From [`CLAUDE.md`](../CLAUDE.md) Rule 1: watch is the workflow.

When a partial changes, the engine emits a structured event:

```json
{
  "type": "partial_changed",
  "path": "_partials/paint-metal.md",
  "dependents": ["specs/ROOF.md", "specs/GATE-01.md"],
  "edge_type": "partial-include",
  "diff_summary": "effective coverage: 3.5 → 4.0 m²/can"
}
```

When a partial is renamed/deleted, the same engine emits a "broken reference" event listing dangling includes immediately rather than waiting for the next build to fail.

**Subscribers are equal peers** of the Project Agent's WebSocket fan-out:
- Browser editor: shows toast notifications, highlights affected files
- CLI watcher: prints to terminal (replaces today's local watch behavior)
- Built-in AI: receives events, decides whether to surface in chat, may propose updates

The AI is *one* subscriber, not above the engine. The graph is deterministic and fast (sub-millisecond at one project's scale); the AI consumes events from it like any other client.

This is the expression of a design principle: **deterministic graph walks belong in tooling; LLMs handle judgment.** The engine answers "which files depend on this?" The AI decides "should I update the can counts in those files?"

---

## 14. Project bootstrap

**Approach C — template-based repo.** A platform-maintained template repo on GitHub holds the canonical starter content for new projects: folder structure, `TEMPLATE.md`, default `theme.typ`, `mise.toml`, fonts list, catalog references, frontmatter conventions.

Bootstrap flow:

1. New project requested (manual or self-serve, depending on phase)
2. Provisioning step clones the template repo at a specific tag
3. Seeds the new Project Agent's Workspace with the cloned content
4. Sets the ACL (requesting user becomes `owner`)
5. Optionally creates a linked GitHub repo (initial push of the seeded content)
6. Returns project URL

Template evolution rides on the same git mechanics as catalogs. Tags are versions. Old projects bootstrapped from `template@v3` are unaffected when the platform team ships `template@v4`. New projects pick up the latest by default.

**Phase 1 (manual provisioning):** a `mise run new-project -- <name>` task on the platform team's local machine. Slow but trivial to build. Fine for the first handful of projects.

**Phase 2 (self-serve):** the Platform Agent (§7) handles provisioning requests via its own UI. Same template-repo mechanism, just behind a logged-in flow.

---

## 15. Sequencing — what we build first

**Phase 0 (today, Laem Chabang renovation):** the existing `quick/` pipeline. Single project, local-only Rust CLI, manual deploy. The current state is the first proof point.

**Phase 1 (next):** harden Phase 0 into a single-tenant platform.

- Watch-time dep-graph notifications (engine + event payloads + CLI subscriber)
- Project Agent skeleton on CF (one project = one DO)
- WorkspaceFS backend behind the existing `vfs` abstraction
- Browser frontend: Leptos + Cloudflare Agents GUI in side dock
- Better-auth migration from CF Access email allowlist
- Template-repo + manual `mise run new-project` task
- typst-WASM browser path stripped-build size measurement

**Phase 2:** multi-tenant scale-out.

- Platform Agent for cross-project state and self-serve project creation
- GitHubFS backend for git-as-canonical deployments
- Tauri desktop + mobile shells
- Per-user-state slicing in DO SQLite
- Catalog repo with versioned releases; first non-paint catalog (fixtures, hardware)
- Cross-project user identity, ACL UI

**Phase 3+:** depth.

- Real-time co-editing (CRDT layer, if/when needed)
- Platform Agent billing + quota
- Workers-rs migration of the Agent (only if Agents SDK API stabilizes and bundle savings warrant)
- typst CLI in CF Containers as headless fallback
- On-device ML (Llama for offline translation, image classification)
- Search across specs and catalogs

---

## Deferred / Phase-2+ flags

These are real concerns but not Phase 1:

- **Async notifications** (email/SMS for offline review prompts) — Phase 2
- **Search** (cross-spec, cross-project) — Phase 2
- **Cost / quota / billing** — Phase 2 when first paying tenant lands
- **Mobile-specific UX refinements** — Phase 2 with Tauri mobile
- **Cross-version migration tooling** (engine ↔ Agent ↔ Agents SDK bumps) — Phase 3
- **Observability** (per-project metrics, AI cost attribution) — covered by CF's built-in dashboards via the `cloudflare-observability` MCP; refine when Phase 2 lands
- **CRDT-based simultaneous editing** — only if a real use case demands it

---

## Open architectural questions

Things still genuinely undecided:

1. **Workers-rs migration timing for the Agent.** Current TS Agent works. Workers-rs becomes worth it if the engine stabilizes and bundle savings justify the toolchain cost. Defer until both are true.
2. **The `catalog-ref` edge type.** Pinned catalog version refs aren't structured today (they live as text in include directives). Worth adding a structured form once we have a real platform-catalog repo to test against.
3. **Cross-project search.** When projects share catalogs, "where else is this paint product used?" becomes a useful query. Out of scope for Phase 1; design hint: the Platform Agent is the natural place to maintain this index.
4. **Tauri mobile build pipeline.** Not yet attempted. Build pipeline, App Store / Play Store distribution, native IPC patterns are all open.

---

## Design rules (for future-Claude and humans alike)

These distill the principles that ran through the conversation that produced this doc:

- **Deterministic graph walks belong in tooling; LLMs handle judgment.** Don't ask an LLM to remember dependency graphs across context resets. Build the tool.
- **Watch is the primary feedback layer.** Pre-commit is a backstop; CI is the last-resort net. Anything in row 2 or 3 that could run cheaply should move to row 1.
- **Compute travels to where users are; state and coordination stay on CF.** The Worker is a coordinator, not a compute platform. Big WASM bundles are fine in browsers — they're cached.
- **One implementation, many consumption surfaces.** The Rust core compiles to native (CLI, Tauri) and WASM (browser, Worker). No parallel TypeScript reimplementation of the engine.
- **The chat UI is donated, not built.** Use Cloudflare Agents GUI, Kumo, agents-starter patterns. Compose domain panels alongside; never fork the chat surface.
- **Git is the versioning, history, and sync substrate.** Three FS backends, all with git semantics. No custom version systems. No custom sync protocols beyond what git already gives us.
- **Determinism over freshness.** Catalog refs are pinned. Builds are reproducible. Live links are rejected.
- **Configurable paths, not hardcoded ones.** Every directory the Rust core reads or writes goes through a `Config` field backed by an env var.
- **The `_partials/` partial pattern is the SSOT primitive.** Each partial owns its substrate's metadata; consumers compute from it. Edits propagate via the dep-graph.
