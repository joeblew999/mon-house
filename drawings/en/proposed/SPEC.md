# Technical Specification - Proposed Design

**AUTO-GENERATED FROM SVG DRAWINGS - DO NOT EDIT MANUALLY**

This specification is derived by AI from reading the tagged SVG elements in `proposed/plan.svg` and `proposed/section.svg` according to the vocabulary defined in `code/drawing-standards.json`.

---

## Scale

All measurements use the standard scale: **1 meter = 100 pixels**

---

## Building Envelope (Exterior Walls)

The building perimeter is defined by the following exterior walls (tagged as `wall-exterior`):

### Plan View - Perimeter Coordinates

**North Perimeter:**
- Open Living Space top wall: (100, 100) → (400, 100) = 3.0m width
- Living Room top wall: (400, 200) → (690, 200) = 2.9m width
- Kitchen top wall: (690, 540) → (890, 540) = 2.0m width (connects at different y-level)

**West Perimeter:**
- Left wall full height: (100, 100) → (100, 840) = 7.4m total depth
  - Open Living Space section: (100, 100) → (100, 740) = 6.4m
  - Bathroom section: (100, 740) → (100, 840) = 1.0m

**East Perimeter:**
- Open Living Space right wall (upper): (400, 100) → (400, 200) = 1.0m (EXTERIOR section only)
- Living Room right wall: (690, 200) → (690, 540) = 3.4m
- Kitchen right wall: (890, 540) → (890, 840) = 3.0m

**South Perimeter:**
- Bathroom bottom wall: (100, 840) → (290, 840) = 1.9m
- Corridor south wall: (290, 840) → (400, 840) = 1.1m
- Living Room bottom wall: (400, 840) → (690, 840) = 2.9m
- Kitchen bottom wall: (690, 840) → (890, 840) = 2.0m

**Total Building Dimensions:**
- Maximum width (east-west): 7.9m (at various y-levels)
- Maximum depth (north-south): 7.4m
- Building footprint: Irregular L-shaped plan (identical to existing)

---

## Interior Partitions (Interior Walls)

Non-structural partition walls (tagged as `wall-interior`):

**MAJOR CHANGE:** All interior bedroom partition walls have been REMOVED to create open living space.

**Remaining Interior Partitions:**

**Bathroom Partitions (MAINTAINED):**
- Bathroom top wall: (100, 740) → (290, 740) = 1.9m width (INTERIOR)
- Bathroom right wall: (290, 740) → (290, 840) = 1.0m height (INTERIOR)

**All Other Interior Walls REMOVED:**
- Former wall between Bedroom 1 and Bedroom 2: REMOVED
- Former Bedroom 1 right wall (lower section): REMOVED
- Former Bedroom 2 right wall: REMOVED
- Former wall between Bedroom 2 and Bathroom: NOW part of bathroom top wall
- Former Corridor/Living Room partition: REMOVED

---

## Room Dimensions and Floor Areas

### Open Living Space (Former Bedrooms 1 & 2)
- **Size:** 3.0m (width) × 7.4m (depth)
- **Floor Area (Ground Level):** 22.2 m²
- **Perimeter walls:** North (EXTERIOR), West (EXTERIOR), partial East (1.0m EXTERIOR at top)
- **Windows:** 2 windows (maintained from Bedroom 1)
  - Window 1: Top wall, 0.8m wide × 0.08m frame thickness
  - Window 2: Left wall, 0.8m tall × 0.08m frame thickness
- **Doors:** REMOVED (all bedroom doors removed for open plan)
- **Ground Floor Ceiling Height:** 2.8m (from section view)
- **Loft Above:** Loft bedroom platform occupies portion of vertical space above ceiling
- **Design Change:** Combines former Bedroom 1 (3.0m × 3.4m) and Bedroom 2 (3.0m × 3.0m) by removing all interior walls

