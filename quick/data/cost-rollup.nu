#!/usr/bin/env nu

# data/cost-rollup.nu — aggregates per-spec material costs from all data-layer
# scopes (tiles + paints + windows) into specs/_partials/cost-summary-<spec-stem>.md.
#
# Reads tiles.json + paints.json + windows.json + rooms.json + scope-picks.json.
# For each unique `spec` field across all scope types, emits one rollup partial
# listing the contributing scopes and their subtotals plus a grand total.
#
# Cost model: every row contributes (cost_min, cost_max). Point estimates
# (tile, paint) set min == max. Range estimates (windows) set them apart.
# The total per spec sums min and max separately and renders as either
# "฿X" (point) or "฿X-Y" (range).
#
# Hand-edited line items (fixtures, labour, TBC entries) are NOT in scope —
# they live in each spec's `## Cost Summary` table. This partial complements
# that table with the auto-computed materials.

def main [] {
  let data_dir = ($env.QUICK_DATA_DIR? | default "data")
  let specs_dir = ($env.QUICK_SPECS_DIR? | default "specs")
  let partials_dir = $"($specs_dir)/_partials"

  let tiles    = (open $"($data_dir)/tiles.json"    | get tiles)
  let rooms    = (open $"($data_dir)/rooms.json"    | get rooms)
  let paints   = (open $"($data_dir)/paints.json"   | get paints)
  let windows  = (open $"($data_dir)/windows.json"  | get windows)
  let curtains = (open $"($data_dir)/curtains.json" | get curtains)
  let picks    = (open $"($data_dir)/scope-picks.json")

  let tile_rows = ($picks.tiles | columns | each {|sid|
    let s = ($picks.tiles | get $sid)
    if $s.spec == null { null } else {
      let cost = (tile_cost $s $tiles $rooms)
      {
        spec: $s.spec
        type: "Tile"
        label: $"($s.title) — ($sid)"
        min: $cost
        max: $cost
      }
    }
  } | where {|r| $r != null })

  let paint_rows = ($picks.paints | columns | each {|sid|
    let s = ($picks.paints | get $sid)
    if $s.spec == null { null } else {
      let cost = (paint_cost $s $paints)
      {
        spec: $s.spec
        type: "Paint"
        label: $"($s.title) — ($sid)"
        min: $cost
        max: $cost
      }
    }
  } | where {|r| $r != null })

  let window_rows = ($picks.windows | columns | each {|sid|
    let s = ($picks.windows | get $sid)
    if $s.spec == null { null } else {
      let cost = (window_cost $s $windows)
      {
        spec: $s.spec
        type: "Windows"
        label: $"($s.title) — ($sid)"
        min: $cost.min
        max: $cost.max
      }
    }
  } | where {|r| $r != null })

  let curtain_rows = ($picks.curtains | columns | each {|sid|
    let s = ($picks.curtains | get $sid)
    if $s.spec == null { null } else {
      let cost = (curtain_cost $s $curtains $windows $picks.windows)
      {
        spec: $s.spec
        type: "Curtains"
        label: $"($s.title) — ($sid)"
        min: $cost.min
        max: $cost.max
      }
    }
  } | where {|r| $r != null })

  let all_rows = ($tile_rows ++ $paint_rows ++ $window_rows ++ $curtain_rows)
  let specs = ($all_rows | get spec | uniq)

  mut wrote = 0
  mut skipped = 0

  for spec in $specs {
    let rows = ($all_rows | where {|r| $r.spec == $spec})
    let md = (gen_spec_md $spec $rows)
    let stem = ($spec | str replace ".md" "" | str downcase | str replace --all "_" "-")
    let out = $"($partials_dir)/cost-summary-($stem).md"

    let existing = (try { open --raw $out } catch { "" })
    if $existing == $md {
      $skipped += 1
    } else {
      $md | save --force --raw $out
      print $"  wrote ($out)"
      $wrote += 1
    }
  }

  print $"gen-cost-rollup: ($wrote) written, ($skipped) skipped"
}

