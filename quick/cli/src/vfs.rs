/// Virtual filesystem — single abstraction layer for all I/O.
///
/// Every filesystem operation in quick-tool goes through this module.
///
/// ## Two surfaces, same backend
///
/// - **Free functions** (`read_to_string`, `glob`, etc.) are convenience entry
///   points hardcoded to the local OS filesystem via `std::fs`. Most existing
///   call sites use these. They will eventually be gated to non-wasm targets.
/// - **`Vfs` trait** is the pluggable abstraction. New code that needs to
///   work across deployment targets (CLI, browser via WASM, CF Worker via
///   Workspace) takes a `&dyn Vfs` and routes through whichever backend is
///   mounted. `LocalVfs` is the std::fs-backed default; future backends:
///   `BrowserVfs` (File System Access API), `WorkspaceVfs` (@cloudflare/shell).
///
/// ## What belongs here
/// - File read/write/delete
/// - Directory create/delete/list
/// - Path existence and metadata (mtime)
/// - Glob pattern expansion
///
/// ## What does NOT belong here
/// - Path construction (`Path::join`, `PathBuf::from`, etc.) — pure math, no I/O
/// - Process execution (`Command::new`) — separate concern
/// - File watching (`notify`) — local-only subsystem, not needed on Cloudflare
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};

// ── Trait surface ──────────────────────────────────────────────────────────────

/// Pluggable filesystem interface. Each deployment target provides one impl.
///
/// Methods are **async** because the browser File System Access API is
/// inherently async and we want one trait that works in both native and WASM
/// contexts. Native impls (`LocalVfs`) return ready futures with zero overhead;
/// browser impls (`BrowserVfs`) await JS Promises via `wasm-bindgen-futures`.
///
/// Sync call sites in the CLI bridge to async via `pollster::block_on`.
///
/// Bound on `&self` rather than `&mut self` so multiple call sites can share
/// one backend without locking.
#[allow(async_fn_in_trait)]
pub trait Vfs {
    async fn read_to_string(&self, path: &Path) -> Result<String>;
    async fn write(&self, path: &Path, data: &[u8]) -> Result<()>;
    async fn exists(&self, path: &Path) -> bool;
    async fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
}

/// Default backend: real OS filesystem via `std::fs`.
///
/// Zero-sized — instantiate freely. Available everywhere except WASM targets,
/// where `std::fs` is unimplemented (gate this impl out on wasm32 once the
/// browser path is the primary one).
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Default, Debug)]
pub struct LocalVfs;

#[cfg(not(target_arch = "wasm32"))]
impl Vfs for LocalVfs {
    async fn read_to_string(&self, path: &Path) -> Result<String> {
        read_to_string(path)
    }
    async fn write(&self, path: &Path, data: &[u8]) -> Result<()> {
        write(path, data)
    }
    async fn exists(&self, path: &Path) -> bool {
        exists(path)
    }
    async fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        read_dir(path)
    }
}

// ── Browser backend ───────────────────────────────────────────────────────────
//
// `BrowserVfs` is backed by a JS object the host SPA constructs from a
// `FileSystemDirectoryHandle` (File System Access API). The Rust side calls
// the JS object's methods via wasm-bindgen; JS does the actual file I/O.
//
// The JS-side contract (defined in cf/src/wasm-vfs.ts) is a minimal shape
// with four async methods matching the trait: readToString, write, exists,
// readDir. Each returns a Promise; the Rust side awaits via wasm_bindgen_futures.
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
mod browser {
    use super::*;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    #[wasm_bindgen]
    extern "C" {
        /// JS-side filesystem handle. Constructed by the SPA from a
        /// `FileSystemDirectoryHandle` (FS Access API). Methods return Promises.
        #[derive(Clone)]
        pub type JsVfs;

        #[wasm_bindgen(method, js_name = "readToString")]
        fn js_read_to_string(this: &JsVfs, path: &str) -> js_sys::Promise;

        #[wasm_bindgen(method, js_name = "write")]
        fn js_write(this: &JsVfs, path: &str, data: &[u8]) -> js_sys::Promise;

        #[wasm_bindgen(method, js_name = "exists")]
        fn js_exists(this: &JsVfs, path: &str) -> js_sys::Promise;

        #[wasm_bindgen(method, js_name = "readDir")]
        fn js_read_dir(this: &JsVfs, path: &str) -> js_sys::Promise;
    }

    /// Browser-backed Vfs. Holds a JS handle; clones are cheap (Rc internally).
    pub struct BrowserVfs {
        pub(crate) handle: JsVfs,
    }

    impl BrowserVfs {
        /// Wrap a JS-side vfs handle. Called from `crate::wasm` when the SPA
        /// hands a directory handle in.
        pub fn new(handle: JsVfs) -> Self {
            Self { handle }
        }
    }

