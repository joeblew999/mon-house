/// Watch subcommand — cross-platform file watcher with explicit per-trigger pipelines.
///
/// ## Design
///
/// This command does NOT call `mise run` as a subprocess.  It calls the Rust
/// pipeline functions directly, which means it works without mise installed.
///
/// Events are classified into three kinds, each with its own pipeline:
///
/// | Kind   | Trigger                     | Pipeline                    |
/// |--------|-----------------------------|-----------------------------|
/// | Theme  | .typ file changed           | fonts → build               |
/// | Spec   | specs/*.md changed          | translate → build           |
/// | Image  | image file changed          | build only                  |
///
/// The build step is guarded by `build::needs_build_in` — the single source
/// of truth for the stamp check (layer 1 idempotency).  Fonts and translate
/// carry their own internal idempotency (layers 2+3), but they are only called
/// when the trigger kind actually requires them.
///
/// ## Idempotency
///
/// | Step      | Where                         | Mechanism                    |
/// |-----------|-------------------------------|------------------------------|
/// | fonts     | fonts.rs::cmd_download        | hash + per-file (layers 2+3) |
/// | translate | translate.rs::cmd_translate   | SHA-256 hash (layer 2)       |
/// | build     | build.rs::needs_build_in      | stamp mtime (layer 1)        |
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::{build, fonts, translate, Config};

// ── event classification ───────────────────────────────────────────────────────

/// What kind of file changed — drives which pipeline steps run.
#[derive(Debug, Clone, Copy)]
enum TriggerKind {
    /// A .typ file changed (theme wrapper or a theme definition).
    /// Runs: fonts → build
    Theme,
    /// A spec .md file changed (EN source, not generated .th.md or .hash).
    /// Runs: translate → build
    Spec,
    /// An image file changed.
    /// Runs: build only
    Image,
}

/// Classify an event into a `TriggerKind`, or `None` for irrelevant events
/// (generated files, other file types, non-create/modify/remove events).
fn classify(event: &Event) -> Option<TriggerKind> {
    if !matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) {
        return None;
    }
    event.paths.iter().find_map(|p| {
        let ext  = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Skip generated files — they are outputs, not inputs
        if name.ends_with(".th.md") || name.ends_with(".hash") {
            return None;
        }

        match ext {
            "typ"                                                      => Some(TriggerKind::Theme),
            "md"                                                       => Some(TriggerKind::Spec),
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg"           => Some(TriggerKind::Image),
            _                                                          => None,
        }
    })
}

// ── stamp wrapper ──────────────────────────────────────────────────────────────

/// Thin wrapper: extracts paths from cfg and delegates to `build::needs_build_in`.
fn needs_build(cfg: &Config) -> bool {
    let images_dir = cfg.resolved_images_dir();
    let images_opt = if images_dir.exists() { Some(images_dir.as_path()) } else { None };
    crate::idempotency::needs_build_in(
        &cfg.resolved_theme_file(),
        &cfg.out_dir,
        &cfg.specs_dir,
        images_opt,
    )
}

// ── subcommand: watch ──────────────────────────────────────────────────────────

pub fn cmd_watch(cfg: &Config) -> Result<()> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher =
        RecommendedWatcher::new(tx, notify::Config::default())
            .context("creating file watcher")?;

    // Root dir — catches TEMPLATE.md and any stray .typ at project root
    watcher
        .watch(Path::new("."), RecursiveMode::NonRecursive)
        .context("watching current directory")?;

    // specs/ — EN source .md files
    if cfg.specs_dir.exists() {
        watcher
            .watch(&cfg.specs_dir, RecursiveMode::NonRecursive)
            .with_context(|| format!("watching {}/", cfg.specs_dir.display()))?;
    }

    // scripts/ and scripts/themes/ — theme wrappers and definitions
    if cfg.scripts_dir.exists() {
        watcher
            .watch(&cfg.scripts_dir, RecursiveMode::NonRecursive)
            .with_context(|| format!("watching {}/", cfg.scripts_dir.display()))?;
    }
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

    // images/ — recursive so subdirectories (e.g. resources/images/gate/) are covered
    let images_dir = cfg.resolved_images_dir();
    if images_dir.exists() {
        watcher
            .watch(&images_dir, RecursiveMode::Recursive)
            .with_context(|| format!("watching {}/", images_dir.display()))?;
    }

    println!(
        "Watching {specs}/*.md  {scripts}/theme.typ  {scripts}/themes/*.typ  {images}/**/*",
        specs   = cfg.specs_dir.display(),
        scripts = cfg.scripts_dir.display(),
        images  = images_dir.display(),
    );
    println!("Theme→fonts+build  Spec→translate+build  Image→build  (Ctrl+C to stop)\n");

    let debounce = Duration::from_millis(300);
    let mut last_trigger: Option<Instant> = None;

    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(Ok(event)) => {
                let Some(kind) = classify(&event) else { continue };

                let now   = Instant::now();
                let ready = last_trigger.map_or(true, |t| now.duration_since(t) >= debounce);
                if !ready { continue; }
                last_trigger = Some(now);

                let names: Vec<_> = event
                    .paths
                    .iter()
                    .filter_map(|p| p.file_name()?.to_str().map(str::to_owned))
                    .collect();

                // Drain buffered events in debounce window
                while rx.try_recv().is_ok() {}

                match kind {
                    TriggerKind::Theme => {
                        println!("↺  {} [theme]  fonts → build", names.join(", "));
                        if let Err(e) = fonts::cmd_download(cfg) {
                            eprintln!("  fonts error: {e:#}");
                        }
                        if needs_build(cfg) {
                            if let Err(e) = build::cmd_build(cfg, None) {
                                eprintln!("  build error: {e:#}");
                            }
                        } else {
                            println!("  ✓ build up to date");
                        }
                    }
                    TriggerKind::Spec => {
                        println!("↺  {} [spec]  translate → build", names.join(", "));
                        if let Err(e) = translate::cmd_translate(cfg, vec![]) {
                            eprintln!("  translate error: {e:#}");
                        }
                        if needs_build(cfg) {
                            if let Err(e) = build::cmd_build(cfg, None) {
                                eprintln!("  build error: {e:#}");
                            }
                        } else {
                            println!("  ✓ build up to date");
                        }
                    }
                    TriggerKind::Image => {
                        println!("↺  {} [image]  build", names.join(", "));
                        if needs_build(cfg) {
                            if let Err(e) = build::cmd_build(cfg, None) {
                                eprintln!("  build error: {e:#}");
                            }
                        } else {
                            println!("  ✓ build up to date");
                        }
                    }
                }
                println!();
            }
            Ok(Err(e)) => eprintln!("watcher error: {e}"),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                anyhow::bail!("watcher channel disconnected unexpectedly");
            }
        }
    }
}
