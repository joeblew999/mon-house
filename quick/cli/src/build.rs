/// Build subcommand — cross-platform replacement for the bash loops in tasks.all and tasks.one.
///
/// ## Idempotency
///
/// This command has NO idempotency logic of its own.  That is intentional.
///
/// When invoked via `mise run all`, mise's sources/outputs check (layer 1) fires
/// *before* this binary ever runs.  If every `out/*.pdf` is newer than every
/// `[A-Z]*.th.md` and `scripts/theme.typ`, mise skips the task entirely and this
/// code never executes.
///
/// Do NOT add timestamp or hash checks here — they would duplicate mise's layer 1
/// and create two sources of truth that can drift.  The contract is:
///
///   mise layer 1  →  decides whether to run
///   this code     →  actually runs pandoc + typst (no skipping)
///
/// The only place that needs its own idempotency is `quick-tool watch`, and it
/// preserves layer 1 by calling `mise run all` as a subprocess rather than
/// calling this function directly.
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::Config;

const SKIP: &[&str] = &["CLAUDE.md", "README.md", "TEMPLATE.md"];
const TMP: &str = "_tmp.typ";

// ── internal helpers ───────────────────────────────────────────────────────────

fn run_pandoc(src: &Path, theme: &Path, lang: &str, region: &str) -> Result<()> {
    // Pandoc requires forward slashes in -V template= even on Windows.
    // Path::display() gives backslashes on Windows, so normalise explicitly.
    let src_str = src.to_str()
        .with_context(|| format!("source path '{}' contains non-UTF-8 characters", src.display()))?;
    let theme_fwd = theme.to_string_lossy().replace('\\', "/");
    let status = Command::new("pandoc")
        .args([
            src_str,
            "-t", "typst",
            "--standalone",
            "-V", &format!("template={theme_fwd}"),
            "-V", &format!("lang={lang}"),
            "-V", &format!("region={region}"),
            "-o", TMP,
        ])
        .status()
        .context("running pandoc — is it installed?")?;
    if !status.success() {
        bail!("pandoc failed for {}", src.display());
    }
    Ok(())
}

fn run_typst(font_dir: &Path, out: &str) -> Result<()> {
    let font_dir_str = font_dir.to_str()
        .with_context(|| format!("font-dir '{}' contains non-UTF-8 characters", font_dir.display()))?;
    let status = Command::new("typst")
        .args([
            "compile",
            "--ignore-system-fonts",
            "--font-path",
            font_dir_str,
            TMP,
            out,
        ])
        .status()
        .context("running typst — is it installed?")?;
    if !status.success() {
        bail!("typst compile failed → {out}");
    }
    Ok(())
}

fn build_one(stem: &str, cfg: &Config) -> Result<()> {
    let en_src = std::path::PathBuf::from(format!("{stem}.md"));
    let th_src = std::path::PathBuf::from(format!("{stem}.th.md"));

    if !en_src.exists() {
        bail!("{stem}.md not found");
    }
    if !th_src.exists() {
        bail!(
            "{stem}.th.md not found — run `quick-tool translate` first"
        );
    }

    println!("→ {stem}");

    // EN PDF
    run_pandoc(&en_src, &cfg.theme_file, "en", "US")?;
    run_typst(&cfg.font_dir, &format!("out/{stem}.pdf"))?;

    // Thai PDF
    run_pandoc(&th_src, &cfg.theme_file, "th", "TH")?;
    run_typst(&cfg.font_dir, &format!("out/{stem}.th.pdf"))?;

    Ok(())
}

// ── subcommand: build ──────────────────────────────────────────────────────────

pub fn cmd_build(cfg: &Config, name: Option<String>) -> Result<()> {
    std::fs::create_dir_all("out").context("creating out/")?;

    let full_build = name.is_none();

    if let Some(stem) = name {
        build_one(&stem, cfg)?;
    } else {
        let mut count = 0u32;
        for entry in glob::glob("[A-Z]*.md").context("invalid glob pattern")? {
            let path = entry.context("glob error")?;
            let fname = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            if fname.ends_with(".th.md") || SKIP.contains(&fname) {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            build_one(stem, cfg)?;
            count += 1;
        }
        println!("✓ out/ updated ({count} spec(s))");
    }

    // Write stamp file only on a successful full build.
    // tasks.all uses this as its output (not the glob out/*.pdf) so that
    // `mise run one` — which builds a single spec without writing the stamp —
    // cannot cause tasks.all to skip a full rebuild on next invocation.
    if full_build {
        std::fs::write("out/.build-stamp", "")?;
    }

    // Clean up intermediate file — ignore error (may not exist on failure)
    let _ = std::fs::remove_file(TMP);
    Ok(())
}

// ── subcommand: clean ──────────────────────────────────────────────────────────

pub fn cmd_clean() -> Result<()> {
    if Path::new("out").exists() {
        std::fs::remove_dir_all("out").context("removing out/")?;
        println!("✓ cleaned");
    } else {
        println!("✓ nothing to clean");
    }
    Ok(())
}
