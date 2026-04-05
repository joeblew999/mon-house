/// Virtual filesystem — single abstraction layer for all I/O.
///
/// Every filesystem operation in quick-tool goes through this module.
/// Currently backed by `std::fs`. When targeting Cloudflare Workers
/// (R2, KV, or a WASM-compatible shim), swap the implementations here —
/// no other file changes required.
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
