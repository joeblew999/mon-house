# ADR 002: Global CSS Stylesheet for SVG Drawings

## Status
Proposed

## Context

Currently, each SVG file contains a `<style>` section that duplicates the same CSS classes across all 4 drawing files:
- `drawings/en/existing/plan.svg`
- `drawings/en/existing/section.svg`
- `drawings/en/proposed/plan.svg`
- `drawings/en/proposed/section.svg`

This creates several problems:

1. **Duplication**: Same CSS repeated in 4 files (8 files if counting Thai translations)
2. **Synchronization burden**: When visual styles change in `drawing-standards.json`, must update all SVG files
3. **Maintenance complexity**: Easy to have inconsistent styles across files
4. **File size**: Each SVG carries redundant CSS payload

## Decision

**OPTION A: Global External CSS File** (RECOMMENDED)

Create a single global CSS file that all SVG drawings reference:

```
code/
  drawing-standards.json          # Source of truth
  drawing-standards.css           # Generated CSS (auto-generated from JSON)

drawings/
  en/
    existing/
      plan.svg                    # References ../../../code/drawing-standards.css
      section.svg                 # References ../../../code/drawing-standards.css
    proposed/
      plan.svg                    # References ../../../code/drawing-standards.css
      section.svg                 # References ../../../code/drawing-standards.css
```

**SVG file structure:**
```svg
<?xml version="1.0" encoding="UTF-8"?>
<?xml-stylesheet href="../../../code/drawing-standards.css" type="text/css"?>
<svg xmlns="http://www.w3.org/2000/svg"
     xmlns:xlink="http://www.w3.org/1999/xlink"
     viewBox="0 0 800 1000">

  <!-- NO <style> section needed! -->

  <rect class="wall-exterior" x="100" y="100" width="600" height="800"/>
  <!-- ... rest of drawing ... -->
</svg>
```

**Generation workflow:**
```bash
# 1. Edit drawing-standards.json
vim code/drawing-standards.json

# 2. Generate CSS from JSON (automated tool)
./code/tools/generate-css-from-json.sh

# 3. CSS is automatically applied to all SVG files via reference
# No need to edit individual SVG files!
```

**OPTION B: Embedded CSS (Current Approach)**

Keep `<style>` sections in each SVG file, but generate them from JSON.

**Pros:**
- Self-contained SVG files (work offline, can be moved)
- No relative path issues
- Simpler for AI to read (all info in one file)

**Cons:**
- Duplication across 4-8 files
- Must regenerate all SVG files when styles change
- Higher maintenance burden

## Trade-offs

### External CSS Advantages:
1. **Single source of truth**: Only `code/drawing-standards.css` needs updating
2. **DRY principle**: CSS defined once, referenced many times
3. **Easier maintenance**: Change one file → affects all drawings
4. **Smaller SVG files**: No embedded CSS payload
5. **Clearer separation**: Visual styles separate from geometry

### External CSS Disadvantages:
1. **Additional lookup required**: AI must read two files (SVG + CSS) to understand rendering
2. **Relative path fragility**: `<?xml-stylesheet href="../../../code/drawing-standards.css"?>` must be correct
3. **Distribution complexity**: Must include CSS file when sharing SVG
4. **Viewer compatibility**: Some SVG viewers may not support external stylesheets

### Embedded CSS Advantages:
1. **Self-contained**: SVG file has everything needed to render
2. **AI comprehension**: Single file contains both geometry and styles
3. **Portability**: Can copy/move SVG without dependencies
4. **Universal compatibility**: All SVG viewers support embedded `<style>`

### Embedded CSS Disadvantages:
1. **Duplication**: Same CSS in 4-8 files
2. **Sync burden**: Must regenerate all files when styles change
3. **File bloat**: Each file carries full CSS payload

## Recommendation

**Use External CSS with automated validation**

**Rationale:**
1. The project already has a sophisticated generation workflow (JSON → CSS → SVG)
2. Adding one more reference step is minor compared to maintenance savings
3. AI can easily read two files (SVG + CSS) - this is not a significant burden
4. The project uses git, so relative paths are stable
5. Pre-commit hooks can validate CSS references

**Implementation plan:**

1. Create `code/tools/generate-css-from-json.sh` (already exists as `/tmp/generate_all_css.sh`)
2. Generate `code/drawing-standards.css` from JSON
3. Update all SVG files to reference external CSS via `<?xml-stylesheet?>`
4. Remove `<style>` sections from SVG files
5. Add pre-commit hook check: verify all SVG files reference correct CSS path
6. Update CLAUDE.md with new workflow

**AI workflow adjustment:**

When AI reads SVG to generate SPEC.md:
```
1. Read code/drawing-standards.json  # Semantic definitions
2. Read code/drawing-standards.css   # Visual styles
3. Read drawings/en/existing/plan.svg # Geometry
4. Generate SPEC.md                  # Technical parameters
```

One additional file read is acceptable given the maintenance benefits.

## Consequences

### Positive:
- **Reduced duplication**: CSS defined once in `code/drawing-standards.css`
- **Easier updates**: Change JSON → regenerate CSS → all SVGs automatically updated
- **Clearer architecture**: Visual styles separate from geometry
- **Smaller SVG files**: No embedded CSS payload

### Negative:
- **Two-file dependency**: SVG + CSS must both be present
- **AI must read CSS file**: Additional file read required for full context
- **Relative path maintenance**: Must ensure `<?xml-stylesheet?>` paths are correct

### Mitigation:
- Pre-commit hook validates CSS references
- Documentation in CLAUDE.md clearly explains the dependency
- Automated tools handle CSS generation (no manual editing)

## Related ADRs
- [ADR 001: JSON Reference Resolution](001-json-reference-resolution.md) - Defines $ref system for DRY JSON

## Notes

The key insight is that **we already have a generation pipeline** (JSON → CSS → SVG). Moving from "generate CSS into SVG" to "generate CSS as external file" is a small incremental change with significant maintenance benefits.

The additional AI lookup burden (read one more file) is minimal compared to the cost of maintaining synchronized CSS across 4-8 SVG files.

---

**Date**: 2025-10-30
**Author**: Claude (AI assistant)
**Reviewers**: User
