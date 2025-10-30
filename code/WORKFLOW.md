# Complete Workflow Documentation

## For AI (Claude) - How to Update Drawings

### The Golden Rule
**ALWAYS test first, then production:**
```bash
make test    # Step 1: Test on test/test.svg (SAFE)
make all     # Step 2: Run on production (ONLY if test passed)
```

---

## What `make test` Does (Step-by-Step)

```
┌─────────────────────────────────────────────────┐
│  make test  (Safe sandbox on test/test.svg)    │
└─────────────────────────────────────────────────┘
                     ↓
         ┌───────────────────────┐
         │ 1. Build mon-tool     │
         │    (if source changed)│
         └───────────────────────┘
                     ↓
         ┌───────────────────────┐
         │ 2. Generate CSS       │
         │    drawing-standards  │
         │    .json → CSS        │
         └───────────────────────┘
                     ↓
         ┌───────────────────────┐
         │ 3. Inject CSS         │
         │    Embed in test.svg  │
         │    (makes standalone) │
         └───────────────────────┘
                     ↓
         ┌───────────────────────┐
         │ 4. Validate Syntax    │
         │    - CSS classes OK?  │
         │    - No inline styles?│
         └───────────────────────┘
                     ↓
         ┌───────────────────────┐
         │ 5. Validate Semantics │
         │    - Metadata present?│
         │    - Format correct?  │
         └───────────────────────┘
                     ↓
         ┌───────────────────────┐
         │ 6. Show Summary       │
         │    ✓ What passed      │
         │    ⚠ What needs work  │
         └───────────────────────┘
```

---

## What `make all` Does (Same as test, but on production)

Runs the EXACT SAME workflow on:
- `../drawings/en/existing/plan.svg`
- `../drawings/en/existing/section.svg`
- `../drawings/en/proposed/plan.svg`
- `../drawings/en/proposed/section.svg`

---

## Understanding the Output

### Good Output (Everything Passed)
```
=== Summary ===
✓ CSS generated: drawing-standards_gen.css
✓ CSS injected into 4 SVG files
✓ Syntax validation passed
✓ All semantic metadata present

Complete - All checks passed!
```

### Warning Output (Issues Found)
```
=== Summary ===
✓ CSS generated: drawing-standards_gen.css
✓ CSS injected into 4 SVG files
⚠ 1 syntax validation warnings
⚠ 47 semantic metadata issues found

Run 'mon-tool semantic validate <file>' for details

Complete - SVG files updated (warnings above)
```

**What to do:** Run the suggested command to see details:
```bash
./mon-tool/mon-tool semantic validate ../drawings/en/existing/plan.svg
```

---

## Two Types of Flows

### Type 1: Unidirectional (Source → Derived)

These flows are **one-way** - tools regenerate derived files:

```
drawing-standards.json  →  [Generate CSS]  →  drawing-standards_gen.css
                              ↓
drawing-standards_gen.css  →  [Inject]  →  SVG files (embedded CSS)
                              ↓
SVG files  →  [Validate]  →  Pass/Fail report
```

**For AI:** Just run `make test` or `make all` - tools handle the rest.

### Type 2: Bidirectional (Must Manually Sync)

These flows are **two-way** - both are sources of truth:

```
plan.svg  ↔  section.svg
```

**Rules:**
- Section cut line in plan defines what appears in section
- X-coordinates MUST match between plan and section
- If you change plan at x=300, you MUST update section at x=300
- Cannot regenerate one from the other - both are hand-drawn

**For AI:** Tools CHECK consistency (future), but YOU must manually keep them synchronized.

---

## Validation Levels

### Level 1: Syntax (SVG Structure)
✓ CSS classes are defined
✓ No inline styles (use classes instead)
✓ CSS is embedded (not external)

**Tool:** `mon-tool svg validate`

### Level 2: Semantics (Required Metadata)
✓ Windows have data-width, data-height
✓ Doors have data-width, data-height, data-type
✓ Numeric values have no units (e.g., "1.2" not "1.2m")
✓ Elements have required attributes per drawing-standards.json

**Tool:** `mon-tool semantic validate`

### Level 3: Geometric (TODO - Future)
✓ Interior elements are INSIDE building envelope
✓ Plan ↔ Section x-coordinates match
✓ Section elements reference valid plan elements

**Tool:** `mon-tool semantic sync` (not yet implemented)

---

## Decision Tree for AI

```
┌─────────────────────────────────────┐
│ Need to update SVG drawings?        │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│ 1. Test on test/test.svg FIRST      │
│    $ make test                       │
└─────────────────────────────────────┘
              ↓
         Did it pass?
              ↓
     YES ─────┴───── NO
      ↓              ↓
┌─────────────┐  ┌──────────────────────┐
│ 2. Run on   │  │ Fix issues:          │
│ production  │  │ - Check output       │
│ $ make all  │  │ - Run semantic       │
└─────────────┘  │   validate for       │
                 │   details            │
                 │ - Fix SVG            │
                 │ - Try make test again│
                 └──────────────────────┘
```

---

## File Locations

```
mon-house/
├── code/
│   ├── Makefile                       ← Run 'make test' or 'make all' here
│   ├── drawing-standards.json         ← Visual styles (SOURCE)
│   ├── drawing-standards_gen.css      ← Generated CSS
│   ├── drawings.json                  ← Production SVG file list
│   └── mon-tool/
│       └── mon-tool                   ← The unified tool binary
├── drawings/
│   └── en/
│       ├── existing/
│       │   ├── plan.svg               ← Production drawings
│       │   └── section.svg            ← (4 files total)
│       └── proposed/
│           ├── plan.svg
│           └── section.svg
└── test/
    ├── test.svg                       ← Safe test drawing
    └── drawings.json                  ← Test configuration
```

---

## Common Commands Reference

```bash
# === FOR AI (CLAUDE) ===

# Test before touching production
cd code && make test

# Update production drawings
cd code && make all

# Clean generated files
cd code && make clean


# === FOR DEBUGGING ===

# See detailed semantic errors
./code/mon-tool/mon-tool semantic validate test/test.svg

# Validate specific production file
./code/mon-tool/mon-tool semantic validate drawings/en/existing/plan.svg

# List all production drawings
./code/mon-tool/mon-tool drawing list

# Show drawing details
./code/mon-tool/mon-tool drawing info en/existing/plan.svg
```

---

## Summary

**The workflow is simple:**
1. AI edits SVG or JSON
2. Run `make test` (safe sandbox)
3. Check output - all passed?
4. If yes → `make all` (production)
5. If no → fix issues, repeat from step 2

**The tools handle:**
- ✅ CSS generation (unidirectional)
- ✅ CSS injection (unidirectional)
- ✅ Syntax validation (unidirectional)
- ✅ Semantic validation (unidirectional)
- ⏳ Plan↔Section sync (bidirectional - TODO)

This ensures drawings stay valid and complete!