### Loft Bedroom (Above Open Living Space)
- **Platform Extent:** x=200 to x=550, positioned at y=320 (sits on suspended ceiling)
- **Platform Length:** 3.5m (350px) extending from west wall toward center
- **Platform Width:** 3.0m (same as Open Living Space width below)
- **Loft Floor Area:** 10.5 m² (3.5m × 3.0m)
- **Loft Floor Elevation:** 2.8m above ground floor (sits on suspended ceiling)
- **Headroom Calculation:**
  - At platform edge (x=200, west wall): 4.9m ridge - 2.8m loft floor = 2.1m headroom (adequate)
  - At platform end (x=550, extended toward center): Distance from ridge center (x=595) = 45px = 0.45m
    - Roof height at x=550: Interpolated between eave (3.4m at x=200) and ridge (4.9m at x=595)
    - Rise from eave to ridge: 4.9m - 3.4m = 1.5m over run of 3.95m
    - Rise at x=550 (3.5m from west wall): (3.5m / 3.95m) × 1.5m = 1.33m above eave = 4.73m total height
    - Headroom at x=550: 4.73m - 2.8m = 1.93m (adequate for bed, slightly tight for standing)
  - Person positioned at x=500 (on loft): Head at y=165 (1.65m above loft floor + 2.8m loft elevation = 4.45m total)
    - Roof at x=500: (3.0m from west wall / 3.95m run) × 1.5m rise = 1.14m above eave = 4.54m total
    - Clearance: 4.54m roof - 4.45m head = 0.09m (90mm) - TIGHT but person not standing fully upright
- **Furniture:** Bed (1.0m × 2.0m) positioned at x=350 to x=550, on loft platform
- **Access:** Not shown in drawings (ladder or stairs required)
- **Design Feature:** Platform positioned toward ridge center to maximize headroom under sloped roof

### Bathroom
- **Size:** 1.9m (width) × 1.0m (depth)
- **Floor Area:** 1.9 m²
- **Perimeter walls:** West (EXTERIOR), South-left (EXTERIOR)
- **Partition walls:** North (INTERIOR - maintained), East (INTERIOR - maintained)
- **Door:** East wall, 0.6m opening width, swings into bathroom (MAINTAINED)
- **Ceiling Height:** 2.8m (from section view)
- **Design Change:** UNCHANGED from existing design

### Living Room
- **Size:** 2.9m (width) × 6.4m (depth)
- **Floor Area:** 18.56 m²
- **Perimeter walls:** North (EXTERIOR), East (EXTERIOR), South (EXTERIOR)
- **Partition walls:** None (fully open to corridor area and kitchen)
- **Opening:** 3.0m wide opening to kitchen (no separating wall between y=540 and y=840)
- **Ceiling Height:** 2.8m (from section view)
- **Design Change:** UNCHANGED from existing design

### Kitchen
- **Size:** 2.0m (width) × 3.0m (depth)
- **Floor Area:** 6.0 m²
- **Perimeter walls:** North (EXTERIOR), East (EXTERIOR), South (EXTERIOR)
- **Opening:** 3.0m wide opening to living room (no separating wall on west side)
- **Ceiling Height:** 2.8m (from section view)
- **Design Change:** UNCHANGED from existing design

---

## Total Floor Areas

- **Ground Floor Total:** 48.66 m²
  - Open Living Space: 22.2 m² (replaces Bedroom 1: 10.2 m² + Bedroom 2: 9.0 m² + Corridor: 1.1 m² + partial wall removal gains)
  - Bathroom: 1.9 m²
  - Living Room: 18.56 m²
  - Kitchen: 6.0 m²

- **Loft Floor:** 10.5 m²
  - Loft Bedroom: 10.5 m² (new addition above Open Living Space)

- **Total Conditioned Space (Ground + Loft):** 59.16 m²

**Floor Area Comparison vs. Existing:**
- Existing total: 46.76 m² (single level)
- Proposed total: 59.16 m² (ground floor 48.66 m² + loft 10.5 m²)
- Net increase: +12.4 m² (+26.5% increase)
- Increase source: Loft bedroom addition (10.5 m²) + corridor/partition removal efficiency gains (1.9 m²)

---

## Section A-A (Vertical Dimensions)

Section cut location: **y=620** (plan view), cutting horizontally through open living space, living room, and kitchen area

View direction: **Looking south** (arrows point downward on section cut line)

### Vertical Heights (Floor to Key Elements)

