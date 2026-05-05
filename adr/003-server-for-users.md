# ADR 003: Server for users (early sketch)

## Status
**Rejected (2026-05-04)** — superseded by ADR 008 (Everything on Cloudflare) and ADR 007 (Typst-WASM in Browser).

This ADR was a 13-line sketch from when the project was thinking about a Datastar-served local GUI for non-VSCode users. The actual direction is now:

- The user-facing GUI is the **React SPA in `cf/`** served by the existing Cloudflare Worker — see ADR 008.
- Live preview and PDF compile happen **in the browser via typst-WASM**, not via a server-side renderer — see ADR 007.
- File watching and FS access use the **File System Access API** (Chromium) or **GitHub OAuth + REST API** (Safari/Firefox/mobile) — also ADR 008.

**No Datastar.** No local Go server. The browser does the work; CF Workers do the AI calls and serve assets.

If a feature listed in the original sketch is still wanted (chat-with-Claude UI, drawings file tree, live FS reload), it lives inside the React SPA covered by ADR 008.

---

## Original sketch (kept for context)

> the tools are thre
>
> we need to alow users to do what we do in vscode now.
>
> a very basic GUI that uses datastar to:
>
> allow users to call claude and see the out put from claude just like using it in vscode. this is a simple chat gui using datastar.
>
> allow users to see the drawings folder as a tree.
> whjen each svg or readme changes on the FS, the web gui updates. Datastar is create for this.
