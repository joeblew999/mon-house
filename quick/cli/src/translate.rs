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
/// Section-granular: each `## ` section's BLAKE3 hash is the cache key in
/// `<stem>.th.md.cache.json`. Cached chunks skip the API call; only sections
/// whose source content changed are re-translated. See `translate_file` and
/// `chunks::split` for details.
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{http, idempotency, vfs};

/// Shared spec prompt — keep in sync with cf/src/prompt.ts SYSTEM_PROMPT.
pub const SYSTEM_PROMPT: &str = r#"You are a professional translator specialising in Thai construction and renovation documents.

Rules:
- Translate ALL English text to Thai
- Keep ALL numbers, measurements, prices, SKUs, and URLs exactly as-is
- Keep ALL markdown formatting (headings, tables, bold, links) exactly as-is
- Keep table structure identical — only translate the text inside cells
- Use correct Thai construction terminology (not word-for-word literal translation)
- Formal register (ภาษาทางการ) appropriate for contractor documents
- Preserve HTML comments (<!-- ... -->) exactly as-is — do not translate, expand, or remove them
- Do NOT add explanations or notes — output ONLY the translated markdown
"#;

/// Single short label prompt — for SVG <text> elements. Keep in sync with cf/src/prompt.ts LABEL_PROMPT.
pub const LABEL_PROMPT: &str = r#"You translate single short labels from English to Thai. The labels are used in technical floor plans and construction drawings.

Rules:
- Output ONLY the Thai translation, on a single line
- Keep ALL numbers, dimensions, units (mm, m², etc.) exactly as-is
- Use construction terminology
- NO markdown, NO tables, NO headings, NO bullet points, NO explanations
- NO quotation marks, NO labels like "Translation:"
- If the input has no English words (numbers / units only), output it unchanged
"#;

/// Translation modes — `spec` for full markdown specs, `label` for single short SVG labels.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TranslateMode {
    Spec,
    Label,
}

impl Default for TranslateMode {
    fn default() -> Self { TranslateMode::Spec }
}

fn system_prompt_for(mode: TranslateMode) -> &'static str {
    match mode {
        TranslateMode::Spec => SYSTEM_PROMPT,
        TranslateMode::Label => LABEL_PROMPT,
    }
}

fn user_prompt_for(mode: TranslateMode, content: &str) -> String {
    match mode {
        TranslateMode::Spec => format!("Translate this construction spec to Thai:\n\n{content}"),
        TranslateMode::Label => format!("Translate this label to Thai: {content}"),
    }
}

// ── Wire types (shared with CF Worker via ts-rs) ───────────────────────────────

/// Request body for `POST /translate`.
#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "TranslateRequest.ts")]
pub struct TranslateRequest {
    pub content: String,
    /// Optional — `"spec"` (default) for markdown specs, `"label"` for SVG labels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
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
        self.translate_with(content, TranslateMode::Spec)
    }

    fn translate_with(&self, content: &str, mode: TranslateMode) -> Result<String> {
        match self {
            TranslateBackend::Worker { url } => call_worker(url, content, mode),
            TranslateBackend::Api { api_key, model } => call_claude_api(api_key, model, content, mode),
            #[cfg(feature = "local")]
            TranslateBackend::Cli(path) => call_claude_cli(path, content, mode),
        }
    }
}

// ── Worker HTTP backend ────────────────────────────────────────────────────────

#[derive(Serialize)]
struct WorkerRequest<'a> {
    content: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<&'a str>,
}

#[derive(Deserialize)]
struct WorkerResponse { thai: String }

fn call_worker(url: &str, content: &str, mode: TranslateMode) -> Result<String> {
    let mode_str = match mode { TranslateMode::Spec => None, TranslateMode::Label => Some("label") };
    let resp: WorkerResponse = http::post_json(url, &[], &WorkerRequest { content, mode: mode_str })?;
    Ok(clean_output(resp.thai))
}

// ── Claude API backend ─────────────────────────────────────────────────────────

