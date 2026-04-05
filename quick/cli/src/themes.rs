/// Theme registry — discover, list, switch, and test Typst themes.
///
/// ## How themes work
///
/// `scripts/theme.typ` is an import-wrapper file auto-managed by this module:
///
///   // Active theme: default
///   #import "themes/default.typ": *
///
/// Pandoc's generated `_tmp.typ` imports `conf` from `scripts/theme.typ`.
/// Because `scripts/theme.typ` wildcard-imports from the active theme file,
/// `conf` (and `grid-images`) are re-exported automatically.
///
/// Typst resolves `themes/default.typ` relative to `scripts/`, giving
/// `scripts/themes/default.typ` — the correct location regardless of CWD.
///
/// ## Registry
///
/// `scripts/themes/registry.toml` lists available themes and descriptions.
/// The `name` field must match a `{name}.typ` file in the same directory.
///
/// ## Font detection
///
/// `fonts::parse_families()` already follows `#import` one level deep, so
/// switching themes also updates which fonts are downloaded on next `fonts`.
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::Config;

// ── registry ───────────────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct ThemeEntry {
    name: String,
    description: String,
}

#[derive(Deserialize, Debug)]
struct Registry {
    themes: Vec<ThemeEntry>,
}

fn themes_dir(cfg: &Config) -> PathBuf {
    cfg.resolved_theme_file()
        .parent()
        .unwrap_or_else(|| Path::new("scripts"))
        .join("themes")
}

fn registry_path(cfg: &Config) -> PathBuf {
    themes_dir(cfg).join("registry.toml")
}

