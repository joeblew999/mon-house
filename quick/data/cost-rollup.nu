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
        is_alternative: false
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
        is_alternative: false
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
        is_alternative: false
      }
    }
  } | where {|r| $r != null })

  let curtain_rows = ($picks.curtains | columns | each {|sid|
    let s = ($picks.curtains | get $sid)
    if $s.spec == null { null } else {
      let cost = (curtain_cost $s $curtains $windows $picks.windows)
      let alt_of = ($s | get -o is_alternative_to | default null)
      {
        spec: $s.spec
        type: "Curtains"
        label: $"($s.title) — ($sid)"
        min: $cost.min
        max: $cost.max
        is_alternative: ($alt_of != null)
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

# Compute a curtain scope's cost — branches on style (pleated-on-track vs
# eyelet-on-rod). Mirrors curtains.nu math and returns {min, max}.
def curtain_cost [scope curtains windows window_scopes] {
  let style = ($scope | get -o style | default "pleated-on-track")
  if $style == "eyelet-on-rod" {
    eyelet_cost $scope $curtains $windows $window_scopes
  } else {
    pleated_cost $scope $curtains $windows $window_scopes
  }
}

def pleated_cost [scope curtains windows window_scopes] {
  let win_scope = ($window_scopes | get $scope.from_window_scope)
  let included = ($win_scope.items | where {|item|
    $item.window_id in $scope.include_window_ids
  })

  let fabric_p = ($curtains | get $scope.fabric_product)
  let bolt_w = ($fabric_p.bolt_width_cm | default 150)
  let vert_allow = ($scope | get -o vertical_allowance_cm | default 30)

  let win_rows = ($included | each {|item|
    let w = ($windows | get $item.window_id)
    let track_m_per = (($w.size_w_cm + $scope.track_overlap_cm) / 100.0)
    let curtain_w_cm = ($w.size_w_cm * $scope.fullness + $scope.fabric_overlap_cm)
    let curtain_width_m_per = ($curtain_w_cm / 100.0)
    let effective_drop = ($w.size_h_cm + $vert_allow)
    let fabric_m_per = if $effective_drop > $bolt_w {
      let panels = (($curtain_w_cm / $bolt_w) | math ceil)
      (($effective_drop / 100.0) * $panels)
    } else {
      $curtain_width_m_per
    }
    {
      qty: $item.qty
      track_m_per: $track_m_per
      curtain_width_m_per: $curtain_width_m_per
      fabric_m_per: $fabric_m_per
      track_m: ($track_m_per * $item.qty)
      fabric_m: ($fabric_m_per * $item.qty)
    }
  })

  let track_m  = ($win_rows | get track_m  | append 0 | math sum)
  let fabric_m = ($win_rows | get fabric_m | append 0 | math sum)

  let track_p  = ($curtains | get $scope.track_product)

  let track_min  = ($track_m  * $track_p.baht_per_metre_min)
  let track_max  = ($track_m  * $track_p.baht_per_metre_max)
  let fabric_min = ($fabric_m * $fabric_p.baht_per_metre_min)
  let fabric_max = ($fabric_m * $fabric_p.baht_per_metre_max)

  let fittings = ($scope | get -o fittings | default [])
  let fittings_min = ($fittings | each {|fid|
    let f = ($curtains | get $fid)
    let qty = ($win_rows | each {|wr| compute_fitting_qty $f $wr} | append 0 | math sum)
    ($qty * $f.baht_per_unit_min)
  } | append 0 | math sum)
  let fittings_max = ($fittings | each {|fid|
    let f = ($curtains | get $fid)
    let qty = ($win_rows | each {|wr| compute_fitting_qty $f $wr} | append 0 | math sum)
    ($qty * $f.baht_per_unit_max)
  } | append 0 | math sum)

  {
    min: ($track_min + $fabric_min + $fittings_min | math round | into int)
    max: ($track_max + $fabric_max + $fittings_max | math round | into int)
  }
}

def eyelet_cost [scope curtains windows window_scopes] {
  let win_scope = ($window_scopes | get $scope.from_window_scope)
  let included = ($win_scope.items | where {|item|
    $item.window_id in $scope.include_window_ids
  })

  let panel_p = ($curtains | get $scope.panel_product)
  let panel_w = $panel_p.panel_width_cm

  let win_rows = ($included | each {|item|
    let w = ($windows | get $item.window_id)
    let rod_m_per = (($w.size_w_cm + $scope.rod_overlap_cm) / 100.0)
    let panels_per = (($w.size_w_cm * $scope.fullness / $panel_w) | math ceil)
    {
      qty: $item.qty
      rod_m_per: $rod_m_per
      track_m_per: $rod_m_per   # alias for per-track-metre fittings
      fabric_m_per: 0           # eyelet has no fabric metres
      rod_m: ($rod_m_per * $item.qty)
      panels: ($panels_per * $item.qty)
    }
  })

  let rod_m = ($win_rows | get rod_m | append 0 | math sum)
  let panels = ($win_rows | get panels | append 0 | math sum)

  let rod_p = ($curtains | get $scope.rod_product)

  let rod_min = ($rod_m * $rod_p.baht_per_metre_min)
  let rod_max = ($rod_m * $rod_p.baht_per_metre_max)
  let panels_min = ($panels * $panel_p.baht_per_panel_min)
  let panels_max = ($panels * $panel_p.baht_per_panel_max)

  let fittings = ($scope | get -o fittings | default [])
  let fittings_min = ($fittings | each {|fid|
    let f = ($curtains | get $fid)
    let qty = ($win_rows | each {|wr| compute_fitting_qty $f $wr} | append 0 | math sum)
    ($qty * $f.baht_per_unit_min)
  } | append 0 | math sum)
  let fittings_max = ($fittings | each {|fid|
    let f = ($curtains | get $fid)
    let qty = ($win_rows | each {|wr| compute_fitting_qty $f $wr} | append 0 | math sum)
    ($qty * $f.baht_per_unit_max)
  } | append 0 | math sum)

  {
    min: ($rod_min + $panels_min + $fittings_min | math round | into int)
    max: ($rod_max + $panels_max + $fittings_max | math round | into int)
  }
}

# Compute the qty of one fitting needed for one window in scope.
# Mirrors the function in curtains.nu — kept in sync.
def compute_fitting_qty [fitting win_row] {
  let pc = $fitting.per_calc
  if $pc == "per-window" {
    $win_row.qty
  } else if $pc == "per-track-metre" {
    let interval = ($fitting.interval_cm | default 80)
    let per_window = (($win_row.track_m_per * 100.0 / $interval) | math ceil)
    ($per_window * $win_row.qty)
  } else if $pc == "per-curtain-width-metre" {
    let covers = ($fitting.covers_cm_per_unit | default 240)
    let width_m = ($win_row | get -o curtain_width_m_per | default $win_row.fabric_m_per)
    let per_window = (($width_m * 100.0 / $covers) | math ceil)
    ($per_window * $win_row.qty)
  } else {
    0
  }
}

def gen_spec_md [spec rows] {
  # Sum only primary rows (alternatives are listed but excluded from the total —
  # they're "pick one of these" options, not additive items).
  let primary_rows = ($rows | where {|r| not $r.is_alternative})
  let total_min = ($primary_rows | get min | append 0 | math sum | into int)
  let total_max = ($primary_rows | get max | append 0 | math sum | into int)

  let header_rows = [
    "| Source | Type | Subtotal \(THB\) |"
    "|---|---|---|"
  ]
  let body_rows = ($rows | each {|r|
    let tag = if $r.is_alternative { " *(alternative — not in total)*" } else { "" }
    $"| ($r.label)($tag) | ($r.type) | (price_label $r.min $r.max) |"
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

  let any_alt = ($rows | any {|r| $r.is_alternative})
  let alt_note = if $any_alt {
    " Rows marked *alternative* are pick-one options — only the primary alternative is included in the total."
  } else { "" }

  let header = $"### Auto-computed materials cost

Aggregated from `data/scope-picks.json` by `data/cost-rollup.nu`. Covers tile, paint, window, and curtain quantities only — fixtures, labour, and TBC items live in this spec's `## Cost Summary` table below. Range values \(฿X-Y\) reflect supplier-quote variance for custom-order items.($alt_note)

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