fn call_claude_api(api_key: &str, model: &str, content: &str, mode: TranslateMode) -> Result<String> {
    const API_URL: &str = "https://api.anthropic.com/v1/messages";
    let prompt = user_prompt_for(mode, content);
    let max_tokens = match mode { TranslateMode::Label => 256, TranslateMode::Spec => 16384 };
    let body = ApiRequest {
        model,
        max_tokens,
        system: system_prompt_for(mode),
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
fn call_claude_cli(claude: &Path, content: &str, mode: TranslateMode) -> Result<String> {
    use std::process::Command;
    let prompt = format!(
        "{system}\n{user}",
        system = system_prompt_for(mode),
        user = user_prompt_for(mode, content),
    );
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
    if !trimmed.starts_with("```") {
        return trimmed;
    }
    let lines: Vec<&str> = trimmed.lines().collect();
    let last_idx = if lines.last().map(|l| l.trim()) == Some("```") {
        lines.len() - 1 // drop the closing fence
    } else {
        lines.len()     // truncated response — no closing fence, keep all remaining lines
    };
    lines[1..last_idx].join("\n")
}

// ── SVG translation ────────────────────────────────────────────────────────────

/// Translate the visible text inside `<text>...</text>` elements of an SVG.
///
/// The non-text parts of the SVG (geometry, attributes, comments) pass through unchanged,
/// so the diagram still scales / renders identically — only the labels switch language.
/// `font-family` attributes on each `<text>` are rewritten to `"Noto Sans Thai, Arial"` so
/// Thai characters fall back gracefully when the spec PDF is rendered.
pub fn translate_svg_content(backend: &TranslateBackend, content: &str) -> Result<String> {
    let text_re = regex::Regex::new(r#"<text([^>]*)>([^<]+)</text>"#)?;
    let font_re = regex::Regex::new(r#"font-family="[^"]*""#)?;

    let mut out = String::with_capacity(content.len());
    let mut last = 0;
    for cap in text_re.captures_iter(content) {
        let whole = cap.get(0).expect("whole match");
        let attrs = cap.get(1).expect("attrs").as_str();
        let inner = cap.get(2).expect("inner").as_str();
        out.push_str(&content[last..whole.start()]);

        let translated = if inner.trim().is_empty() {
            inner.to_string()
        } else {
            backend.translate_with(inner, TranslateMode::Label)?
        };
        let new_attrs = font_re
            .replace(attrs, r#"font-family="Noto Sans Thai, Arial""#)
            .into_owned();
        out.push_str(&format!("<text{new_attrs}>{translated}</text>"));

        last = whole.end();
    }
    out.push_str(&content[last..]);
    Ok(out)
}

fn translate_svg_file(
    backend: &mut Option<TranslateBackend>,
    cfg: &crate::Config,
    input: &Path,
) -> Result<bool> {
    let name = input.file_stem().and_then(|s| s.to_str()).context("invalid filename")?;
    let output    = input.with_file_name(format!("{name}.th.svg"));
    let hash_file = input.with_file_name(format!("{name}.th.svg.hash"));
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
    let result = translate_svg_content(b, &content)?;
    vfs::write(&output, result.as_bytes())?;
    vfs::write(&hash_file, current_hash.as_bytes())?;
    Ok(true)
}

// ── Image-ref + include substitution ───────────────────────────────────────────

/// In the translated markdown, swap any `![..](path/to/foo.svg|png|jpg|jpeg)` or
/// `<!-- include: path/to/foo.md -->` reference for the `.th.<ext>` Thai sibling
/// if it exists on disk. Lets a Thai PDF pick up both Thai-labelled diagrams and
/// translated partials automatically without the author touching `.th.md`.
///
/// The two base_dirs differ because the path conventions differ:
///   * `image_base` — image paths are resolved by typst at compile time relative
///     to the project root (where `_tmp.typ` lives), so this should be the
///     parent of `specs_dir`.
///   * `include_base` — include paths are relative to the markdown file that
///     contains them, the same convention `build::write_typ_wrapper` uses.
fn substitute_th_refs(content: &str, image_base: &Path, include_base: &Path) -> String {
    let after_images = if let Ok(re) = regex::Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)") {
        re.replace_all(content, |caps: &regex::Captures| {
            let alt = &caps[1];
            let path = &caps[2];
            let new_path = th_variant_if_exists(path, image_base, &["svg", "png", "jpg", "jpeg"]);
            format!("![{alt}]({new_path})")
        })
        .into_owned()
    } else {
        content.to_string()
    };

    if let Ok(re) = regex::Regex::new(r"(?m)^[ \t]*<!--[ \t]*include:[ \t]*([^\s>]+)[ \t]*-->[ \t]*$") {
        re.replace_all(&after_images, |caps: &regex::Captures| {
            let path = &caps[1];
            let new_path = th_variant_if_exists(path, include_base, &["md"]);
            format!("<!-- include: {new_path} -->")
        })
        .into_owned()
    } else {
        after_images
    }
}

fn th_variant_if_exists(path: &str, base_dir: &Path, allowed_exts: &[&str]) -> String {
    let p = Path::new(path);
    let Some(stem) = p.file_stem().and_then(|s| s.to_str()) else { return path.into() };
    let Some(ext) = p.extension().and_then(|s| s.to_str()) else { return path.into() };
    if !allowed_exts.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
        return path.into();
    }
    if stem.ends_with(".th") { return path.into(); } // already Thai variant
    let parent = p.parent().unwrap_or(Path::new(""));
    let th_rel = parent.join(format!("{stem}.th.{ext}"));
    let th_abs = base_dir.join(&th_rel);
    if vfs::exists(&th_abs) {
        th_rel.to_string_lossy().into_owned()
    } else {
        path.into()
    }
}

// ── Per-file translation ───────────────────────────────────────────────────────
//
// Translation is **section-granular**:
//
//   1. The source `.md` is split on `## ` heading boundaries (`chunks::split`).
//   2. Each chunk's BLAKE3 hash is looked up in `<stem>.th.md.cache.json`.
//   3. Hits → reuse the cached Thai chunk (no API call).
//   4. Misses → call the backend on just that chunk, store result in the
//      rebuilt cache.
//   5. Concat translated chunks → run image / include ref substitution → write
//      `<stem>.th.md`. Save the rebuilt cache (which prunes stale entries).
//
// Editing one paragraph in section "Plumbing tidy" therefore re-translates
// only that one section, while every other section reuses its cached Thai.

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct ChunkCache {
    #[serde(default = "default_cache_version")]
    version: u32,
    /// `blake3(en_chunk_bytes)` (hex) → translated Thai chunk text.
    chunks: std::collections::HashMap<String, String>,
}

fn default_cache_version() -> u32 { 1 }

fn load_cache(path: &Path) -> ChunkCache {
    if !vfs::exists(path) { return ChunkCache::default(); }
    match vfs::read_to_string(path) {
        Ok(text) => serde_json::from_str::<ChunkCache>(&text).unwrap_or_default(),
        Err(_)   => ChunkCache::default(),
    }
}

fn save_cache(path: &Path, cache: &ChunkCache) -> Result<()> {
    let json = serde_json::to_string_pretty(cache).context("serialise cache")?;
    vfs::write(path, json.as_bytes())
}

fn translate_file(backend: &mut Option<TranslateBackend>, cfg: &crate::Config, input: &Path) -> Result<bool> {
    let name = input.file_stem().and_then(|s| s.to_str()).context("invalid filename")?;
    let output     = input.with_file_name(format!("{name}.th.md"));
    let cache_path = input.with_file_name(format!("{name}.th.md.cache.json"));
    let old_hash_path = input.with_file_name(format!("{name}.th.md.hash"));

    let content = vfs::read_to_string(input)?;
    let en_chunks = crate::chunks::split(&content);
    let prev_cache = load_cache(&cache_path);

    let mut next_cache = ChunkCache { version: 1, chunks: std::collections::HashMap::new() };
    let mut th_chunks = Vec::with_capacity(en_chunks.len());
    let mut hits = 0u32;
    let mut misses = 0u32;

    for chunk in &en_chunks {
        let key = idempotency::blake3_hex(chunk.as_bytes());
        if let Some(cached) = prev_cache.chunks.get(&key) {
            th_chunks.push(cached.clone());
            next_cache.chunks.insert(key, cached.clone());
            hits += 1;
            continue;
        }
        if backend.is_none() { *backend = Some(TranslateBackend::resolve(cfg)?); }
        let b = backend.as_ref().expect("set above");
        let translated = b.translate(chunk)?;
        next_cache.chunks.insert(key, translated.clone());
        th_chunks.push(translated);
        misses += 1;
    }

    let translated_full = crate::chunks::join(&th_chunks);
    let image_base   = cfg.specs_dir.parent().unwrap_or(Path::new("."));
    let include_base = input.parent().unwrap_or(image_base);
    let result = substitute_th_refs(&translated_full, image_base, include_base);

    // Decide whether anything actually changed on disk. Re-writing identical
    // bytes is harmless but it does bump mtime, which would force the build
    // step to redo typst. So only write if needed.
    let prev_output = vfs::read_to_string(&output).unwrap_or_default();
    let did_write = if prev_output != result {
        vfs::write(&output, result.as_bytes())?;
        true
    } else {
        false
    };

    save_cache(&cache_path, &next_cache)?;

    // Migrate away from the old whole-file `.hash` sibling: delete it on first
    // run so future tooling only has to look at `.cache.json`.
    if vfs::exists(&old_hash_path) { let _ = std::fs::remove_file(&old_hash_path); }

    if misses == 0 && !did_write {
        println!("  skip {} (all {} chunks cached)", input.display(), hits);
        Ok(false)
    } else {
        println!(
            "  translated {} → {} ({} chunks: {} cached, {} via API)",
            input.display(),
            output.display(),
            hits + misses,
            hits,
            misses,
        );
        Ok(true)
    }
}

// ── Subcommand ─────────────────────────────────────────────────────────────────

pub fn cmd_translate(cfg: &crate::Config, files: Vec<PathBuf>) -> Result<()> {
    let primary_md_paths: Vec<PathBuf> = if files.is_empty() {
        let pattern = cfg.specs_dir.join("[A-Z]*.md");
        vfs::glob(pattern.to_str().context("specs_dir path not UTF-8")?)?
            .into_iter()
            .filter(|p| !p.file_name().and_then(|n| n.to_str()).unwrap_or_default().ends_with(".th.md"))
            .collect()
    } else {
        files.into_iter()
            .filter(|p| {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
                name.ends_with(".md") && !name.ends_with(".th.md")
            })
            .collect()
    };

    // Partials in `specs/_partials/` — always globbed regardless of file args, so
    // that includer specs find a translated `.th.md` partial during ref substitution.
    // Translated FIRST so the substitution step downstream sees fresh `.th.md` siblings.
    let partial_md_paths: Vec<PathBuf> = {
        let pattern = cfg.specs_dir.join("_partials/*.md");
        vfs::glob(pattern.to_str().context("partials path not UTF-8")?)
            .unwrap_or_default()
            .into_iter()
            .filter(|p| !p.file_name().and_then(|n| n.to_str()).unwrap_or_default().ends_with(".th.md"))
            .collect()
    };

    // SVG inputs — always glob (kept simple; explicit file args still translate matching .md).
    let svg_paths: Vec<PathBuf> = {
        let parent = cfg.specs_dir.parent().unwrap_or(Path::new("."));
        let pattern = parent.join("resources/images/*.svg");
        vfs::glob(pattern.to_str().context("images path not UTF-8")?)
            .unwrap_or_default()
            .into_iter()
            .filter(|p| {
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                !stem.ends_with(".th")
            })
            .collect()
    };

    let mut backend: Option<TranslateBackend> = None;
    let mut translated = 0u32;
    let mut skipped    = 0u32;
    // Order matters: SVGs first so their `.th.svg` exists for spec image-ref
    // substitution; partials second so their `.th.md` exists for spec
    // include-ref substitution; then the specs themselves.
    for path in &svg_paths {
        if translate_svg_file(&mut backend, cfg, path)? { translated += 1; } else { skipped += 1; }
    }
    for path in &partial_md_paths {
        if translate_file(&mut backend, cfg, path)? { translated += 1; } else { skipped += 1; }
    }
    for path in &primary_md_paths {
        if translate_file(&mut backend, cfg, path)? { translated += 1; } else { skipped += 1; }
    }
    println!("Done. {translated} translated, {skipped} skipped.");
    Ok(())
}