fn load_registry(cfg: &Config) -> Result<Registry> {
    let path = registry_path(cfg);
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("reading registry {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

/// Parse the active theme name from the `// Active theme: <name>` comment.
pub fn active_theme_name(cfg: &Config) -> Result<String> {
    let text = std::fs::read_to_string(&cfg.resolved_theme_file())
        .with_context(|| format!("reading {}", cfg.resolved_theme_file().display()))?;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("// Active theme:") {
            return Ok(rest.trim().to_string());
        }
    }
    bail!(
        "could not find '// Active theme: <name>' in {}.\n\
         Run `quick-tool themes switch default` to initialise.",
        cfg.resolved_theme_file().display()
    )
}

fn theme_file_path(cfg: &Config, name: &str) -> PathBuf {
    themes_dir(cfg).join(format!("{name}.typ"))
}

// ── subcommand: list ───────────────────────────────────────────────────────────

pub fn cmd_list(cfg: &Config) -> Result<()> {
    let reg = load_registry(cfg)?;
    let active = active_theme_name(cfg).unwrap_or_default();
    let dir = themes_dir(cfg);

    println!("Available themes ({}):\n", dir.display());
    println!("  {:<12}  {:<6}  {}", "NAME", "FILE?", "DESCRIPTION");
    println!("  {}", "-".repeat(70));

    for t in &reg.themes {
        let file_exists = dir.join(format!("{}.typ", t.name)).exists();
        let marker = if t.name == active { "▶" } else { " " };
        let file_ok = if file_exists { "✓" } else { "✗ MISSING" };
        println!("  {marker} {:<10}  {:<6}  {}", t.name, file_ok, t.description);
    }

    println!();
    println!("Active: {active}");
    println!("Switch: quick-tool themes switch <name>");
    Ok(())
}

// ── subcommand: current ────────────────────────────────────────────────────────

pub fn cmd_current(cfg: &Config) -> Result<()> {
    let name = active_theme_name(cfg)?;
    let path = theme_file_path(cfg, &name);
    let exists = if path.exists() { "✓" } else { "✗ FILE MISSING" };
    println!("Active theme: {name}  {exists}");
    println!("File:         {}", path.display());
    println!("Wrapper:      {}", cfg.resolved_theme_file().display());
    Ok(())
}

// ── subcommand: switch ─────────────────────────────────────────────────────────

pub fn cmd_switch(cfg: &Config, name: &str) -> Result<()> {
    let reg = load_registry(cfg)?;

    // Validate name is in registry
    if !reg.themes.iter().any(|t| t.name == name) {
        let names: Vec<_> = reg.themes.iter().map(|t| t.name.as_str()).collect();
        bail!(
            "unknown theme '{}'. Available: {}",
            name,
            names.join(", ")
        );
    }

    let target = theme_file_path(cfg, name);
    if !target.exists() {
        bail!("{} not found. The theme file is missing.", target.display());
    }

    let wrapper = format!(
        "// Active theme: {name}\n\
         // Auto-managed by `quick-tool themes switch` — do not edit manually.\n\
         // To change theme:   quick-tool themes switch <name>\n\
         // To list themes:    quick-tool themes list\n\
         #import \"themes/{name}.typ\": *\n"
    );

    std::fs::write(&cfg.resolved_theme_file(), wrapper)
        .with_context(|| format!("writing {}", cfg.resolved_theme_file().display()))?;

    println!("✓ Switched to theme: {name}");
    println!("  {}", cfg.resolved_theme_file().display());
    println!();
    println!("Next steps:");
    println!("  • If fonts changed: mise run fonts  (or quick-tool fonts download)");
    println!("  • Rebuild PDFs:     mise run all    (or quick-tool build)");
    Ok(())
}

// ── subcommand: test ───────────────────────────────────────────────────────────

/// Compile a minimal test PDF with the given theme file.
///
/// Files are written to CWD (same as normal `_tmp.typ` builds) so that typst
/// resolves `#import "scripts/themes/foo.typ"` relative to the .typ file's
/// location — which is CWD, where `scripts/` actually exists.
///
/// Test files use a `_theme_test_` prefix so they are never matched by the
/// `[A-Z]*.md` glob patterns used by translate and build.
fn compile_test(cfg: &Config, theme_path: &Path, label: &str) -> Result<PathBuf> {
    // Forward-slash theme path for pandoc -V template= (required on Windows too)
    let theme_fwd = theme_path.to_string_lossy().replace('\\', "/");

    // Minimal CommonMark spec document that exercises all theme elements:
    // cover block, H1/H2/H3, paragraph, table, blockquote, hr
    let md = "---\ntitle: Theme Test\nstatus: Draft\nrev: \"1\"\n---\n\n\
              # Section One\n\n\
              A paragraph of body text to check font and line spacing.\n\n\
              ## Subsection\n\n\
              | Column A | Column B | Column C |\n\
              |----------|----------|----------|\n\
              | Row 1 A  | Row 1 B  | Row 1 C  |\n\
              | Row 2 A  | Row 2 B  | Row 2 C  |\n\n\
              ### Detail\n\n\
              A third-level heading with some text below.\n\n\
              # Section Two\n\n\
              > A blockquote paragraph for checking indentation.\n\n\
              ---\n\n\
              Final paragraph after a horizontal rule.\n";

    // Write to CWD so typst can resolve `#import "scripts/themes/..."` correctly.
    // Prefix `_theme_test_` keeps these out of [A-Z]*.md glob patterns.
    let md_file  = PathBuf::from(format!("_theme_test_{label}.md"));
    let typ_file = PathBuf::from(format!("_theme_test_{label}.typ"));
    let pdf_file = PathBuf::from(format!("_theme_test_{label}.pdf"));

    // Clean up on exit regardless of success/failure
    let _cleanup = Cleanup(vec![md_file.clone(), typ_file.clone()]);

    std::fs::write(&md_file, md)?;

    let md_str  = md_file.to_str().context("md path contains non-UTF-8")?;
    let typ_str = typ_file.to_str().context("typ path contains non-UTF-8")?;
    let pdf_str = pdf_file.to_str().context("pdf path contains non-UTF-8")?;
    let font_dir = cfg.resolved_font_dir();
    let font_str = font_dir.to_str()
        .context("font-dir path contains non-UTF-8")?;

    // pandoc: md → typ
    let pandoc = which::which("pandoc").context("pandoc not found in PATH")?;
    let status = Command::new(&pandoc)
        .args([
            md_str,
            "-t", "typst",
            "--standalone",
            "-V", &format!("template={theme_fwd}"),
            "-V", "lang=en",
            "-V", "region=US",
            "-o", typ_str,
        ])
        .status()
        .context("running pandoc")?;
    if !status.success() {
        bail!("pandoc failed for theme '{label}'");
    }

    // typst: typ → pdf
    let typst = which::which("typst").context("typst not found in PATH")?;
    let status = Command::new(&typst)
        .args([
            "compile",
            "--ignore-system-fonts",
            "--font-path",
            font_str,
            typ_str,
            pdf_str,
        ])
        .status()
        .context("running typst")?;
    if !status.success() {
        bail!("typst compile failed for theme '{label}'");
    }

    Ok(pdf_file)
}

/// RAII cleanup: removes files when dropped, ignoring errors.
struct Cleanup(Vec<PathBuf>);
impl Drop for Cleanup {
    fn drop(&mut self) {
        for p in &self.0 {
            let _ = std::fs::remove_file(p);
        }
    }
}

pub fn cmd_test(cfg: &Config, name: Option<&str>, all: bool) -> Result<()> {
    let reg = load_registry(cfg)?;

    if all {
        // Test every theme in the registry
        let mut passed = 0u32;
        let mut failed = 0u32;
        println!("Testing all {} theme(s):\n", reg.themes.len());
        for t in &reg.themes {
            let theme_path = theme_file_path(cfg, &t.name);
            print!("  {:<12} ", t.name);
            std::io::stdout().flush().ok();
            match compile_test(cfg, &theme_path, &t.name) {
                Ok(pdf) => {
                    println!("PASS  ({})", pdf.display());
                    passed += 1;
                }
                Err(e) => {
                    println!("FAIL  {e:#}");
                    failed += 1;
                }
            }
        }
        println!();
        if failed == 0 {
            println!("All {passed} theme(s) passed.");
            Ok(())
        } else {
            bail!("{failed} theme(s) failed.");
        }
    } else {
        // Test one theme (named or current)
        let target_name = match name {
            Some(n) => n.to_string(),
            None => active_theme_name(cfg)?,
        };
        let theme_path = theme_file_path(cfg, &target_name);
        if !theme_path.exists() {
            bail!("{} not found", theme_path.display());
        }
        println!("Testing theme '{target_name}'...");
        match compile_test(cfg, &theme_path, &target_name) {
            Ok(pdf) => {
                println!("  PASS  ({})", pdf.display());
                Ok(())
            }
            Err(e) => bail!("  FAIL  {e:#}"),
        }
    }
}

// ── subcommand: check ──────────────────────────────────────────────────────────

/// Check wrapper file + active theme file exist and are parseable.
pub fn cmd_check(cfg: &Config) -> Result<()> {
    let mut failures: Vec<String> = Vec::new();

    // 1. Wrapper file exists
    println!("Check 1: wrapper {} exists", cfg.resolved_theme_file().display());
    if cfg.resolved_theme_file().exists() {
        println!("  PASS");
    } else {
        println!("  FAIL: file missing");
        failures.push(format!("{} missing", cfg.resolved_theme_file().display()));
    }

    // 2. Active theme name is readable
    println!("Check 2: active theme name parseable");
    let active = match active_theme_name(cfg) {
        Ok(n) => {
            println!("  PASS: '{n}'");
            n
        }
        Err(e) => {
            println!("  FAIL: {e}");
            failures.push(e.to_string());
            String::new()
        }
    };

    // 3. Theme file exists in themes/
    if !active.is_empty() {
        let path = theme_file_path(cfg, &active);
        println!("Check 3: theme file {} exists", path.display());
        if path.exists() {
            println!("  PASS");
        } else {
            println!("  FAIL: file missing");
            failures.push(format!("{} missing", path.display()));
        }
    }

    // 4. Theme is in registry
    println!("Check 4: theme registered in registry.toml");
    match load_registry(cfg) {
        Ok(reg) => {
            if !active.is_empty() && reg.themes.iter().any(|t| t.name == active) {
                println!("  PASS");
            } else if !active.is_empty() {
                println!("  FAIL: '{active}' not in registry");
                failures.push(format!("'{active}' missing from registry.toml"));
            }
        }
        Err(e) => {
            println!("  FAIL: {e}");
            failures.push(e.to_string());
        }
    }

    // 5. Compile test
    if !active.is_empty() {
        println!("Check 5: theme compiles with pandoc + typst");
        let theme_path = theme_file_path(cfg, &active);
        if theme_path.exists() {
            match compile_test(cfg, &theme_path, &active) {
                Ok(pdf) => println!("  PASS  ({})", pdf.display()),
                Err(e) => {
                    println!("  FAIL: {e}");
                    failures.push(e.to_string());
                }
            }
        }
    }

    println!();
    if failures.is_empty() {
        println!("All checks passed. Theme '{active}' is healthy.");
        Ok(())
    } else {
        let msg = failures.join("\n  • ");
        bail!("FAILED ({} issue(s)):\n  • {msg}", failures.len())
    }
}