- **Floor Level:** y=600 (0.0m datum)
- **Suspended Ceiling:** y=320 (2.8m above floor)
- **Loft Platform:** y=320 (sits on suspended ceiling at 2.8m elevation)
- **Structural Beam (Eave Level):** y=245 to y=260 (3.4m above floor, beam depth 150mm)
- **Eave Height:** y=260 (3.4m above floor) - top of beam, where exterior walls meet roof
- **Ridge (Roof Peak):** y=110 (4.9m above floor)

### Height Measurements

- **Floor to Ceiling (Ground Floor):** 2.8m (280px) - suspended ceiling height
- **Floor to Loft Platform:** 2.8m (280px) - loft floor sits on suspended ceiling
- **Floor to Eave (Beam Top):** 3.4m (340px) - eave level where roof meets walls
- **Floor to Ridge:** 4.9m (490px) - peak of gable roof
- **Loft Floor to Ridge:** 2.1m (210px) at west wall, decreasing toward east due to roof slope
- **Loft Headroom:** Variable from 2.1m (at west wall) to 1.93m (at platform edge x=550)

### Structural Elements - Section View

**Foundation:**
- Type: Concrete slab
- Coordinates: (200, 600) to (990, 630)
- Width: 7.9m (full building width in section)
- Depth: 0.3m (30px)
- Tagged as: `foundation`
- **Design Change:** UNCHANGED from existing

**Exterior Walls (Vertical):**
- Left exterior wall: (200, 260) → (200, 600) = 3.4m height (to eave level)
- Right exterior wall: (990, 260) → (990, 600) = 3.4m height (to eave level)
- Thickness: 0.08m (8px stroke width)
- Tagged as: `wall-exterior`
- **Design Change:** Height extended from 2.8m to 3.4m (now reach full eave height instead of ceiling height)

