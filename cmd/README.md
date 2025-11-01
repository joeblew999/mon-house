# mon-tool

Unified CLI tool for the mon-house project.

## Quick Start

```bash
# Build
go build

# Translation workflow (main focus)
./mon-tool translate sync              # Sync EN → TH translations
./mon-tool translate sync --dry-run    # Preview sync
./mon-tool translate apply <task>      # Apply translations
./mon-tool translate events            # View event log

# Headless AI translation
export ANTHROPIC_API_KEY=sk-ant-...
./mon-tool translate auto <task>       # AI translates automatically

# CSS/SVG tools (legacy)
./mon-tool all                         # Generate CSS + inject + validate
./mon-tool css generate                # Generate CSS from JSON
./mon-tool svg validate                # Validate SVG files
```

## What It Does

mon-tool combines multiple systems:

**Translation** (primary focus):
- **CQRS architecture** - Clean separation of commands and queries
- **Event sourcing** - Complete audit trail of all operations
- **Headless AI translation** - Optional Claude API integration

**CSS/SVG** (legacy tools):
- CSS generation from drawing-standards.json
- CSS injection into SVG files
- SVG validation

## Architecture

### Data Flow

```
translate.json (config)
    ↓
[Sync Command] → Copies EN → TH, generates task files
    ↓
tasks/translate-th.json (extraction tasks)
    ↓
[Auto Command] → AI fills translations (optional)
    ↓
[Apply Command] → Writes translations to TH files
    ↓
.mon-tool/events.jsonl (audit trail)
```

### CQRS Pattern

**Commands** (write operations):
- `SyncCommand` - Syncs EN to TH folders, generates tasks
- `ApplyCommand` - Applies translations to files

**Queries** (read operations):
- `LoadConfig()` - Reads translate.json
- `ReadAll()` - Reads event log

**Handlers**:
- `SyncHandler` - Executes sync command, emits events
- `ApplyHandler` - Executes apply command, emits events

### Event Sourcing

All operations emit events to `.mon-tool/events.jsonl`:

**Event types:**
- `DirectoryCreated` - Folder created
- `FileCopied` - File copied (with size)
- `TaskGenerated` - Translation task created
- `TranslationApplied` - Translation written to file
- `AITranslationStarted` - AI translation began
- `AITranslationCompleted` - AI translation finished (with costs)
- `AITranslationFailed` - AI translation failed (with error)

**View events:**
```bash
./mon-tool translate events
```

## Commands

### translate sync

**Syncs EN → TH translations and generates task files.**

```bash
./mon-tool translate sync              # Execute sync
./mon-tool translate sync --dry-run    # Preview actions
```

**What it does:**
1. Loads `code/translate.json` configuration
2. Plans sync actions (mkdir, copy files)
3. Executes sync (if not dry-run)
4. Generates translation task file
5. Emits events for all operations

**Output:**
- TH folder structure (mirror of EN)
- `tasks/translate-th.json` (extraction tasks)
- Events in `.mon-tool/events.jsonl`

### translate apply

**Applies translations from task file to TH files.**

```bash
./mon-tool translate apply tasks/translate-th.json              # Execute apply
./mon-tool translate apply tasks/translate-th.json --dry-run    # Preview apply
```

**What it does:**
1. Reads task file
2. For each file, replaces source text with target text
3. Writes updated files
4. Emits TranslationApplied events

**Requires:** Task file with `target_text` filled in (manual or via `translate auto`)

### translate auto

**AI translates task file using Claude API (headless).**

```bash
export ANTHROPIC_API_KEY=sk-ant-...
./mon-tool translate auto tasks/translate-th.json
```

**What it does:**
1. Reads task file
2. Sends extractions to Claude API
3. Fills in `target_text` for all items
4. Writes updated task file
5. Emits AI events (with token usage and cost)

**Cost tracking:** Events include:
- Input/output token counts
- Estimated cost in USD
- Model used
- Duration

**Requires:** `ANTHROPIC_API_KEY` environment variable

### translate events

**Views event log with filtering.**

