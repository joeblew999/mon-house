# Technical Specification - Existing Design

**AUTO-GENERATED FROM SVG DRAWINGS - DO NOT EDIT MANUALLY**

This specification is derived by AI from reading the tagged SVG elements in `existing/plan.svg` and `existing/section.svg` according to the vocabulary defined in `code/drawing-standards.json`.

---

## Scale

All measurements use the standard scale: **1 meter = 100 pixels**

---

## Building Envelope (Exterior Walls)

The building perimeter is defined by the following exterior walls (tagged as `wall-exterior`):

### Plan View - Perimeter Coordinates

**North Perimeter:**
- Bedroom 1 top wall: (100, 100) → (400, 100) = 3.0m width
- Living Room top wall: (400, 200) → (690, 200) = 2.9m width
- Kitchen top wall: (690, 540) → (890, 540) = 2.0m width (connects at different y-level)

**West Perimeter:**
- Left wall full height: (100, 100) → (100, 840) = 7.4m total depth
  - Bedroom 1 section: (100, 100) → (100, 440) = 3.4m
  - Bedroom 2 section: (100, 440) → (100, 740) = 3.0m
  - Bathroom section: (100, 740) → (100, 840) = 1.0m

**East Perimeter:**
- Bedroom 1 right wall (upper): (400, 100) → (400, 200) = 1.0m (EXTERIOR section only)
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
- Building footprint: Irregular L-shaped plan

---

## Interior Partitions (Interior Walls)

Non-structural partition walls (tagged as `wall-interior`):

**Between Bedrooms:**
- Wall between Bedroom 1 and Bedroom 2: (100, 440) → (400, 440) = 3.0m width

**Bedroom/Living Room Shared:**
- Bedroom 1 right wall (lower section): (400, 200) → (400, 440) = 2.4m height (backed by living room wall)

**Bedroom/Corridor Shared:**
- Bedroom 2 right wall: (400, 440) → (400, 740) = 3.0m height

**Bedroom/Bathroom Shared:**
- Wall between Bedroom 2 and Bathroom: (100, 740) → (290, 740) = 1.9m width
- Wall between Bathroom and Corridor: (290, 740) → (400, 740) = 1.1m width

**Bathroom/Corridor:**
- Bathroom right wall: (290, 740) → (290, 840) = 1.0m height

**Living Room/Corridor:**
- Corridor right wall (shared with living room): (400, 740) → (400, 840) = 1.0m height

---

## Room Dimensions and Floor Areas

### Bedroom 1
- **Size:** 3.0m (width) × 3.4m (depth)
- **Floor Area:** 10.2 m²
- **Perimeter walls:** North (EXTERIOR), West (EXTERIOR), partial East (1.0m EXTERIOR + 2.4m INTERIOR), South (INTERIOR)
- **Windows:** 2 windows
  - Window 1: Top wall, 0.8m wide × 0.08m frame thickness
  - Window 2: Left wall, 0.8m tall × 0.08m frame thickness
- **Door:** Right wall, 0.6m opening width, swings into bedroom
- **Ceiling Height:** 2.8m (from section view)

### Bedroom 2
- **Size:** 3.0m (width) × 3.0m (depth)
- **Floor Area:** 9.0 m²
- **Perimeter walls:** West (EXTERIOR), all others INTERIOR
- **Windows:** None
- **Door:** Right wall, 0.6m opening width, swings into bedroom
- **Ceiling Height:** 2.8m (from section view)

### Bathroom
- **Size:** 1.9m (width) × 1.0m (depth)
- **Floor Area:** 1.9 m²
- **Perimeter walls:** West (EXTERIOR), South-left (EXTERIOR)
- **Partition walls:** North (INTERIOR shared with Bedroom 2), East (INTERIOR shared with corridor)
- **Door:** East wall, 0.6m opening width, swings into bathroom
- **Ceiling Height:** 2.8m (from section view)

### Corridor
- **Size:** 1.1m (width) × 1.0m (depth)
- **Floor Area:** 1.1 m²
- **Perimeter walls:** South (EXTERIOR)
- **Partition walls:** All others INTERIOR (North shared with Bedroom 2, East shared with Living Room, West shared with Bathroom)
- **Function:** Connects bathroom to living spaces
- **Ceiling Height:** 2.8m (from section view)

### Living Room
- **Size:** 2.9m (width) × 6.4m (depth)
- **Floor Area:** 18.56 m²
- **Perimeter walls:** North (EXTERIOR), East (EXTERIOR), South (EXTERIOR)
- **Partition walls:** West wall shared with bedrooms/corridor (INTERIOR)
- **Opening:** 3.0m wide opening to kitchen (no separating wall between y=540 and y=840)
- **Ceiling Height:** 2.8m (from section view)

### Kitchen
- **Size:** 2.0m (width) × 3.0m (depth)
- **Floor Area:** 6.0 m²
- **Perimeter walls:** North (EXTERIOR), East (EXTERIOR), South (EXTERIOR)
- **Opening:** 3.0m wide opening to living room (no separating wall on west side)
- **Ceiling Height:** 2.8m (from section view)

---

## Total Floor Areas

- **Total Conditioned Space:** 46.76 m²
  - Bedroom 1: 10.2 m²
  - Bedroom 2: 9.0 m²
  - Bathroom: 1.9 m²
  - Corridor: 1.1 m²
  - Living Room: 18.56 m²
  - Kitchen: 6.0 m²

---

