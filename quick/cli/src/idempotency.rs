/// Idempotency primitives — the single place for skip/stamp/hash logic.
///
/// ## Three layers
///
/// | Layer | Primitive         | Used by                        |
/// |-------|-------------------|--------------------------------|
/// | L1    | `needs_build_in`  | watch (via `needs_build`)      |
/// | L2    | `sha256_hex`      | fonts (theme hash), translate  |
/// | L3    | per-file exists   | fonts (inside cmd_download)    |
///
/// L3 is trivial (`Path::exists()`) so it stays inline at the call site.
/// L1 and L2 live here so there is exactly one implementation of each.
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use sha2::{Digest, Sha256};

use crate::vfs;

// ── L2: content hashing ────────────────────────────────────────────────────────

/// SHA-256 hash of arbitrary bytes, returned as a lowercase hex string.
///
/// Used by fonts (hashing the theme source file) and translate (hashing EN
/// spec content) — both store the result in a stamp file to detect changes.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex::encode(h.finalize())
}

// ── L1: build stamp check ──────────────────────────────────────────────────────

/// Returns true if the build stamp is absent or older than any source file.
///
/// This replicates mise's `sources`/`outputs` layer 1 check so that `watch`
/// can call build functions directly without spawning a mise subprocess.
///
/// All paths are explicit — no Config dependency — so this can be tested with
/// temp directories and called from any context. Pass `None` for `images_dir`
/// when the images directory does not exist.
pub fn needs_build_in(
    theme_file: &Path,
    out_dir: &Path,
    specs_dir: &Path,
    images_dir: Option<&Path>,
) -> bool {
    let stamp = out_dir.join(".build-stamp");
    let stamp_mtime: SystemTime = match vfs::modified(&stamp) {
        Ok(t) => t,
        Err(_) => return true, // stamp absent → always build
    };

    let mut sources: Vec<PathBuf> = vec![theme_file.to_path_buf()];

    // .th.md translated specs
    let pattern = specs_dir.join("[A-Z]*.th.md");
    if let Some(pat_str) = pattern.to_str() {
        if let Ok(entries) = vfs::glob(pat_str) {
            sources.extend(entries);
        }
    }

    // Image files (recursive — subdirectories like resources/images/gate/ are covered)
    if let Some(img_dir) = images_dir {
        for ext in &["jpg", "jpeg", "png", "gif", "webp", "svg"] {
            let pattern = img_dir.join(format!("**/*.{ext}"));
            if let Some(pat_str) = pattern.to_str() {
                if let Ok(entries) = vfs::glob(pat_str) {
                    sources.extend(entries);
                }
            }
        }
    }

    sources.iter().any(|src| {
        vfs::modified(src)
            .map(|t| t > stamp_mtime)
            .unwrap_or(false)
    })
}

// ── unit tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    struct TempDir(PathBuf);
    impl TempDir {
        fn new(suffix: &str) -> Self {
            use std::sync::atomic::{AtomicU64, Ordering};
            static CTR: AtomicU64 = AtomicU64::new(0);
            let n = CTR.fetch_add(1, Ordering::Relaxed);
            let p = std::env::temp_dir()
                .join(format!("quick-idempotency-test-{}-{n}-{suffix}", std::process::id()));
            std::fs::create_dir_all(&p).unwrap();
            TempDir(p)
        }
        fn path(&self) -> &Path { &self.0 }
    }
    impl Drop for TempDir {
        fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
    }

    #[test]
    fn no_stamp_means_needs_build() {
        let d = TempDir::new("no_stamp");
        let theme = d.path().join("theme.typ");
        std::fs::write(&theme, "// theme").unwrap();
        assert!(needs_build_in(&theme, &d.path().join("out"), &d.path().join("specs"), None));
    }

    #[test]
    fn newer_th_md_triggers_build() {
        let d = TempDir::new("newer_th");
        std::fs::create_dir(d.path().join("out")).unwrap();
        std::fs::create_dir(d.path().join("specs")).unwrap();
        let stamp = d.path().join("out/.build-stamp");
        std::fs::write(&stamp, "").unwrap();
        sleep(Duration::from_millis(50));

        let th = d.path().join("specs/SPEC.th.md");
        std::fs::write(&th, "# Thai").unwrap();
        let theme = d.path().join("theme.typ");
        std::fs::write(&theme, "// theme").unwrap();

        assert!(needs_build_in(&theme, &d.path().join("out"), &d.path().join("specs"), None));
    }

    #[test]
    fn newer_theme_triggers_build() {
        let d = TempDir::new("newer_theme");
        std::fs::create_dir(d.path().join("out")).unwrap();
        let stamp = d.path().join("out/.build-stamp");
        std::fs::write(&stamp, "").unwrap();
        sleep(Duration::from_millis(50));

        let theme = d.path().join("theme.typ");
        std::fs::write(&theme, "// updated theme").unwrap();

        assert!(needs_build_in(&theme, &d.path().join("out"), &d.path().join("specs"), None));
    }

    #[test]
    fn fresh_stamp_means_no_build() {
        let d = TempDir::new("fresh_stamp");
        std::fs::create_dir(d.path().join("out")).unwrap();
        std::fs::create_dir(d.path().join("specs")).unwrap();
        let theme = d.path().join("theme.typ");
        std::fs::write(&theme, "// theme").unwrap();
        let th = d.path().join("specs/SPEC.th.md");
        std::fs::write(&th, "# Thai").unwrap();
        sleep(Duration::from_millis(50));

        let stamp = d.path().join("out/.build-stamp");
        std::fs::write(&stamp, "").unwrap();

        assert!(!needs_build_in(&theme, &d.path().join("out"), &d.path().join("specs"), None));
    }

    #[test]
    fn newer_image_triggers_build() {
        let d = TempDir::new("newer_image");
        std::fs::create_dir(d.path().join("out")).unwrap();
        std::fs::create_dir(d.path().join("specs")).unwrap();
        std::fs::create_dir(d.path().join("images")).unwrap();
        let theme = d.path().join("theme.typ");
        std::fs::write(&theme, "// theme").unwrap();
        let stamp = d.path().join("out/.build-stamp");
        std::fs::write(&stamp, "").unwrap();
        sleep(Duration::from_millis(50));

        std::fs::write(d.path().join("images/photo.jpg"), b"\xff\xd8\xff").unwrap();

        assert!(needs_build_in(
            &theme,
            &d.path().join("out"),
            &d.path().join("specs"),
            Some(&d.path().join("images")),
        ));
    }

    #[test]
    fn sha256_hex_is_deterministic() {
        let a = sha256_hex(b"hello");
        let b = sha256_hex(b"hello");
        let c = sha256_hex(b"world");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 64); // 32 bytes → 64 hex chars
    }
}