```bash
./mon-tool translate events                    # All events
./mon-tool translate events --session <id>     # Specific session
./mon-tool translate events --type Sync        # Specific event type
```

**Output:** Formatted event log with timestamps, types, and details

## Configuration

**Location:** `code/translate.json`

**Structure:**
```json
{
  "source": {
    "language": "en",
    "folder": "drawings/en"
  },
  "targets": [{
    "language": "th",
    "language_name": "Thai",
    "folder": "drawings/th",
    "rename_rules": { ".md": ".th.md" },
    "translation_notes": ["Use formal Thai", "..."]
  }],
  "file_types": {
    "translatable": [".svg", ".md"],
    "copy_only": [".png", ".jpg"]
  },
  "paths": {
    "tasks": "tasks",
    "events": ".mon-tool"
  }
}
```

**Key principle:** This is the single source of truth. All paths, languages, and rules come from this file.

## Complete Workflow

### Manual Translation

```bash
# 1. Sync (generates task file)
./mon-tool translate sync

# 2. Fill translations manually
vim tasks/translate-th.json

# 3. Apply translations
./mon-tool translate apply tasks/translate-th.json

# 4. View what happened
./mon-tool translate events
```

### Headless AI Translation

```bash
# 1. Sync
./mon-tool translate sync

# 2. AI translates (headless)
export ANTHROPIC_API_KEY=sk-ant-...
./mon-tool translate auto tasks/translate-th.json

# 3. Apply translations
./mon-tool translate apply tasks/translate-th.json

# 4. Check costs
./mon-tool translate events --type AITranslation
```

## Error Handling

**All commands:**
- Exit code 0 = success
- Exit code 1 = error
- Errors printed to stderr

**Dry-run mode:**
- Always safe (read-only)
- Shows what would happen
- No events emitted

## Development

**Project structure:**
```
mon-tool/
├── main.go                    # CLI entry point
├── cmd/
│   └── translate.go           # translate subcommand
└── pkg/translate/
    ├── types.go               # Data structures
    ├── config.go              # Config loading
    ├── sync.go                # Sync logic
    ├── apply.go               # Apply logic
    ├── task.go                # Task generation
    ├── commands/              # CQRS handlers
    │   ├── types.go
    │   ├── sync_handler.go
    │   └── apply_handler.go
    ├── events/                # Event sourcing
    │   ├── types.go
    │   └── store.go
    └── ai/                    # AI translation
        ├── types.go
        └── claude.go
```

**Testing:**
```bash
# Run tests
go test ./...

# Build
go build

# Test on safe data
cd ../../project_test
../code/mon-tool/mon-tool translate sync
```

See [../adr/](../adr/) for architecture decision records.

---

## CSS/SVG Commands (Legacy)

These commands are still available for CSS generation and SVG management:

### all

**Run complete CSS workflow.**

```bash
./mon-tool all
```

**What it does:**
1. Generates CSS from `drawing-standards.json`
2. Injects CSS into all SVG files
3. Validates all SVG files

### css generate

**Generate CSS from drawing-standards.json.**

```bash
./mon-tool css generate [standards.json]    # Output to stdout
./mon-tool css generate > styles.css        # Save to file
```

### css inject

**Inject CSS into SVG files.**

```bash
./mon-tool css inject <css-file>
```

Reads `drawings.json` and injects CSS into listed SVG files.

### svg validate

**Validate SVG files against standards.**

```bash
./mon-tool svg validate [files...]          # Validate specific files
./mon-tool svg validate                     # Validate all (from drawings.json)
```

**Checks:**
- No inline styles (must use CSS classes)
- Has embedded CSS in `<style>` section
- All CSS classes are defined

### drawing list

**List all drawings from drawings.json.**

```bash
./mon-tool drawing list
```

### drawing info

**Show detailed drawing information.**

```bash
./mon-tool drawing info <svg-file>
```

### semantic validate

**Validate semantic metadata in SVG files.**

```bash
./mon-tool semantic validate <files...>
```

**Checks required metadata** from drawing-standards.json (e.g., doors must have width, height, type).
