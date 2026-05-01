/// quick-tool as a library — exposes the platform-agnostic modules so that
/// other crates (e.g. the Cloudflare Worker in `cf/`) can reuse them without
/// duplicating prompt text, Serde types, or pure-logic functions.
///
/// ## What is exported
///
/// | Module         | Reusable by CF Worker |
/// |----------------|-----------------------|
/// | `config`       | shared Config type    |
/// | `http`         | stubs only (CF uses worker::Fetch) |
/// | `idempotency`  | `blake3_hex` (pure)   |
/// | `translate`    | types, prompt, clean  |
/// | `vfs`          | stubs only (CF uses R2/KV) |
///
/// CLI-only modules (fonts, build, themes, new, watch) are NOT exported —
/// they depend on local tools (typst, pandoc) that do not run on Cloudflare.
pub mod config;
pub mod http;
pub mod idempotency;
pub mod translate;
pub mod vfs;

// Re-export Config at the crate root so `crate::Config` resolves from both
// the binary (main.rs) and the library (lib.rs).
pub use config::Config;
