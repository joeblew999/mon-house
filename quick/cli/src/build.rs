/// Build subcommand — cross-platform replacement for the bash loops in tasks.all and tasks.one.
///
/// ## Idempotency
///
/// This command has NO idempotency logic of its own.  That is intentional.
///
/// When invoked via `mise run all`, mise's sources/outputs check (layer 1) fires
/// *before* this binary ever runs.  If every `out/*.pdf` is newer than every
/// `specs/[A-Z]*.th.md` and `scripts/theme.typ`, mise skips the task entirely and
/// this code never executes.
///
/// Do NOT add timestamp or hash checks here — they would duplicate mise's layer 1
/// and create two sources of truth that can drift.  The contract is:
///
///   mise layer 1  →  decides whether to run
///   this code     →  actually runs typst (no skipping)
///
/// The only place that needs its own idempotency is `quick-tool watch`, which
/// calls the Rust functions directly and implements the stamp check itself.
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::Config;

const TMP: &str = "_tmp.typ";

// ── internal helpers ───────────────────────────────────────────────────────────

/// Write a cmarker-based Typst wrapper that replaces the pandoc md→typ step.
///
/// cmarker parses frontmatter and renders CommonMark + GFM tables natively inside
/// Typst — no external binary required.  The `scope: image` override is required
/// because cmarker uses eval() internally, which would otherwise resolve image
/// paths relative to the cmarker package directory rather than the project root.
/// Closures defined in this wrapper resolve paths relative to _tmp.typ's location
/// (the project CWD), which is correct.
pub fn write_typ_wrapper(src: &Path, theme: &Path, lang: &str, region: &str) -> Result<()> {
    // Forward slashes required in Typst import paths on all platforms (including Windows).
    let src_fwd   = src.to_string_lossy().replace('\\', "/");
    let theme_fwd = theme.to_string_lossy().replace('\\', "/");

    let wrapper = format!(
        "#import \"{theme_fwd}\": *\n\
         #import \"@preview/cmarker:0.1.8\": render-with-metadata\n\
         \n\
         #let (meta, body) = render-with-metadata(\n\
           read(\"{src_fwd}\"),\n\
           metadata-block: \"frontmatter-yaml\",\n\
           scope: (image: (src, ..args) => image(src, ..args)),\n\
         )\n\
         #show: conf.with(\n\
           title:  meta.at(\"title\",  default: \"\"),\n\
           status: meta.at(\"status\", default: \"Draft\"),\n\
           rev:    meta.at(\"rev\",    default: \"1\"),\n\
           lang: \"{lang}\",\n\
           region: \"{region}\",\n\
         )\n\
         #body\n"
    );

    std::fs::write(TMP, wrapper).context("writing _tmp.typ")?;
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
    let en_src = cfg.specs_dir.join(format!("{stem}.md"));
    let th_src = cfg.specs_dir.join(format!("{stem}.th.md"));

    if !en_src.exists() {
        bail!("{} not found", en_src.display());
    }
    if !th_src.exists() {
        bail!("{} not found — run `quick-tool translate` first", th_src.display());
    }

    println!("→ {stem}");

    // EN PDF — _tmp.typ is compiled from quick/ CWD so image() paths resolve correctly
    run_pandoc(&en_src, &cfg.resolved_theme_file(), "en", "US")?;
    let en_pdf = cfg.out_dir.join(format!("{stem}.pdf"));
    run_typst(&cfg.resolved_font_dir(), en_pdf.to_str().context("out_dir path contains non-UTF-8")?)?;

    // Thai PDF
    run_pandoc(&th_src, &cfg.resolved_theme_file(), "th", "TH")?;
    let th_pdf = cfg.out_dir.join(format!("{stem}.th.pdf"));
    run_typst(&cfg.resolved_font_dir(), th_pdf.to_str().context("out_dir path contains non-UTF-8")?)?;

    Ok(())
}

// ── subcommand: build ──────────────────────────────────────────────────────────

pub fn cmd_build(cfg: &Config, name: Option<String>) -> Result<()> {
    std::fs::create_dir_all(&cfg.out_dir)
        .with_context(|| format!("creating {}/", cfg.out_dir.display()))?;

    let full_build = name.is_none();

    if let Some(stem) = name {
        build_one(&stem, cfg)?;
    } else {
        let mut count = 0u32;
        let pattern = cfg.specs_dir.join("[A-Z]*.md");
        let pattern_str = pattern.to_str().context("specs_dir path contains non-UTF-8")?;
        for entry in glob::glob(pattern_str).context("invalid glob pattern")? {
            let path = entry.context("glob error")?;
            let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or_default();
            // Skip generated .th.md files — only EN sources trigger a build
            if fname.ends_with(".th.md") {
                continue;
            }
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
            build_one(stem, cfg)?;
            count += 1;
        }
        println!("✓ {} updated ({count} spec(s))", cfg.out_dir.display());
    }

    // Write stamp file only on a successful full build.
    // tasks.all uses this as its output (not the glob out/*.pdf) so that
    // `mise run one` — which builds a single spec without writing the stamp —
    // cannot cause tasks.all to skip a full rebuild on next invocation.
    if full_build {
        std::fs::write(cfg.build_stamp(), "")?;
    }

    // Clean up intermediate file — ignore error (may not exist on failure)
    let _ = std::fs::remove_file(TMP);
    Ok(())
}

// ── subcommand: clean ──────────────────────────────────────────────────────────

pub fn cmd_clean(cfg: &Config) -> Result<()> {
    if cfg.out_dir.exists() {
        std::fs::remove_dir_all(&cfg.out_dir)
            .with_context(|| format!("removing {}/", cfg.out_dir.display()))?;
        println!("✓ cleaned {}/", cfg.out_dir.display());
    } else {
        println!("✓ nothing to clean");
    }
    Ok(())
}
