/// Markdown translation — Rust port of scripts/translate_markdown.py
///
/// Translates all [A-Z]*.md spec files from English to Thai via the Claude CLI.
/// Idempotent: skips files whose SHA-256 hash matches the stored .hash file.
///
/// Claude is found via:
///   1. PATH lookup (covers `mise use cargo:...` installs, homebrew, etc.)
///   2. VSCode extension fallback (platform-specific paths)
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::vfs;


const SYSTEM_PROMPT: &str = r#"You are a professional translator specialising in Thai construction and renovation documents.

Rules:
- Translate ALL English text to Thai
- Keep ALL numbers, measurements, prices, SKUs, and URLs exactly as-is
- Keep ALL markdown formatting (headings, tables, bold, links) exactly as-is
- Keep table structure identical — only translate the text inside cells
- Use correct Thai construction terminology (not word-for-word literal translation)
- Formal register (ภาษาทางการ) appropriate for contractor documents
- Do NOT add explanations or notes — output ONLY the translated markdown
"#;

// ── finding the Claude binary ──────────────────────────────────────────────────

/// Look for claude in VSCode extension installs.
/// Path structure differs by OS; `which` already covered PATH lookup above.
#[cfg(windows)]
fn vscode_claude() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    let ext_dir = PathBuf::from(appdata).join("Code").join("extensions");
    claude_in_extensions(&ext_dir, "claude.exe")
}

#[cfg(not(windows))]
fn vscode_claude() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let ext_dir = home.join(".vscode").join("extensions");
    claude_in_extensions(&ext_dir, "claude")
}

fn claude_in_extensions(ext_dir: &Path, binary: &str) -> Option<PathBuf> {
    let pattern = ext_dir
        .join("anthropic.claude-code-*")
        .join("resources")
        .join("native-binary")
        .join(binary);
    let pattern_str = pattern.to_str()?;
    // glob handles the * wildcard in the extension version directory
    glob::glob(pattern_str)
        .ok()?
        .filter_map(|e| e.ok())
        .find(|p| {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                p.metadata().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
            }
            #[cfg(windows)]
            {
                p.exists()
            }
        })
}

fn find_claude() -> Result<PathBuf> {
    // 1. PATH — covers most installs (homebrew, cargo, mise, npm global, etc.)
    if let Ok(path) = which::which("claude") {
        return Ok(path);
    }
    // 2. VSCode extension fallback
    if let Some(path) = vscode_claude() {
        return Ok(path);
    }
    bail!("claude CLI not found in PATH or VSCode extensions. Install Claude Code.")
}

// ── translation ────────────────────────────────────────────────────────────────

fn translate(claude: &Path, content: &str) -> Result<String> {
    let prompt = format!(
        "{SYSTEM_PROMPT}\nTranslate this construction spec to Thai:\n\n{content}"
    );
    let output = Command::new(claude)
        .args(["-p", &prompt, "--output-format", "text"])
        .output()
        .context("running claude CLI")?;

    if !output.status.success() {
        bail!(
            "claude CLI error:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let mut result = String::from_utf8(output.stdout)
        .context("claude output was not valid UTF-8")?;
    result = result.trim().to_string();

    // Strip markdown code fence if Claude wrapped output in ```markdown ... ```
    if result.starts_with("```") {
        let lines: Vec<&str> = result.lines().collect();
        if lines.last().map(|l| l.trim()) == Some("```") {
            result = lines[1..lines.len() - 1].join("\n");
        }
    }
    Ok(result)
}

/// Returns true if the file was translated, false if skipped (unchanged).
///
/// `claude` is a lazily-resolved cache: None means not yet looked up.
/// find_claude() is only called when a file actually needs translating,
/// so running in CI with all hashes up-to-date works without claude installed.
fn translate_file(claude: &mut Option<PathBuf>, input: &Path) -> Result<bool> {
    let name = input
        .file_stem()
        .and_then(|s| s.to_str())
        .context("invalid filename")?;
    let output = input.with_file_name(format!("{name}.th.md"));
    let hash_file = input.with_file_name(format!("{name}.th.md.hash"));

    let content = vfs::read_to_string(input)?;
    let current_hash = crate::idempotency::sha256_hex(content.as_bytes());

    // Skip if unchanged since last translation — no claude needed
    if vfs::exists(&output) && vfs::exists(&hash_file) {
        let stored = vfs::read_to_string(&hash_file)?;
        if stored.trim() == current_hash {
            println!("  skip {} (unchanged)", input.display());
            return Ok(false);
        }
    }

    // Only resolve claude when we actually need to translate
    if claude.is_none() {
        *claude = Some(find_claude()?);
    }
    // Safe: we just set it in the if-block above; ? would have returned on error
    let claude_path = claude.as_ref().expect("claude is Some: set just above");

    println!("  translating {} → {} ...", input.display(), output.display());
    let result = translate(claude_path, &content)?;

    vfs::write(&output, result.as_bytes())?;
    vfs::write(&hash_file, current_hash.as_bytes())?;
    Ok(true)
}

// ── subcommand: translate ──────────────────────────────────────────────────────

pub fn cmd_translate(cfg: &crate::Config, files: Vec<std::path::PathBuf>) -> Result<()> {
    let mut claude: Option<PathBuf> = None; // resolved lazily, only if a file needs translating
    let mut translated = 0u32;
    let mut skipped = 0u32;

    if files.is_empty() {
        // No args → translate all [A-Z]*.md in specs_dir
        let pattern = cfg.specs_dir.join("[A-Z]*.md");
        let pattern_str = pattern.to_str().context("specs_dir path contains non-UTF-8")?;
        for path in vfs::glob(pattern_str)? {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or_default();
            if name.ends_with(".th.md") { continue; }
            if translate_file(&mut claude, &path)? { translated += 1; } else { skipped += 1; }
        }
    } else {
        // Specific files requested (e.g. from `mise run one -- GATE`)
        for path in &files {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or_default();
            if name.ends_with(".th.md") {
                continue;
            }
            if translate_file(&mut claude, path)? { translated += 1; } else { skipped += 1; }
        }
    }

    println!("Done. {translated} translated, {skipped} skipped.");
    Ok(())
}
