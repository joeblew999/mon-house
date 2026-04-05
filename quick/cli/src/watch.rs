/// Watch subcommand — cross-platform file watcher with inline idempotency.
///
/// ## Design
///
/// This command does NOT call `mise run` as a subprocess.  It calls the Rust
/// pipeline functions directly, which means it works without mise installed:
///
///   1. `fonts::cmd_download(cfg)` — layers 2+3 already in Rust (hash + per-file)
///   2. `translate::cmd_translate(vec![])` — layer 2 already in Rust (SHA-256)
///   3. `needs_build()` timestamp check — replicates mise's layer 1 sources/outputs
///      then calls `build::cmd_build(cfg, None)` if stale
///
/// ## Idempotency
///
/// | Step      | How idempotency is preserved              |
/// |-----------|-------------------------------------------|
/// | fonts     | hash check + per-file exists (layers 2+3) |
/// | translate | SHA-256 hash in .th.md.hash (layer 2)     |
/// | build     | stamp mtime vs source mtimes (layer 1)    |
///
/// The stamp file `out/.build-stamp` is written only by a full `build` run.
/// `mise run one` builds a single spec but does NOT write the stamp, so a
/// subsequent watch trigger still sees stale stamp and rebuilds all.  This
/// matches the contract established in build.rs.
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime};

use anyhow::{Context, Result};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::{build, fonts, translate, Config};

// ── helpers ────────────────────────────────────────────────────────────────────

fn is_relevant(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) && event.paths.iter().any(|p| {
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Source files (.md, .typ) — skip generated .th.md and .hash variants
        let is_source = (ext == "md" || ext == "typ")
            && !name.ends_with(".th.md")
            && !name.ends_with(".hash");

        // Image files — a new or changed image requires a PDF rebuild
        let is_image = matches!(ext, "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg");

        is_source || is_image
    })
}

/// Replicates mise's layer 1 sources/outputs check for the `all` task.
///
/// Returns true if `out/.build-stamp` is missing or older than any
/// `[A-Z]*.th.md` file, `scripts/theme.typ`, or any image in `images_dir`.
///
/// All paths are passed explicitly so they can be overridden via `QUICK_*`
/// env vars without hard-coding anything here. Pass `None` for `images_dir`
/// when no images directory exists (e.g. in unit tests).
pub(crate) fn needs_build_in(
    theme_file: &Path,
    out_dir: &Path,
    specs_dir: &Path,
    images_dir: Option<&Path>,
) -> bool {
    let stamp = out_dir.join(".build-stamp");
    let stamp_mtime: SystemTime = match std::fs::metadata(&stamp).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return true, // stamp absent → always build
    };

    // theme.typ is always a source
    let mut sources: Vec<PathBuf> = vec![theme_file.to_path_buf()];

    // Glob .th.md files in specs_dir
    let pattern = specs_dir.join("[A-Z]*.th.md");
    if let Some(pat_str) = pattern.to_str() {
        if let Ok(entries) = glob::glob(pat_str) {
            for entry in entries.flatten() {
                sources.push(entry);
            }
        }
    }

    // Include all image files — a new/updated image requires a PDF rebuild
    if let Some(img_dir) = images_dir {
        for ext in &["jpg", "jpeg", "png", "gif", "webp", "svg"] {
            let pattern = img_dir.join(format!("**/*.{ext}"));
            if let Some(pat_str) = pattern.to_str() {
                if let Ok(entries) = glob::glob(pat_str) {
                    for entry in entries.flatten() {
                        sources.push(entry);
                    }
                }
            }
        }
    }

    sources.iter().any(|src| {
        std::fs::metadata(src)
            .and_then(|m| m.modified())
            .map(|t| t > stamp_mtime)
            .unwrap_or(false) // source unreadable → treat as not stale
    })
}

fn needs_build(cfg: &Config) -> bool {
    let images_dir = cfg.resolved_images_dir();
    let images_opt = if images_dir.exists() { Some(images_dir.as_path()) } else { None };
    needs_build_in(&cfg.resolved_theme_file(), &cfg.out_dir, &cfg.specs_dir, images_opt)
}

