# Claude Context for mon-house

## Project Overview
mon-house is docs for a house renovation project.

wil need to be in English and translated to Thai, because the builder is Thai.

## Project Structure
This is a project located at `/Users/apple/workspace/go/src/github.com/joeblew999/mon-house`.

### Directory Structure
```
mon-house/
├── photos/
│   ├── before/      # Photos before renovation starts
│   ├── during/      # Progress photos during renovation
│   ├── after/       # Completed renovation photos
│   └── reference/   # Reference images for design ideas
├── docs/            # Documentation files
└── CLAUDE.md        # This file
```

## Documentation Approach
- **Visual Documentation**: Uses photos instead of CAD drawings
- **Photo Organization**: Photos organized by renovation phase (before/during/after)
- **Bilingual**: Documentation in English with Thai translations for the builder
- **Claude can read photos**: Upload photos and Claude can analyze them to help with documentation

## Photo Guidelines
- Name photos descriptively (e.g., `kitchen-sink-area-2025-10-23.jpg`)
- Include dates in filenames for tracking progress
- Take multiple angles of each area
- Include close-ups of specific issues or features
- Keep reference photos separate for design inspiration

## Documentation Guidelines
- Create Markdown files in the `docs/` folder
- Include both English and Thai versions
- Reference photos in documentation using relative paths
- Track timeline, budget, and contractor communications

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