# Compute a tile scope's cost by replicating the tiles.nu math.
def tile_cost [scope tiles rooms] {
  let tile = ($tiles | get $scope.tile_id)
  let tile_area_m2 = (($tile.size_w_mm * $tile.size_h_mm) / 1000000.0)
  let waste_factor = (1.0 + ($scope.wastage_pct / 100.0))

  let floor_area = ($scope.rooms | each {|rid|
    let r = ($rooms | get $rid)
    ($r.width_m * $r.length_m)
  } | math sum)

  let wall_surfaces_raw = ($scope | get -o wall_surfaces | default [])
  let wall_area = ($wall_surfaces_raw | each {|w| $w.w_m * $w.h_m} | append 0 | math sum)

  let total_area = ($floor_area + $wall_area)
  let tiles_with_waste = (($total_area / $tile_area_m2) * $waste_factor | math ceil)

  let has_box = ($tile.tiles_per_box != null and $tile.baht_per_box != null)
  if $has_box {
    let boxes = (($tiles_with_waste / $tile.tiles_per_box) | math ceil)
    ($boxes * $tile.baht_per_box | into int)
  } else if ($tile.baht_per_tile != null) {
    ($tiles_with_waste * $tile.baht_per_tile | math round | into int)
  } else if ($tile.baht_per_m2 != null) {
    ($total_area * $waste_factor * $tile.baht_per_m2 | math round | into int)
  } else { 0 }
}

# Compute a paint scope's cost by replicating the paint.nu math.
def paint_cost [scope paints] {
  let total_area = ($scope.surfaces | get area_m2 | math sum)
  $scope.products | each {|pid|
    let p = ($paints | get $pid)
    let cans = (($total_area * $p.default_coats / $p.coverage_m2_per_can_per_coat) | math ceil)
    ($cans * $p.baht_per_can | into int)
  } | math sum
}

# Compute a window scope's cost — returns {min, max} since prices are ranges.
def window_cost [scope windows] {
  let mins = ($scope.items | each {|item|
    let w = ($windows | get $item.window_id)
    ($item.qty * $w.baht_per_unit_min)
  } | append 0 | math sum | into int)
  let maxs = ($scope.items | each {|item|
    let w = ($windows | get $item.window_id)
    ($item.qty * $w.baht_per_unit_max)
  } | append 0 | math sum | into int)
  {min: $mins, max: $maxs}
}

# Compute a curtain scope's cost — reuses window dimensions from the
# referenced window scope. Mirrors curtains.nu math but returns just the
# bottom-line totals as {min, max}.
def curtain_cost [scope curtains windows window_scopes] {
  let win_scope = ($window_scopes | get $scope.from_window_scope)
  let included = ($win_scope.items | where {|item|
    $item.window_id in $scope.include_window_ids
  })

  let track_m = ($included | each {|item|
    let w = ($windows | get $item.window_id)
    (($w.size_w_cm + $scope.track_overlap_cm) / 100.0) * $item.qty
  } | append 0 | math sum)

  let fabric_m = ($included | each {|item|
    let w = ($windows | get $item.window_id)
    (($w.size_w_cm * $scope.fullness + $scope.fabric_overlap_cm) / 100.0) * $item.qty
  } | append 0 | math sum)

  let track_p  = ($curtains | get $scope.track_product)
  let fabric_p = ($curtains | get $scope.fabric_product)

  let min_total = (
    ($track_m  * $track_p.baht_per_metre_min) +
    ($fabric_m * $fabric_p.baht_per_metre_min) | math round | into int
  )
  let max_total = (
    ($track_m  * $track_p.baht_per_metre_max) +
    ($fabric_m * $fabric_p.baht_per_metre_max) | math round | into int
  )
  {min: $min_total, max: $max_total}
}

def gen_spec_md [spec rows] {
  let total_min = ($rows | get min | math sum | into int)
  let total_max = ($rows | get max | math sum | into int)

  let header_rows = [
    "| Source | Type | Subtotal \(THB\) |"
    "|---|---|---|"
  ]
  let body_rows = ($rows | each {|r|
    $"| ($r.label) | ($r.type) | (price_label $r.min $r.max) |"
  })
  let total_row = $"| **Total \(data-layer materials\)** | | **(price_label $total_min $total_max)** |"
  let table = ($header_rows ++ $body_rows ++ [$total_row]) | str join "\n"

  let frontmatter = $"---
title: Materials cost rollup — ($spec)
status: Draft
rev: \"1\"
generated: true
---

"

  let header = $"### Auto-computed materials cost

Aggregated from `data/scope-picks.json` by `data/cost-rollup.nu`. Covers tile, paint, and window quantities only — fixtures, labour, and TBC items live in this spec's `## Cost Summary` table below. Range values \(฿X-Y\) reflect supplier-quote variance for custom-order items.

"

  let footer = "

*Generated by `data/cost-rollup.nu` from `data/*.json` — do not edit this file by hand; change the JSON and run `mise run gen`.*
"

  $frontmatter + $header + $table + $footer
}

# Format a price as a range "฿X-Y" if min != max, else "฿X".
def price_label [min max] {
  if $min == $max {
    $"฿($min)"
  } else {
    $"฿($min)-($max)"
  }
}
