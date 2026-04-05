/// Watch subcommand — cross-platform replacement for the watchexec external tool.
///
/// ## Idempotency
///
/// On every triggered rebuild this command calls `mise run fonts && mise run all`
/// as a subprocess.  It does NOT call the Rust build/translate/fonts functions
/// directly.  This is the critical design choice:
///
///   watchexec (old):  watchexec -e md,typ -- sh -c 'mise run fonts && mise run all'
///   quick-tool watch: notify crate detects change → spawn mise run fonts + mise run all
///
/// The ONLY thing that changes is how file changes are detected (notify crate
/// instead of the watchexec binary).  All three idempotency layers of every mise
/// task remain intact:
///
///   fonts:     layer 1 (mise stamp), layer 2 (hash), layer 3 (per-file)
///   translate: layer 2 (SHA-256 hash in .th.md.hash)
///   all:       layer 1 (mise sources/outputs — outputs newer than sources → skip)
///
/// Running `mise run` inside a subprocess inherits the parent's environment
/// (QUICK_FONT_DIR, QUICK_THEME_FILE, etc.) so all config stays consistent.
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

// ── helpers ────────────────────────────────────────────────────────────────────

fn is_relevant(event: &Event) -> bool {
    // Only react to actual file content changes, not metadata or access events
    matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) && event.paths.iter().any(|p| {
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // Watch .md and .typ files; skip generated .th.md and .hash files
        (ext == "md" || ext == "typ")
            && !name.ends_with(".th.md")
            && !name.ends_with(".hash")
    })
}

/// Run `mise run <task>` as a subprocess, inheriting the current environment.
/// mise is always in PATH on mise-managed machines; which::which is a safety check.
fn mise_run(task: &str) -> Result<()> {
    let mise = which::which("mise").context(
        "mise not found in PATH — is it installed? https://mise.jdx.dev",
    )?;
    let status = Command::new(mise)
        .args(["run", task])
        .status()
        .with_context(|| format!("running mise run {task}"))?;
    if !status.success() {
        bail!("mise run {task} failed");
    }
    Ok(())
}

// ── subcommand: watch ──────────────────────────────────────────────────────────

pub fn cmd_watch() -> Result<()> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher =
        RecommendedWatcher::new(tx, notify::Config::default())
            .context("creating file watcher")?;

    // Watch the current directory (non-recursive — spec .md files live here)
    watcher
        .watch(Path::new("."), RecursiveMode::NonRecursive)
        .context("watching current directory")?;

    // Watch scripts/ for theme.typ changes
    if Path::new("scripts").exists() {
        watcher
            .watch(Path::new("scripts"), RecursiveMode::NonRecursive)
            .context("watching scripts/")?;
    }

    println!("Watching *.md and scripts/theme.typ — Ctrl+C to stop");
    println!("Runs: mise run fonts && mise run all\n");

    // Debounce: ignore events within 300 ms of the last triggered run.
    // This prevents double-fires when editors write + rename temp files.
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

                    // Drain any buffered events before running so we don't
                    // double-trigger on rapid successive saves
                    while rx.try_recv().is_ok() {}

                    let names: Vec<_> = event
                        .paths
                        .iter()
                        .filter_map(|p| p.file_name()?.to_str().map(str::to_owned))
                        .collect();
                    println!("↺  {}", names.join(", "));

                    // fonts first (downloads new fonts if theme.typ changed),
                    // then all (translate + build, both idempotent via mise)
                    if let Err(e) = mise_run("fonts") {
                        eprintln!("  fonts error: {e:#}");
                    }
                    if let Err(e) = mise_run("all") {
                        eprintln!("  build error: {e:#}");
                    }
                    println!();
                }
            }
            Ok(Ok(_)) => {} // irrelevant event — ignore
            Ok(Err(e)) => eprintln!("watcher error: {e}"),
            Err(mpsc::RecvTimeoutError::Timeout) => {} // normal poll interval
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                bail!("watcher channel disconnected unexpectedly");
            }
        }
    }
}
