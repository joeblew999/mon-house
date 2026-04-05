/// End-to-end pipeline integration tests.
///
/// ## Two tiers
///
/// **Tier 1 — unit logic (always run, no tools)**
/// Lives in the src modules as `#[cfg(test)]`.  Run with `cargo test`.
///
/// **Tier 2 — integration (marked `#[ignore]`, requires pandoc + typst + fonts)**
/// These tests call the `quick-tool` binary from the real `quick/` directory
/// and verify that the correct OUTPUT FILES exist and have sensible content.
/// They DO NOT just check exit codes — they check side effects.
///
/// Run:
///   cargo test                    # tier 1 only (fast, no tools)
///   cargo test -- --ignored       # tier 2 only (requires mise tools)
///   mise run test                 # both tiers
///
/// ## Configuring the theme under test
///
///   QUICK_TEST_THEME=compact cargo test -- --ignored
///
/// Tests that switch themes always restore the original on exit.
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// ── harness helpers ────────────────────────────────────────────────────────────

fn quick_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ must have a parent directory")
        .to_path_buf()
}

fn quick_tool() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../target/release/quick-tool")
}

fn qt(args: &[&str]) -> std::process::Output {
    Command::new(quick_tool())
        .args(args)
        .current_dir(quick_dir())
        .output()
        .unwrap_or_else(|e| panic!(
            "quick-tool not found — run `cargo build --release` first: {e}"
        ))
}

fn stdout(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}
fn stderr(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// Switch theme before test, restore original on drop.
struct ThemeGuard { original: String }
impl ThemeGuard {
    fn set(target: &str) -> Self {
        let cur = qt(&["themes", "current"]);
        let original = stdout(&cur)
            .lines()
            .find_map(|l| l.strip_prefix("Active theme:"))
            .unwrap_or("default")
            .split_whitespace()
            .next()
            .unwrap_or("default")
            .to_owned();
        if target != original {
            let sw = qt(&["themes", "switch", target]);
            assert!(sw.status.success(),
                "themes switch {target} failed: {}", stderr(&sw));
        }
        ThemeGuard { original }
    }
}
impl Drop for ThemeGuard {
    fn drop(&mut self) {
        let _ = Command::new(quick_tool())
            .args(["themes", "switch", &self.original])
            .current_dir(quick_dir())
            .status();
    }
}

/// Collect all buildable spec stems from quick/ — same filter as build.rs and translate.rs.
fn spec_stems() -> Vec<String> {
    // Must match the SKIP list in build.rs and translate.rs exactly
    const SKIP: &[&str] = &["CLAUDE.md", "README.md", "TEMPLATE.md"];
    let dir = quick_dir();
    let pattern = dir.join("[A-Z]*.md").to_string_lossy().into_owned();
    glob::glob(&pattern)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|p| {
            let name = p.file_name()?.to_str()?.to_owned();
            if name.ends_with(".th.md") { return None; }
            if SKIP.contains(&name.as_str()) { return None; }
            Some(p.file_stem()?.to_str()?.to_owned())
        })
        .collect()
}

// ── tier 2 integration tests ───────────────────────────────────────────────────

#[test]
#[ignore]
fn tier2_translate_produces_th_md_files() {
    let out = qt(&["translate"]);
    assert!(out.status.success(), "translate failed: {}", stderr(&out));

    // Side-effect check: every EN spec must have a corresponding .th.md
    for stem in spec_stems() {
        let th_path = quick_dir().join(format!("{stem}.th.md"));
        assert!(th_path.exists(), "{stem}.th.md missing after translate");

        let content = fs::read_to_string(&th_path)
            .unwrap_or_else(|_| panic!("{stem}.th.md not readable"));
        assert!(!content.trim().is_empty(), "{stem}.th.md is empty");
        assert!(content.len() > 50,
            "{stem}.th.md suspiciously short ({} bytes)", content.len());
    }
}

#[test]
#[ignore]
fn tier2_translate_is_idempotent() {
    let out = qt(&["translate"]);
    assert!(out.status.success(), "translate failed: {}", stderr(&out));
    // No file should have triggered an actual translation
    assert!(stdout(&out).contains("0 translated"),
        "expected 0 translated on repeat run:\n{}", stdout(&out));
    assert!(!stdout(&out).contains("translating "),
        "unexpected translation triggered:\n{}", stdout(&out));
}

#[test]
#[ignore]
fn tier2_build_produces_pdfs_for_all_specs() {
    let theme = std::env::var("QUICK_TEST_THEME").unwrap_or_else(|_| "default".into());
    let _guard = ThemeGuard::set(&theme);

    let out = qt(&["build"]);
    assert!(out.status.success(), "build failed: {}", stderr(&out));

    let qdir = quick_dir();
    let stems = spec_stems();
    assert!(!stems.is_empty(), "no spec files found in quick/");

    for stem in &stems {
        // EN PDF
        let en_pdf = qdir.join("out").join(format!("{stem}.pdf"));
        assert!(en_pdf.exists(),
            "EN PDF missing: out/{stem}.pdf\nbuild output:\n{}", stdout(&out));
        let en_size = fs::metadata(&en_pdf).unwrap().len();
        assert!(en_size > 1024,
            "out/{stem}.pdf suspiciously small ({en_size} bytes)");

        // Thai PDF
        let th_pdf = qdir.join("out").join(format!("{stem}.th.pdf"));
        assert!(th_pdf.exists(),
            "Thai PDF missing: out/{stem}.th.pdf");
        let th_size = fs::metadata(&th_pdf).unwrap().len();
        assert!(th_size > 1024,
            "out/{stem}.th.pdf suspiciously small ({th_size} bytes)");
    }

    // Stamp file must exist after a full build
    let stamp = qdir.join("out/.build-stamp");
    assert!(stamp.exists(), "out/.build-stamp missing after build");

    println!(
        "✓ {n} EN + {n} Thai PDFs present for theme '{theme}'",
        n = stems.len()
    );
}

