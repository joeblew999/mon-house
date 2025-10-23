# Claude Context for mon-house

## Project Overview
mon-house is docs for a house renovation project.

wil need to be in English and translated to Thai, because the builder is Thai.

## Project Structure
This is a project located at `/Users/apple/workspace/go/src/github.com/joeblew999/mon-house`.



## Documentation Approach

- **Bilingual**: Documentation in English with Thai translations for the builder.




## Floor Plan / SVG Schematic Guidelines

### Dimensioning Standards
All floor plans should use **chained dimensions** on all exterior walls. This means:

1. **Individual dimensions**: Show each room/segment dimension
2. **Total dimensions**: Show the overall exterior wall length
3. **Placement**: Dimensions should be placed outside the building perimeter
4. **Format**: Use blue dashed lines with blue text for dimensions

### Example of Chained Dimensions:
```
LEFT SIDE (exterior wall):
- Bedroom 1 height: 3.4m (individual)
- Bedroom 2 height: 3.0m (individual)
- Bathroom height: 1.0m (individual)
- TOTAL: 7.4m (sum of all)
```

All four sides (top, bottom, left, right) should have chained dimensions showing:
- Each room's contribution to that wall
- The total length of that exterior wall

### Scale
- Use centerline dimensions (dimensions measured to wall centerlines)
- Standard scale: 1 meter = 100 pixels in SVG
- This is a schematic - accuracy to centerlines is acceptable

### Other SVG Elements
- Walls: Black, 8px stroke width
- Doors: Brown, with swing arc showing direction
- Windows: Light blue fill, blue stroke
- Dimension lines: Blue, dashed (5,5 pattern)
- Dimension text: Blue, 16px Arial

### Notes
- It's OK for doors to overlap walls - this is a schematic
- All dimensions should be clearly visible and not overlapping
- Use comments in SVG to label sections clearly

## Bilingual Documentation Structure

The project uses **language-specific folders** (`en/` and `th/`) to clearly separate English and Thai documentation for builders.

### Folder Structure
```
drawings/
├── README.md        # Bilingual index explaining structure
├── en/              # English documentation
│   ├── README.md    # English specs and assumptions
│   ├── existing/    # Current layout (English labels)
│   │   ├── plan.svg
│   │   └── section.svg
│   └── proposed/    # Renovation plan (English labels)
│       ├── plan.svg
│       └── section.svg
└── th/              # Thai documentation (ภาษาไทย)
    ├── README.th.md # Thai specs and assumptions
    ├── existing/    # Current layout (Thai labels)
    │   ├── plan.svg
    │   └── section.svg
    └── proposed/    # Renovation plan (Thai labels)
        ├── plan.svg
        └── section.svg
```

### Sources of Truth

1. **English Documentation**: `drawings/en/README.md`
   - Primary source of truth
   - Contains all design intent, assumptions, and specifications
   - Defines semantic wall types (EXTERIOR vs INTERIOR)
   - Documents building dimensions and structural elements

2. **Thai Documentation**: `drawings/th/README.th.md`
   - Must mirror all content from `en/README.md`
   - Same structure and detail level
   - References same folder structure

3. **English Drawings**: `drawings/en/existing/` and `drawings/en/proposed/`
   - SVG files with English labels
   - Uses semantic CSS classes: `.wall-exterior` (black, 8px solid) and `.wall-interior` (red, 4px dashed)

4. **Thai Drawings**: `drawings/th/existing/` and `drawings/th/proposed/`
   - **Same SVG structure** as English drawings
   - **Only labels translated** to Thai
   - Must maintain identical coordinates and styling

### Semantic Wall Markup

**CRITICAL**: All floor plans use semantic CSS classes to distinguish wall types:

- **`.wall-exterior`** (black, 8px solid): Building perimeter/envelope - **MUST be maintained** in renovation
- **`.wall-interior`** (red, 4px dashed): Internal partitions - **CAN be removed** in renovation

Wall classification requires understanding of building structure, not just room adjacency.
**Example**: A 1.0m wall with no room behind it is EXTERIOR even if between two interior rooms.

### Translation Guidelines

When updating drawings or documentation:

1. **Always update BOTH** English (`en/`) and Thai (`th/`) versions
2. Keep structure identical - only translate text labels
3. Maintain same SVG coordinates and styling
4. **Thai room labels**:
   - ห้องนอน (bedroom)
   - ห้องน้ำ (bathroom)
   - ห้องนั่งเล่น (living room)
   - ห้องครัว (kitchen)
   - พื้นที่นั่งเล่นแบบเปิด (open living space)
   - ห้องนอนลอฟท์ (loft bedroom)
5. **Thai technical terms**:
   - ผนัง (wall), ผนังภายนอก (exterior wall), ผนังภายใน (interior wall)
   - หน้าต่าง (window), ประตู (door)
   - คาน (beam), สันหลังคา (ridge), ชายคา (eave)
   - หน้าตัด (section), แบบผัง (plan)
   - มาตราส่วน (scale), ขนาด (dimension)

### File Path Rules

- **ALWAYS** use relative paths starting with `./`
- Project root: `/Users/apple/workspace/go/src/github.com/joeblew999/mon-house`
- English drawings: `./drawings/en/`
- Thai drawings: `./drawings/th/`
- Never use absolute paths with username in them

## Design Intent

**Goal**: Convert 2-bedroom house to open-plan living space with loft bedroom

**Key Changes**:
- **REMOVE**: Interior bedroom walls (red dashed lines)
- **KEEP**: All exterior walls (black solid lines)
- **KEEP**: Bathroom unchanged
- **ADD**: Loft bedroom above at 3.0m level on beam

