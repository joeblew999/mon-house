#!/usr/bin/env nu

# data/curtains.nu — generates specs/_partials/curtain-quantity-<scope>.md
# from data/curtains.json + data/windows.json + data/scope-picks.json.
#
# Curtain scopes REUSE the window dimensions from a referenced window scope
# (`from_window_scope`). For each window in the covered set:
#   track_m  = (window.size_w_cm + scope.track_overlap_cm)  / 100
#   fabric_m = (window.size_w_cm × scope.fullness + scope.fabric_overlap_cm) / 100
# Both multiplied by the window's qty.
#
# A real curtain run also needs FITTINGS: end-brackets (per window), mount
# brackets (per metre of track at a fixed interval), and gliders/hooks
# (per metre of curtain width at a fixed coverage). Each fitting in the
# catalog declares a `per_calc` rule that the generator follows:
#   - "per-window"              → 1 × window.qty
#   - "per-track-metre"         → ceil(track_m_per_window × 100 / interval_cm) × window.qty
#   - "per-curtain-width-metre" → ceil(fabric_m_per_window × 100 / covers_cm_per_unit) × window.qty
#
# Run via `mise run gen` (or `quick-tool gen`). Idempotent per the project
# rule: a second run with no input change writes zero files.

def main [] {
  let data_dir = ($env.QUICK_DATA_DIR? | default "data")
  let specs_dir = ($env.QUICK_SPECS_DIR? | default "specs")
  let partials_dir = $"($specs_dir)/_partials"

  let curtains = (open $"($data_dir)/curtains.json" | get curtains)
  let windows  = (open $"($data_dir)/windows.json"  | get windows)
  let picks    = (open $"($data_dir)/scope-picks.json")

  mut wrote = 0
  mut skipped = 0

  for scope_id in ($picks.curtains | columns) {
    let scope = ($picks.curtains | get $scope_id)
    let md = (gen_scope_md $scope $curtains $windows $picks.windows)
    let out = $"($partials_dir)/curtain-quantity-($scope_id).md"

    let existing = (try { open --raw $out } catch { "" })
    if $existing == $md {
      $skipped += 1
    } else {
      $md | save --force --raw $out
      print $"  wrote ($out)"
      $wrote += 1
    }
  }

  print $"gen-curtains: ($wrote) written, ($skipped) skipped"
}

