/// quick-tool as a library — exposes the platform-agnostic modules so that
/// other crates (e.g. the Cloudflare Worker in `cf/`) can reuse them without
/// duplicating prompt text, Serde types, or pure-logic functions.
///
/// ## What is exported
///
/// | Module         | Reusable by CF Worker | Reusable in browser via WASM |
/// |----------------|-----------------------|-------------------------------|
/// | `chunks`       | yes                   | yes                           |
/// | `config`       | shared Config type    | yes                           |
/// | `http`         | stubs only (CF uses worker::Fetch) | stubs only        |
/// | `idempotency`  | `blake3_hex` (pure)   | yes                           |
/// | `includes`     | yes                   | yes                           |
/// | `translate`    | types, prompt, clean  | yes (types + chunk logic)     |
/// | `vfs`          | stubs only (CF uses R2/KV) | trait + impls per backend |
///
/// CLI-only modules (fonts, build, themes, new, watch) are NOT exported —
/// they depend on local tools (typst, pandoc) that do not run on Cloudflare
/// or in the browser.
pub mod chunks;
pub mod config;
pub mod http;
pub mod idempotency;
pub mod includes;
pub mod translate;
pub mod vfs;

// Browser WASM entry points — exposed via wasm-bindgen so JS can call into
// the Rust engine. Gated to the `wasm` feature so native builds don't pull in
// wasm-bindgen unnecessarily.
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub mod wasm;

// Re-export Config at the crate root so `crate::Config` resolves from both
// the binary (main.rs) and the library (lib.rs).
pub use config::Config;
