/// New subcommand — scaffold a new spec file from TEMPLATE.md.
use std::path::PathBuf;

use anyhow::{bail, Result};

const DEFAULT_TEMPLATE: &str = "\
---
title: {TITLE}
status: Draft
rev: \"1\"
---

# {TITLE} - Laem Chabang

[One paragraph describing scope]

---

## Dimensions
| Element | Measurement |
|---|---|
| Item | Value |

---

## Shopping List
| Product | What it does | Qty | Est. Price (THB) |
|---|---|---|---|
| Product name | Description | 1 | ~0 |

---

## Cost Summary
| Category | Est. Cost (THB) |
|---|---|
| Materials | ~0 |
| Labour | ~0 |
| **TOTAL** | **~0** |

---

## Steps
1. Step one
2. Step two
3. Step three

---

## Notes
- Note one
- Note two
";

pub fn cmd_new(name: &str) -> Result<()> {
    // Reject path separators and traversals — spec names must be plain identifiers
    if name.contains('/') || name.contains('\\') || name.contains("..") || name.is_empty() {
        bail!("spec name must be a plain word (no slashes or dots), got: {name:?}");
    }
    let filename = format!("{}.md", name.to_uppercase());
    let path = PathBuf::from(&filename);
    if path.exists() {
        bail!("{filename} already exists");
    }

    // Prefer the project's TEMPLATE.md; fall back to the embedded default
    let content = if PathBuf::from("TEMPLATE.md").exists() {
        std::fs::read_to_string("TEMPLATE.md")?
            .replace("Spec Title", name)
    } else {
        DEFAULT_TEMPLATE.replace("{TITLE}", name)
    };

    std::fs::write(&path, content)?;
    println!("✓ created {filename}");
    println!("  Edit it, then save — `mise run watch` will pick it up.");
    Ok(())
}
