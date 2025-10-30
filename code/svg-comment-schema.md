# SVG Comment Schema for Architectural Drawings

## Purpose

SVG files are the **source of truth** for all architectural drawings. To prevent merge conflicts and maintain design intent, we embed structured metadata as XML comments within the SVG files.

## Comment Types

### 1. FILE-LEVEL Metadata

Place at the top of the SVG file, after the XML declaration:

```xml
<!--
  FILE: drawings/en/proposed/plan.svg
  TYPE: floor-plan
  SCALE: 1m = 100px
  LAST-UPDATED: 2025-10-30
  BILINGUAL-PAIR: drawings/th/proposed/plan.svg
  DESIGN-PHASE: proposed

  DESIGN-INTENT:
  - Remove interior walls to create open-plan living space
  - Remove ceiling to expose full 5.5m roof height
  - Maintain all exterior walls (building envelope)
  - Keep bathroom unchanged

  CONSTRAINTS:
  - All EXTERIOR walls must remain (black, 8px solid)
  - Can remove INTERIOR walls (red, 4px dashed)
  - Total dimensions must match: 7.9m × 7.4m
-->
```

### 2. ELEMENT-LEVEL Metadata

Attach to specific SVG elements (walls, rooms, dimensions):

#### Walls

```xml
<!-- WALL: west-exterior
     ID: wall-west-exterior
     TYPE: exterior
     SEMANTIC: building-envelope
     CONSTRAINT: load-bearing, must-keep
     COORDINATES: x=100, y=100 to y=840
     THICKNESS: centerline (actual wall thickness not shown)
     MATERIAL: concrete/brick (TBD by builder)
     BILINGUAL-PAIR: th#wall-west-exterior
-->
<line id="wall-west-exterior" class="wall-exterior"
      x1="100" y1="100" x2="100" y2="840"
      stroke="black" stroke-width="8"/>
```

```xml
<!-- WALL: bedroom-divider (REMOVED IN PROPOSED)
     ID: wall-bedroom-divider
     TYPE: interior
     SEMANTIC: partition
     CONSTRAINT: can-remove
     COORDINATES: y=440, x=100 to x=400
     STATUS: existing-only (removed in proposed plan)
     REASON-REMOVED: Create open-plan living space
-->
<line id="wall-bedroom-divider" class="wall-interior"
      x1="100" y1="440" x2="400" y2="440"
      stroke="red" stroke-width="4" stroke-dasharray="5,5"/>
```

#### Rooms

```xml
<!-- ROOM: open-living-space
     ID: room-open-living
     NAME-EN: Open Living Space
     NAME-TH: พื้นที่นั่งเล่นแบบเปิด
     DIMENSIONS: 3.0m × 7.4m (width × depth)
     AREA: 22.2 m²
     CEILING-HEIGHT: 2.8m (existing) → 5.5m (exposed roof)
     REPLACES: bedroom-1, bedroom-2
     DESIGN-INTENT: Convert bedrooms to high-ceiling open space
     BILINGUAL-PAIR: th#room-open-living

     COORDINATES:
     - Top-left: (100, 100)
     - Bottom-right: (400, 840)
     - Centerpoint: (250, 470)
-->
<text x="250" y="400" id="room-open-living-label">
  OPEN LIVING SPACE
</text>
```

#### Dimensions

```xml
<!-- DIMENSION: total-depth
     ID: dim-total-depth
     VALUE: 7.4m
     TYPE: chained-total
     CALCULATION: 3.4m + 3.0m + 1.0m
     COMPONENTS:
     - bedroom-1-depth: 3.4m
     - bedroom-2-depth: 3.0m
     - bathroom-depth: 1.0m
     PLACEMENT: outside building perimeter (left side)
     CONSTRAINT: must-equal sum of components
     BILINGUAL-PAIR: th#dim-total-depth
-->
<line class="dimension-line" x1="50" y1="100" x2="50" y2="840"/>
<text class="dimension-text" x="30" y="470">7.4m</text>
```

#### Doors & Windows

```xml
<!-- DOOR: bathroom-entry
     ID: door-bathroom
     TYPE: hinged-door
     WIDTH: 0.8m (800mm standard)
     SWING: inward-to-bathroom
     COORDINATES: wall at y=740, x=290 to x=370
     HINGE-SIDE: left (when facing bathroom)
     CLEARANCE: verify 800mm opening
     BILINGUAL-PAIR: th#door-bathroom
-->
<path id="door-bathroom" d="M 290 740 L 290 820 A 80 80 0 0 1 370 740"
      stroke="brown" fill="none"/>
```

```xml
<!-- WINDOW: living-room-north
     ID: window-living-north
     TYPE: fixed-window
     WIDTH: 1.5m
     HEIGHT: 1.2m (typical)
     COORDINATES: wall at y=100, x=545 to x=695
     SILL-HEIGHT: ~1.0m from floor (typical)
     DESIGN-NOTE: Provides north light to living room
     BILINGUAL-PAIR: th#window-living-north
-->
<rect id="window-living-north" x="545" y="90" width="150" height="20"
      fill="lightblue" stroke="blue"/>
```

### 3. STRUCTURAL Elements

```xml
<!-- BEAM: ceiling-beam
     ID: beam-ceiling
     TYPE: structural-beam
     HEIGHT: 2.8m above floor
     DEPTH: 100mm
     SPAN: 7.9m (full width)
     MATERIAL: wood/steel (TBD)
     LOAD: supports roof structure
     DESIGN-NOTE: Remains in place, ceiling removed below it
     CONSTRAINT: cannot-remove (structural)
-->
<rect id="beam-ceiling" x="200" y="290" width="790" height="10"
      fill="#8B4513" stroke="black"/>
```

