# Claude Context for mon-house

## Project Overview

**mon-house** is documentation for a house renovation project in Thailand.

**Github repo**: https://github.com/joeblew999/mon-house

**File path**: `/Users/apple/workspace/go/src/github.com/joeblew999/mon-house`

---

## What This Project Contains

This project documents:
- **Existing house design** (current state)
- **Proposed house design** (renovation plan)
- **Technical specifications** (measurements, coordinates, materials)
- **Bilingual documentation** (English and Thai)

---

## The Information Flow: How It All Works

```
1. code/drawing-standards.json
   ↓ (defines vocabulary - what elements mean)

2. SVG drawings (drawings/en/*.svg)
   ↓ (SOURCE OF TRUTH - tagged geometry)

3. AI reads SVG and generates SPEC.md
   ↓ (technical parameters extracted)

4. README.md references SPEC.md
   ↓ (human-friendly explanation)

5. Translation: EN → TH
   ↓ (brute force copy + translate)
```

**Key principle**: SVG drawings are the source of truth. Everything else is derived or documented from them.

---

## File Structure

```
mon-house/
├── README.md                           # Language router (links to en/ or th/)
├── CLAUDE.md                           # This file (AI instructions)
├── code/
│   └── drawing-standards.json         # Vocabulary: what elements mean
├── drawings/
│   ├── en/                            # English documentation (SOURCE)
│   │   ├── README.md                  # Human explanation
│   │   ├── existing/
│   │   │   ├── plan.svg               # Current floor plan
│   │   │   ├── section.svg            # Current section view
│   │   │   └── SPEC.md                # Technical specs (AI-generated)
│   │   └── proposed/
│   │       ├── plan.svg               # Proposed floor plan
│   │       ├── section.svg            # Proposed section view
│   │       └── SPEC.md                # Technical specs (AI-generated)
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

### 2. code/drawing-standards.json (Design Vocabulary)
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

### Keeping Standards and SVG in Sync

**These THREE must always match:**
1. `code/drawing-standards.json` (definition)
2. SVG `<style>` section (CSS classes)
3. SVG elements (use the classes)

**If you add a new element type:**
1. Add to `drawing-standards.json`
2. Add CSS class to SVG `<style>`
3. Use class in SVG elements

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

**Step 1: Delete TH folder, Copy EN folder**
```bash
# Delete entire TH folder
rm -rf drawings/th/

# Copy entire EN folder
cp -r drawings/en/ drawings/th/
```

**Step 2: Translate ALL text in SVG files**
- Open every `.svg` file in `drawings/th/`
- Translate ALL `<text>` element content
- **Don't change**: coordinates, CSS classes, structure

**Step 3: Translate ALL text in Markdown files**
- Rename `SPEC.md` → `SPEC.th.md`
- Rename `README.md` → `README.th.md`
- Translate ALL text
- **Don't change**: numbers, coordinates, measurements

### Thai Architectural Terminology

**Rule**: Translate the SEMANTIC MEANING, not word-for-word.

**Source of terminology**: See the `vocabulary` field in `code/drawing-standards.json` for standard architectural terms and their semantic meanings.

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

1. Add definition to `code/drawing-standards.json`
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

From `code/drawing-standards.json`:

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
4. **EN is source, TH is derived** - always edit EN first, then translate to TH
5. **Translation = brute force** - delete TH folder, copy EN, translate all text
6. **Structure must match** - TH files must have identical coordinates/CSS/structure as EN
7. **Use classes, not inline styles** - all SVG elements use classes from drawing-standards.json
8. **SPEC.md is auto-generated** - regenerate when SVG changes
9. **README.md references SPEC** - no numbers in README, only concepts
10. **Pre-commit hook enforces sync** - cannot commit EN without translating TH

---

**This file was simplified on 2025-10-23 to remove accumulated complexity.**
