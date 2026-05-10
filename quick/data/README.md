# `data/` — JSON-driven quantity calculations

Hand-edited JSON files plus colocated `.nu` generators. The generators read the
JSON, compute quantities (tiles needed, boxes to buy, cost subtotals), and
write committed markdown partials into `specs/_partials/tile-quantity-*.md`.

**Run:** `mise run gen` *(or `quick-tool gen` once the Rust wrapper lands)*

**Watch:** the watch loop picks up changes to `data/*.json` and `data/*.nu`
and re-runs gen before letting the existing translate→build pipeline fire.

## Files

| File | Role | Hand-edited? |
|---|---|---|
| `tiles.json` | Tile catalog: SKU, dimensions, finish, slip rating, pricing | ✓ |
| `rooms.json` | Room dimensions in metres | ✓ |
| `scope-picks.json` | Maps each spec scope → rooms[] + tile_id + wastage % | ✓ |
| `tiles.nu` | Generator: reads the JSON, emits quantity-table partials | ✓ |
| `paint-*.json` *(future)* | Paint catalog + room surface areas | ✓ |
| `paint.nu` *(future)* | Same pattern for paint | ✓ |

## Why JSON + nushell

- **Markdown is not a calculator.** Quantity tables that are typed by hand drift the moment a price or dimension changes.
- **Numbers are SSOT in JSON.** Prose (catalogs, recommendations, design rationale) stays in `_partials/*.md` hand-edited; numbers move here.
- **Nushell is Rust.** Same generator can run via the local `nu` binary now and embedded in `quick-tool` later — see `quick/CLAUDE.md` for the migration roadmap.

## Idempotency (Rule 2)

Every generator does **hash-and-skip**: it formats the markdown, compares to
the existing file, and writes only if the content differs. Running
`mise run gen` twice with no input change must do zero work.

## Output convention

Each scope in `scope-picks.json` produces one partial:

```
data/scope-picks.json:scopes."bathroom-basic"
    │
    ▼  (generator)
specs/_partials/tile-quantity-bathroom-basic.md
    │
    ▼  spec includes via:  <!-- include: _partials/tile-quantity-bathroom-basic.md -->
    │
    ▼  flows through translate → cmarker → typst → PDF as any other partial
```

Generated partials are **committed**. CI doesn't need `nu` — it builds from
the committed artifacts. A drift guard in CI re-runs gen and `git diff
--exit-code` to catch contributors who edit JSON without regenerating.

## Schema rules

### `tiles.json`

Each tile entry must specify dimensions (`size_w_mm`, `size_h_mm`). At least
one pricing tier must be present; the generator picks the most accurate
available in this priority order:

1. Box-priced (`baht_per_box` + `tiles_per_box` + `m2_per_box`) — used by most porcelain
2. Per-tile (`baht_per_tile`) — used when sold loose
3. Per-m² (`baht_per_m2`) — used when only the headline price is known

`null` is fine for fields that aren't yet known (pull a value out of `null` /
TBC by visiting the showroom or calling the supplier; update the JSON; run
`mise run gen`; the price flows to every spec that uses the tile).

### `rooms.json`

Every room is `width_m × length_m` in metres. The catalog is dimension-only;
no opinion about what gets tiled — that's per-scope.

### `scope-picks.json`

Each scope has an id (kebab-case), a `title` (used as the partial's `## ` heading),
a list of `rooms` (must exist in `rooms.json`), a `tile_id` (must exist in
`tiles.json`), and a `wastage_pct`.

`spec` is a free-form pointer to which file consumes the partial — used for
documentation, not for routing.
