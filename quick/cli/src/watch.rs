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
        // Watch .md and .typ files; skip generated .th.md and .hash files;
        // skip registry.toml (it doesn't affect PDF output)
        (ext == "md" || ext == "typ")
            && !name.ends_with(".th.md")
            && !name.ends_with(".hash")
    })
}

/// Replicates mise's layer 1 sources/outputs check for the `all` task.
///
/// Returns true if `out/.build-stamp` is missing or older than any
/// `[A-Z]*.th.md` file or `scripts/theme.typ`.
fn needs_build(theme_file: &Path) -> bool {
    let stamp = Path::new("out/.build-stamp");
    let stamp_mtime: SystemTime = match std::fs::metadata(stamp).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return true, // stamp absent → always build
    };

    // Sources: theme.typ + all translated specs
    let mut sources: Vec<PathBuf> = vec![theme_file.to_path_buf()];
    if let Ok(entries) = glob::glob("[A-Z]*.th.md") {
        for entry in entries.flatten() {
            sources.push(entry);
        }
    }

    sources.iter().any(|src| {
        std::fs::metadata(src)
            .and_then(|m| m.modified())
            .map(|t| t > stamp_mtime)
            .unwrap_or(false) // source unreadable → treat as not stale
    })
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

    if Path::new("scripts").exists() {
        watcher
            .watch(Path::new("scripts"), RecursiveMode::NonRecursive)
            .context("watching scripts/")?;
    }

    // Watch scripts/themes/ — editing a theme file triggers a full rebuild
    let themes_dir = cfg.theme_file
        .parent()
        .unwrap_or_else(|| Path::new("scripts"))
        .join("themes");
    if themes_dir.exists() {
        watcher
            .watch(&themes_dir, RecursiveMode::NonRecursive)
            .context("watching scripts/themes/")?;
    }

    println!("Watching *.md, scripts/theme.typ, scripts/themes/*.typ — Ctrl+C to stop");
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
                    if let Err(e) = translate::cmd_translate(vec![]) {
                        eprintln!("  translate error: {e:#}");
                    }

                    // Step 3: build only if sources are newer than stamp (layer 1)
                    if needs_build(&cfg.theme_file) {
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