### 4. SECTION-SPECIFIC Metadata

```xml
<!-- SECTION-CUT: A-A
     ID: section-line-aa
     LOCATION: y=620 (1m south of original y=520)
     DIRECTION: looking-north
     SHOWS: roof structure, floor levels, ceiling removal
     REFERENCES: section.svg for elevation view
     BILINGUAL-PAIR: th#section-line-aa
-->
<line id="section-line-aa" x1="100" y1="620" x2="900" y2="620"
      stroke="red" stroke-width="2" stroke-dasharray="10,5"/>
```

### 5. CALCULATION Documentation

```xml
<!-- CALCULATION: roof-ridge-height
     FORMULA: floor-to-ceiling + ceiling-to-eaves + eaves-to-ridge
     VALUES: 2.8m + 0.6m + 2.1m = 5.5m

     BREAKDOWN:
     - Floor to existing ceiling: 2.8m (not 3.0m - corrected)
     - Ceiling to eaves: 0.6m (600mm)
     - Eaves to ridge: 2.1m
     - TOTAL floor to ridge: 5.5m

     DESIGN-NOTE: Ceiling removed to expose full 5.5m height
     SOURCE: measured from photos (roof_01.JPG, roof_02.JPG)
-->
```

### 6. CROSS-REFERENCE Comments

```xml
<!-- CROSS-REFERENCE:
     - English version: drawings/en/proposed/plan.svg
     - Thai version: drawings/th/proposed/plan.svg
     - Section view: drawings/en/proposed/section.svg
     - Existing state: drawings/en/existing/plan.svg
     - Specifications: drawings/en/proposed/SPEC.md
     - Photos: photos/roof_01.JPG (shows actual ceiling height)
-->
```

### 7. CHANGE-LOG Comments

```xml
<!-- CHANGE-LOG:
     2025-10-30: Removed loft bedroom concept
                 - Deleted loft bed and person from y=290 level
                 - Updated label from "LOFT BEDROOM" to "(Exposed Roof Structure)"
                 - Corrected ceiling height from 3.0m to 2.8m
                 - Updated total ridge height from 5.0m to 5.5m
                 - Reason: Client decision to remove ceiling entirely

     2025-10-25: Added loft bedroom above open living space
                 - Added bed and person at y=290 (on beam)
                 - Label: "LOFT BEDROOM"
-->
```

## Best Practices

### 1. Placement
- File-level comments at top of `<svg>` element
- Element comments directly above the element they describe
- Group related elements with section comments

### 2. IDs and References
- Use consistent ID naming: `wall-west-exterior`, `room-open-living`
- Reference IDs in comments for relationships
- Use same IDs in bilingual pairs (enables automated sync checking)

### 3. Bilingual Consistency
- Always include `BILINGUAL-PAIR` references
- Use same IDs in both EN and TH files
- Only labels/text differ, coordinates/structure identical

### 4. Units
- Always specify units: `3.0m`, `800mm`, `100px`
- Document scale conversion: `1m = 100px`
- Show calculations with units: `2.8m + 0.6m + 2.1m = 5.5m`

### 5. Design Intent
- Document WHY not just WHAT
- Explain constraints and trade-offs
- Reference photos or measurements when applicable

### 6. Semantic Markup
- Use CSS classes for meaning: `.wall-exterior`, `.wall-interior`
- Document class semantics in comments
- Explain visual encoding: "black 8px = exterior, red 4px dashed = interior"

## Schema Validation

While these are human-readable comments, they follow a structured format that could be:
- Parsed by scripts for validation
- Used to generate documentation
- Checked for bilingual consistency
- Converted to machine-readable metadata later

## Example: Fully Annotated Wall

```xml
<!-- WALL: west-exterior-full
     ID: wall-west-exterior
     TYPE: exterior
     SEMANTIC: building-envelope
     CONSTRAINT: load-bearing, must-keep, cannot-modify
     COORDINATES:
       - Start: (100, 100) - northwest corner
       - End: (100, 840) - southwest corner
       - Length: 7.4m (740px)
     THICKNESS: centerline dimension (actual thickness ~200mm, not shown in plan)
     MATERIAL: concrete or brick (builder to specify)
     STRUCTURAL: yes - supports roof load
     ADJACENT-SPACES:
       - East: Open Living Space (interior)
       - West: exterior (building perimeter)
     CSS-CLASS: wall-exterior
     VISUAL: black solid line, 8px stroke
     BILINGUAL-PAIR: th/proposed/plan.svg#wall-west-exterior
     CROSS-REFS:
       - Section view: section.svg shows wall height (2.8m)
       - Existing: Same wall in existing/plan.svg (unchanged)
     DESIGN-NOTE: Critical load-bearing wall, forms west edge of building envelope
-->
<line id="wall-west-exterior" class="wall-exterior"
      x1="100" y1="100" x2="100" y2="840"
      stroke="black" stroke-width="8"/>
```

## Maintenance

- Update comments when editing SVG
- Keep CHANGE-LOG current
- Verify BILINGUAL-PAIR consistency
- Review comments during merge conflicts (they explain intent)

## Future Enhancements

Comments could be:
- Extracted to generate documentation
- Used for automated bilingual sync checking
- Converted to RDF/semantic web format
- Used as basis for CRDT implementation
- Parsed for dimension validation
