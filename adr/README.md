# Architecture Decision Records

| # | Title | Status | Notes |
|---|---|---|---|
| 001 | [JSON Reference Resolution System](001-json-reference-resolution.md) | **Deferred** | Never built — styling moved to Typst themes |
| 002 | [Global CSS Stylesheet for SVG Drawings](002-global-css-stylesheet.md) | **Deferred** | Depends on 001 |
| 003 | [Server for users](003-server-for-users.md) | **Rejected** | Superseded by ADR 007 + 008 |
| 004 | [Visible Call Flow Architecture](004-code-flow.md) | **Superseded** | Go pathfinder removed 2026-05-04 |
| 005 | [Headless AI Translation Architecture](005-headless-ai-translation.md) | **Superseded** | Translation now runs on CF Workers AI |
| 006 | [Builder Quote System](006-builder-quote-system.md) | Proposed | Active — capability URLs + quotes-in-git |
| 007 | [Typst-WASM in Browser](007-typst-wasm-in-browser.md) | Proposed | Active — Track A of 008's Phase 2 |
| 008 | [Everything-on-Cloudflare](008-everything-on-cloudflare.md) | Proposed | **North-star** |

## Reading order

For someone new to the project:

1. **ADR 008** first — north-star architecture, sets the direction.
2. **ADR 007** — how PDF preview works in the browser (Track A).
3. **ADR 006** — how the builder submits quotes.
4. **ADRs 001–005** — historical context only. The Go pipeline they describe was removed on 2026-05-04; the active pipeline lives in `quick/`.

## Conventions

- Status values: **Proposed**, **Accepted**, **Superseded**, **Deferred**, **Rejected**.
- When superseding an ADR, link to the replacement in the status block.
- Date stamps in ISO format (`YYYY-MM-DD`).
