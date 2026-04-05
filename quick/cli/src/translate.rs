/// Markdown translation — EN spec files → Thai via Claude.
///
/// ## Backend selection (at runtime, lazy — only when a file actually needs translating)
///
/// 1. **API** (`TranslateBackend::Api`) — uses Claude Messages API over HTTPS.
///    Selected when `ANTHROPIC_API_KEY` (or `--anthropic-api-key`) is set.
///    Works on all platforms including Cloudflare Workers.
///
/// 2. **CLI** (`TranslateBackend::Cli`) — spawns the `claude` subprocess.
///    Selected when no API key is present and the `local` cargo feature is enabled.
///    Not available on Cloudflare (WASM has no process execution).
///
/// ## Idempotency
///
/// Files are skipped when their SHA-256 hash matches the stored `.th.md.hash` stamp.
/// The claude CLI / API is only invoked when a file actually changed.
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::{http, idempotency, vfs};

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

// ── Claude Messages API types ──────────────────────────────────────────────────

#[derive(Serialize)]
struct ApiRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: Vec<ApiMessage<'a>>,
}

#[derive(Serialize)]
struct ApiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ApiResponse {
    content: Vec<ApiContent>,
}

#[derive(Deserialize)]
struct ApiContent {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

// ── Translation backends ───────────────────────────────────────────────────────

enum TranslateBackend {
    /// REST call to api.anthropic.com/v1/messages — works everywhere.
    Api { api_key: String, model: String },
    /// Subprocess `claude -p "..."` — local desktop only.
    #[cfg(feature = "local")]
    Cli(PathBuf),
}

impl TranslateBackend {
    /// Resolve which backend to use. Only called on first file that actually needs translating.
    fn resolve(cfg: &crate::Config) -> Result<Self> {
        // API key present → always use the REST API.
        if let Some(key) = cfg.anthropic_api_key.clone() {
            return Ok(TranslateBackend::Api {
                api_key: key,
                model: cfg.claude_model.clone(),
            });
        }

        // No API key — fall back to CLI if we're on a local build.
        #[cfg(feature = "local")]
        {
            return Ok(TranslateBackend::Cli(find_claude()?));
        }

        // Cloudflare / no-local build with no API key → hard error.
        #[cfg(not(feature = "local"))]
        bail!(
            "ANTHROPIC_API_KEY is required for translation (no local CLI available on this platform). \
             Set ANTHROPIC_API_KEY in your environment or mise.local.toml."
        );
    }

    fn translate(&self, content: &str) -> Result<String> {
        match self {
            TranslateBackend::Api { api_key, model } => {
                call_claude_api(api_key, model, content)
            }
            #[cfg(feature = "local")]
            TranslateBackend::Cli(path) => call_claude_cli(path, content),
        }
    }
}

// ── API backend ────────────────────────────────────────────────────────────────

fn call_claude_api(api_key: &str, model: &str, content: &str) -> Result<String> {
    const API_URL: &str = "https://api.anthropic.com/v1/messages";

    let prompt = format!("Translate this construction spec to Thai:\n\n{content}");
    let body = ApiRequest {
        model,
        max_tokens: 8096,
        system: SYSTEM_PROMPT,
        messages: vec![ApiMessage { role: "user", content: &prompt }],
    };
    let headers = [
        ("x-api-key", api_key),
        ("anthropic-version", "2023-06-01"),
    ];

    let resp: ApiResponse = http::post_json(API_URL, &headers, &body)?;

    let text = resp
        .content
        .into_iter()
        .find(|c| c.kind == "text")
        .and_then(|c| c.text)
        .context("Claude API returned no text content block")?;

    Ok(clean_output(text))
}

// ── CLI backend (local-only) ───────────────────────────────────────────────────

#[cfg(feature = "local")]
fn call_claude_cli(claude: &Path, content: &str) -> Result<String> {
    use std::process::Command;

    let prompt = format!("{SYSTEM_PROMPT}\nTranslate this construction spec to Thai:\n\n{content}");
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

    let text = String::from_utf8(output.stdout)
        .context("claude output was not valid UTF-8")?;
    Ok(clean_output(text))
}

/// Strip surrounding whitespace and markdown code fences if Claude wrapped the output.
fn clean_output(raw: String) -> String {
    let trimmed = raw.trim().to_string();
    if trimmed.starts_with("```") {
        let lines: Vec<&str> = trimmed.lines().collect();
        if lines.last().map(|l| l.trim()) == Some("```") {
            return lines[1..lines.len() - 1].join("\n");
        }
    }
    trimmed
}

// ── CLI discovery (local-only) ─────────────────────────────────────────────────

#[cfg(feature = "local")]
fn find_claude() -> Result<PathBuf> {
    // 1. PATH — covers most installs (homebrew, cargo, mise, npm global, etc.)
    if let Ok(path) = which::which("claude") {
        return Ok(path);
    }
    // 2. VSCode extension fallback
    if let Some(path) = vscode_claude() {
        return Ok(path);
    }
    bail!("claude CLI not found in PATH or VSCode extensions. Install Claude Code or set ANTHROPIC_API_KEY.")
}

#[cfg(all(feature = "local", windows))]
fn vscode_claude() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    let ext_dir = PathBuf::from(appdata).join("Code").join("extensions");
    claude_in_extensions(&ext_dir, "claude.exe")
}

