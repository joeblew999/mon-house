# Test Folder - Safe Sandbox for AI Changes

## Purpose

This folder provides a **safe testing environment** for Claude (AI) to test changes before touching production drawings.

## What's Here

- `test.svg` - A simple test drawing with windows, doors, walls
- `drawings.json` - Configuration pointing to test.svg
- `drawing-standards_gen.css` - Generated CSS (created by `make test`)

## Workflow for AI (Claude)

### Step 1: Test Changes Here FIRST

```bash
cd code
make test
```

This runs the complete workflow on `test.svg`:
1. ✓ Generates CSS from drawing-standards.json
2. ✓ Injects CSS into test.svg
3. ✓ Validates syntax (inline styles, CSS classes)
4. ✓ Validates semantics (required metadata)

**Output shows:**
- What was updated
- What passed
- What has warnings/errors

### Step 2: Review Output

The output tells you:
```
✓ CSS generated: drawing-standards_gen.css
✓ CSS injected into 1 SVG files
⚠ 1 syntax validation warnings
⚠ 14 semantic metadata issues found

Run 'mon-tool semantic validate <file>' for details
```

### Step 3: Fix Issues (if any)

If there are semantic errors:
```bash
cd ..
./code/mon-tool/mon-tool semantic validate test/test.svg
```

This shows EXACTLY what's missing:
```
✗ 14 metadata errors:

  Line ~55: test-door (class=door)
    data-width: Missing required attribute 'data-width'

  Line ~72: (unnamed window) (class=window)
    data-width: Missing required attribute 'data-width'
    data-height: Missing required attribute 'data-height'
```

### Step 4: If Test Looks Good → Production

```bash
make all
```

This runs the SAME workflow on production drawings in `../drawings/`.

## What Gets Tested

### 1. Visual/Syntactic Checks
- ✓ CSS classes are defined
- ✓ No inline styles (should use classes)
- ✓ CSS is embedded (not external)

### 2. Semantic Checks
- ✓ Windows have data-width, data-height
- ✓ Doors have data-width, data-height, data-type
- ✓ Numeric values don't have units (e.g., "1.2" not "1.2m")
- ✓ Required metadata is present

## Test Cases

### Current test.svg contains:
- Exterior walls (wall-exterior)
- Interior walls (wall-interior)
- Windows (window) - SOME missing metadata
- Doors (door) - SOME missing metadata
- Sliding doors (door-sliding) - SOME missing metadata

### Intentional Issues:
The test.svg INTENTIONALLY has issues to verify the tools catch them:
- ⚠ 14 metadata errors (missing data-width, data-height, etc.)
- ⚠ 10 inline font-size attributes

This ensures the validation tools are working!

## For User (Developer)

You can add more test cases by:

1. Edit `test.svg` - Add elements with/without metadata
2. Run `make test` - See what the tools catch
3. Verify tools correctly identify issues

## For AI (Claude)

Before making ANY changes to production drawings:
1. Test the change pattern on `test.svg` first
2. Run `make test`
3. Verify the output is what you expect
4. Only then run `make all` on production

**This prevents breaking production drawings!**
