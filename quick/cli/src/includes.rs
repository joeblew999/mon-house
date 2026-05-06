// Markdown include expander.
//
// Looks for `<!-- include: relative/path.md -->` directives anywhere in the
// markdown source and replaces them inline with the included file's contents.
// Recurses through nested includes (capped at MAX_DEPTH so a typo can't loop).
//
// Used by `build::write_typ_wrapper` so SSOT partials (e.g. shared parts list
// across BATHROOM.md and BATHROOM-COMPACT.md) can live in one file.
//
// Translate keeps directives as raw text so the same Thai pipeline works:
// each partial translates once into its own `.th.md`, and translate's
// `substitute_th_refs` rewrites `<!-- include: foo.md -->` →
// `<!-- include: foo.th.md -->` for the Thai output.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::vfs;

const MAX_DEPTH: u32 = 5;

/// Match `<!-- include: PATH -->` on its own line. Whitespace tolerant.
/// Multi-line flag: `(?m)`. Anchor to a line start to avoid false matches in code blocks.
const RE: &str = r"(?m)^[ \t]*<!--[ \t]*include:[ \t]*(?P<path>[^\s>]+)[ \t]*-->[ \t]*$";

/// Recursively expand include directives in `content`, with paths resolved
/// relative to `base_dir`. The outermost call typically passes the spec's
/// parent directory.
///
/// Reads file contents through `vfs::read_to_string` (the OS filesystem on
/// native builds). For browser/in-memory contexts where you have file
/// content already loaded, use `expand_with_map` instead.
pub fn expand(content: &str, base_dir: &Path) -> Result<String> {
    expand_with_reader(content, base_dir, 0, &|p: &Path| {
        // Path-traversal guards apply only to filesystem reads — the in-memory
        // variant doesn't need them because it can only return values from a
        // pre-validated map of file content.
        let canon_base = base_dir.canonicalize().unwrap_or_else(|_| base_dir.to_path_buf());
        let canon_inc = p
            .canonicalize()
            .with_context(|| format!("include not found: {}", p.display()))?;
        if !canon_inc.starts_with(&canon_base) {
            bail!(
                "include escapes base directory: {} -> {} (base {})",
                p.display(),
                canon_inc.display(),
                canon_base.display(),
            );
        }
        vfs::read_to_string(p)
            .with_context(|| format!("include not found: {}", p.display()))
    })
}

/// Vfs-backed variant of `expand`: reads partials on-demand through any
/// `Vfs` impl. Used by the browser/WASM build (with `BrowserVfs`) and any
/// future async-native callers (with `LocalVfs`). The CLI's sync paths keep
/// using `expand` — no need to async-ify them.
///
/// Path-traversal guards work the same as the sync variant: include paths
/// must be relative; the Vfs impl is responsible for any sandboxing it needs
/// to enforce (e.g. BrowserVfs only sees files inside the user-granted
/// directory handle).
pub async fn expand_with_vfs<V: crate::vfs::Vfs>(
    vfs: &V,
    content: &str,
    base_dir: &Path,
) -> Result<String> {
    expand_with_vfs_inner(vfs, content, base_dir, 0).await
}

// Boxing the recursive future is the standard trick for async recursion —
// without it, the compiler can't size the future (recursive type).
#[allow(clippy::manual_async_fn)]
fn expand_with_vfs_inner<'a, V: crate::vfs::Vfs>(
    vfs: &'a V,
    content: &'a str,
    base_dir: &'a Path,
    depth: u32,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + 'a>> {
    Box::pin(async move {
        if depth > MAX_DEPTH {
            bail!("include depth > {MAX_DEPTH} — likely a cycle");
        }
        let re = regex::Regex::new(RE)?;
        let mut out = String::with_capacity(content.len());
        let mut last = 0;
        for caps in re.captures_iter(content) {
            let whole = caps.get(0).expect("whole match");
            let path_str = caps.name("path").expect("path group").as_str();
            if Path::new(path_str).is_absolute() {
                bail!("include path must be relative: {path_str}");
            }
            let included_path = base_dir.join(path_str);
            let included = vfs
                .read_to_string(&included_path)
                .await
                .with_context(|| format!("include not found: {}", included_path.display()))?;
            let nested_base = included_path.parent().unwrap_or(base_dir).to_path_buf();
            let expanded =
                expand_with_vfs_inner(vfs, &included, &nested_base, depth + 1).await?;

            out.push_str(&content[last..whole.start()]);
            out.push_str(&strip_frontmatter(&expanded));
            last = whole.end();
        }
        out.push_str(&content[last..]);
        Ok(out)
    })
}