#[cfg(all(feature = "local", not(windows)))]
fn vscode_claude() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let ext_dir = home.join(".vscode").join("extensions");
    claude_in_extensions(&ext_dir, "claude")
}

#[cfg(feature = "local")]
fn claude_in_extensions(ext_dir: &Path, binary: &str) -> Option<PathBuf> {
    let pattern = ext_dir
        .join("anthropic.claude-code-*")
        .join("resources")
        .join("native-binary")
        .join(binary);
    let pattern_str = pattern.to_str()?;
    vfs::glob(pattern_str)
        .ok()?
        .into_iter()
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

// ── per-file translation ───────────────────────────────────────────────────────

/// Returns `true` if the file was translated, `false` if skipped (unchanged).
///
/// `backend` is lazily resolved: `None` means not yet initialised.
/// The backend (and therefore claude CLI / API key) is only required when at
/// least one file actually needs translating — so CI with committed hash stamps
/// works without ANTHROPIC_API_KEY or claude installed.
fn translate_file(backend: &mut Option<TranslateBackend>, cfg: &crate::Config, input: &Path) -> Result<bool> {
    let name = input
        .file_stem()
        .and_then(|s| s.to_str())
        .context("invalid filename")?;
    let output = input.with_file_name(format!("{name}.th.md"));
    let hash_file = input.with_file_name(format!("{name}.th.md.hash"));

    let content = vfs::read_to_string(input)?;
    let current_hash = idempotency::sha256_hex(content.as_bytes());

    // Skip if unchanged since last translation — no backend needed
    if vfs::exists(&output) && vfs::exists(&hash_file) {
        let stored = vfs::read_to_string(&hash_file)?;
        if stored.trim() == current_hash {
            println!("  skip {} (unchanged)", input.display());
            return Ok(false);
        }
    }

    // Lazy-resolve backend on first file that actually needs translating
    if backend.is_none() {
        *backend = Some(TranslateBackend::resolve(cfg)?);
    }
    let b = backend.as_ref().expect("backend is Some: set just above");

    println!("  translating {} → {} ...", input.display(), output.display());
    let result = b.translate(&content)?;

    vfs::write(&output, result.as_bytes())?;
    vfs::write(&hash_file, current_hash.as_bytes())?;
    Ok(true)
}

// ── subcommand: translate ──────────────────────────────────────────────────────

pub fn cmd_translate(cfg: &crate::Config, files: Vec<PathBuf>) -> Result<()> {
    let mut backend: Option<TranslateBackend> = None;
    let mut translated = 0u32;
    let mut skipped = 0u32;

    if files.is_empty() {
        // No args → translate all [A-Z]*.md in specs_dir
        let pattern = cfg.specs_dir.join("[A-Z]*.md");
        let pattern_str = pattern.to_str().context("specs_dir path contains non-UTF-8")?;
        for path in vfs::glob(pattern_str)? {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or_default();
            if name.ends_with(".th.md") {
                continue;
            }
            if translate_file(&mut backend, cfg, &path)? {
                translated += 1;
            } else {
                skipped += 1;
            }
        }
    } else {
        // Specific files requested (e.g. from `mise run one -- GATE`)
        for path in &files {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or_default();
            if name.ends_with(".th.md") {
                continue;
            }
            if translate_file(&mut backend, cfg, path)? {
                translated += 1;
            } else {
                skipped += 1;
            }
        }
    }

    println!("Done. {translated} translated, {skipped} skipped.");
    Ok(())
}
