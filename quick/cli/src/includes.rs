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

use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::vfs;

const MAX_DEPTH: u32 = 5;

/// Match `<!-- include: PATH -->` on its own line. Whitespace tolerant.
/// Multi-line flag: `(?m)`. Anchor to a line start to avoid false matches in code blocks.
const RE: &str = r"(?m)^[ \t]*<!--[ \t]*include:[ \t]*(?P<path>[^\s>]+)[ \t]*-->[ \t]*$";

/// Recursively expand include directives in `content`, with paths resolved
/// relative to `base_dir`. The outermost call typically passes the spec's
/// parent directory.
pub fn expand(content: &str, base_dir: &Path) -> Result<String> {
    expand_inner(content, base_dir, 0)
}

fn expand_inner(content: &str, base_dir: &Path, depth: u32) -> Result<String> {
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

        // Guard: reject absolute paths and any include that would escape the
        // top-level base. Keeps a stray `<!-- include: ../../../etc/passwd -->`
        // from quietly inlining whatever happens to be on disk.
        if Path::new(path_str).is_absolute() {
            bail!("include path must be relative: {path_str}");
        }
        let canon_base = base_dir.canonicalize().unwrap_or_else(|_| base_dir.to_path_buf());
        let canon_inc  = included_path.canonicalize()
            .with_context(|| format!("include not found: {}", included_path.display()))?;
        if !canon_inc.starts_with(&canon_base) {
            bail!(
                "include escapes base directory: {} -> {} (base {})",
                path_str, canon_inc.display(), canon_base.display(),
            );
        }

        let included = vfs::read_to_string(&included_path)
            .with_context(|| format!("include not found: {}", included_path.display()))?;
        let nested_base = included_path.parent().unwrap_or(base_dir);
        let expanded = expand_inner(&included, nested_base, depth + 1)?;

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