**Structural Beam:**
- Location: (200, 245) → (990, 245) top surface
- Span: 7.9m (790px) - full width of building
- Depth: 0.15m (15px) - beam from y=245 to y=260
- Elevation: 3.4m above floor (eave level)
- Purpose: Supports roof structure AND loft platform load
- Material: Wood (fill: #8B4513)
- Tagged as: `beam`
- **Design Change:** Now serves dual purpose - roof support AND loft platform support

**Suspended Ceiling:**
- Location: y=320 (2.8m above floor)
- Type: Horizontal suspended ceiling hung from beam structure above
- Coordinates: (200, 320) → (990, 320) full width shown
- Span: 7.9m (in section view)
- Visual: Gray dashed line (stroke-dasharray: 5,5)
- **Design Change:** NOW serves as LOFT PLATFORM floor structure (load-bearing function added)

**Loft Platform:**
- Location: (200, 320) → (550, 320)
- Length: 3.5m (350px) extending from west wall toward building center
- Elevation: 2.8m above ground floor
- Structure: Sits on suspended ceiling system, hung from structural beam at 3.4m
- Load path: Platform → ceiling structure → beam (3.4m) → exterior walls → foundation
- Visual: Black solid line (stroke-width: 4px)
- **Design Change:** NEW ELEMENT - loft platform added for bedroom above

**Gable Roof:**
- Type: Symmetrical gable roof
- Ridge location: x=595 (center of building), y=110 (4.9m above floor)
- Left roof slope: (140, 260) → (595, 110)
  - Eave at: (140, 260) with 0.6m overhang beyond wall at x=200
  - Rise: 4.9m - 3.4m = 1.5m
  - Run: 595 - 200 = 3.95m (to center)
  - Slope: 1.5m / 3.95m = 0.38 (approx. 20.8° angle)
- Right roof slope: (595, 110) → (1050, 260)
  - Eave at: (1050, 260) with 0.6m overhang beyond wall at x=990
  - Rise: 4.9m - 3.4m = 1.5m
  - Run: 990 - 595 = 3.95m (to center)
  - Slope: 1.5m / 3.95m = 0.38 (approx. 20.8° angle)
- Eave overhang: 0.6m (60px) on both left and right sides
- Tagged as: `roof`
- **Design Change:** UNCHANGED geometry, but now creates usable loft space below

### Reference Elements (Scale and Context)

**Bed (Furniture in Loft):**
- Coordinates: (350, 270) → (550, 320)
- Dimensions: 2.0m length × 0.5m height (side view)
- Location: Loft bedroom, positioned on loft platform at y=320
- Positioning: Bed placed toward center (higher headroom area) under roof slope
- Tagged as: `furniture`
- **Design Change:** NEW - bed relocated to loft bedroom

**Human Figure (Scale Reference):**
- Position: x=500, standing on loft platform (y=320)
- Height: 1.7m (170px) - standard standing height
- Head center: (500, 165) = 1.65m above loft floor
- Loft floor elevation: 2.8m above ground floor
- Total head height: 4.45m above ground floor
- Roof clearance at x=500: 4.54m - 4.45m = 0.09m (90mm clearance) - tight but passable
- Components: Circle head (r=15px) + stick body + arms + legs
- Purpose: Demonstrates loft headroom and human scale on platform
- Tagged as: `human-scale`
- **Design Change:** NEW - figure relocated to loft to demonstrate headroom

---

## Openings

### Doors

**MAJOR CHANGE:** Bedroom doors REMOVED for open plan design.

**Remaining Door:**

1. **Bathroom Door (MAINTAINED)**
   - Location: Right wall (x=290), y=760 to y=820
   - Width: 0.6m (60px opening)
   - Swing: Into bathroom (arc swings west)

**Doors Removed:**
- Former Bedroom 1 Door: REMOVED (open plan)
- Former Bedroom 2 Door: REMOVED (open plan)

### Windows

All windows tagged as `window`:

**MAINTAINED from Existing:**

1. **Window 1 (North Wall)**
   - Location: Top wall (north), centered at x=190, y=96 to y=104
   - Dimensions: 0.8m wide × 0.08m frame thickness
   - Visual: Blue stroke, lightblue fill, 50% opacity
   - **Design Change:** UNCHANGED position, now serves Open Living Space

2. **Window 2 (West Wall)**
   - Location: Left wall (west), centered at y=240, x=96 to x=104
   - Dimensions: 0.8m tall × 0.08m frame thickness
   - Visual: Blue stroke, lightblue fill, 50% opacity
   - **Design Change:** UNCHANGED position, now serves Open Living Space

### Openings Between Rooms

**Living Room to Kitchen Opening (MAINTAINED):**
- Location: East wall of Living Room (x=690), y=540 to y=840
- Width: 3.0m (300px)
- Type: Open passage (no door)
- Note: No separating wall between these spaces

**Open Living Space (NEW CONCEPT):**
- Former Bedroom 1, Bedroom 2, and Corridor are now one continuous open space
- No interior doors or partitions
- Loft bedroom above accessible via ladder/stairs (not shown)

---

## Section Cut Reference

**Section A-A:**
- Cut location on plan: y=620 (horizontal line across full width)
- Cut extent: x=100 to x=890 (7.9m width)
- View direction: South (looking down from north)
- Section markers: Red arrows at both ends pointing downward
- Label: "SECTION A-A" displayed at cut line
- Tagged as: `section-cut`, `section-arrow`, `section-label`

**What Section Shows:**
- Cuts through: Open Living Space, Loft Bedroom, Living Room, and Kitchen areas
- Reveals: Vertical relationships, roof structure, ceiling height, beam location, loft platform, foundation
- Building width in section: 7.9m (from west exterior wall to east exterior wall at kitchen)
- **Design Change:** Now shows loft bedroom platform and vertical spatial relationship

---

## Building Envelope Components

### Vertical Envelope (Exterior Walls)
- **Material:** Structural walls (specific material not specified in drawing)
- **Total perimeter:** Irregular L-shaped footprint
- **Height:** 3.4m to eave level (full height from floor to roof structure)
- **Function:** Load-bearing, weather barrier
- **Design Change:** Wall height shown to full eave level (3.4m) rather than ceiling level (2.8m)

### Top Envelope (Roof)
- **Type:** Gable roof with symmetrical slopes
- **Ridge height:** 4.9m above floor
- **Eave height:** 3.4m above floor
- **Overhang:** 0.6m on both sides
- **Structural support:** Beam at eave level (3.4m)
- **Function:** Weather barrier, structural roof system
- **Design Change:** UNCHANGED geometry, but now encloses loft space as usable area

### Bottom Envelope (Foundation)
- **Type:** Concrete slab
- **Thickness:** 0.3m
- **Width:** 7.9m (in section view)
- **Function:** Structural foundation, weather barrier, slab-on-grade
- **Design Change:** UNCHANGED

---

## Key Design Changes from Existing Design

### 1. Interior Layout Transformation
- **REMOVED:** All bedroom interior walls and doors
- **CREATED:** Open Living Space (22.2 m²) from former Bedrooms 1 & 2 + Corridor
- **MAINTAINED:** Bathroom as separate enclosed space
- **IMPACT:** Improved flow, better natural light distribution, modern open-plan living

### 2. Loft Bedroom Addition
- **ADDED:** Loft platform (3.5m × 3.0m = 10.5 m²) above Open Living Space
- **STRUCTURE:** Platform sits on suspended ceiling at 2.8m elevation, supported by structural beam at 3.4m
- **HEADROOM:** Variable 1.93m to 2.1m under sloped roof (adequate for sleeping area)
- **ACCESS:** Requires ladder or stairs (not shown in drawings)
- **IMPACT:** +10.5 m² additional floor area without increasing building footprint

### 3. Structural Modifications
- **BEAM FUNCTION:** Structural beam now serves dual purpose (roof support + loft load)
- **CEILING FUNCTION:** Suspended ceiling now serves as loft platform floor (load-bearing)
- **WALL HEIGHT:** Exterior walls shown to full 3.4m eave height (vs. 2.8m ceiling height in existing)
- **LOAD PATH:** Added load path for loft: platform → ceiling → beam → walls → foundation

### 4. Spatial Efficiency
- **FLOOR AREA GAIN:** +12.4 m² total (+26.5% increase)
  - Loft addition: +10.5 m²
  - Partition removal efficiency: +1.9 m²
- **CEILING VOLUME:** Open Living Space now has access to full 2.8m ceiling height without corridor compression
- **NATURAL LIGHT:** Two windows now serve larger Open Living Space instead of separate small bedrooms

### 5. Functional Changes
- **BEDROOM COUNT:** Reduced from 2 bedrooms (ground floor) to 1 bedroom (loft)
- **LIVING SPACE:** Increased open living/gathering space significantly
- **PRIVACY:** Reduced (open plan + loft bedroom less private than enclosed bedrooms)
- **FLEXIBILITY:** Increased (open space can be used for multiple functions)

### 6. Unchanged Elements
- **BUILDING ENVELOPE:** Identical footprint, roof geometry, foundation
- **BATHROOM:** Completely unchanged (size, location, fixtures)
- **LIVING ROOM & KITCHEN:** Unchanged (size, location, relationship)
- **WINDOWS:** Same positions and sizes (now serve different spaces)
- **ROOF STRUCTURE:** Same geometry (now encloses usable loft space)

---

## Drawing Standards Applied

This specification references the following element types from `code/drawing-standards.json`:

- **wall-exterior:** Building perimeter walls (structural, load-bearing, black 8px solid stroke)
- **wall-interior:** Interior partition walls (non-structural, removable, red 4px dashed stroke)
- **window:** Glazed openings (blue 4px stroke, lightblue fill, 50% opacity)
- **door:** Door openings with swing indication (brown 4px stroke)
- **door-arc:** Door swing arcs (brown 2px stroke)
- **dimension-line:** Measurement lines (blue 1px dashed stroke)
- **dimension-text:** Dimension labels (blue text, Arial 16px)
- **foundation:** Concrete slab foundation (gray #cccccc fill, black 2px stroke)
- **roof:** Roof structure (black 6px solid stroke)
- **beam:** Structural beam (wood #8B4513 fill, black 2px stroke)
- **furniture:** Context elements like bed (tan #D2B48C fill, black 2px stroke)
- **human-scale:** Human figure for scale reference (black 2px stroke, 1.7m standard height)
- **section-cut:** Section cut line (red 3px dashed stroke, dasharray 15,10)
- **section-arrow:** Section view direction arrows (red 3px stroke)
- **section-label:** Section identifier text (red bold Arial 14px)

---

**Generated:** 2025-10-30
**Source Files:** `drawings/en/proposed/plan.svg`, `drawings/en/proposed/section.svg`
**Standards:** `code/drawing-standards.json` v1.0
**Design:** Proposed renovation - open plan with loft bedroom addition