#[test]
#[ignore]
fn tier2_themes_switch_updates_wrapper_file() {
    let theme = std::env::var("QUICK_TEST_THEME").unwrap_or_else(|_| "minimal".into());
    let _guard = ThemeGuard::set(&theme);

    // Side-effect: scripts/theme.typ must contain the import line for the active theme
    let wrapper = fs::read_to_string(quick_dir().join("scripts/theme.typ"))
        .expect("scripts/theme.typ not readable");
    assert!(
        wrapper.contains(&format!("themes/{theme}.typ")),
        "scripts/theme.typ does not import '{theme}':\n{wrapper}"
    );
    assert!(
        wrapper.contains(&format!("// Active theme: {theme}")),
        "scripts/theme.typ missing '// Active theme: {theme}' comment:\n{wrapper}"
    );
}

#[test]
#[ignore]
fn tier2_themes_test_all_every_theme_compiles() {
    let out = qt(&["themes", "test", "--all"]);
    assert!(out.status.success(), "themes test --all failed:\n{}", stderr(&out));
    // Side-effect: all three themes must pass
    let s = stdout(&out);
    assert!(s.contains("default") && s.contains("PASS"),
        "default theme did not PASS:\n{s}");
    assert!(s.contains("minimal") && s.contains("PASS"),
        "minimal theme did not PASS:\n{s}");
    assert!(s.contains("compact") && s.contains("PASS"),
        "compact theme did not PASS:\n{s}");
    assert!(s.contains("All 3 theme(s) passed"), "summary line missing:\n{s}");
}

#[test]
#[ignore]
fn tier2_fonts_download_all_files_present() {
    let out = qt(&["fonts", "download"]);
    assert!(out.status.success(), "fonts download failed: {}", stderr(&out));

    // Side-effect: every expected .ttf file must exist in fonts/
    let fonts_dir = quick_dir().join("fonts");
    assert!(fonts_dir.exists(), "fonts/ directory missing");

    // At minimum: Inter, Noto Sans, Noto Sans Thai — 2 weights each = 6 files
    let ttf_count = fs::read_dir(&fonts_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .and_then(|x| x.to_str())
                .map(|x| x == "ttf")
                .unwrap_or(false)
        })
        .count();
    assert!(ttf_count >= 6,
        "expected at least 6 .ttf files in fonts/, found {ttf_count}");

    // Stamp file must exist
    let done = quick_dir().join("fonts/.done");
    assert!(done.exists(), "fonts/.done stamp missing");
}

#[test]
#[ignore]
fn tier2_full_watch_pipeline_side_effects() {
    // This is the integration version of `mise run watch` firing on a file save.
    // Runs fonts → translate → build and verifies ALL side effects end-to-end.
    let theme = std::env::var("QUICK_TEST_THEME").unwrap_or_else(|_| "default".into());
    let _guard = ThemeGuard::set(&theme);
    let qdir = quick_dir();

    // 1. fonts: succeeds + all .ttf files physically present + stamp exists
    // (We don't assert "up to date" because a prior test may have switched the theme,
    // invalidating the stamp — what matters is fonts are correct after this call.)
    let f = qt(&["fonts", "download"]);
    assert!(f.status.success(), "fonts: {}", stderr(&f));
    assert!(qdir.join("fonts/.done").exists(), "fonts/.done missing");
    // Verify at least the expected TTF count (3 families × 2 weights = 6)
    let ttf_count = std::fs::read_dir(qdir.join("fonts")).unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("ttf"))
        .count();
    assert!(ttf_count >= 6, "expected ≥6 .ttf files after fonts download, got {ttf_count}");

    // 2. translate: idempotent + all .th.md files present and non-empty
    let t = qt(&["translate"]);
    assert!(t.status.success(), "translate: {}", stderr(&t));
    assert!(stdout(&t).contains("0 translated"),
        "translate should skip everything:\n{}", stdout(&t));
    for stem in spec_stems() {
        let th = qdir.join(format!("{stem}.th.md"));
        assert!(th.exists(), "{stem}.th.md missing");
        assert!(th.metadata().unwrap().len() > 50, "{stem}.th.md too small");
    }

    // 3. build: all PDFs present, non-empty, stamp written
    let b = qt(&["build"]);
    assert!(b.status.success(), "build: {}", stderr(&b));
    let stamp = qdir.join("out/.build-stamp");
    assert!(stamp.exists(), "out/.build-stamp missing after build");
    for stem in spec_stems() {
        for suffix in &["", ".th"] {
            let pdf = qdir.join("out").join(format!("{stem}{suffix}.pdf"));
            assert!(pdf.exists(), "{} missing", pdf.display());
            assert!(pdf.metadata().unwrap().len() > 1024,
                "{} too small", pdf.display());
        }
    }

    // 4. themes check: the wrapper and active theme are healthy
    let chk = qt(&["themes", "check"]);
    assert!(chk.status.success(),
        "themes check failed for '{theme}':\n{}", stderr(&chk));

    println!("✓ Full pipeline verified for theme '{theme}': fonts → translate �� build → check");
}
