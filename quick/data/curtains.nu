#!/usr/bin/env nu

# data/curtains.nu — generates specs/_partials/curtain-quantity-<scope>.md
# from data/curtains.json + data/windows.json + data/scope-picks.json.
#
# Curtain scopes REUSE the window dimensions from a referenced window scope
# (`from_window_scope`). For each window in the covered set:
#   track_m  = (window.size_w_cm + scope.track_overlap_cm)  / 100
#   fabric_m = (window.size_w_cm × scope.fullness + scope.fabric_overlap_cm) / 100
# Both multiplied by the window's qty (taken from the windows scope).
#
# Cost = fabric_m × baht_per_metre + track_m × baht_per_metre, with each
# product priced as a (min, max) range.
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

  # Per-window curtain math
  let rows = ($included | each {|item|
    let w = ($windows | get $item.window_id)
    let track_m_per = (($w.size_w_cm + $scope.track_overlap_cm) / 100.0)
    let fabric_m_per = (($w.size_w_cm * $scope.fullness + $scope.fabric_overlap_cm) / 100.0)
    {
      label: $item.label
      window_name: $w.name
      width_cm: $w.size_w_cm
      drop_cm: $w.size_h_cm
      qty: $item.qty
      track_m: ($track_m_per * $item.qty | math round --precision 2)
      fabric_m: ($fabric_m_per * $item.qty | math round --precision 2)
    }
  })

  let total_track_m  = ($rows | get track_m  | math sum | math round --precision 2)
  let total_fabric_m = ($rows | get fabric_m | math sum | math round --precision 2)

  # Pricing — products are per linear metre with min/max ranges
  let track_p  = ($curtains | get $scope.track_product)
  let fabric_p = ($curtains | get $scope.fabric_product)

  let track_min  = ($total_track_m  * $track_p.baht_per_metre_min  | math round | into int)
  let track_max  = ($total_track_m  * $track_p.baht_per_metre_max  | math round | into int)
  let fabric_min = ($total_fabric_m * $fabric_p.baht_per_metre_min | math round | into int)
  let fabric_max = ($total_fabric_m * $fabric_p.baht_per_metre_max | math round | into int)

  let total_min = ($track_min + $fabric_min)
  let total_max = ($track_max + $fabric_max)

  # Per-window table
  let win_header = [
    "| Window | Frame size \(W × H cm\) | Qty | Track \(m\) | Fabric \(m\) |"
    "|---|---|---|---|---|"
  ]
  let win_body = ($rows | each {|r|
    $"| ($r.label) | ($r.width_cm) × ($r.drop_cm) | ($r.qty) | ($r.track_m) | ($r.fabric_m) |"
  })
  let win_total = $"| **Total** | | | **($total_track_m)** | **($total_fabric_m)** |"
  let win_table = ($win_header ++ $win_body ++ [$win_total]) | str join "\n"

  # Per-product cost table
  let cost_header = [
    "| Product | Metres | Unit price | Subtotal |"
    "|---|---|---|---|"
  ]
  let track_link = if $track_p.url != null {
    $"[($track_p.name)]\(($track_p.url)\)"
  } else { $track_p.name }
  let fabric_link = if $fabric_p.url != null {
    $"[($fabric_p.name)]\(($fabric_p.url)\)"
  } else { $fabric_p.name }
  let cost_body = [
    $"| ($track_link) | ($total_track_m) m | (price_label_per_m $track_p.baht_per_metre_min $track_p.baht_per_metre_max) | (price_label $track_min $track_max) |"
    $"| ($fabric_link) | ($total_fabric_m) m | (price_label_per_m $fabric_p.baht_per_metre_min $fabric_p.baht_per_metre_max) | (price_label $fabric_min $fabric_max) |"
  ]
  let cost_total = $"| **Total** | | | **(price_label $total_min $total_max)** |"
  let cost_table = ($cost_header ++ $cost_body ++ [$cost_total]) | str join "\n"

  let frontmatter = $"---
title: ($scope.title)
status: Draft
rev: \"1\"
generated: true
---

"

  let header_md = $"### ($scope.title)

Window dimensions are reused from the `($scope.from_window_scope)` scope in `scope-picks.json` — never duplicated here. Curtain math: track length = window width + ($scope.track_overlap_cm) cm overlap; fabric = window width × ($scope.fullness) fullness + ($scope.fabric_overlap_cm) cm overlap, per linear metre off the 280 cm bolt.

"

  let footer = "

*Generated by `data/curtains.nu` from `data/*.json` — do not edit this file by hand; change the JSON and run `mise run gen`.*
"

  $frontmatter + $header_md + $win_table + "\n\n" + $cost_table + $footer
}

# Format a price as a range "฿X-Y" if min != max, else "฿X".
def price_label [min max] {
  if $min == $max { $"฿($min)" } else { $"฿($min)-($max)" }
}

def price_label_per_m [min max] {
  if $min == $max { $"฿($min)/m" } else { $"฿($min)-($max)/m" }
}
