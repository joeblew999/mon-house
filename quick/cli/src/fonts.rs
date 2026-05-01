/// Font management — Rust port of scripts/fonts.py
///
/// All paths and API endpoints come from Config (set by mise [env] or CLI flags).
/// No hardcoded paths remain — override anything in mise.local.toml.
///
/// Idempotency (three layers):
///   Layer 1 — mise sources/outputs (task level, zero Rust cost)
///   Layer 2 — SHA-256 hash + all files present (script level)
///   Layer 3 — per-file exists check (download loop)
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::Deserialize;

use crate::{http, idempotency, vfs, Config};

// ── shared helpers ─────────────────────────────────────────────────────────────

fn extract_font_stack(text: &str) -> Option<Vec<String>> {
    let re = Regex::new(r#"font:\s*\(([^)]+)\)"#).ok()?;
    let caps = re.captures(text)?;
    let families: Vec<String> = caps[1]
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if families.is_empty() { None } else { Some(families) }
}

/// Return the path of the file that actually contains the `font: (...)` declaration.
///
/// The active theme wrapper (`scripts/theme.typ`) may just be a one-liner
/// `#import "themes/default.typ": *` — in that case the font stack lives in the
/// imported file.  This function follows that import one level deep (no recursion,
/// no absolute paths, no traversals) and returns whichever file has the stack.
///
/// Both `theme_hash` and `parse_families` use this so the resolution logic lives
/// in exactly one place.
fn font_source_file(cfg: &Config) -> Result<std::path::PathBuf> {
    let theme_file = cfg.resolved_theme_file();
    let text = vfs::read_to_string(&theme_file)?;

    // If the wrapper has a font stack directly, use it as-is
    if extract_font_stack(&text).is_some() {
        return Ok(theme_file);
    }

    // Follow one level of `#import "path": *`
    let import_re = Regex::new(r#"#import\s+"([^"]+)""#)?;
    if let Some(caps) = import_re.captures(&text) {
        let import_path = &caps[1];
        if import_path.starts_with('/') || import_path.starts_with('\\') || import_path.contains("..") {
            anyhow::bail!(
                "refusing to follow unsafe import path '{}' in {}",
                import_path, theme_file.display()
            );
        }
        let base = theme_file.parent().unwrap_or_else(|| std::path::Path::new("."));
        let imported = base.join(import_path);
        if let Ok(imported_text) = vfs::read_to_string(&imported) {
            if extract_font_stack(&imported_text).is_some() {
                return Ok(imported);
            }
        }
    }

    anyhow::bail!(
        "no 'font: (...)' declaration found in {} or its imports",
        theme_file.display()
    )
}

/// Hash the file that actually contains the font stack.
///
/// Switching themes rewrites the wrapper but not the theme file itself —
/// hashing the source file means font stamps survive a `themes switch` when
/// the font stack hasn't changed.
fn theme_hash(cfg: &Config) -> Result<String> {
    let path = font_source_file(cfg)?;
    let bytes = vfs::read_bytes(&path)?;
    Ok(idempotency::blake3_hex(&bytes))
}

pub fn parse_families(cfg: &Config) -> Result<Vec<String>> {
    let path = font_source_file(cfg)?;
    let text = vfs::read_to_string(&path)?;
    extract_font_stack(&text)
        .ok_or_else(|| anyhow::anyhow!("no 'font: (...)' in {}", path.display()))
}

fn family_to_gwfh_id(family: &str) -> String {
    family.to_lowercase().replace(' ', "-")
}

fn family_to_filename_slug(family: &str) -> String {
    family_to_gwfh_id(family).replace('-', "_")
}

fn expected_files(cfg: &Config, families: &[String]) -> Vec<PathBuf> {
    let weights = cfg.parsed_weights();
    families
        .iter()
        .flat_map(|f| {
            let slug = family_to_filename_slug(f);
            let dir = cfg.resolved_font_dir().clone();
            weights
                .iter()
                .map(move |w| dir.join(format!("{}_{}.ttf", slug, w)))
        })
        .collect()
}

fn all_files_present(cfg: &Config, families: &[String]) -> bool {
    expected_files(cfg, families).iter().all(|p| vfs::exists(p))
}


// ── subcommand: download ───────────────────────────────────────────────────────

fn subsets_for(family: &str) -> Vec<&'static str> {
    let lower = family.to_lowercase();
    if lower.contains("thai") {
        return vec!["thai"];
    }
    if lower.contains("arabic") || lower.contains("naskh") {
        return vec!["arabic"];
    }
    if lower.contains("devanagari") || lower.contains("hindi") {
        return vec!["devanagari"];
    }
    if lower.contains("chinese") || lower.contains("cjk") {
        return vec!["chinese-simplified"];
    }
    if lower.contains("japanese") {
        return vec!["japanese"];
    }
    if lower.contains("korean") {
        return vec!["korean"];
    }
    vec!["latin"]
}

#[derive(Deserialize)]
struct GwfhVariant {
    #[serde(rename = "fontWeight")]
    font_weight: String,
    ttf: Option<String>,
}

#[derive(Deserialize)]
struct GwfhFontDetail {
    variants: Vec<GwfhVariant>,
}

fn fetch_gwfh(cfg: &Config, gwfh_id: &str, subsets: &[&str]) -> Result<Vec<(String, String)>> {
    let url = format!("{}/{}?subsets={}&formats=ttf", cfg.gwfh_api, gwfh_id, subsets.join(","));
    println!("  Querying GWFH: {url}");
    let detail: GwfhFontDetail = http::get_json(&url)?;
    let weights = cfg.parsed_weights();
    let weight_set: std::collections::HashSet<String> =
        weights.iter().map(|w| w.to_string()).collect();
    let slug = gwfh_id.replace('-', "_");
    let mut seen: HashMap<String, String> = HashMap::new();
    for variant in detail.variants {
        if let Some(ttf_url) = variant.ttf {
            if weight_set.contains(&variant.font_weight) {
                let fname = format!("{}_{}.ttf", slug, variant.font_weight);
                seen.entry(fname).or_insert(ttf_url); // first URL wins (dedup)
            }
        }
    }
    Ok(seen.into_iter().collect())
}

pub fn cmd_download(cfg: &Config) -> Result<()> {
    if !vfs::exists(&cfg.resolved_theme_file()) {
        bail!("{} not found. Run from the quick/ directory.", cfg.resolved_theme_file().display());
    }
    let families = parse_families(cfg)?;
    let current_hash = theme_hash(cfg)?;

    // Layer 2: skip if theme unchanged AND all files on disk
    let done = cfg.done_file();
    if vfs::exists(&done)
        && vfs::read_to_string(&done)?.trim() == current_hash
        && all_files_present(cfg, &families)
    {
        println!("✓ fonts up to date (theme.typ unchanged, all files present)");
        return Ok(());
    }

    vfs::create_dir_all(&cfg.resolved_font_dir())?;
    println!("Font stack from {}: {families:?}\n", cfg.resolved_theme_file().display());
    let mut total_downloaded: u32 = 0;
    let mut total_skipped: u32 = 0;
    let mut fetch_errors: Vec<String> = Vec::new();

    for family in &families {
        let gwfh_id = family_to_gwfh_id(family);
        let subsets = subsets_for(family);
        println!("► {family}  (gwfh: {gwfh_id}, subsets: {subsets:?})");
        let pairs = match fetch_gwfh(cfg, &gwfh_id, &subsets) {
            Ok(p) => p,
            Err(e) => {
                // Collect the error — don't write the stamp at the end.
                // Silently continuing here would let a partial font set get
                // stamped as complete, breaking typst compilation silently.
                let msg = format!("GWFH fetch failed for '{family}': {e}");
                println!("  ERROR: {e}");
                fetch_errors.push(msg);
                continue;
            }
        };
        if pairs.is_empty() {
            println!("  WARNING: no TTF variants found for weights {:?}", cfg.parsed_weights());
        }
        for (fname, url) in &pairs {
            let dest = cfg.resolved_font_dir().join(fname);
            if vfs::exists(&dest) {
                // Layer 3: skip existing files
                println!("  ✓ {fname} (already exists, skipping)");
                total_skipped += 1;
            } else {
                println!("    ↓ {url}");
                let data = http::get_bytes(url)?;
                // Atomic write via vfs: write to .tmp then rename
                vfs::write_atomic(&dest, &data)?;
                println!("  ✓ {fname} ({} KB)", data.len() / 1024);
                total_downloaded += 1;
            }
        }
        println!();
    }

    println!("Done. {total_downloaded} downloaded, {total_skipped} skipped.");

    // Only write the stamp when every family was fetched successfully.
    // A partial stamp would cause the next run to skip re-fetching missing fonts.
    if fetch_errors.is_empty() {
        vfs::write(&done, format!("{current_hash}\n").as_bytes())?;
    } else {
        let msg = fetch_errors.join("\n  • ");
        bail!("Font download incomplete — stamp NOT written.\n  • {msg}");
    }
    Ok(())
}

// ── subcommand: test ───────────────────────────────────────────────────────────

pub fn cmd_test(cfg: &Config) -> Result<()> {
    let mut failures: Vec<String> = Vec::new();

    println!("Check 1: {} has parseable font stack", cfg.resolved_theme_file().display());
    if !vfs::exists(&cfg.resolved_theme_file()) {
        bail!("{} not found", cfg.resolved_theme_file().display());
    }
    let families = match parse_families(cfg) {
        Ok(f) => {
            println!("  PASS: {f:?}");
            f
        }
        Err(e) => bail!("  FAIL: {e}"),
    };

    println!("Check 2: font files present in {}/", cfg.resolved_font_dir().display());
    for path in expected_files(cfg, &families) {
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        if vfs::exists(&path) {
            println!("  PASS: {name}");
        } else {
            let msg = format!("missing: {}", path.display());
            println!("  FAIL: {name} missing");
            failures.push(msg);
        }
    }

    println!("Check 3: typst recognises each font family");
    let output = Command::new("typst")
        .args(["fonts", "--font-path", cfg.resolved_font_dir().to_str().unwrap_or("fonts"), "--ignore-system-fonts"])
        .output()
        .context("running `typst fonts` — is typst installed and on PATH?")?;
    let available: std::collections::HashSet<&str> = std::str::from_utf8(&output.stdout)?
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    for family in &families {
        if available.contains(family.as_str()) {
            println!("  PASS: '{family}'");
        } else {
            println!("  FAIL: '{family}' not seen by typst");
            failures.push(format!("typst cannot find family: {family}"));
        }
    }

    println!("Check 4: stamp valid");
    let done = cfg.done_file();
    if !vfs::exists(&done) {
        println!("  FAIL: {} missing — run `mise run fonts`", done.display());
        failures.push(format!("{} missing", done.display()));
    } else {
        let current = theme_hash(cfg)?;
        let stored = vfs::read_to_string(&done)?;
        if stored.trim() == current {
            println!("  PASS: stamp hash matches {}", cfg.resolved_theme_file().display());
        } else {
            println!("  FAIL: stamp hash mismatch — run `mise run fonts`");
            failures.push("stamp hash mismatch".to_string());
        }
    }

    println!();
    if failures.is_empty() {
        println!(
            "All checks passed. ({} families, {} files)",
            families.len(),
            expected_files(cfg, &families).len()
        );
        Ok(())
    } else {
        let msg = failures.join("\n  • ");
        bail!("FAILED ({} issue(s)):\n  • {msg}", failures.len())
    }
}

// ── subcommand: idempotency ────────────────────────────────────────────────────

fn run_download_subprocess() -> Result<String> {
    // Call ourselves — same pattern as Python's sys.executable.
    // We pass no extra flags so env vars from mise are inherited automatically.
    let exe = std::env::current_exe().context("finding current executable")?;
    let output = Command::new(&exe)
        .args(["fonts", "download"])
        .output()
        .context("running fonts download subprocess")?;
    if !output.status.success() {
        bail!(
            "download subprocess failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn assert_output(label: &str, output: &str, expected: &str, failures: &mut Vec<String>) {
    if output.contains(expected) {
        println!("  PASS: {label}");
    } else {
        println!("  FAIL: {label}");
        println!("        expected '{expected}' in output");
        println!("        got: {:?}", output.trim());
        failures.push(label.to_string());
    }
}

/// RAII guard that restores the .done stamp and optionally one font file on drop.
/// Owns all paths and data so it can restore state even if the surrounding scope panics.
struct IdempotencyGuard {
    done_path: std::path::PathBuf,
    original_stamp: String,
    victim: Option<(std::path::PathBuf, Vec<u8>)>,
}
impl Drop for IdempotencyGuard {
    fn drop(&mut self) {
        let _ = vfs::write(&self.done_path, self.original_stamp.as_bytes());
        if let Some((path, data)) = &self.victim {
            if !vfs::exists(path) {
                let _ = vfs::write(path, data.as_slice());
            }
        }
    }
}

pub fn cmd_idempotency(cfg: &Config) -> Result<()> {
    let families = parse_families(cfg)?;
    let done = cfg.done_file();
    let mut failures: Vec<String> = Vec::new();

    println!("Setup: ensuring fonts are downloaded...");
    run_download_subprocess()?;
    let original_stamp = vfs::read_to_string(&done)?;

    // Guard restores stamp (and victim file) on any exit path — panic, error, or Ctrl+C.
    let mut guard = IdempotencyGuard {
        done_path: done.clone(),
        original_stamp: original_stamp.clone(),
        victim: None,
    };

    // Layer 2a: everything valid → skip with no network calls
    println!("\nLayer 2 — hash match, all files present → skip");
    let out = run_download_subprocess()?;
    assert_output("skips with 'up to date' message", &out, "up to date", &mut failures);
    if out.contains("↓ https") {
        println!("  FAIL: made unexpected network calls");
        failures.push("Layer 2 made network calls when stamp+files valid".to_string());
    } else {
        println!("  PASS: no network calls made");
    }

    // Layer 2b: bad stamp, files intact → re-runs but downloads nothing
    println!("\nLayer 2 — hash mismatch → runs but no downloads");
    vfs::write(&done, b"badhash\n")?;
    let out = run_download_subprocess()?;
    vfs::write(&done, original_stamp.as_bytes())?; // restore early (guard also does this on drop)
    assert_output("queries GWFH (hash mismatch triggers run)", &out, "Querying GWFH", &mut failures);
    assert_output("skips all files (already exist)", &out, "already exists, skipping", &mut failures);
    assert_output("zero downloads", &out, "0 downloaded", &mut failures);

    // Layer 3: bad stamp + one file deleted → downloads exactly that file
    println!("\nLayer 3 — one file deleted → downloads only that file");
    let victim = expected_files(cfg, &families).into_iter().next()
        .context("no font files configured")?;
    let victim_data = vfs::read_bytes(&victim)?;
    let victim_name = victim.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| victim.display().to_string());
    // Register victim in guard BEFORE deleting so drop() can restore it
    guard.victim = Some((victim.clone(), victim_data.clone()));
    vfs::remove_file(&victim)?;
    vfs::write(&done, b"badhash\n")?;
    let out = run_download_subprocess()?;
    // Restore explicitly — guard is a safety net, not the primary path
    vfs::write(&victim, &victim_data)?;
    vfs::write(&done, original_stamp.as_bytes())?;
    assert_output(&format!("downloads missing {victim_name}"), &out, &victim_name, &mut failures);
    assert_output("only 1 downloaded", &out, "1 downloaded", &mut failures);

    // Confirm system still healthy
    println!("\nPost-test: verifying system still healthy");
    let out = run_download_subprocess()?;
    assert_output("back to up-to-date", &out, "up to date", &mut failures);

    println!();
    if failures.is_empty() {
        println!("All idempotency checks passed.");
        Ok(())
    } else {
        let msg = failures.join("\n  • ");
        bail!("FAILED ({} issue(s)):\n  • {msg}", failures.len())
    }
}

// ── subcommand: search ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct GwfhSearchResult {
    id: String,
    family: String,
    variants: Vec<serde_json::Value>, // strings ("regular","700") in search results
    subsets: Vec<String>,
}

pub fn cmd_search(cfg: &Config, query: &str) -> Result<()> {
    if query.is_empty() {
        bail!("Usage: quick-tool fonts search <font name>");
    }
    let url = format!("{}?search={}", cfg.gwfh_api, urlencoding::encode(query));
    println!("Searching GWFH registry for: '{query}'");
    println!("API: {url}\n");
    let results: Vec<GwfhSearchResult> = http::get_json(&url)?;

    let query_words: Vec<&str> = query.split_whitespace().collect();
    let mut matches: Vec<&GwfhSearchResult> = results
        .iter()
        .filter(|f| {
            let lower = f.family.to_lowercase();
            query_words.iter().all(|w| lower.contains(w))
        })
        .collect();

    if matches.is_empty() {
        matches = results.iter().take(5).collect();
        if matches.is_empty() {
            bail!("No results found for '{query}'.");
        }
        println!("No exact matches for '{query}'. Closest results:");
    } else {
        println!("Found {} match(es):", matches.len());
    }

    println!();
    for font in matches.iter().take(5) {
        let mut weights: Vec<u32> = font
            .variants
            .iter()
            .filter_map(|v| {
                let s = v.as_str()?;
                if s == "regular" {
                    Some(400)
                } else {
                    s.parse::<u32>().ok()
                }
            })
            .collect();
        weights.sort_unstable();
        weights.dedup();

        println!("  family:  {}", font.family);
        println!("  gwfh_id: {}", font.id);
        println!("  weights: {weights:?}");
        println!("  subsets: {:?}", font.subsets);
        println!();
        println!("  Add to {} font stack:", cfg.resolved_theme_file().display());
        println!("  ─────────────────────────────────────");
        println!(r#"  font: (..., "{}"),"#, font.family);
        println!();
    }
    Ok(())
}