    impl Vfs for BrowserVfs {
        async fn read_to_string(&self, path: &Path) -> Result<String> {
            let p = path.to_string_lossy().to_string();
            let val = JsFuture::from(self.handle.js_read_to_string(&p))
                .await
                .map_err(|e| anyhow::anyhow!("readToString({p}): {:?}", e))?;
            val.as_string()
                .ok_or_else(|| anyhow::anyhow!("readToString({p}) returned non-string"))
        }

        async fn write(&self, path: &Path, data: &[u8]) -> Result<()> {
            let p = path.to_string_lossy().to_string();
            JsFuture::from(self.handle.js_write(&p, data))
                .await
                .map_err(|e| anyhow::anyhow!("write({p}): {:?}", e))?;
            Ok(())
        }

        async fn exists(&self, path: &Path) -> bool {
            let p = path.to_string_lossy().to_string();
            match JsFuture::from(self.handle.js_exists(&p)).await {
                Ok(v) => v.as_bool().unwrap_or(false),
                Err(_) => false,
            }
        }

        async fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
            let p = path.to_string_lossy().to_string();
            let val = JsFuture::from(self.handle.js_read_dir(&p))
                .await
                .map_err(|e| anyhow::anyhow!("readDir({p}): {:?}", e))?;
            // JS returns an array of strings (relative paths or names).
            // Convert to PathBuf, joined to the input dir.
            let arr = js_sys::Array::from(&val);
            let mut out = Vec::with_capacity(arr.length() as usize);
            for i in 0..arr.length() {
                if let Some(s) = arr.get(i).as_string() {
                    out.push(PathBuf::from(s));
                }
            }
            Ok(out)
        }
    }
}

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use browser::{BrowserVfs, JsVfs};

// ── read ───────────────────────────────────────────────────────────────────────

/// Read a file's contents as a UTF-8 string.
pub fn read_to_string(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))
}

/// Read a file's contents as raw bytes.
pub fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path)
        .with_context(|| format!("reading {}", path.display()))
}

// ── write ──────────────────────────────────────────────────────────────────────

/// Write data to a file, creating or overwriting it.
pub fn write(path: &Path, data: impl AsRef<[u8]>) -> Result<()> {
    std::fs::write(path, data)
        .with_context(|| format!("writing {}", path.display()))
}

/// Atomically write a file via a `.tmp` sibling, then rename.
///
/// Prevents readers from seeing a partial write. The temp file is in the
/// same directory as `dest` so the rename is atomic on the same filesystem.
pub fn write_atomic(dest: &Path, data: impl AsRef<[u8]>) -> Result<()> {
    let tmp = dest.with_extension("tmp");
    write(&tmp, data)?;
    rename(&tmp, dest)
}

// ── directories ────────────────────────────────────────────────────────────────

/// Create a directory and all missing parent directories.
pub fn create_dir_all(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)
        .with_context(|| format!("creating directory {}", path.display()))
}

// ── delete ─────────────────────────────────────────────────────────────────────

/// Delete a single file.
pub fn remove_file(path: &Path) -> Result<()> {
    std::fs::remove_file(path)
        .with_context(|| format!("removing {}", path.display()))
}

/// Recursively delete a directory and all its contents.
pub fn remove_dir_all(path: &Path) -> Result<()> {
    std::fs::remove_dir_all(path)
        .with_context(|| format!("removing {}/", path.display()))
}

// ── move ───────────────────────────────────────────────────────────────────────

/// Rename (move) a file or directory.
pub fn rename(from: &Path, to: &Path) -> Result<()> {
    std::fs::rename(from, to)
        .with_context(|| format!("renaming {} → {}", from.display(), to.display()))
}

// ── query ──────────────────────────────────────────────────────────────────────

/// Returns true if the path exists (file or directory).
pub fn exists(path: &Path) -> bool {
    path.exists()
}

/// Returns the last-modified time of a file or directory.
#[cfg_attr(not(feature = "local"), allow(dead_code))]
pub fn modified(path: &Path) -> Result<SystemTime> {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .with_context(|| format!("getting mtime of {}", path.display()))
}

/// List all direct children of a directory as paths.
#[allow(dead_code)] // not yet called from production code; present for Cloudflare port
pub fn read_dir(path: &Path) -> Result<Vec<PathBuf>> {
    let entries = std::fs::read_dir(path)
        .with_context(|| format!("listing directory {}", path.display()))?;
    let mut paths = Vec::new();
    for entry in entries {
        paths.push(
            entry
                .with_context(|| format!("reading entry in {}", path.display()))?
                .path(),
        );
    }
    Ok(paths)
}

// ── glob ───────────────────────────────────────────────────────────────────────

/// Expand a glob pattern into matching paths.
///
/// Returns `Err` if the pattern is syntactically invalid.
/// Individual unreadable entries are silently skipped (same behaviour as
/// iterating `glob::glob(...).flatten()`).
pub fn glob(pattern: &str) -> Result<Vec<PathBuf>> {
    let entries = glob::glob(pattern)
        .with_context(|| format!("invalid glob pattern: {pattern}"))?;
    Ok(entries.flatten().collect())
}
