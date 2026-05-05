// Markdown chunker — splits a spec on `## ` (level-2) heading boundaries.
//
// Used by `translate::translate_file` so that editing one section only
// re-translates that section, not the whole spec. Each chunk's BLAKE3 hash
// becomes the cache key; the cache (a JSON sidecar `.th.md.cache.json`) maps
// EN-chunk hash → cached Thai chunk text.
//
// Chunk 0 is whatever lives BEFORE the first `## ` (typically frontmatter +
// intro paragraph + any embedded images). Subsequent chunks start at a `## `
// heading and run to the next one. Joining is lossless — `chunks.concat()`
// reproduces the original input byte-for-byte.

/// Split markdown content into level-2-section chunks.
///
/// Behaviour:
///   * If the file has no `## ` headings: one chunk == whole content.
///   * If `## ` is the very first text: chunk 0 is empty (preserved so the
///     chunk-index stays stable as later chunks change).
///   * Joining: the concatenation of the returned chunks IS the input.
///
/// Code-fence aware: a `## ` line **inside** a fenced block (between two
/// matching ` ``` ` lines) is NOT treated as a section boundary. Otherwise
/// a code sample that happens to contain `## ` (e.g. a shell comment or a
/// markdown-about-markdown example) would silently shred the spec.
pub fn split(content: &str) -> Vec<String> {
    let starts = find_section_starts(content);

    if starts.is_empty() {
        return vec![content.to_string()];
    }

    let mut chunks = Vec::with_capacity(starts.len() + 1);
    chunks.push(content[..starts[0]].to_string()); // everything before first `## `
    for (i, &start) in starts.iter().enumerate() {
        let end = starts.get(i + 1).copied().unwrap_or(content.len());
        chunks.push(content[start..end].to_string());
    }
    chunks
}

/// Walk the input line-by-line and return the byte offset of every `## `
/// heading that is NOT inside a fenced code block.
fn find_section_starts(content: &str) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut in_fence = false;
    let mut offset = 0usize;
    for line in content.split_inclusive('\n') {
        // A line that begins with ``` (any fence flavour) flips fence state.
        // We test against `trim_start` so indented fences still toggle.
        let trimmed_start = line.trim_start();
        if trimmed_start.starts_with("```") || trimmed_start.starts_with("~~~") {
            in_fence = !in_fence;
        } else if !in_fence && line.starts_with("## ") {
            starts.push(offset);
        }
        offset += line.len();
    }
    starts
}

/// Concatenate chunks back into a single string. Inverse of `split`.
pub fn join(chunks: &[String]) -> String {
    chunks.concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_headings_one_chunk() {
        let s = "no headings here\njust prose\n";
        let chunks = split(s);
        assert_eq!(chunks.len(), 1);
        assert_eq!(join(&chunks), s);
    }

    #[test]
    fn three_sections_round_trip() {
        let s = "intro paragraph\n\n## A\n\nbody A\n\n## B\n\nbody B\n";
        let chunks = split(s);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], "intro paragraph\n\n");
        assert!(chunks[1].starts_with("## A"));
        assert!(chunks[2].starts_with("## B"));
        assert_eq!(join(&chunks), s);
    }

    #[test]
    fn frontmatter_in_first_chunk() {
        let s = "---\ntitle: x\n---\n\nintro\n\n## A\n\nbody\n";
        let chunks = split(s);
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].contains("title: x"));
        assert!(chunks[1].starts_with("## A"));
        assert_eq!(join(&chunks), s);
    }

    #[test]
    fn first_byte_is_heading_keeps_empty_first_chunk() {
        let s = "## Heading\n\nbody\n";
        let chunks = split(s);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], "");
        assert!(chunks[1].starts_with("## Heading"));
        assert_eq!(join(&chunks), s);
    }

    #[test]
    fn heading_inside_code_fence_is_not_a_section_boundary() {
        let s = "## Real\n\n```bash\n## not a heading\necho hi\n```\n\n## Also Real\nbody\n";
        let chunks = split(s);
        // Two real sections (after the empty pre-chunk).
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], "");
        assert!(chunks[1].starts_with("## Real"));
        assert!(chunks[1].contains("## not a heading")); // stayed inside chunk 1
        assert!(chunks[2].starts_with("## Also Real"));
        assert_eq!(join(&chunks), s);
    }

    #[test]
    fn tilde_fence_also_protects_headings() {
        let s = "intro\n~~~\n## inside tilde fence\n~~~\n## Real\n";
        let chunks = split(s);
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].contains("## inside tilde fence"));
        assert!(chunks[1].starts_with("## Real"));
        assert_eq!(join(&chunks), s);
    }
}
