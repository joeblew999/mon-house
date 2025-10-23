# House Renovation Drawings

## Design Intent

**Renovation Goal**: Convert existing 2-bedroom house into open-plan design with loft bedroom

**Key Changes**:
- Remove existing Bedroom 1 and Bedroom 2 **interior** walls:
  - Remove wall between Bedroom 1 and Bedroom 2 (y=440, INTERIOR)
  - Remove wall between Bedroom 2 and Bathroom (y=740, x=100 to x=290, INTERIOR)
  - Remove interior portion of right wall shared with Living Room (x=400, y=200 to y=440, INTERIOR)
  - Remove interior portion of right wall shared with Corridor (x=400, y=440 to y=740, INTERIOR)
  - **KEEP** 1.0m exterior wall section (x=400, y=100 to y=200, EXTERIOR - building perimeter)
- Keep all **exterior** walls:
  - Top wall (y=100, EXTERIOR)
  - Left wall (x=100, EXTERIOR)
  - South wall including 1.1m corridor section (y=840, EXTERIOR)
- Create open-plan living space (3.0m × 7.4m total height)
- Add loft bedroom above at 3.0m level (on beam)
- Loft provides sleeping area while maximizing ground floor space
- **MAINTAIN existing bathroom** (1.9m × 1.0m) - all walls kept
- Remove corridor walls (now open to living room and open living space)

## Assumptions

### Building Structure
- Single floor house with loft potential
- Floor to ceiling height: 3.0 m
- Gable ridge line runs top to bottom (north-south direction based on plan)
- Ridge height from floor: 5.0 m
- Eaves overhang: 0.6 m (600 mm)

### Floor Plan Dimensions
- Total width (east-west): 7.9 m
  - Bedroom 1: 3.0 m width
  - Living Room: 2.9 m width
  - Kitchen: 2.0 m width
- Total depth (north-south): 7.4 m
  - Bedroom 1: 3.4 m depth
  - Bedroom 2: 3.0 m depth
  - Bathroom: 1.0 m depth
- Corridor: 1.1 m width
- Bathroom: 1.9 m depth
- Living Room height: 6.4 m (spans from Bedroom 1 level to bottom)
- Kitchen height: 3.0 m
- Opening between Living Room and Kitchen: 3.0 m

### Structural Elements
- Beam height: 3.0 m (at ceiling level, spans full width)
- Beam depth: 100 mm (0.1 m)
- Foundation: Concrete slab

### Reference Elements (for scale)
- Bed size: 1.0 m x 2.0 m (single bed)
- Bed height: ~0.5 m
- Person height: 1.7 m (standing adult)

### Drawing Standards
- Scale: 1 meter = 100 pixels in SVG
- Centerline dimensions used throughout
- Chained dimensions on all exterior walls
- Section cut labeled as "SECTION A-A" at y=620 (moved 1m south from original position)

### Wall Types (Semantic Markup)
**EXTERIOR walls** (black, 8px solid):
- Building perimeter / envelope
- Must be maintained in renovation
- Examples: north wall (y=100), west wall (x=100), south wall (y=840)

**INTERIOR walls** (red, 4px dashed):
- Internal partitions
- Can be removed in renovation
- Examples: wall between bedrooms (y=440), bedroom/bathroom wall (y=740)

## Drawings

### existing/
Contains drawings of the current house layout:

**plan.svg** - Existing floor plan showing:
- 2 bedrooms, bathroom, corridor, living room, kitchen
- **Semantic wall markup**: EXTERIOR walls (black solid), INTERIOR walls (red dashed)
- All rooms with dimensions
- Doors and windows
- Section cut line at y=620 (red dashed line showing where section is taken)
- Chained dimensions on all four exterior walls

**section.svg** - Section A-A showing existing structure:
- 3.0 m wall height
- 5.0 m ridge height
- Gable roof with 0.6 m eaves overhang
- Beam at 3.0 m ceiling level (100mm deep)
- Ground floor level with bed and person for scale
- Concrete slab foundation
- All vertical and horizontal dimensions

### proposed/
Contains drawings of the proposed renovation:

**plan.svg** - Proposed floor plan showing:
- Open living space (3.0m × 7.4m) replacing Bedrooms 1 & 2
- **Bathroom maintained** (1.9m × 1.0m) - all walls kept
- Corridor walls removed - now open plan
- Only EXTERIOR walls kept from bedroom area:
  - Top wall (y=100)
  - Left wall (x=100, full height)
  - 1.0m wall section (x=400, y=100 to y=200) - building perimeter
  - South wall (y=840) including 1.1m corridor section
- All INTERIOR walls removed
- Living room and kitchen unchanged
- Section cut line at y=620
- Note: "(Loft Bedroom Above)" indicating vertical space usage

**section.svg** - Proposed Section A-A showing:
- Same structure as existing
- Open living space at ground level
- Loft bedroom above at 3.0m level on beam
- Labels updated to show "OPEN LIVING SPACE" and "LOFT BEDROOM"