## Section A-A (Vertical Dimensions)

Section cut location: **y=620** (plan view), cutting horizontally through living room and kitchen area

View direction: **Looking south** (arrows point downward on section cut line)

### Vertical Heights (Floor to Key Elements)

- **Floor Level:** y=600 (0.0m datum)
- **Suspended Ceiling:** y=320 (2.8m above floor)
- **Structural Beam (Eave Level):** y=245 to y=260 (3.4m above floor, beam centerline at 3.4m)
- **Ridge (Roof Peak):** y=110 (4.9m above floor)

### Height Measurements

- **Floor to Ceiling:** 2.8m (280px) - suspended ceiling height
- **Floor to Eave (Beam Top):** 3.4m (340px) - eave level where roof meets walls
- **Floor to Ridge:** 4.9m (490px) - peak of gable roof

### Structural Elements - Section View

**Foundation:**
- Type: Concrete slab
- Coordinates: (200, 600) to (990, 630)
- Width: 7.9m (full building width in section)
- Depth: 0.3m (30px)
- Tagged as: `foundation`

**Exterior Walls (Vertical):**
- Left exterior wall: (200, 320) → (200, 600) = 2.8m height (to ceiling)
- Right exterior wall: (990, 320) → (990, 600) = 2.8m height (to ceiling)
- Thickness: 0.08m (8px stroke width)
- Tagged as: `wall-exterior`

**Structural Beam:**
- Location: (200, 245) → (990, 245) top surface
- Span: 7.9m (790px) - full width of building
- Depth: 0.15m (15px) - beam height from y=245 to y=260
- Elevation: 3.4m above floor (eave level)
- Purpose: Supports roof structure at eave level
- Material: Wood (fill: #8B4513)
- Tagged as: `beam`
- Note: Beam centerline at eave height (3.4m)

**Suspended Ceiling:**
- Location: y=320 (2.8m above floor)
- Type: Horizontal suspended ceiling hung from beam structure above
- Coordinates: (200, 320) → (990, 320)
- Span: 7.9m
- Visual: Gray dashed line (stroke-dasharray: 5,5)
- Note: Creates 2.8m clear ceiling height below 3.4m eave/beam level

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

### Reference Elements (Scale and Context)

**Bed (Furniture):**
- Coordinates: (250, 550) → (450, 600)
- Dimensions: 2.0m length × 0.5m height
- Location: Bedroom 1 area
- Tagged as: `furniture`

**Human Figure (Scale Reference):**
- Position: x=500, standing on floor (y=600)
- Height: 1.7m (170px) - standard standing height
- Head center: (500, 445)
- Components: Circle head (r=15px) + stick body
- Purpose: Provides human scale reference for vertical dimensions
- Tagged as: `human-scale`

---

## Openings

### Doors

All doors are 0.6m wide openings with swing arcs (tagged as `door` and `door-arc`):

1. **Bedroom 1 Door**
   - Location: Right wall (x=400), y=260 to y=320
   - Width: 0.6m (60px opening)
   - Swing: Into bedroom (arc swings west)

2. **Bedroom 2 Door**
   - Location: Right wall (x=400), y=560 to y=620
   - Width: 0.6m (60px opening)
   - Swing: Into bedroom (arc swings west)

3. **Bathroom Door**
   - Location: Right wall (x=290), y=760 to y=820
   - Width: 0.6m (60px opening)
   - Swing: Into bathroom (arc swings west)

### Windows

All windows tagged as `window`:

1. **Bedroom 1 Window 1**
   - Location: Top wall (north), centered at x=190, y=96 to y=104
   - Dimensions: 0.8m wide × 0.08m frame thickness
   - Visual: Blue stroke, lightblue fill, 50% opacity

2. **Bedroom 1 Window 2**
   - Location: Left wall (west), centered at y=240, x=96 to x=104
   - Dimensions: 0.8m tall × 0.08m frame thickness
   - Visual: Blue stroke, lightblue fill, 50% opacity

### Openings Between Rooms

**Living Room to Kitchen Opening:**
- Location: East wall of Living Room (x=690), y=540 to y=840
- Width: 3.0m (300px)
- Type: Open passage (no door)
- Note: No separating wall between these spaces

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
- Cuts through: Bedroom 1, Living Room, and Kitchen areas
- Reveals: Vertical relationships, roof structure, ceiling height, beam location, foundation
- Building width in section: 7.9m (from west exterior wall to east exterior wall at kitchen)

---

## Building Envelope Components

### Vertical Envelope (Exterior Walls)
- **Material:** Structural walls (specific material not specified in drawing)
- **Total perimeter:** Irregular L-shaped footprint
- **Height:** 2.8m to ceiling level (walls), 3.4m to eave level (full structure)
- **Function:** Load-bearing, weather barrier

### Top Envelope (Roof)
- **Type:** Gable roof with symmetrical slopes
- **Ridge height:** 4.9m above floor
- **Eave height:** 3.4m above floor
- **Overhang:** 0.6m on both sides
- **Structural support:** Beam at eave level (3.4m)
- **Function:** Weather barrier, structural roof system

### Bottom Envelope (Foundation)
- **Type:** Concrete slab
- **Thickness:** 0.3m
- **Width:** 7.9m (in section view)
- **Function:** Structural foundation, weather barrier, slab-on-grade

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
**Source Files:** `drawings/en/existing/plan.svg`, `drawings/en/existing/section.svg`
**Standards:** `code/drawing-standards.json` v1.0
