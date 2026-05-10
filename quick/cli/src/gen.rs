//! `gen` — invoke the colocated `.nu` data-layer generators in `data/`.
//!
//! Subprocesses to `nu <script>` for each `*.nu` file under `QUICK_DATA_DIR`.
//! Each script reads JSON catalogues from that directory and writes
//! `_partials/*.md` into `QUICK_SPECS_DIR`. Idempotency is the script's
//! responsibility (hash-and-skip on each output).
//!
//! Phase 2 will embed the nu engine via the `nu-*` crates. For now this is
//! a thin subprocess wrapper that gives `watch.rs` and the `mise run gen`
//! task a single entry point.

use std::process::Command;

use anyhow::{Context, Result};

use crate::Config;

pub fn cmd_gen(cfg: &Config) -> Result<()> {
    let data_dir = &cfg.data_dir;
    if !data_dir.is_dir() {
        anyhow::bail!("data dir {:?} not found", data_dir);
    }

    let mut scripts: Vec<_> = std::fs::read_dir(data_dir)
        .with_context(|| format!("reading data dir {:?}", data_dir))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "nu"))
        .collect();
    scripts.sort();

    if scripts.is_empty() {
        eprintln!("gen: no .nu generators found in {:?} — nothing to do", data_dir);
        return Ok(());
    }

    for script in &scripts {
        eprintln!("gen: running {}", script.display());
        let status = Command::new("nu")
            .arg(script)
            .env("QUICK_DATA_DIR", data_dir.as_os_str())
            .env("QUICK_SPECS_DIR", cfg.specs_dir.as_os_str())
            .status()
            .with_context(|| {
                format!(
                    "invoking `nu {}` — is nushell installed (mise install pulls 0.112)?",
                    script.display()
                )
            })?;
        if !status.success() {
            anyhow::bail!(
                "generator {} failed (exit {})",
                script.display(),
                status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".into())
            );
        }
    }

    Ok(())
}
