---
title: House Exterior — Painting
status: Draft
rev: "2"
---

# House Exterior — Painting

DIY work, Joe + Mon. **Not part of any builder quote.** Repaint the existing house exterior — concrete walls, both sides of all four fence sections, and the timber eaves + fascia boards.

Old concrete plaster house, ~10 years old. Existing paint on all surfaces, dusty/chalky in places. Small cans preferred for easier handling.

Paint products (coverage, price, SKU) and computed can counts are bundled together in the auto-generated quantity partials below — see "Paint Quantities". The shared catalog partials at [`paint-concrete.md`](_partials/paint-concrete.md) / [`paint-timber.md`](_partials/paint-timber.md) remain for ROOF.md and GATE-01.md but are no longer needed inline here. Project-wide formula and rollup: [PAINT.md](PAINT.md).

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

Surface areas + can counts are auto-generated from `data/scope-picks.json` (the per-spec surface inventory) and `data/paints.json` (the product catalog). Edit either source and run `mise run gen` to regenerate; never edit the included partials by hand.

<!-- include: _partials/paint-quantity-house-exterior-concrete.md -->

<!-- include: _partials/paint-quantity-house-exterior-timber.md -->

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

<!-- include: _partials/cost-summary-house-exterior.md -->

| Category | Est. Cost (THB) |
|---|---|
| Labour (DIY) | 0 |
| **HOUSE EXTERIOR PAINT TOTAL** | **see auto-computed total above (currently ~฿14,930)** |
