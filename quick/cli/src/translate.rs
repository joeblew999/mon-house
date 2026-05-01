/// Markdown translation — EN spec files → Thai.
///
/// This is the **local fallback** backend used by `mise run translate` and CI.
/// The primary path (file watch → Agent → Workers AI) is handled by the
/// TypeScript watcher in `quick/local/src/watch.ts` via AgentClient.
///
/// ## Backend selection (lazy — only when a file actually needs translating)
///
/// 1. **Worker** (`QUICK_TRANSLATE_URL` set) — POSTs to a CF Worker endpoint.
///    No API key needed — uses CF Workers AI. Fastest for CI.
///
/// 2. **API** (`ANTHROPIC_API_KEY` set) — Claude Messages API over HTTPS.
///
/// 3. **CLI** — spawns the `claude` subprocess (local-only, no key needed).
///
/// ## Idempotency
///
/// Files are skipped when their BLAKE3 hash matches the stored `.th.md.hash`.
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{http, idempotency, vfs};

/// Shared system prompt — keep in sync with cf/src/prompt.ts.
pub const SYSTEM_PROMPT: &str = r#"You are a professional translator specialising in Thai construction and renovation documents.

Rules:
- Translate ALL English text to Thai
- Keep ALL numbers, measurements, prices, SKUs, and URLs exactly as-is
- Keep ALL markdown formatting (headings, tables, bold, links) exactly as-is
- Keep table structure identical — only translate the text inside cells
- Use correct Thai construction terminology (not word-for-word literal translation)
- Formal register (ภาษาทางการ) appropriate for contractor documents
- Do NOT add explanations or notes — output ONLY the translated markdown
"#;

// ── Wire types (shared with CF Worker via ts-rs) ───────────────────────────────

/// Request body for `POST /translate`.
#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "TranslateRequest.ts")]
pub struct TranslateRequest {
    pub content: String,
}

/// Success response from `POST /translate`.
#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "TranslateResponse.ts")]
pub struct TranslateResponse {
    pub thai: String,
}

/// Error response (4xx / 5xx).
#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "ErrorResponse.ts")]
pub struct ErrorResponse {
    pub error: String,
}

// ── Claude Messages API types ──────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ApiRequest<'a> {
    pub model: &'a str,
    pub max_tokens: u32,
    pub system: &'a str,
    pub messages: Vec<ApiMessage<'a>>,
}

#[derive(Serialize)]
pub struct ApiMessage<'a> {
    pub role: &'a str,
    pub content: &'a str,
}

#[derive(Deserialize)]
pub struct ApiResponse {
    pub content: Vec<ApiContent>,
}

#[derive(Deserialize)]
pub struct ApiContent {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: Option<String>,
}

// ── Backend selection ──────────────────────────────────────────────────────────

enum TranslateBackend {
    /// POST to a CF Worker endpoint — no API key, uses CF Workers AI.
    Worker { url: String },
    /// REST call to api.anthropic.com/v1/messages.
    Api { api_key: String, model: String },
    /// Subprocess `claude -p "..."` — local desktop only.
    #[cfg(feature = "local")]
    Cli(PathBuf),
}

impl TranslateBackend {
    fn resolve(cfg: &crate::Config) -> Result<Self> {
        if let Some(url) = cfg.translate_url.clone() {
            return Ok(TranslateBackend::Worker { url });
        }
        if let Some(key) = cfg.anthropic_api_key.clone() {
            return Ok(TranslateBackend::Api {
                api_key: key,
                model: cfg.claude_model.clone(),
            });
        }
        #[cfg(feature = "local")]
        {
            return Ok(TranslateBackend::Cli(find_claude()?));
        }
        #[cfg(not(feature = "local"))]
        bail!(
            "No translation backend. Set QUICK_TRANSLATE_URL or ANTHROPIC_API_KEY."
        );
    }

    fn translate(&self, content: &str) -> Result<String> {
        match self {
            TranslateBackend::Worker { url } => call_worker(url, content),
            TranslateBackend::Api { api_key, model } => call_claude_api(api_key, model, content),
            #[cfg(feature = "local")]
            TranslateBackend::Cli(path) => call_claude_cli(path, content),
        }
    }
}

// ── Worker HTTP backend ────────────────────────────────────────────────────────

#[derive(Serialize)]
struct WorkerRequest<'a> { content: &'a str }

#[derive(Deserialize)]
struct WorkerResponse { thai: String }

fn call_worker(url: &str, content: &str) -> Result<String> {
    let resp: WorkerResponse = http::post_json(url, &[], &WorkerRequest { content })?;
    Ok(clean_output(resp.thai))
}

// ── Claude API backend ─────────────────────────────────────────────────────────

