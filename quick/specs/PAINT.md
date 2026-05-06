---
title: Paint
status: Draft
rev: "5"
---

# Paint — Product Catalog (Reference)

This file is **reference-only**. It declares the paint products used across the project — coverage, price, SKU, number of coats — and the formula every other spec uses to compute its own can quantities. **No work is described here.**

The catalog is split into three substrate partials so each work-spec can include only what it needs:

| Spec | Includes | Surface |
|---|---|---|
| [HOUSE-EXTERIOR.md](HOUSE-EXTERIOR.md) | `paint-concrete`, `paint-timber` | walls, fence, eaves, fascia |
| [ROOF.md](ROOF.md) | `paint-metal` | sheet undersides + steel beams |
| [GATE-01.md](GATE-01.md) | `paint-metal` | gate (both sides) |

Joe + Mon do **all** painting across **all** phases — never in any builder quote.

---

## Formula

```
cans = ceil(area_m² ÷ effective_m²_per_can)
```

Effective coverage already accounts for the number of coats — no need to multiply by coats again.

**Example:** ROOF metalwork = 30 m² → `ceil(30 ÷ 3.5)` = **9 cans** of `metal-2in1`.

When buying, round up by 1–2 cans per substrate to handle wastage.

---

<!-- include: _partials/paint-concrete.md -->

<!-- include: _partials/paint-timber.md -->

<!-- include: _partials/paint-metal.md -->

---

## Project-wide rollup

Sum of cans declared by each work-spec. Update this rollup when a work-spec changes — or regenerate it from the linked specs.

| Catalog ID | HOUSE-EXTERIOR | ROOF | GATE-01 | Total cans | Total cost (THB) |
|---|---|---|---|---|---|
| `concrete-primer` | 5 | — | — | 5 | ~3,690 |
| `concrete-topcoat` | 8 | — | — | 8 | ~9,488 |
| `timber-undercoat` | 1 | — | — | 1 | ~812 |
| `timber-topcoat` | 1 | — | — | 1 | ~940 |
| `metal-2in1` | — | 9 (buy 10) | 4 | 13–14 | ~2,600–2,800 |
| **Totals** | **15** | **9–10** | **4** | **~28** | **~17,530–17,730** |

---

## Notes

- **Catalog is SSOT.** Coverage, price, and SKU live only in the three `_partials/paint-*.md` files. Work-specs ([HOUSE-EXTERIOR](HOUSE-EXTERIOR.md), [ROOF](ROOF.md), [GATE-01](GATE-01.md)) include the partial(s) they need and compute can counts using the formula above — they do not duplicate metadata.
- When a price or coverage changes, edit the relevant partial once. The change appears in every spec that includes it.
