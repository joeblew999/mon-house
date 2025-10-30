# mon-tool

Unified CLI tool for managing SVG drawings in the mon-house project.

The 

## Quick Start

```bash
# Build the tool
go build

# Run complete workflow
./mon-tool all

# Or use via Makefile
cd ..
make all
```

## What It Does

mon-tool consolidates 5 separate tools into one unified CLI:
- generate-css → `mon-tool css generate`
- inject-css → `mon-tool css inject`
- validate-svg → `mon-tool svg validate`
- generate-element → `mon-tool svg gen element` (coming soon)
- generate-titleblock → `mon-tool svg gen titleblock` (coming soon)

## Data Flow

```
drawing-standards.json  →  [CSS generation]  →  drawing-standards_gen.css
                                ↓
drawing-standards_gen.css + drawings.json  →  [CSS injection]  →  SVG files
                                ↓
                          SVG files  →  [Validation]  →  Pass/Fail report
```

## Commands

### Main Workflow

**`mon-tool all`** - Run complete workflow (generate → inject → validate)

This is the SINGLE SOURCE OF TRUTH for the workflow. It:
1. Generates CSS from [drawing-standards.json](../drawing-standards.json)
2. Injects CSS into all SVG files listed in [drawings.json](../drawings.json)
3. Validates all SVG files against standards

### CSS Commands

**`mon-tool css generate [standards.json]`**
- Input: [drawing-standards.json](../drawing-standards.json)
- Output: Stdout (redirect to [drawing-standards_gen.css](../drawing-standards_gen.css))
- Purpose: Generate CSS rules from element definitions

**`mon-tool css inject <css-file>`**
- Input: CSS file + [drawings.json](../drawings.json)
- Output: Updates SVG files in place
- Purpose: Embed CSS in SVG `<style>` sections (makes them self-contained)

### SVG Commands

**`mon-tool svg validate [files...]`**
- Input: SVG files (or [drawings.json](../drawings.json) if no args)
- Output: Validation report
- Checks:
  - No inline styles (must use CSS classes)
  - No external stylesheets (must use embedded `<style>`)
  - Has embedded CSS
  - All CSS classes are defined

**`mon-tool svg gen element <type> [opts]`** (not yet implemented)
- Generate SVG element snippets (door, window, wall, etc.)
- Use [generate-element](../generate-element/) tool for now

**`mon-tool svg gen titleblock`** (not yet implemented)
- Generate consistent title blocks from [drawings.json](../drawings.json)
- Use [generate-titleblock](../generate-titleblock/) tool for now

### Drawing Commands

**`mon-tool drawing list`**
- Input: [drawings.json](../drawings.json)
- Output: List of all drawings with metadata

**`mon-tool drawing info <path>`**
- Input: Drawing path from [drawings.json](../drawings.json)
- Output: Detailed info about a specific drawing

## Architecture

```
mon-tool/
├── main.go                    # CLI entry point, command routing
├── cmd/
│   ├── all.go                # Complete workflow (SINGLE SOURCE OF TRUTH)
│   ├── css.go                # CSS commands (generate, inject)
│   ├── svg.go                # SVG commands (validate, gen)
│   └── drawing.go            # Drawing commands (list, info)
├── internal/
│   ├── config/
│   │   └── config.go         # JSON parsing (drawings.json, drawing-standards.json)
│   ├── generator/
│   │   └── css.go            # CSS generation logic
│   ├── injector/
│   │   └── css.go            # CSS injection logic
│   └── validator/
│       └── svg.go            # SVG validation logic
├── go.mod
└── README.md                  # This file
```

## Why Unified Tool?

Previously, we had 5 separate Go tools that all parsed the same JSON files and duplicated code:
- generate-css/main.go
- inject-css/main.go
- validate-svg/main.go
- generate-element/main.go
- generate-titleblock/main.go

**Problems:**
- Code duplication (JSON parsing, error handling, etc.)
- Inconsistent interfaces
- Hard to maintain
- Unclear workflow

**Solution:**
Consolidate into `mon-tool` with:
- Shared internal packages (no duplication)
- Consistent CLI interface
- Single `all` command that defines the workflow
- Clear data flow documentation

## Files Used

| File | Purpose | Used By |
|------|---------|---------|
| [drawing-standards.json](../drawing-standards.json) | Element definitions (source of truth) | `css generate` |
| [drawings.json](../drawings.json) | List of SVG files with metadata | `css inject`, `svg validate`, `drawing list/info` |
| [drawing-standards_gen.css](../drawing-standards_gen.css) | Generated CSS (git-ignored) | `css inject` |
| SVG files | Drawing geometry | `css inject`, `svg validate` |

## Dependencies

- [github.com/itchyny/gojq](https://github.com/itchyny/gojq) - JSON querying for $ref resolution

## Development

```bash
# Build
go build

# Run tests
go test ./...

# Update dependencies
go mod tidy

# Install locally
go install
```

## Examples

```bash
# Complete workflow
./mon-tool all

# Individual steps
./mon-tool css generate > drawing-standards_gen.css
./mon-tool css inject drawing-standards_gen.css
./mon-tool svg validate

# List drawings
./mon-tool drawing list

# Get drawing info
./mon-tool drawing info en/existing/plan.svg

# Validate specific files
./mon-tool svg validate ../drawings/en/*/*.svg

# Help
./mon-tool help
```