// ── unit tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    /// Minimal inline temp-dir: no external crate needed.
    struct TempDir(std::path::PathBuf);
    impl TempDir {
        fn new(suffix: &str) -> Self {
            use std::sync::atomic::{AtomicU64, Ordering};
            static CTR: AtomicU64 = AtomicU64::new(0);
            let n = CTR.fetch_add(1, Ordering::Relaxed);
            let p = std::env::temp_dir()
                .join(format!("quick-watch-test-{}-{n}-{suffix}", std::process::id()));
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
        // Write stamp first, then add image after
        let stamp = d.path().join("out/.build-stamp");
        std::fs::write(&stamp, "").unwrap();
        sleep(Duration::from_millis(50));

        let img = d.path().join("images/photo.jpg");
        std::fs::write(&img, b"\xff\xd8\xff").unwrap(); // minimal JPEG header

        assert!(needs_build_in(
            &theme,
            &d.path().join("out"),
            &d.path().join("specs"),
            Some(&d.path().join("images")),
        ));
    }
}

// ── subcommand: watch ──────────────────────────────────────────────────────────

pub fn cmd_watch(cfg: &Config) -> Result<()> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher =
        RecommendedWatcher::new(tx, notify::Config::default())
            .context("creating file watcher")?;

    watcher
        .watch(Path::new("."), RecursiveMode::NonRecursive)
        .context("watching current directory")?;

    if cfg.scripts_dir.exists() {
        watcher
            .watch(&cfg.scripts_dir, RecursiveMode::NonRecursive)
            .with_context(|| format!("watching {}/", cfg.scripts_dir.display()))?;
    }

    // Watch specs_dir — all EN source .md files live here
    if cfg.specs_dir.exists() {
        watcher
            .watch(&cfg.specs_dir, RecursiveMode::NonRecursive)
            .with_context(|| format!("watching {}/", cfg.specs_dir.display()))?;
    }

    // Watch <scripts_dir>/themes/ — editing a theme file triggers a full rebuild
    let theme_file = cfg.resolved_theme_file();
    let themes_dir = theme_file
        .parent()
        .unwrap_or(&cfg.scripts_dir)
        .join("themes");
    if themes_dir.exists() {
        watcher
            .watch(&themes_dir, RecursiveMode::NonRecursive)
            .with_context(|| format!("watching {}/", themes_dir.display()))?;
    }

    // Watch images dir — adding/replacing an image requires a PDF rebuild
    let images_dir = cfg.resolved_images_dir();
    if images_dir.exists() {
        watcher
            .watch(&images_dir, RecursiveMode::Recursive)
            .with_context(|| format!("watching {}/", images_dir.display()))?;
    }

    println!(
        "Watching {specs}/*.md, {scripts}/theme.typ, {scripts}/themes/*.typ, {images}/**/* — Ctrl+C to stop",
        specs   = cfg.specs_dir.display(),
        scripts = cfg.scripts_dir.display(),
        images  = images_dir.display(),
    );
    println!("Pipeline: fonts → translate → build (all idempotent, no mise required)\n");

    // None = never triggered yet, so the first event always fires immediately.
    let debounce = Duration::from_millis(300);
    let mut last_trigger: Option<Instant> = None;

    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(Ok(event)) if is_relevant(&event) => {
                let now = Instant::now();
                let ready = last_trigger
                    .map_or(true, |t| now.duration_since(t) >= debounce);
                if ready {
                    last_trigger = Some(now);

                    // Print the triggering filename BEFORE draining — the drain
                    // discards subsequent events so `event.paths` is the one that
                    // actually matters.
                    let names: Vec<_> = event
                        .paths
                        .iter()
                        .filter_map(|p| p.file_name()?.to_str().map(str::to_owned))
                        .collect();
                    println!("↺  {}", names.join(", "));

                    // Drain any additional events buffered during debounce window
                    while rx.try_recv().is_ok() {}

                    // Step 1: fonts (layers 2+3 inside cmd_download)
                    if let Err(e) = fonts::cmd_download(cfg) {
                        eprintln!("  fonts error: {e:#}");
                    }

                    // Step 2: translate (layer 2 inside cmd_translate)
                    if let Err(e) = translate::cmd_translate(cfg, vec![]) {
                        eprintln!("  translate error: {e:#}");
                    }

                    // Step 3: build only if sources are newer than stamp (layer 1)
                    if needs_build(cfg) {
                        if let Err(e) = build::cmd_build(cfg, None) {
                            eprintln!("  build error: {e:#}");
                        }
                    } else {
                        println!("  ✓ build up to date (stamp newer than sources)");
                    }

                    println!();
                }
            }
            Ok(Ok(_)) => {} // irrelevant event — ignore
            Ok(Err(e)) => eprintln!("watcher error: {e}"),
            Err(mpsc::RecvTimeoutError::Timeout) => {} // normal poll interval
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                anyhow::bail!("watcher channel disconnected unexpectedly");
            }
        }
    }
}
