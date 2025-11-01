# Shopping Directory

## Purpose

This directory contains product research, shopping links, and vendor-specific information for items specified in the parent `SPEC.md` file.

**Why separate?**
- Keep `SPEC.md` clean with requirements only (dimensions, specifications, placement)
- Product links and vendor info are volatile (prices change, products discontinued, links break)
- Shopping research can be updated independently without cluttering specifications
- Easier to compare products and track purchasing decisions

## Naming Pattern

**CRITICAL**: SPEC.md is the **SOURCE OF TRUTH** for naming. Shopping files MUST follow SPEC.md section structure.

**Pattern**: `{section-number}-{kebab-case-name}.md`

**Rules**:
1. **Source of truth**: Always check SPEC.md for the exact section number and title
2. Use the exact section number from SPEC.md (e.g., `2.1`, `2.15`, `3.2`)
3. Convert the SPEC.md section title to kebab-case (lowercase, hyphens for spaces)
4. Use `.md` extension for markdown files
5. If SPEC.md section is renumbered or renamed, shopping file MUST be renamed to match

**Examples**:

| SPEC.md Section | Shopping File |
|----------------|---------------|
| 2.1 Toilet | `2.1-toilet.md` |
| 2.2 Basin | `2.2-basin.md` |
| 2.5 Shower Taps | `2.5-shower-taps.md` |
| 2.11 Ventilation/Exhaust Fan | `2.11-ventilation-fan.md` |
| 2.14 Towel Bars/Hooks | `2.14-towel-bars-hooks.md` |
| 2.15 Toilet Paper Holder | `2.15-toilet-paper-holder.md` |

## File Structure

```
shopping/
├── README.md                      # This file
├── images/                        # Product images and screenshots
│   ├── 1-chaise-lounge-molesun.webp
│   ├── 2.1-toilet-cotto-option1.jpg
│   └── ... (organized by section number)
├── 1-chaise-lounge.md            # Chaise lounge product research
├── 2.1-toilet.md                 # Toilet product research
├── 2.2-basin.md                  # Basin product research
├── 2.3-shower.md                 # Shower product research
├── 2.4-shower-glass-panel.md     # Glass panel product research
├── 2.5-shower-taps.md            # Shower taps product research
├── 2.6-basin-taps.md             # Basin taps product research
├── 2.7-mirror.md                 # Mirror product research
├── 2.8-floor-tiles.md            # Floor tiles product research
├── 2.9-wall-tiles.md             # Wall tiles product research
├── 2.10-water-heater.md          # Water heater product research
├── 2.11-ventilation-fan.md       # Ventilation fan product research
├── 2.12-lighting.md              # Lighting fixtures product research
├── 2.13-bathroom-door.md         # Bathroom door product research
├── 2.14-towel-bars-hooks.md      # Towel bars/hooks product research
├── 2.15-toilet-paper-holder.md   # Toilet paper holder product research
├── 2.16-bathroom-accessories.md  # Bathroom accessories product research
├── 2.17-floor-drain.md           # Floor drain product research
└── 2.18-electrical-outlets.md    # Electrical outlets product research
```

## Product Images

Product images should be stored in the `images/` subdirectory with the following naming convention:

**Pattern**: `{section-number}-{product-name}-{variant}.{ext}`

**Examples**:
- `1-chaise-lounge-molesun.webp` - Chaise lounge from Molesun
- `2.1-toilet-cotto-option1.jpg` - COTTO toilet option 1
- `2.1-toilet-toto-premium.jpg` - TOTO toilet premium option
- `2.5-shower-taps-chrome.jpg` - Shower taps in chrome finish

**Image Guidelines**:
- Keep images reasonably sized (< 500KB when possible)
- Use descriptive filenames
- Include product variant/brand in filename
- Prefer `.webp` or `.jpg` formats
- Reference images in markdown files using: `![Description](images/filename.ext)`

---

## What to Include in Shopping Files

Each shopping file should contain:

1. **Product Research**
   - Links to products from Thailand vendors (HomePro, Global House, Boonthavorn, etc.)
   - Online shopping links (Lazada, Shopee, etc.)
   - Manufacturer product pages
   - Product images (stored in `images/` folder)

2. **Pricing Information**
   - Current prices in Thai Baht (THB)
   - Date of price check
   - Price range for different quality levels

3. **Vendor Comparison**
   - Which vendors stock the product
   - Delivery options
   - Warranty information

4. **Product Codes/SKUs**
   - Manufacturer part numbers
   - Vendor SKUs
   - Alternative product codes

5. **Notes**
   - Availability status
   - Lead times
   - Special ordering requirements
   - Installation considerations

## Example Shopping File Template

```markdown
# 2.X Product Name

**Last Updated**: YYYY-MM-DD

## Specification Reference
See [SPEC.md Section 2.X](../SPEC.md#2x-product-name) for detailed requirements.

## Product Options

### Option 1: [Brand/Model Name]
- **Link**: [URL]
- **Price**: ฿X,XXX (as of YYYY-MM-DD)
- **Vendor**: HomePro / Global House / etc.
- **SKU**: ABC-123
- **Notes**: [Availability, features, etc.]

### Option 2: [Brand/Model Name]
- **Link**: [URL]
- **Price**: ฿X,XXX (as of YYYY-MM-DD)
- **Vendor**: HomePro / Global House / etc.
- **SKU**: DEF-456
- **Notes**: [Availability, features, etc.]

## Recommended Choice
[Your recommendation with reasoning]

## Installation Notes
[Any special installation requirements or considerations]
```

## Guidelines for Humans

1. Always check SPEC.md first for requirements before researching products
2. Update prices with date stamps when checking vendors
3. Include multiple options when available (budget, mid-range, premium)
4. Note if products are discontinued or hard to find
5. Keep Thailand-specific vendor information (HomePro, Global House, Boonthavorn, etc.)

## Guidelines for AI (Claude)

**SPEC.md is the source of truth** - always start there:

1. **Before creating shopping files**: Read SPEC.md to get the exact section number and title
2. **Before renaming shopping files**: Check SPEC.md for the current section structure
3. When asked about products for a SPEC item, look for corresponding shopping file
4. When creating new shopping files, follow the naming pattern exactly from SPEC.md
5. Always include date stamps for price information
6. Cross-reference SPEC.md section numbers to ensure consistency
7. **If SPEC.md section is renamed/renumbered, shopping file MUST be renamed to match**
8. Prioritize Thailand-based vendors and local availability
9. Include both online (Lazada, Shopee) and physical store options (HomePro, Global House)

## Adding New Items

When a new section is added to SPEC.md:

1. Note the section number (e.g., `2.19`)
2. Convert section title to kebab-case (e.g., "Water Softener" → `water-softener`)
3. Create file: `2.19-water-softener.md`
4. Use the template above
5. Update this README if needed

## Maintenance

- Review shopping files quarterly to update prices and availability
- Mark discontinued products clearly
- Archive old product options if no longer relevant
- Keep the most current and available options at the top of each file