def gen_scope_md [scope curtains windows window_scopes] {
  let win_scope = ($window_scopes | get $scope.from_window_scope)
  let included = ($win_scope.items | where {|item|
    $item.window_id in $scope.include_window_ids
  })

  # Per-window per-metre math: track_m_per_window, fabric_m_per_window
  let win_rows = ($included | each {|item|
    let w = ($windows | get $item.window_id)
    let track_m_per = (($w.size_w_cm + $scope.track_overlap_cm) / 100.0)
    let fabric_m_per = (($w.size_w_cm * $scope.fullness + $scope.fabric_overlap_cm) / 100.0)
    {
      label: $item.label
      window_id: $item.window_id
      width_cm: $w.size_w_cm
      drop_cm: $w.size_h_cm
      qty: $item.qty
      track_m_per: $track_m_per
      fabric_m_per: $fabric_m_per
      track_m: ($track_m_per * $item.qty | math round --precision 2)
      fabric_m: ($fabric_m_per * $item.qty | math round --precision 2)
    }
  })

  let total_track_m  = ($win_rows | get track_m  | math sum | math round --precision 2)
  let total_fabric_m = ($win_rows | get fabric_m | math sum | math round --precision 2)

  # Per-fitting qty calc — sum qty across all windows in scope
  let fitting_rows = ($scope.fittings | each {|fid|
    let f = ($curtains | get $fid)
    let qty = ($win_rows | each {|wr|
      compute_fitting_qty $f $wr
    } | append 0 | math sum | into int)
    let cost_min = ($qty * $f.baht_per_unit_min | into int)
    let cost_max = ($qty * $f.baht_per_unit_max | into int)
    {
      product_id: $fid
      name: $f.name
      url: $f.url
      qty: $qty
      unit_min: $f.baht_per_unit_min
      unit_max: $f.baht_per_unit_max
      cost_min: $cost_min
      cost_max: $cost_max
    }
  })

  # Per-metre product costs (track + fabric)
  let track_p  = ($curtains | get $scope.track_product)
  let fabric_p = ($curtains | get $scope.fabric_product)
  let track_min  = ($total_track_m  * $track_p.baht_per_metre_min  | math round | into int)
  let track_max  = ($total_track_m  * $track_p.baht_per_metre_max  | math round | into int)
  let fabric_min = ($total_fabric_m * $fabric_p.baht_per_metre_min | math round | into int)
  let fabric_max = ($total_fabric_m * $fabric_p.baht_per_metre_max | math round | into int)

  let fittings_min = ($fitting_rows | get cost_min | append 0 | math sum | into int)
  let fittings_max = ($fitting_rows | get cost_max | append 0 | math sum | into int)

  let total_min = ($track_min + $fabric_min + $fittings_min)
  let total_max = ($track_max + $fabric_max + $fittings_max)

  # Per-window table
  let win_header = [
    "| Window | Frame size \(W × H cm\) | Qty | Track \(m\) | Fabric \(m\) |"
    "|---|---|---|---|---|"
  ]
  let win_body = ($win_rows | each {|r|
    $"| ($r.label) | ($r.width_cm) × ($r.drop_cm) | ($r.qty) | ($r.track_m) | ($r.fabric_m) |"
  })
  let win_total = $"| **Total** | | | **($total_track_m)** | **($total_fabric_m)** |"
  let win_table = ($win_header ++ $win_body ++ [$win_total]) | str join "\n"

  # Per-product cost table — track + fittings + fabric
  let cost_header = [
    "| Product | Qty | Unit price | Subtotal |"
    "|---|---|---|---|"
  ]

  let track_link = if $track_p.url != null { $"[($track_p.name)]\(($track_p.url)\)" } else { $track_p.name }
  let fabric_link = if $fabric_p.url != null { $"[($fabric_p.name)]\(($fabric_p.url)\)" } else { $fabric_p.name }

  let track_row = $"| ($track_link) | ($total_track_m) m | (price_label_per_m $track_p.baht_per_metre_min $track_p.baht_per_metre_max) | (price_label $track_min $track_max) |"
  let fitting_body = ($fitting_rows | each {|r|
    let link = if $r.url != null { $"[($r.name)]\(($r.url)\)" } else { $r.name }
    $"| ($link) | ($r.qty) | (price_label $r.unit_min $r.unit_max)/ea | (price_label $r.cost_min $r.cost_max) |"
  })
  let fabric_row = $"| ($fabric_link) | ($total_fabric_m) m | (price_label_per_m $fabric_p.baht_per_metre_min $fabric_p.baht_per_metre_max) | (price_label $fabric_min $fabric_max) |"
  let cost_total = $"| **Total** | | | **(price_label $total_min $total_max)** |"

  let cost_table = ($cost_header ++ [$track_row] ++ $fitting_body ++ [$fabric_row] ++ [$cost_total]) | str join "\n"

  let frontmatter = $"---
title: ($scope.title)
status: Draft
rev: \"1\"
generated: true
---

"

  let header_md = $"### ($scope.title)

Window dimensions are reused from the `($scope.from_window_scope)` scope in `scope-picks.json` — never duplicated here. Curtain math: track length = window width + ($scope.track_overlap_cm) cm overlap; fabric = window width × ($scope.fullness) fullness + ($scope.fabric_overlap_cm) cm overlap, per linear metre off the 280 cm bolt. Fittings \(end-brackets, mount-brackets, gliders\) follow each product's per_calc rule from `curtains.json`.

"

  let footer = "

*Generated by `data/curtains.nu` from `data/*.json` — do not edit this file by hand; change the JSON and run `mise run gen`.*
"

  $frontmatter + $header_md + $win_table + "\n\n" + $cost_table + $footer
}

# Compute the qty of one fitting needed for one window in scope.
# Reads the fitting's per_calc rule + optional interval_cm / covers_cm_per_unit.
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
    let per_window = (($win_row.fabric_m_per * 100.0 / $covers) | math ceil)
    ($per_window * $win_row.qty)
  } else {
    0
  }
}

# Format a price as a range "฿X-Y" if min != max, else "฿X".
def price_label [min max] {
  if $min == $max { $"฿($min)" } else { $"฿($min)-($max)" }
}

def price_label_per_m [min max] {
  if $min == $max { $"฿($min)/m" } else { $"฿($min)-($max)/m" }
}
