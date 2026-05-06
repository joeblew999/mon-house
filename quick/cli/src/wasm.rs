//! Browser WASM entry points.
//!
//! Exposes a JS-friendly surface so JavaScript can drive the Rust engine
//! running in the browser. The boundary is intentionally coarse — every
//! cross-language call has serialization overhead, so each entry point
//! does a meaningful unit of work, not chatty per-byte access.
//!
//! ## Async-first
//!
//! All FS-touching functions are async because the underlying File System
//! Access API is async. The Rust side awaits JS Promises via
//! `wasm-bindgen-futures`; JS callers `await` the returned Promise normally.
//!
//! ## What gets exposed
//!
//! - `engine_version()` — sanity-check / build identifier
//! - `find_dependents(vfs, specsDir, partialPath)` — async; returns the list
//!    of specs that include the given partial. Uses the JS-supplied vfs
//!    handle for on-demand file reads — does NOT preload the project.
//! - `expand_includes(vfs, baseDir, content)` — async; recursive include
//!    expansion using the JS-supplied vfs.
//!
//! The JS-side `vfs` object is a minimal interface (see cf/src/wasm-vfs.ts)
//! with: `readToString(path) → Promise<string>`, `write(path, bytes) →
//! Promise<void>`, `exists(path) → Promise<bool>`, `readDir(path) →
//! Promise<string[]>`. The SPA constructs one from a
//! `FileSystemDirectoryHandle`.

use std::path::Path;

use wasm_bindgen::prelude::*;

use crate::vfs::{BrowserVfs, JsVfs};

/// Returns the engine version string. Useful for cache-busting and sanity
/// checks ("am I talking to the build I just deployed?").
#[wasm_bindgen]
pub fn engine_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Find which top-level specs include the given partial.
///
/// Reads files on-demand through the JS-supplied `vfs` handle (no preload).
/// `specs_dir` is the project's specs root (e.g. `"specs"`); `partial_path`
/// is the partial's relative path (e.g. `"specs/_partials/paint-metal.md"`).
///
/// JS API:
/// ```js
/// import init, { find_dependents } from "./pkg/quick_tool.js";
/// await init();
/// const deps = await find_dependents(vfs, "specs", "specs/_partials/paint-metal.md");
/// // → ["specs/GATE-01.md", "specs/PAINT.md", "specs/ROOF.md"]
/// ```
#[wasm_bindgen]
pub async fn find_dependents(
    vfs: JsVfs,
    specs_dir: String,
    partial_path: String,
) -> Result<JsValue, JsValue> {
    let backend = BrowserVfs::new(vfs);
    let deps = crate::includes::find_dependents(
        &backend,
        Path::new(&specs_dir),
        Path::new(&partial_path),
    )
    .await
    .map_err(|e| JsValue::from_str(&format!("{e:#}")))?;

    let strings: Vec<String> = deps
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    serde_wasm_bindgen::to_value(&strings)
        .map_err(|e| JsValue::from_str(&format!("serializing result: {e}")))
}

/// Expand `<!-- include: ... -->` directives in `content`.
///
/// Reads partial bodies on-demand through the JS-supplied `vfs` handle.
/// `base_dir` is the directory the directives resolve relative to (typically
/// the parent of the file `content` came from).
#[wasm_bindgen]
pub async fn expand_includes(
    vfs: JsVfs,
    base_dir: String,
    content: String,
) -> Result<String, JsValue> {
    let backend = BrowserVfs::new(vfs);
    crate::includes::expand_with_vfs(&backend, &content, Path::new(&base_dir))
        .await
        .map_err(|e| JsValue::from_str(&format!("{e:#}")))
}
