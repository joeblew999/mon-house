---
title: House Exterior — Painting
status: Draft
rev: "2"
---

# House Exterior — Painting

DIY work, Joe + Mon. **Not part of any builder quote.** Repaint the existing house exterior — concrete walls, both sides of all four fence sections, and the timber eaves + fascia boards.

Old concrete plaster house, ~10 years old. Existing paint on all surfaces, dusty/chalky in places. Small cans preferred for easier handling.

Paint product metadata (coverage, price, SKU) is included inline below from the shared catalog partials. Cans are then computed for this spec's surface areas using `cans = ceil(area ÷ effective_m²_per_can)` — see [PAINT.md](PAINT.md) for the formula and project-wide rollup.

<!-- include: _partials/paint-concrete.md -->

<!-- include: _partials/paint-timber.md -->

---

## Scope

Three substrates, all on the existing house exterior + perimeter:

- **Concrete walls** — four exterior walls of the house (minus windows + sliding door).
- **Concrete fence** — four perimeter fence sections, both sides.
- **Timber** — eaves and fascia boards around the roof line.

Out of scope: roof metalwork (see [ROOF.md](ROOF.md)), gate metalwork (see [GATE-01.md](GATE-01.md)), driveway (see [CONCRETE.md](CONCRETE.md)), interior painting.

---

## Dimensions

### House (concrete walls)

| Wall | Length (m) | Height (m) | Area (m²) |
|---|---|---|---|
| Front | 5 | 3 | 15 |
| Back | 5 | 3 | 15 |
| Left | 5 | 3 | 15 |
| Right | 5 | 3 | 15 |
| Minus 5x windows (1.29x1.06) | | | -6.8 |
| Minus 1x bathroom window (0.6x0.4) | | | -0.2 |
| Minus 1x sliding door (1.725x1.98) | | | -3.4 |
| **House walls total** | | | **49.6** |

### Fence (concrete) — both sides

| Section | Length (m) | Height (m) | Sides | Area (m²) |
|---|---|---|---|---|
| Fence 1 | 7 | 1.5 | 2 | 21 |
| Fence 2 | 7 | 1.5 | 2 | 21 |
| Fence 3 | 7 | 1.5 | 2 | 21 |
| Fence 4 | 7 | 1.5 | 2 | 21 |
| **Fence total** | | | | **84** |

### Timber (eaves + fascia boards)

| Element | Length (m) | Width (m) | Area (m²) |
|---|---|---|---|
| Eaves (4 sides x 5m) | 20 | 0.3 | 6 |
| Fascia boards (4 sides x 5m) | 20 | 0.2 | 4 |
| **Timber total** | | | **10** |

### Summary

| Surface | Area (m²) |
|---|---|
| House walls (minus windows/door) | 49.6 |
| Fence (4× 7m × 1.5m × 2 sides) | 84 |
| Eaves + fascia boards | 10 |
| **Concrete total (house + fence)** | **132.9** |
| **Timber total (eaves + fascia)** | **10** |
| **Grand total** | **142.9** |

---

## Paint Quantities

Catalog rows are included above; can counts below are derived using `cans = ceil(area ÷ effective_m²_per_can)`.

### Concrete walls + fence

Area = **132.9 m²**.

| Catalog ID | Calc | Cans | Subtotal (THB) |
|---|---|---|---|
| `concrete-primer` | ceil(132.9 ÷ 30) | 5 | ~3,690 |
| `concrete-topcoat` | ceil(132.9 ÷ 17.5) | 8 | ~9,488 |
| **Concrete subtotal** | | **13** | **~13,178** |

### Timber eaves + fascia

Area = **10 m²**.

| Catalog ID | Calc | Cans | Subtotal (THB) |
|---|---|---|---|
| `timber-undercoat` | ceil(10 ÷ 30) | 1 | ~812 |
| `timber-topcoat` | ceil(10 ÷ 17.5) | 1 | ~940 |
| **Timber subtotal** | | **2** | **~1,752** |

---

## Order of work

1. **Prep** — brush off loose dust/chalk with a stiff broom. No pressure washing needed.
2. **Prime walls + fence** — `concrete-primer`, 1 coat. Bonds to chalky old plaster.
3. **Topcoat walls + fence** — `concrete-topcoat`, 2 coats. Self-cleaning titanium tech.
4. **Prime timber** — `timber-undercoat`, 1–2 coats. Blocks tannin bleed.
5. **Topcoat timber** — `timber-topcoat`, 2 coats. Flexible acrylic for wood.

---

## Notes

- House walls are dusty/chalky — `concrete-primer` is designed for this.
- `concrete-topcoat` must be **Sheen** finish (hides old plaster imperfections) — see catalog for the exact SKU.
- Round up by 1 can per substrate when buying, to handle wastage.

---

## Cost Summary

| Category | Est. Cost (THB) |
|---|---|
| Concrete materials (house walls + fence) | ~13,178 |
| Timber materials (eaves + fascia) | ~1,752 |
| Labour (DIY) | 0 |
| **HOUSE EXTERIOR PAINT TOTAL** | **~14,930** |