fn call_claude_api(api_key: &str, model: &str, content: &str) -> Result<String> {
    const API_URL: &str = "https://api.anthropic.com/v1/messages";
    let prompt = format!("Translate this construction spec to Thai:\n\n{content}");
    let body = ApiRequest {
        model,
        max_tokens: 8096,
        system: SYSTEM_PROMPT,
        messages: vec![ApiMessage { role: "user", content: &prompt }],
    };
    let headers = [("x-api-key", api_key), ("anthropic-version", "2023-06-01")];
    let resp: ApiResponse = http::post_json(API_URL, &headers, &body)?;
    let text = resp.content.into_iter()
        .find(|c| c.kind == "text")
        .and_then(|c| c.text)
        .context("Claude API returned no text content")?;
    Ok(clean_output(text))
}

// ── Claude CLI backend (local only) ───────────────────────────────────────────

#[cfg(feature = "local")]
fn call_claude_cli(claude: &Path, content: &str) -> Result<String> {
    use std::process::Command;
    let prompt = format!("{SYSTEM_PROMPT}\nTranslate this construction spec to Thai:\n\n{content}");
    let output = Command::new(claude)
        .args(["-p", &prompt, "--output-format", "text"])
        .output()
        .context("running claude CLI")?;
    if !output.status.success() {
        bail!("claude CLI error:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(clean_output(String::from_utf8(output.stdout).context("claude output not UTF-8")?))
}

#[cfg(feature = "local")]
fn find_claude() -> Result<PathBuf> {
    if let Ok(path) = which::which("claude") { return Ok(path); }
    if let Some(path) = vscode_claude() { return Ok(path); }
    bail!("claude CLI not found. Install Claude Code or set ANTHROPIC_API_KEY.")
}

#[cfg(all(feature = "local", windows))]
fn vscode_claude() -> Option<PathBuf> {
    let ext_dir = PathBuf::from(std::env::var("APPDATA").ok()?).join("Code").join("extensions");
    claude_in_extensions(&ext_dir, "claude.exe")
}

#[cfg(all(feature = "local", not(windows)))]
fn vscode_claude() -> Option<PathBuf> {
    let ext_dir = dirs::home_dir()?.join(".vscode").join("extensions");
    claude_in_extensions(&ext_dir, "claude")
}

#[cfg(feature = "local")]
fn claude_in_extensions(ext_dir: &Path, binary: &str) -> Option<PathBuf> {
    let pattern = ext_dir.join("anthropic.claude-code-*").join("resources").join("native-binary").join(binary);
    vfs::glob(pattern.to_str()?).ok()?.into_iter().find(|p| {
        #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; p.metadata().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false) }
        #[cfg(windows)] { p.exists() }
    })
}

// ── Output cleanup ─────────────────────────────────────────────────────────────

pub fn clean_output(raw: String) -> String {
    let trimmed = raw.trim().to_string();
    if trimmed.starts_with("```") {
        let lines: Vec<&str> = trimmed.lines().collect();
        if lines.last().map(|l| l.trim()) == Some("```") {
            return lines[1..lines.len() - 1].join("\n");
        }
    }
    trimmed
}

// ── Per-file translation ───────────────────────────────────────────────────────

fn translate_file(backend: &mut Option<TranslateBackend>, cfg: &crate::Config, input: &Path) -> Result<bool> {
    let name = input.file_stem().and_then(|s| s.to_str()).context("invalid filename")?;
    let output    = input.with_file_name(format!("{name}.th.md"));
    let hash_file = input.with_file_name(format!("{name}.th.md.hash"));
    let content   = vfs::read_to_string(input)?;
    let current_hash = idempotency::blake3_hex(content.as_bytes());

    if vfs::exists(&output) && vfs::exists(&hash_file) {
        if vfs::read_to_string(&hash_file)?.trim() == current_hash {
            println!("  skip {} (unchanged)", input.display());
            return Ok(false);
        }
    }

    if backend.is_none() { *backend = Some(TranslateBackend::resolve(cfg)?); }
    let b = backend.as_ref().expect("set above");

    println!("  translating {} → {} ...", input.display(), output.display());
    let result = b.translate(&content)?;
    vfs::write(&output, result.as_bytes())?;
    vfs::write(&hash_file, current_hash.as_bytes())?;
    Ok(true)
}

// ── Subcommand ─────────────────────────────────────────────────────────────────

pub fn cmd_translate(cfg: &crate::Config, files: Vec<PathBuf>) -> Result<()> {
    let paths: Vec<PathBuf> = if files.is_empty() {
        let pattern = cfg.specs_dir.join("[A-Z]*.md");
        vfs::glob(pattern.to_str().context("specs_dir path not UTF-8")?)?
            .into_iter()
            .filter(|p| !p.file_name().and_then(|n| n.to_str()).unwrap_or_default().ends_with(".th.md"))
            .collect()
    } else {
        files.into_iter()
            .filter(|p| !p.file_name().and_then(|n| n.to_str()).unwrap_or_default().ends_with(".th.md"))
            .collect()
    };

    let mut backend: Option<TranslateBackend> = None;
    let mut translated = 0u32;
    let mut skipped    = 0u32;
    for path in &paths {
        if translate_file(&mut backend, cfg, path)? { translated += 1; } else { skipped += 1; }
    }
    println!("Done. {translated} translated, {skipped} skipped.");
    Ok(())
}
