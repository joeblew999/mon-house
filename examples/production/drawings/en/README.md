# House Renovation Drawings

## phasng

1. outside

- high pressure cleaning from top to bottom.

2. inside bathroom

3. inside all

- make open plan

- make kitchen

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

## Building Overview

The house is a single-story structure with gable roof and loft potential. For precise measurements and coordinates, see the SPEC.md files in each folder:
- **[existing/SPEC.md](existing/SPEC.md)** - Current house dimensions and layout
- **[proposed/SPEC.md](proposed/SPEC.md)** - Renovation design dimensions and changes

### Key Concepts

**Building Envelope:** The exterior perimeter walls that define the building boundary - these are structural and must be maintained.

**Loft Potential:** The existing beam at ceiling level (3.0m height) can support a loft platform, creating a second level sleeping area while maintaining the open ground floor.

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

**What it shows:** The current house layout with 2 bedrooms, bathroom, corridor, living room, and kitchen.

**Files:**
- **[plan.svg](existing/plan.svg)** - Floor plan with semantic wall markup (EXTERIOR vs INTERIOR)
- **[section.svg](existing/section.svg)** - Vertical section showing roof, walls, beam, and foundation
- **[SPEC.md](existing/SPEC.md)** - Complete technical specifications and measurements

**Key features:** All walls are tagged to show which are structural (EXTERIOR - must keep) vs removable (INTERIOR - can demo).

### proposed/

**What it shows:** The renovation design with open living space and loft bedroom.

**Files:**
- **[plan.svg](proposed/plan.svg)** - Floor plan with interior walls removed, bathroom maintained
- **[section.svg](proposed/section.svg)** - Vertical section showing loft bedroom platform on beam
- **[SPEC.md](proposed/SPEC.md)** - Complete specifications including what was removed vs maintained

**Key changes:**
- Ground floor: Open living space (former bedrooms with all interior walls removed)
- Upper level: Loft bedroom platform supported by existing beam
- Bathroom: Fully maintained for privacy
- Living room & kitchen: Unchanged