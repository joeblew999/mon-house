// Local file watcher — re-runs gen + translate + build whenever a relevant
// file changes on disk. Replaces the TypeScript watcher that used to live in
// quick/local/ and pushed changes to the CF PipelineAgent over WebSocket.
//
// What's watched:
//   * specs_dir (recursive)         — *.md + _partials/*.md
//   * scripts_dir (recursive)       — theme.typ + themes/*.typ
//   * data_dir (recursive)          — *.json + *.nu (data layer)
//   * <project>/resources/images/   — *.svg (translated to .th.svg)
//
// Strategy on each event:
//   1. Drop our own outputs (.th.md / .th.svg / .cache.json) so we don't loop.
//   2. If any data/*.json or data/*.nu changed, run `gen` first so the
//      generated partials in specs/_partials/ are fresh before translate.
//   3. Run translate (chunk cache hits make this cheap for unchanged sections).
//   4. Run build for the affected stem if we can identify it; otherwise build
//      everything. Build is per-file mtime-driven via mise's sources/outputs,
//      so a "build all" call where nothing changed is a no-op.
//
// Save bursts are debounced — VSCode often emits multiple events per save —
// via notify-debouncer-mini at a 250 ms interval.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};

use crate::{build, gen, includes, translate, vfs, Config};

const DEBOUNCE_MS: u64 = 250;

pub fn cmd_watch(cfg: &Config) -> Result<()> {
    let images_dir = cfg.resolved_images_dir();

    // Friendly intro: what's being watched, and how to stop.
    println!(
        "[watch] specs    : {}\n[watch] scripts  : {}\n[watch] data     : {}\n[watch] images   : {}\n[watch] (Ctrl+C to stop)\n",
        cfg.specs_dir.display(),
        cfg.scripts_dir.display(),
        cfg.data_dir.display(),
        images_dir.display(),
    );

    // Channel for debounced events.
    let (tx, rx) = std::sync::mpsc::channel::<DebounceEventResult>();

    let mut debouncer = new_debouncer(Duration::from_millis(DEBOUNCE_MS), move |res| {
        // Channel send can fail only if the receiver was dropped — at which
        // point the watcher is shutting down anyway, so swallowing is fine.
        let _ = tx.send(res);
    })
    .context("create file-system debouncer")?;

    // Each watched root may not exist yet on a fresh checkout — be lenient.
    for root in [&cfg.specs_dir, &cfg.scripts_dir, &cfg.data_dir, &images_dir] {
        if !vfs::exists(root) {
            eprintln!("[watch] note: {} does not exist (skipping)", root.display());
            continue;
        }
        debouncer
            .watcher()
            .watch(root, RecursiveMode::Recursive)
            .with_context(|| format!("watching {}", root.display()))?;
    }

    for events in rx {
        match events {
            Ok(events) => {
                let touched: Vec<PathBuf> = events.into_iter().map(|e| e.path).collect();
                if let Err(err) = handle_burst(cfg, &touched) {
                    eprintln!("[watch] error: {err:#}");
                }
            }
            Err(err) => eprintln!("[watch] watch error: {err}"),
        }
    }
    Ok(())
}

/// Decide what to rebuild from a debounced burst of paths and run it.
fn handle_burst(cfg: &Config, paths: &[PathBuf]) -> Result<()> {
    // De-dup paths and drop our own outputs so a translate-write doesn't trigger
    // another translate run.
    let mut interesting: Vec<&Path> = paths
        .iter()
        .map(|p| p.as_path())
        .filter(|p| is_interesting(p))
        .collect();
    interesting.sort();
    interesting.dedup();
    if interesting.is_empty() {
        return Ok(());
    }

    println!("[watch] change ({} file{}):", interesting.len(), if interesting.len() == 1 { "" } else { "s" });
    for p in &interesting {
        println!("[watch]   {}", p.display());
    }

    // For partial changes, surface the dependent specs so the user knows
    // which derived values (e.g. hand-computed can counts) need a review pass.
    // Markdown can't auto-recompute these — the notification is the contract.
    notify_partial_dependents(cfg, &interesting);

    // Data-layer step: if any data/*.json or data/*.nu changed in this burst,
    // run gen first so the generated partials in specs/_partials/ are fresh
    // before translate reads them. gen is itself idempotent (hash-and-skip
    // per output) so this costs nothing if nothing relevant changed.
    let data_dir_name = cfg
        .data_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("data");
    let any_data_changed = interesting
        .iter()
        .any(|p| is_under_named_dir(p, data_dir_name));
    if any_data_changed {
        println!("[watch] data change → running gen");
        gen::cmd_gen(cfg)?;
    }

    // Strategy: translate first (cache makes this fast), then build affected
    // stem if exactly one spec changed; otherwise build everything (mise's
    // mtime check makes a no-op build cheap).
    translate::cmd_translate(cfg, vec![])?;

    let single_spec_stem = single_spec_stem(cfg, &interesting);
    build::cmd_build(cfg, single_spec_stem)?;
    Ok(())
}

