# Claude Context for mon-house

## Project Overview

**mon-house** is documentation for a house renovation project in Thailand.

**Github repo**: https://github.com/joeblew999/mon-house

YOU MUST use this file path root:  `/Users/apple/workspace/go/src/github.com/joeblew999/mon-house`

---

## SPA & Markdown Convention

**README.md is the standard index file** for every folder in this project.

- The root `index.html` is a unified SPA that renders all markdown files
- Navigation uses hash-based routing (e.g., `#examples/production/furniture/SPEC.md`)
- Breadcrumbs link to `README.md` for each folder
- Optional YAML frontmatter in README.md can configure folder behavior:

```yaml
---
leaf_folders: [shopping, images, assets]   # Folders without their own README
---
```

**Note**: SPEC.md files still exist for specifications content, but README.md is the entry point/index for each folder.

---

## Taskfile (Single Source of Truth for Automation)

**Location**: `Taskfile.yml` (in project root)

The project uses [go-task](https://taskfile.dev) for all automation. **Taskfile.yml is the single source of truth** for project variables and tasks. You can add new tasks to this file as needed.

### Key Variables (defined in Taskfile.yml)

All paths and URLs are defined as variables with clear prefixes:
- `BIN_*` - Binary/executable names
- `DIR_*` - Directory paths
- `URL_*` - URLs
- `REPO_*` - Repository info

### Common Tasks

```bash
task                  # Show all available tasks
task build            # Build mon-tool
task test             # Test translation sync (safe)
task prod             # Production translation sync

# GitHub Pages helpers
task pages:status     # Check build status
task pages:wait       # Wait for build to complete
task pages:dev        # Open in dev mode (no cache)
task deploy           # Push + wait + open in dev mode
```

### Adding New Tasks

When you need new automation, add tasks to `Taskfile.yml`:
- Use existing variables from the `vars:` section
- Add new variables with appropriate prefix (DIR_, URL_, etc.)
- Follow existing patterns for consistency

### Browser Testing

**For simple page viewing/testing:**
- Use `task pages:dev` - opens user's default browser in dev mode
- User can inspect and verify manually
- No orphaned browser instances

**For automated browser testing (clicking, filling forms, screenshots):**
- Use MCP Playwright tools (`mcp__playwright__browser_*`)
- ALWAYS call `mcp__playwright__browser_close` when done testing
- If "browser already in use" error occurs, run `task browser:kill` first

---

## What This Project Contains

This project documents:
- **Existing house design** (current state)
- **Proposed house design** (renovation plan)
- **Technical specifications** (measurements, coordinates, materials)
- **Bilingual documentation** (English and Thai)

---

## Project Structure (Standard Go Layout)

```
mon-house/
├── main.go                      # Go entry point
├── go.mod                       # Go module
├── cmd/                         # Command implementations
├── pkg/translate/               # Translation packages
├── internal/                    # Internal packages
├── drawing-standards.json       # SVG element definitions
├── adr/                         # Architecture Decision Records
├── examples/                    # Example projects
│   ├── production/              # Production house documentation
│   │   └── code/translate.json  # Production config
│   └── test/                    # Test data (safe sandbox)
│       └── code/translate.json  # Test config
├── Taskfile.yml                 # Task runner (single source of truth for automation)
└── README.md                    # Project entry point
```

---

## Translation Workflow

All commands are run from the **project root** directory using `task`.

### Production vs Test

**Production commands** (operate on `examples/production/` folder):
```bash
task prod    # Production translation sync
```

**Test commands** (operate on `examples/test/` folder):
```bash
task test    # Test translation sync (safe)
```

**Always test first**: Run `task test` before `task prod`.

### How It Works

1. Taskfile.yml → Changes directory (`cd examples/production` or `cd examples/test`)
2. mon-tool → Auto-discovers `examples/*/code/translate.json` in that directory
3. translate.json → Contains ALL paths and configuration (single source of truth)

**No hardcoded paths in Go code** - everything comes from `examples/*/code/translate.json`.

### Go Code Structure (Standard Go Layout)

The translation system follows standard Go project layout:

**`cmd/mon-tool/`** - Main application:
- `main.go` - Entry point
- `cmd/` - Subcommands (translate, css, svg, etc.)

**`pkg/translate/`** - Translation packages:

**Core types** (`types.go`):
- `Config` - Mirrors translate.json structure exactly
- `TargetConfig` - Per-language config (language, language_name, folder, translation_notes)
- `Task` - Translation task with source/target language and extractions
- `TextExtraction` - Single translatable text element

**Key principle**: Go types directly reflect translate.json. Example:

```json
// In examples/*/code/translate.json:
{
  "targets": [{
    "language": "th",
    "language_name": "Thai",
    "translation_notes": ["Use formal Thai", "..."]
  }]
}
```

```go
// In Go (types.go):
type TargetConfig struct {
    Language         string   `json:"language"`
    LanguageName     string   `json:"language_name"`
    TranslationNotes []string `json:"translation_notes"`
}
```

**Where language config flows**:
1. `config.go` → Loads translate.json
2. `task.go` → Generates task files from Config.Targets
3. `ai/claude.go` → Uses LanguageName and TranslationNotes for AI prompts
4. `commands/*_handler.go` → Executes operations using Config

---

## Critical Synchronization Rules

**Four types of sync MUST be maintained:**

### 1. JSON → CSS Generation
`drawing-standards.json` is the source of truth for all visual styles.
- When JSON changes, `drawing-standards.css` MUST be regenerated
- Use `code/generate-css.sh` to regenerate CSS from JSON
- CSS file is referenced by all SVG files via `<?xml-stylesheet?>` directive
- **Rule**: NEVER edit CSS file manually - always regenerate from JSON
- Details: See [ADR 002: Global CSS Stylesheet](adr/002-global-css-stylesheet.md)

### 2. JSON ↔ SVG Schema Sync
`drawing-standards.json` defines requirements that SVG files must follow.
- JSON defines `requiredMetadata`, visual properties, and CSS classes
- SVG files must implement these requirements
- SVG files reference external CSS: `<?xml-stylesheet href="../../../drawing-standards.css"?>`
- **Rule**: NO inline styles in SVG - use CSS classes only
- Details: See "Keeping Standards and SVG in Sync" section below

### 3. Plan ↔ Section Sync
Plans and sections are linked drawings - changes in one affect the other.
- Section cut line on plan defines what appears in section view
- X-coordinates must match between plan and section
- Section elements reference plan elements via `data-source="plan:{id}"`
- Details: See "Critical Synchronization Rules" at top of "Information Flow"

### 4. EN → TH Translation Sync
EN files are SOURCE, TH files are DERIVED translations.
- Delete `drawings/th/`, copy `drawings/en/`, translate all text
- Pre-commit hook enforces this automatically
- Details: See "Translation Workflow" section below

---

## The Information Flow: How It All Works

```
1. drawing-standards.json
   ↓ (defines vocabulary - what elements mean)

2. drawing-standards.css
   ↓ (GENERATED from JSON - visual styles)

3. SVG drawings (drawings/en/*.svg)
   ↓ (SOURCE OF TRUTH - tagged geometry, references CSS)

4. AI reads SVG and generates SPEC.md
   ↓ (technical parameters extracted)

5. README.md references SPEC.md
   ↓ (human-friendly explanation)

6. Translation: EN → TH
   ↓ (brute force copy + translate)

7. Plan ↔ Section consistency maintained
   ↓ (linked drawings must stay synchronized)
```

**Key principles**:
- **SVG drawings are the source of truth** for geometry
- **JSON is the source of truth** for visual styles (generates CSS)
- Everything else is derived or documented from these sources

---

## File Structure

```
mon-house/
├── README.md                           # Language router (links to en/ or th/)
├── CLAUDE.md                           # This file (AI instructions)
├── code/
│   ├── drawing-standards.json         # Vocabulary: what elements mean (SOURCE)
│   ├── drawing-standards.css          # Visual styles (GENERATED from JSON)
│   └── adr/
│       ├── 001-json-reference-resolution.md
│       └── 002-global-css-stylesheet.md
├── drawings/
│   ├── en/                            # English documentation (SOURCE)
│   │   ├── README.md                  # Human explanation
│   │   ├── existing/
│   │   │   ├── plan.svg               # Current floor plan (references CSS)
│   │   │   ├── section.svg            # Current section view (references CSS)
│   │   │   └── SPEC.md                # Technical specs (AI-generated)
│   │   └── proposed/
│   │       ├── plan.svg               # Proposed floor plan (references CSS)
│   │       ├── section.svg            # Proposed section view (references CSS)
│   │       └── SPEC.md                # Technical specs (AI-generated)
│   └── th/                            # Thai translation (DERIVED from en/)
│       └── (mirror of en/ structure with Thai text)
├── furniture/
│   ├── en/                            # English furniture specs (SOURCE)
│   │   ├── SPEC.md                    # Furniture specifications
│   │   └── shopping/                  # Product research and links
│   │       ├── README.md              # Shopping directory naming pattern
│   │       └── *.md                   # Product research files
│   └── th/                            # Thai translation (DERIVED from en/)
│       └── (mirror of en/ structure with Thai text)
└── .git/hooks/pre-commit              # Enforces EN→TH sync
```

---

## The 4 Documentation Layers

### 1. Root README.md (Language Router)
**Purpose**: Language selection only
**Content**: Links to `en/` or `th/` folders
**Rule**: NEVER add project details here

### 2. drawing-standards.json (Design Vocabulary)
**Purpose**: Define what building elements mean semantically
**Content**: Element definitions, CSS classes, visual properties
**Audience**: AI (for programmatic reference)

**Key concepts:**
- `wall-exterior` = building envelope (vertical)
- `roof` = building envelope (top)
- `foundation` = building envelope (bottom)
- `wall-interior` = partition (non-structural)
- **Building envelope** = complete boundary (exterior walls + roof + foundation)

### 3. SVG Drawings (Source of Truth)
**Location**: `drawings/en/existing/*.svg` and `drawings/en/proposed/*.svg`
**Purpose**: Tagged building geometry
**Audience**: AI reads to derive specs; humans view for visualization

**Rule**: This is the SOURCE OF TRUTH. All measurements come from here.

### 4. SPEC.md (AI-Generated Technical Specifications)
**Location**: `drawings/en/existing/SPEC.md` and `drawings/en/proposed/SPEC.md`
**Purpose**: Technical parameters extracted from SVG
**Generation**: AI reads SVG tags → generates SPEC.md
**Content**: Coordinates, dimensions, floor areas, materials
**Header**: Must say "AUTO-GENERATED - DO NOT EDIT MANUALLY"

**Rule**: If SVG changes → regenerate SPEC.md

### 5. README.md (Human Explanation)
**Location**: `drawings/en/README.md`
**Purpose**: Human-friendly explanation of the design
**Audience**: Builders, contractors, stakeholders
**Content**: Design intent (WHY and WHAT), references to SPEC.md for numbers
**Rule**: NO coordinates, NO dimensions in README - just concepts and references

### 6. Thai Translations (TH)
**Location**: `drawings/th/` (mirrors `drawings/en/`)
**Purpose**: Thai translations for Thai builders
**Generation**: Copy entire `en/` folder → translate all text
**Rule**: Must maintain identical structure to EN files

---

## Furniture Directory Structure

**Location**: `examples/production/furniture/` (organized by category)

### Purpose
Contains furniture and fixture specifications for all rooms in the house. Unlike drawing SPEC.md files (which are auto-generated from SVG), furniture SPEC.md is manually curated with detailed product specifications.

### Structure (Category-Based)

```
furniture/
├── SPEC.md              # Master index (links to all categories)
├── SELLERS.md           # Shared list of Thailand retailers
├── README.md            # Naming conventions and guidelines
│
├── living-room/         # Section 1.x
│   ├── SPEC.md          # Living room specifications
│   └── shopping/
│       ├── 1.1-chaise-lounge.md
│       └── images/
│
├── bathroom/            # Section 2.x
│   ├── SPEC.md          # Bathroom specifications (18 items)
│   └── shopping/
│       ├── 2.1-toilet.md
│       ├── 2.2-basin.md
│       └── ...
│
├── dining/              # Section 3.x
│   ├── SPEC.md          # Dining specifications
│   └── shopping/
│
├── lighting/            # Section 4.x
│   ├── SPEC.md          # Lighting specifications
│   └── shopping/
│
└── outdoor/             # Section 5.x
    ├── SPEC.md          # Outdoor specifications
    └── shopping/
```

### Section Numbering Convention

- **1.x** = Living Room (1.1 Chaise Lounge, 1.2 Coffee Table, etc.)
- **2.x** = Bathroom (2.1 Toilet, 2.2 Basin, etc.)
- **3.x** = Dining (3.1 Table, 3.2 Chairs, etc.)
- **4.x** = Lighting (4.1 Living Room Light, 4.2 Bedroom Light, etc.)
- **5.x** = Outdoor (5.1 Plants, 5.2 Outdoor Furniture, etc.)

### Key Principles

1. **Category SPEC.md is source of truth** for each category
   - Each category folder has its own SPEC.md
   - Contains detailed product information, dimensions, specifications
   - Master SPEC.md at root is an index linking to all categories

2. **Shopping directory** separates product research from specifications
   - Each category has its own `shopping/` subfolder
   - Contains volatile information (prices, links, vendor SKUs)
   - Files named using pattern: `{section-number}-{kebab-case-name}.md`
   - Example: Section "2.1 Toilet" → `bathroom/shopping/2.1-toilet.md`

3. **SELLERS.md** is shared at furniture root
   - List of Thailand retailers (HomePro, Lazada, Shopee, etc.)
   - Used for product research across all categories

4. **Naming pattern** ensures consistency
   - Shopping filenames MUST match SPEC.md section numbers exactly
   - Convert section title to kebab-case for filename

5. **Shopping file back links** must include section anchors
   - Links back to SPEC.md MUST include anchor to specific section
   - Without anchor, user lands at top of SPEC instead of the item they were shopping for
   - Format: `[← Back to Category SPEC](../SPEC.md#section-anchor)`
   - Anchor format: lowercase, hyphens for spaces, no special chars
   - Example: Section "2.5 Shower Taps/Faucet" → anchor `#25-shower-tapsfaucet`

6. **Frontmatter configuration** in root SPEC.md
   - `leaf_folders: [shopping, images, assets]` defines folders without their own SPEC.md
   - These folders link back to their parent SPEC.md in breadcrumb navigation
   - Configuration is read once by index.html at startup

### Example Workflow

**Adding a new furniture item:**

1. Determine category (living-room, bathroom, dining, lighting, outdoor)
2. Add to `{category}/SPEC.md` with next section number (e.g., 3.1 Dining Table)
3. Create shopping file: `{category}/shopping/3.1-dining-table.md`
4. Research products using sellers in `SELLERS.md` (at furniture root)

**Adding a new category:**

1. Create folder: `furniture/{category-name}/`
2. Create `{category}/SPEC.md` with template
3. Create `{category}/shopping/` subfolder
4. Update master `furniture/SPEC.md` index with new category link

---

## Working with SVG Drawings

### SVG Structure Requirements

**Every SVG element must:**
1. Use a `class` attribute (from drawing-standards.json)
2. Match a CSS class in the `<style>` section
3. Match a definition in drawing-standards.json

**Example:**
```svg
<!-- In drawing-standards.json -->
"wall-exterior": { "stroke": "black", "stroke-width": 8 }

<!-- In SVG <style> section -->
.wall-exterior { stroke: black; stroke-width: 8; }

<!-- In SVG elements -->
<line class="wall-exterior" x1="100" y1="100" x2="400" y2="100"/>
```

**Rule**: NO inline styles (`stroke="black"`). Always use classes.

### Semantic Metadata in SVG Elements

SVG elements should include **semantic metadata** using SVG's built-in features: `<g>` grouping, `id` attributes, `data-*` attributes, and `<title>` elements.

**Source of truth**: The `requiredMetadata` field in `drawing-standards.json` defines what metadata each element type needs. Always check the JSON first.

**Why semantic metadata?**
- Enables Plan ↔ Section synchronization (section can reference plan data)
- Makes SVG machine-readable for automation
- Documents design intent within the drawing itself
- Allows AI to understand relationships between elements

**Three levels of semantic metadata:**

#### Level 1: Simple Elements (walls, windows, furniture)
Use **CSS class only** - semantics come from drawing-standards.json:
```svg
<line x1="100" y1="100" x2="400" y2="100" class="wall-exterior"/>
<rect x="150" y="100" width="80" height="8" class="window"/>
<rect x="200" y="200" width="200" height="200" class="furniture"/>
```
**When to use**: Walls, windows, simple furniture, structural elements

#### Level 2: Grouped Elements (dining sets, section cuts)
Use **`<g>` with `id`** to group related elements:
```svg
<g id="dining-set">
  <rect x="485" y="525" width="120" height="75" class="furniture"/> <!-- table -->
  <rect x="430" y="555" width="50" height="45" class="furniture"/> <!-- chair -->
  <rect x="610" y="555" width="50" height="45" class="furniture"/> <!-- chair -->
</g>

<g id="section-cut-aa">
  <line x1="100" y1="620" x2="890" y2="620" class="section-cut"/>
  <path d="M 100 620 L 90 640 M 100 620 L 110 640" class="section-arrow"/>
  <text x="495" y="610" class="section-label">SECTION A-A</text>
</g>
```
**When to use**: Multiple elements that move together, section cuts, furniture sets

#### Level 3: Parametric Elements (doors, complex objects)
Use **`<g>` + `data-*` attributes + `<title>`** for elements with dimensions and behavior:
```svg
<g id="door-south-wall" class="door"
   data-width="0.9"
   data-height="2.1"
   data-swing-direction="inward-left"
   data-hinge-side="left"
   data-type="swinging">
  <title>South Wall Door: 0.9m × 2.1m, swinging inward to the left</title>
  <line x1="450" y1="840" x2="540" y2="840" class="door"/>
  <path d="M 450 840 Q 460 850 490 870" class="door-arc"/>
  <text x="495" y="860" font-size="10" fill="brown" text-anchor="middle">Door (0.9×2.1m)</text>
</g>
```
**When to use**: Doors, windows with specific dimensions, stairs, equipment

**Common data-* attributes:**

**IMPORTANT**: See `drawing-standards.json` under `elements.<element-name>.requiredMetadata` for the complete, authoritative list of required and optional attributes for each element type.

**Quick reference** (check JSON for current requirements):

For doors (see `elements.door.requiredMetadata`):
- `data-width` - door width in meters (required)
- `data-height` - door height in meters (required)
- `data-type` - "swinging" or "sliding" (required)
- `data-swing-direction` - swing direction (required if swinging)
- `data-hinge-side` - "left" or "right" (required if swinging)

For sliding doors (see `elements.door-sliding.requiredMetadata`):
- `data-width` - door width in meters (required)
- `data-height` - door height in meters (required)
- `data-type` - "sliding" (required)
- `data-panels` - number of panels (optional, default 2)

For windows (see `elements.window.requiredMetadata`):
- `data-width` - window width in meters (required)
- `data-height` - window height in meters (required)
- `data-sill-height` - sill height above floor (required for sections)

For furniture (optional metadata, not yet in JSON):
- `data-width`, `data-depth`, `data-height` - dimensions in meters
- `data-type` - "bed", "table", "chair", "bench", etc.

For rooms (optional metadata, not yet in JSON):
- `data-name` - room name (e.g., "bedroom-1", "living-room")
- `data-floor-area` - area in square meters

**Plan ↔ Section linking:**
In section drawings, reference plan elements using `data-source`:
```svg
<!-- In section.svg -->
<g id="door-south-wall-section" data-source="plan:door-south-wall">
  <title>South Wall Door (from plan): 0.9m × 2.1m</title>
  <line x1="550" y1="600" x2="550" y2="390" stroke="brown" stroke-width="4"/>
  <!-- ... door frame geometry calculated from plan data -->
</g>
```

**When editing SVGs:**
1. **Check element type** - Does it need semantic metadata?
2. **Determine level** - Simple (class only), Grouped (id), or Parametric (data-*)
3. **Add metadata** - Use appropriate attributes for the level
4. **Link drawings** - If element appears in section, add `data-source` reference
5. **Verify sync** - Ensure Plan → Section dimensions match

### Keeping Standards and SVG in Sync

**These THREE must always match:**
1. `drawing-standards.json` (definition)
2. SVG `<style>` section (CSS classes)
3. SVG elements (use the classes)

**Critical**: The SVG `<style>` section must be generated from the JSON to ensure sync!

#### How to Sync JSON → SVG CSS

**1. Element Visual Properties**
JSON `elements.{type}.visual` → SVG CSS class `.{type}`:

```json
// JSON: elements.door.visual
{
  "stroke": "brown",
  "strokeWidth": 4,
  "fill": "none"
}

// SVG <style>:
.door { stroke: brown; stroke-width: 4; fill: none; }
```

**2. Element Label Properties**  
JSON `elements.{type}.requiredMetadata.childElements.label` → SVG CSS class `.{type}-label`:

```json
// JSON: elements.door.requiredMetadata.childElements.label
{
  "fontSize": "$ref:typography.fontSizes.standard",  // resolves to 16
  "fontFamily": "$ref:typography.fontFamilies.primary",  // resolves to Arial
  "fill": "pink"
}

// SVG <style>:
.door-label { font-size: 16px; font-family: Arial; fill: pink; }
```

**3. Text Elements Must Use CSS Classes**
❌ **WRONG** (inline styles):
```svg
<text x="100" y="100" font-size="16" font-family="Arial" fill="pink">Door</text>
```

✅ **CORRECT** (CSS class):
```svg
<text x="100" y="100" class="door-label">Door</text>
```

**Why?** CSS classes in `<style>` section are the source of truth. Inline styles can be overridden by parent element CSS, causing rendering issues.

**Workflow when JSON changes:**
1. Edit `drawing-standards.json` (change color, size, etc.)
2. Regenerate SVG `<style>` section CSS classes from JSON
3. Verify all `<text>` elements use `class="{type}-label"`, not inline styles
4. Test rendering in SVG viewer

**If you add a new element type:**
1. Add to `drawing-standards.json` with `visual` and `label` properties
2. Generate CSS class `.{type}` from `visual` properties
3. Generate CSS class `.{type}-label` from `label` properties
4. Add both CSS classes to all SVG `<style>` sections
5. Use classes in SVG elements

### Building Envelope Semantic Rules

**Building envelope** = complete boundary separating interior from exterior

**Components:**
- **Exterior walls** (vertical envelope) - `wall-exterior` class
- **Roof** (top envelope) - `roof` class
- **Foundation** (bottom envelope) - `foundation` class

**Critical rule**: ALL interior elements (furniture, people, etc.) must be positioned **inside** the building envelope:
- Between exterior walls (horizontal containment)
- Below roof slope (vertical containment from top)
- Above foundation (vertical containment from bottom)

**Example check**: For loft bedroom in proposed section:
- Person's head must be below roof slope at that x-coordinate
- Bed must be between left/right exterior walls
- All elements on platform above foundation level

---

## Translation Workflow (Brute Force)

### The Simple 3-Step Process

**Step 1: Delete TH folders, Copy EN folders**
```bash
# Delete entire TH drawings folder
rm -rf drawings/th/

# Copy entire EN drawings folder
cp -r drawings/en/ drawings/th/

# Delete entire TH furniture folder
rm -rf furniture/th/

# Copy entire EN furniture folder
cp -r furniture/en/ furniture/th/
```

**Step 2: Translate ALL text in SVG files (drawings only)**
- Open every `.svg` file in `drawings/th/`
- Translate ALL `<text>` element content
- **Don't change**: coordinates, CSS classes, structure

**Step 3: Translate ALL text in Markdown files (drawings AND furniture)**
- **In drawings/th/**: Rename `SPEC.md` → `SPEC.th.md`, `README.md` → `README.th.md`
- **In furniture/th/**: Rename `SPEC.md` → `SPEC.th.md`
- **In furniture/th/shopping/**: Rename `README.md` → `README.th.md`, translate all `*.md` files
- Translate ALL text in all markdown files
- **Don't change**: numbers, coordinates, measurements, section numbers, file naming patterns

### Thai Architectural Terminology

**Rule**: Translate the SEMANTIC MEANING, not word-for-word.

**Source of terminology**: See the `vocabulary` field in `drawing-standards.json` for standard architectural terms and their semantic meanings.

**Translation approach:**
- Read the element's semantic meaning in drawing-standards.json
- Use proper Thai architectural terms that convey that concept
- Prioritize semantic accuracy over literal word translation

**Example**:
- `wall-exterior` has semantic meaning "Forms the building boundary between conditioned interior and exterior environment"
- Thai translation should convey "structural perimeter wall that separates inside from outside"
- Not just literal "wall outside"

---

## Pre-Commit Hook (Automatic Enforcement)

**Location**: `.git/hooks/pre-commit`

### What It Does

When you commit EN files, the hook automatically:
1. Deletes entire `drawings/th/` folder
2. Copies entire `drawings/en/` folder → `drawings/th/`
3. Renames markdown files (`.md` → `.th.md`)
4. Prompts you to translate
5. Blocks commit until TH files are translated and staged
6. Verifies structure matches (line count check)

### How to Use

```bash
# 1. Edit EN files
vim drawings/en/proposed/section.svg

# 2. Stage and commit EN files
git add drawings/en/
git commit -m "Update proposed section"

# Hook runs and blocks:
#   🗑️  Deleting entire TH folder...
#   📋 Copying entire EN folder to TH...
#   ⚠️  TRANSLATION REQUIRED
#   ❌ COMMIT BLOCKED

# 3. Translate ALL TH files
vim drawings/th/existing/plan.svg       # translate text
vim drawings/th/existing/section.svg    # translate text
vim drawings/th/existing/SPEC.th.md     # translate text
vim drawings/th/proposed/plan.svg       # translate text
vim drawings/th/proposed/section.svg    # translate text
vim drawings/th/proposed/SPEC.th.md     # translate text

# 4. Stage TH files and commit again
git add drawings/th/
git commit -m "Update proposed section"

# Hook runs and passes:
#   ✅ Structure verification passed
#   ✅ EN→TH translation sync OK
```

**Why this works**: Guarantees EN and TH are always synchronized in version control.

---

## AI Workflow: Reading SVG and Generating SPEC

When SVG drawings change, AI should:

**Step 1: Read drawing-standards.json**
- Learn what element classes mean semantically
- Understand visual properties

**Step 2: Read SVG drawings**
- Find elements by class (e.g., `wall-exterior`)
- Extract coordinates from attributes (x1, y1, x2, y2, etc.)
- Calculate dimensions and areas
- Identify building envelope, rooms, openings

**Step 3: Generate SPEC.md**
- Document building envelope coordinates
- List all rooms with dimensions and floor areas
- Document structural elements (beams, foundations)
- Calculate totals
- Mark file: "AUTO-GENERATED - DO NOT EDIT MANUALLY"

**Step 4: Update README.md**
- Reference SPEC.md for all numbers
- Explain design intent (WHY and WHAT)
- NO coordinates or dimensions in README

---

## Common Tasks

### When you change an SVG drawing:

1. Edit `drawings/en/existing/*.svg` or `drawings/en/proposed/*.svg`
2. Verify elements use correct classes from drawing-standards.json
3. Verify all interior elements are inside building envelope
4. Regenerate corresponding SPEC.md (AI reads SVG)
5. Run translation workflow (delete TH → copy EN → translate)

### When you add a new element type:

1. Add definition to `drawing-standards.json`
2. Add CSS class to SVG `<style>` section
3. Apply class to SVG elements
4. Verify sync: JSON ↔ CSS ↔ elements

### When committing changes:

1. Edit EN files only
2. Stage and commit: `git add drawings/en/ && git commit`
3. Pre-commit hook will block and prompt for translation
4. Translate all TH files
5. Stage and commit again: `git add drawings/th/ && git commit`

---

## Design Vocabulary (Semantic Terms)

From `drawing-standards.json`:

**envelope**: The complete building boundary separating conditioned interior from exterior environment - includes exterior walls (vertical envelope), roof (top envelope), and foundation (bottom envelope)

**perimeter**: The outer boundary of the building footprint

**partition**: Interior wall dividing interior space (non-structural)

**structural**: Element that carries loads and cannot be removed

**load-bearing**: Element that supports structural loads (roof, floors, etc.)

**weather barrier**: Element that separates interior from exterior environment

**removable**: Element that can be removed without affecting building stability

---

## Critical Rules Summary

1. **SVG is source of truth** - all measurements derive from SVG geometry
2. **Building envelope** = exterior walls + roof + foundation (semantic concept)
3. **All interior elements must be inside envelope** (below roof, between walls, above foundation)
4. **EN is source, TH is derived** - always edit EN first, then translate to TH (applies to drawings/ AND furniture/)
5. **Translation = brute force** - delete TH folders (drawings/th/ AND furniture/th/), copy EN folders, translate all text
6. **Structure must match** - TH files must have identical coordinates/CSS/structure as EN
7. **Use CSS classes, NEVER inline styles** - all SVG styling must use CSS classes from `<style>` section, which are generated from drawing-standards.json. Inline styles (`fill="pink"`, `font-size="16"`) can be overridden by parent CSS and cause rendering bugs. If you need a new style, add it to the JSON first, then generate the CSS class.
8. **SVG `<style>` must match JSON** - CSS classes in SVG must be generated from drawing-standards.json. When JSON changes, regenerate SVG CSS.
9. **SPEC.md is auto-generated** - regenerate when SVG changes (for drawings); SPEC.md is manually curated for furniture
10. **README.md references SPEC** - no numbers in README, only concepts
11. **Pre-commit hook enforces sync** - cannot commit EN without translating TH
12. **furniture/SPEC.md is source of truth** - shopping files must follow SPEC.md section numbering exactly using pattern `{section-number}-{kebab-case-name}.md`
13. **leaf_folders frontmatter** - Root furniture/SPEC.md uses frontmatter to define folders that don't have their own SPEC.md (e.g., shopping, images). The SPA uses this for breadcrumb navigation.
14. **shopping file back links need anchors** - links from shopping files back to SPEC.md MUST include section anchor (e.g., `../SPEC.md#25-shower-tapsfaucet`) so users return to the specific item, not top of page

---

**This file was simplified on 2025-10-23 to remove accumulated complexity.**
---

**This file was simplified on 2025-10-23 to remove accumulated complexity.**
