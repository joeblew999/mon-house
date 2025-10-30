# mon-house/code

Go tools for CSS generation and SVG management.

## CORE CONCEPT 

drawing-standards.json is the truth that the code works off.

The AI can use the mon tools to help it work with the SVG files.

## For Claude (AI updates)

**ALWAYS test first, then run on production:**

```bash
make test    # 1. Test on test/test.svg (SAFE - do this first!)
make all     # 2. Run on production drawings (updates real files)
make clean   # Clean generated files
```

The Makefile runs the complete workflow:
1. Generates CSS from `drawing-standards.json`
2. Injects CSS into all SVG files (makes them self-contained)
3. Validates SVG files follow standards

**Workflow:**
- `make test` runs on `../test/test.svg` (safe sandbox)
- `make all` runs on production drawings in `../drawings/`
- Both auto-compile `mon-tool` if source changes

## For Developers (debugging/coding)

Use `mon-tool` CLI for granular control:

```bash
cd mon-tool && go build

# Visual validation (CSS, syntax)
./mon-tool css generate           # Generate CSS only
./mon-tool css inject <css-file>  # Inject CSS only
./mon-tool svg validate           # Validate CSS/syntax

# Semantic validation (metadata, properties)
./mon-tool semantic validate plan.svg      # Check required metadata
./mon-tool semantic validate *.svg         # Check all drawings

# Information
./mon-tool drawing list           # List all drawings
./mon-tool drawing info plan.svg  # Show drawing details

# Or run complete workflow
./mon-tool all
```

See [mon-tool/README.md](mon-tool/README.md) and [SEMANTIC-VISION.md](mon-tool/SEMANTIC-VISION.md) for full documentation.

## Files

- `drawing-standards.json` - Element definitions (source of truth)
- `drawings.json` - List of production SVG files with metadata
- `../test/drawings.json` - List of test SVG files (for `make test`)
- `drawing-standards_gen.css` - Generated CSS (git-ignored)
