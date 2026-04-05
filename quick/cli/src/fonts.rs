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
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::Config;

// ── shared helpers ─────────────────────────────────────────────────────────────

/// Hash the file that actually contains the font stack declaration.
///
/// For a plain theme file: hashes the file itself.
/// For an import-wrapper (scripts/theme.typ → themes/default.typ): hashes the
/// imported file so that `themes switch` — which only rewrites the wrapper —
/// does NOT invalidate the font stamp when the font stack hasn't changed.
fn theme_hash(cfg: &Config) -> Result<String> {
    let text = std::fs::read_to_string(&cfg.resolved_theme_file())
        .with_context(|| format!("reading {}", cfg.resolved_theme_file().display()))?;

    // If the wrapper imports another file that has the font stack, hash that file.
    let import_re = Regex::new(r#"#import\s+"([^"]+)""#)?;
    if let Some(caps) = import_re.captures(&text) {
        let import_path = &caps[1];
        if !import_path.starts_with('/') && !import_path.starts_with('\\') && !import_path.contains("..") {
            let theme = cfg.resolved_theme_file();
            let base = theme.parent().unwrap_or_else(|| std::path::Path::new("."));
            let imported = base.join(import_path);
            if let Ok(bytes) = std::fs::read(&imported) {
                // Check that the imported file actually contains the font stack
                if extract_font_stack(&String::from_utf8_lossy(&bytes)).is_some() {
                    let mut h = Sha256::new();
                    h.update(&bytes);
                    return Ok(hex::encode(h.finalize()));
                }
            }
        }
    }

    // Fall back to hashing the theme file itself
    let bytes = std::fs::read(&cfg.resolved_theme_file())
        .with_context(|| format!("reading {}", cfg.resolved_theme_file().display()))?;
    let mut h = Sha256::new();
    h.update(&bytes);
    Ok(hex::encode(h.finalize()))
}

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

pub fn parse_families(cfg: &Config) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(&cfg.resolved_theme_file())
        .with_context(|| format!("reading {}", cfg.resolved_theme_file().display()))?;

    // Direct font stack — plain theme file (legacy path or standalone theme)
    if let Some(f) = extract_font_stack(&text) {
        return Ok(f);
    }

    // Follow one level of `#import "path": *` — used by the import-wrapper theme.typ
    let import_re = Regex::new(r#"#import\s+"([^"]+)""#)?;
    if let Some(caps) = import_re.captures(&text) {
        let import_path = &caps[1];
        // Reject absolute paths and traversals — only follow safe relative imports
        if import_path.starts_with('/') || import_path.starts_with('\\') || import_path.contains("..") {
            anyhow::bail!(
                "refusing to follow unsafe import path '{}' in {}",
                import_path,
                cfg.resolved_theme_file().display()
            );
        }
        let theme = cfg.resolved_theme_file();
        let base = theme.parent().unwrap_or_else(|| std::path::Path::new("."));
        let imported = base.join(import_path);
        if let Ok(imported_text) = std::fs::read_to_string(&imported) {
            if let Some(f) = extract_font_stack(&imported_text) {
                return Ok(f);
            }
        }
    }

    anyhow::bail!(
        "no 'font: (...)' declaration found in {} or its imports",
        cfg.resolved_theme_file().display()
    )
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
    expected_files(cfg, families).iter().all(|p| p.exists())
}

fn http_get_bytes(url: &str) -> Result<Vec<u8>> {
    println!("    ↓ {url}");
    let resp = ureq::get(url)
        .set("User-Agent", "typst-font-manager/1.0")
        .call()
        .with_context(|| format!("GET {url}"))?;
    let mut bytes = Vec::new();
    resp.into_reader().read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn http_get_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T> {
    let resp = ureq::get(url)
        .set("User-Agent", "typst-font-manager/1.0")
        .call()
        .with_context(|| format!("GET {url}"))?;
    Ok(resp.into_json()?)
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
    let detail: GwfhFontDetail = http_get_json(&url)?;
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
    if !cfg.resolved_theme_file().exists() {
        bail!("{} not found. Run from the quick/ directory.", cfg.resolved_theme_file().display());
    }
    let families = parse_families(cfg)?;
    let current_hash = theme_hash(cfg)?;

    // Layer 2: skip if theme unchanged AND all files on disk
    let done = cfg.done_file();
    if done.exists()
        && std::fs::read_to_string(&done)?.trim() == current_hash
        && all_files_present(cfg, &families)
    {
        println!("✓ fonts up to date (theme.typ unchanged, all files present)");
        return Ok(());
    }

    std::fs::create_dir_all(&cfg.resolved_font_dir())?;
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
            if dest.exists() {
                // Layer 3: skip existing files
                println!("  ✓ {fname} (already exists, skipping)");
                total_skipped += 1;
            } else {
                let data = http_get_bytes(url)?;
                // Atomic write: write to .tmp then rename so a crash/disk-full
                // can never leave a silently-truncated font file behind.
                let tmp = dest.with_extension("tmp");
                std::fs::write(&tmp, &data)
                    .with_context(|| format!("writing {}", tmp.display()))?;
                std::fs::rename(&tmp, &dest)
                    .with_context(|| format!("renaming {} → {}", tmp.display(), dest.display()))?;
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
        std::fs::write(done, format!("{current_hash}\n"))?;
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
    if !cfg.resolved_theme_file().exists() {
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
        if path.exists() {
            println!("  PASS: {name}");
        } else {
            let msg = format!("missing: {}", path.display());
            println!("  FAIL: {name} missing");
            failures.push(msg);
        }
    }

    println!("Check 3: typst recognises each font family");
    let typst_bin = which::which("typst").context("typst not found in PATH")?;
    let output = Command::new(typst_bin)
        .args(["fonts", "--font-path", cfg.resolved_font_dir().to_str().unwrap_or("fonts"), "--ignore-system-fonts"])
        .output()
        .context("running typst fonts")?;
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
    if !done.exists() {
        println!("  FAIL: {} missing — run `mise run fonts`", done.display());
        failures.push(format!("{} missing", done.display()));
    } else {
        let current = theme_hash(cfg)?;
        let stored = std::fs::read_to_string(&done)?;
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
        let _ = std::fs::write(&self.done_path, &self.original_stamp);
        if let Some((path, data)) = &self.victim {
            if !path.exists() {
                let _ = std::fs::write(path, data);
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
    let original_stamp = std::fs::read_to_string(&done)?;

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
    std::fs::write(&done, "badhash\n")?;
    let out = run_download_subprocess()?;
    std::fs::write(&done, &original_stamp)?; // restore early (guard also does this on drop)
    assert_output("queries GWFH (hash mismatch triggers run)", &out, "Querying GWFH", &mut failures);
    assert_output("skips all files (already exist)", &out, "already exists, skipping", &mut failures);
    assert_output("zero downloads", &out, "0 downloaded", &mut failures);

    // Layer 3: bad stamp + one file deleted → downloads exactly that file
    println!("\nLayer 3 — one file deleted → downloads only that file");
    let victim = expected_files(cfg, &families).into_iter().next()
        .context("no font files configured")?;
    let victim_data = std::fs::read(&victim)?;
    let victim_name = victim.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| victim.display().to_string());
    // Register victim in guard BEFORE deleting so drop() can restore it
    guard.victim = Some((victim.clone(), victim_data.clone()));
    std::fs::remove_file(&victim)?;
    std::fs::write(&done, "badhash\n")?;
    let out = run_download_subprocess()?;
    // Restore explicitly — guard is a safety net, not the primary path
    std::fs::write(&victim, &victim_data)?;
    std::fs::write(&done, &original_stamp)?;
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
    let results: Vec<GwfhSearchResult> = http_get_json(&url)?;

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
