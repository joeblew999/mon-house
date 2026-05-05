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
pub fn split(content: &str) -> Vec<String> {
    let re = regex::Regex::new(r"(?m)^## ").expect("static regex");
    let starts: Vec<usize> = re.find_iter(content).map(|m| m.start()).collect();

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
}