/// In-memory variant of `expand`: instead of reading files from disk, look
/// them up in `files` keyed by the resolved relative path (e.g.
/// `"specs/_partials/paint-metal.md"`).
///
/// Used by the browser/WASM build where the host JS has already loaded file
/// content into memory and hands it to Rust as a `HashMap`. Path-traversal
/// guards aren't enforced here because the map itself is the trust boundary —
/// Rust can only return content the host pre-loaded.
pub fn expand_with_map(
    content: &str,
    base_dir: &Path,
    files: &std::collections::HashMap<String, String>,
) -> Result<String> {
    expand_with_reader(content, base_dir, 0, &|p: &Path| {
        let key = p.to_string_lossy();
        files
            .get(key.as_ref())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("include not found in map: {key}"))
    })
}

fn expand_with_reader<F>(
    content: &str,
    base_dir: &Path,
    depth: u32,
    read: &F,
) -> Result<String>
where
    F: Fn(&Path) -> Result<String>,
{
    if depth > MAX_DEPTH {
        bail!("include depth > {MAX_DEPTH} — likely a cycle");
    }
    let re = regex::Regex::new(RE)?;
    let mut out = String::with_capacity(content.len());
    let mut last = 0;
    for caps in re.captures_iter(content) {
        let whole = caps.get(0).expect("whole match");
        let path_str = caps.name("path").expect("path group").as_str();
        let included_path = base_dir.join(path_str);

        // Reject absolute paths regardless of backend — the directive contract
        // is "relative to including file's dir." Absolute paths in markdown
        // are almost always a typo or a security smell.
        if Path::new(path_str).is_absolute() {
            bail!("include path must be relative: {path_str}");
        }

        let included = read(&included_path)?;
        let nested_base = included_path.parent().unwrap_or(base_dir);
        let expanded = expand_with_reader(&included, nested_base, depth + 1, read)?;

        out.push_str(&content[last..whole.start()]);
        // Strip a trailing frontmatter block from the included partial — the
        // top-level spec already has its own frontmatter, and a second `---`
        // block in the body breaks cmarker.
        out.push_str(&strip_frontmatter(&expanded));
        last = whole.end();
    }
    out.push_str(&content[last..]);
    Ok(out)
}

/// In-memory variant of `find_dependents`: scan an explicit map of file
/// content (key → markdown body) and return the keys whose include directives
/// resolve to `partial_key`.
///
/// Used by the browser/WASM build, where the host JS has loaded the spec set
/// into a `HashMap<String, String>` keyed by relative path (e.g.
/// `"specs/PAINT.md"`, `"specs/_partials/paint-metal.md"`). Path
/// canonicalisation isn't possible without a real filesystem, so this variant
/// uses **path normalisation** (lexical join + strip `./` and `../`) to
/// resolve include directives consistently with how `expand_with_map` resolves
/// them.
///
/// The `specs_root` is the conceptual root of the spec set (used to filter
/// out partial-includes themselves — only entries directly under it are
/// considered "specs"). Empty string means the keys ARE the spec roots.
pub fn find_dependents_in_map(
    files: &std::collections::HashMap<String, String>,
    specs_root: &str,
    partial_key: &str,
) -> Result<Vec<String>> {
    let re = regex::Regex::new(RE)?;
    let mut deps: Vec<String> = Vec::new();

    let target_norm = normalize_path(partial_key);

    for (key, content) in files {
        // Skip the partial itself and other partials. A "spec" is a file
        // directly under specs_root (one path segment past it).
        if key == partial_key {
            continue;
        }
        if !is_top_level_spec(specs_root, key) {
            continue;
        }
        if key.ends_with(".th.md") {
            continue;
        }
        // Each include directive is resolved relative to the SPEC's parent dir.
        let base_dir = key.rsplit_once('/').map(|(p, _)| p).unwrap_or("");
        for caps in re.captures_iter(content) {
            let path_str = caps.name("path").expect("path group").as_str();
            if Path::new(path_str).is_absolute() {
                continue;
            }
            let resolved = if base_dir.is_empty() {
                path_str.to_string()
            } else {
                format!("{}/{}", base_dir, path_str)
            };
            if normalize_path(&resolved) == target_norm {
                deps.push(key.clone());
                break;
            }
        }
    }

    deps.sort();
    Ok(deps)
}

/// Lexical path normalisation (no FS access).
/// - Drops `./` segments
/// - Resolves `..` against the previous segment
/// - Collapses repeated slashes
fn normalize_path(p: &str) -> String {
    let mut stack: Vec<&str> = Vec::new();
    for seg in p.split('/') {
        match seg {
            "" | "." => continue,
            ".." => {
                stack.pop();
            }
            other => stack.push(other),
        }
    }
    stack.join("/")
}

/// True if `key` is a "top-level spec" — i.e. directly under `specs_root`,
/// not nested in `_partials/` or some other subdirectory. Empty `specs_root`
/// means the key has no leading prefix and contains no `/`.
fn is_top_level_spec(specs_root: &str, key: &str) -> bool {
    let inside = if specs_root.is_empty() {
        key
    } else {
        match key.strip_prefix(&format!("{specs_root}/")) {
            Some(rest) => rest,
            None => return false,
        }
    };
    !inside.contains('/')
}