/// True if any ancestor directory of `path` has the given name.
/// Used to detect whether a watched path lives under data/ regardless of
/// whether the path is absolute (notify) or relative (cfg.data_dir).
fn is_under_named_dir(path: &Path, dir_name: &str) -> bool {
    path.ancestors()
        .any(|a| a.file_name().and_then(|s| s.to_str()) == Some(dir_name))
}

/// Drop our own outputs and anything outside the watched roots.
fn is_interesting(path: &Path) -> bool {
    let name = match path.file_name().and_then(|s| s.to_str()) {
        Some(n) => n,
        None => return false,
    };
    // Outputs we ourselves write — would loop forever if not filtered.
    if name.ends_with(".th.md")
        || name.ends_with(".th.svg")
        || name.ends_with(".th.md.cache.json")
        || name.ends_with(".th.md.cache.tmp")
        || name.ends_with(".tmp")
        || name == "_tmp.md"
        || name == "_tmp.typ"
    {
        return false;
    }
    // Only watch files we care about: markdown, SVG, typst, plus the
    // data-layer pair (JSON catalogues + .nu generators). Translation cache
    // outputs (*.th.md.cache.json) are filtered above by suffix.
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("md") | Some("svg") | Some("typ") | Some("json") | Some("nu")
    )
}

/// For each changed partial in the burst, print the list of top-level specs
/// that include it so the user knows which dependents to review. Failures
/// (unreadable specs, broken canonicalization) are silently swallowed —
/// notification is best-effort and must never abort the watch loop.
fn notify_partial_dependents(cfg: &Config, paths: &[&Path]) {
    for path in paths {
        // notify-debouncer hands us absolute paths; cfg.specs_dir is usually
        // relative ("specs"). Compare by ancestors having a "_partials" segment
        // instead of starts_with(specs_dir/_partials), which fails across the
        // relative/absolute boundary.
        let is_partial = path
            .ancestors()
            .any(|a| a.file_name().and_then(|s| s.to_str()) == Some("_partials"));
        if !is_partial {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        // Skip our own outputs (.th.md siblings of partials).
        if path
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|n| n.ends_with(".th.md"))
        {
            continue;
        }
        match pollster::block_on(includes::find_dependents(
            &vfs::LocalVfs,
            &cfg.specs_dir,
            path,
        )) {
            Ok(deps) if deps.is_empty() => {
                println!(
                    "[watch] note: {} has no dependents in specs/",
                    path.display()
                );
            }
            Ok(deps) => {
                println!(
                    "[watch] partial changed: {} → review {} dependent spec{}:",
                    path.display(),
                    deps.len(),
                    if deps.len() == 1 { "" } else { "s" },
                );
                for d in deps {
                    println!("[watch]   • {}", d.display());
                }
            }
            Err(err) => {
                eprintln!(
                    "[watch] dep-scan failed for {}: {err:#}",
                    path.display()
                );
            }
        }
    }
}

/// If exactly one top-level spec was touched, return its stem so build can
/// be scoped to that one PDF. Otherwise (theme change, multi-file edit, SVG-
/// only change, partial change, etc.) return None so build runs for everything.
fn single_spec_stem(cfg: &Config, paths: &[&Path]) -> Option<String> {
    let mut stems: Vec<String> = Vec::new();
    for p in paths {
        // Only top-level specs/[A-Z]*.md count — skip partials, SVGs, themes.
        if p.parent() != Some(cfg.specs_dir.as_path()) {
            return None;
        }
        if p.extension().and_then(|s| s.to_str()) != Some("md") {
            return None;
        }
        let stem = p.file_stem()?.to_str()?;
        if !stem.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
            return None;
        }
        stems.push(stem.to_string());
    }
    stems.dedup();
    if stems.len() == 1 { stems.into_iter().next() } else { None }
}