/// Find specs that include a given partial via `<!-- include: ... -->`.
///
/// Scans every top-level `*.md` file directly under `specs_dir` (not recursing
/// into `_partials/` or other subdirs) and returns the paths of specs whose
/// include directives resolve to `partial_path`.
///
/// Path comparison uses canonicalization, so `../_partials/x.md`,
/// `_partials/x.md`, and `./_partials/x.md` all match the same target.
///
/// Used by the watch loop to surface "this partial changed → these specs
/// include it" notifications, so a content edit prompts the user to review
/// derived values in dependents (e.g. hand-written `cans = ceil(area / X)`
/// numbers in specs that consume the partial's coverage data).
///
/// Unreadable specs are silently skipped — a single broken file shouldn't
/// abort the whole dependent scan.
///
/// `vfs` parameter: the filesystem backend. Pass any `Vfs` impl —
/// `LocalVfs` from native contexts, `BrowserVfs` from browser/WASM.
///
/// **Async because `Vfs` is async** (browser FS Access API requires it).
/// Sync callers in the CLI bridge with `pollster::block_on`.
pub async fn find_dependents<V: crate::vfs::Vfs>(
    vfs: &V,
    specs_dir: &Path,
    partial_path: &Path,
) -> Result<Vec<PathBuf>> {
    // Path normalisation works the same in both backends — neither requires
    // `canonicalize` (which is FS-only) for include resolution. We compare
    // the lexical-normalised forms instead, matching how `find_dependents_in_map`
    // already does it.
    let target_norm = normalize_path(&partial_path.to_string_lossy());
    let re = regex::Regex::new(RE)?;
    let mut deps: Vec<PathBuf> = Vec::new();

    let entries = match vfs.read_dir(specs_dir).await {
        Ok(e) => e,
        Err(_) => return Ok(deps),
    };

    for entry in entries {
        if entry.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        // Skip our own outputs (.th.md siblings).
        if entry
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|n| n.ends_with(".th.md"))
        {
            continue;
        }
        let Ok(content) = vfs.read_to_string(&entry).await else {
            continue;
        };
        let base_dir = entry.parent().unwrap_or(specs_dir);
        for caps in re.captures_iter(&content) {
            let path_str = caps.name("path").expect("path group").as_str();
            if Path::new(path_str).is_absolute() {
                continue; // expand() rejects these; ignore here too.
            }
            let included = base_dir.join(path_str);
            if normalize_path(&included.to_string_lossy()) == target_norm {
                deps.push(entry.clone());
                break;
            }
        }
    }

    deps.sort();
    Ok(deps)
}

/// Drop a leading YAML frontmatter block (`---\n...\n---\n`) if present.
/// Partials may carry their own frontmatter for standalone preview, but when
/// inlined we want only the body.
fn strip_frontmatter(content: &str) -> String {
    let trimmed = content.trim_start_matches('\u{FEFF}');
    if !trimmed.starts_with("---\n") && !trimmed.starts_with("---\r\n") {
        return content.into();
    }
    let after = &trimmed[3..]; // skip leading "---"
    let mut consumed = 3;
    let mut closed = false;
    for line in after.lines() {
        consumed += line.len() + 1; // +1 for the newline
        if line.trim() == "---" {
            closed = true;
            break;
        }
    }
    if !closed { return content.into(); }
    trimmed[consumed.min(trimmed.len())..].trim_start_matches('\n').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_directives_returns_input_unchanged() {
        let s = "# Hello\n\nNo includes here.\n";
        assert_eq!(expand(s, Path::new(".")).unwrap(), s);
    }

    #[test]
    fn frontmatter_stripped() {
        let s = "---\ntitle: x\n---\n\n# Body\n";
        assert_eq!(strip_frontmatter(s), "# Body\n");
    }

    #[test]
    fn frontmatter_absent_passes_through() {
        let s = "# Body\n\nplain.\n";
        assert_eq!(strip_frontmatter(s), s);
    }

    #[test]
    fn absolute_include_path_rejected() {
        let dir = std::env::temp_dir().join("quick-incl-abs");
        let _ = std::fs::create_dir_all(&dir);
        let abs = dir.join("evil.md");
        std::fs::write(&abs, "X").unwrap();
        let src = format!("<!-- include: {} -->", abs.display());
        let err = expand(&src, &dir).unwrap_err().to_string();
        assert!(err.contains("must be relative"), "got: {err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parent_traversal_rejected() {
        // base_dir = .../inner ; include points at .../outer/secret.md (escapes)
        let outer = std::env::temp_dir().join("quick-incl-trav");
        let inner = outer.join("inner");
        let _ = std::fs::create_dir_all(&inner);
        std::fs::write(outer.join("secret.md"), "secret").unwrap();
        let src = "<!-- include: ../secret.md -->\n";
        let err = expand(src, &inner).unwrap_err().to_string();
        assert!(err.contains("escapes base"), "got: {err}");
        let _ = std::fs::remove_dir_all(&outer);
    }
}
