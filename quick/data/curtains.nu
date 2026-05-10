#!/usr/bin/env nu

# data/curtains.nu — generates specs/_partials/curtain-quantity-<scope>.md
# from data/curtains.json + data/windows.json + data/scope-picks.json.
#
# Two curtain styles supported, dispatched by `scope.style`:
#
#   1. "pleated-on-track" — IKEA VIDGA-style. Track + fittings + fabric off
#      the bolt. Math:
#        track_m        = (window_w + track_overlap_cm) / 100
#        curtain_width  = window_w × fullness + fabric_overlap_cm
#        effective_drop = window_h + vertical_allowance_cm
#        fabric_m       = (effective_drop ≤ bolt_width)
#                          ? curtain_width / 100                                   (orient B: drop fits)
#                          : ceil(curtain_width / bolt_width) × effective_drop/100 (orient A: seam vertical drops)
#      Plus fittings whose qty follows their own per_calc rules. The
#      "per-curtain-width-metre" rule reads curtain_width_m_per from the
#      win_row, NOT fabric_m_per (which now varies by orientation).
#
#   2. "eyelet-on-rod" — HomePro-style. Rod + brackets + pre-made eyelet
#      panels (sold per panel, not per metre). Math:
#        rod_m  = (window_w + rod_overlap_cm) / 100
#        panels = ceil(window_w × fullness / panel_width_cm)
#      No fabric per metre, no gliders (eyelets ARE the rings).
#
# Curtain scopes REUSE window dimensions from a referenced window scope —
# never duplicate them in scope-picks.json.
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
    let style = ($scope | get -o style | default "pleated-on-track")
    let md = if $style == "eyelet-on-rod" {
      gen_eyelet_md $scope $curtains $windows $picks.windows
    } else {
      gen_pleated_md $scope $curtains $windows $picks.windows
    }
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

# ── pleated-on-track ──────────────────────────────────────────────────────────

def gen_pleated_md [scope curtains windows window_scopes] {
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
    # Orientation B (drop fits in bolt width) vs Orientation A (seamed vertical drops)
    let needs_seam = ($effective_drop > $bolt_w)
    let fabric_m_per = if $needs_seam {
      let panels = (($curtain_w_cm / $bolt_w) | math ceil)
      (($effective_drop / 100.0) * $panels)
    } else {
      $curtain_width_m_per
    }
    {
      label: $item.label
      width_cm: $w.size_w_cm
      drop_cm: $w.size_h_cm
      qty: $item.qty
      track_m_per: $track_m_per
      curtain_width_m_per: $curtain_width_m_per
      fabric_m_per: $fabric_m_per
      seamed: $needs_seam
      track_m: ($track_m_per * $item.qty | math round --precision 2)
      fabric_m: ($fabric_m_per * $item.qty | math round --precision 2)
    }
  })

  let total_track_m  = ($win_rows | get track_m  | math sum | math round --precision 2)
  let total_fabric_m = ($win_rows | get fabric_m | math sum | math round --precision 2)

  let fitting_rows = ($scope.fittings | each {|fid|
    let f = ($curtains | get $fid)
    let qty = ($win_rows | each {|wr| compute_fitting_qty $f $wr} | append 0 | math sum | into int)
    {
      name: $f.name
      url: $f.url
      qty: $qty
      unit_min: $f.baht_per_unit_min
      unit_max: $f.baht_per_unit_max
      cost_min: ($qty * $f.baht_per_unit_min | into int)
      cost_max: ($qty * $f.baht_per_unit_max | into int)
    }
  })

  let track_p  = ($curtains | get $scope.track_product)
  # fabric_p already resolved at top of function
  let track_min  = ($total_track_m  * $track_p.baht_per_metre_min  | math round | into int)
  let track_max  = ($total_track_m  * $track_p.baht_per_metre_max  | math round | into int)
  let fabric_min = ($total_fabric_m * $fabric_p.baht_per_metre_min | math round | into int)
  let fabric_max = ($total_fabric_m * $fabric_p.baht_per_metre_max | math round | into int)

  let fittings_min = ($fitting_rows | get cost_min | append 0 | math sum | into int)
  let fittings_max = ($fitting_rows | get cost_max | append 0 | math sum | into int)
  let total_min = ($track_min + $fabric_min + $fittings_min)
  let total_max = ($track_max + $fabric_max + $fittings_max)

  let win_table = (build_window_table_pleated $win_rows $total_track_m $total_fabric_m)
  let cost_table = (build_cost_table_pleated $track_p $fabric_p $fitting_rows $total_track_m $total_fabric_m $track_min $track_max $fabric_min $fabric_max $total_min $total_max)

  let intro = $"Window dimensions are reused from the `($scope.from_window_scope)` scope in `scope-picks.json` — never duplicated here. Style: **pleated-on-track**. Track length = window width + ($scope.track_overlap_cm) cm extension. Curtain width = window width × ($scope.fullness) fullness + ($scope.fabric_overlap_cm) cm side-hem allowance. Effective drop = window height + ($vert_allow) cm vertical allowance \(top heading + bottom hem\). Fabric: if effective drop ≤ ($bolt_w) cm bolt width, one horizontal cut \(linear metres = curtain width\); else multiple vertical drops seamed \(metres = ceil\(width / bolt\) × drop\). Fittings follow each product's per_calc rule from `curtains.json`."

  wrap_md $scope $intro ($win_table + "\n\n" + $cost_table)
}

def build_window_table_pleated [win_rows total_track_m total_fabric_m] {
  let h = [
    "| Window | Frame size \(W × H cm\) | Qty | Track \(m\) | Fabric \(m\) |"
    "|---|---|---|---|---|"
  ]
  let body = ($win_rows | each {|r|
    $"| ($r.label) | ($r.width_cm) × ($r.drop_cm) | ($r.qty) | ($r.track_m) | ($r.fabric_m) |"
  })
  let total = $"| **Total** | | | **($total_track_m)** | **($total_fabric_m)** |"
  ($h ++ $body ++ [$total]) | str join "\n"
}

def build_cost_table_pleated [track_p fabric_p fitting_rows total_track_m total_fabric_m track_min track_max fabric_min fabric_max total_min total_max] {
  let h = [
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
  let total_row = $"| **Total** | | | **(price_label $total_min $total_max)** |"

  ($h ++ [$track_row] ++ $fitting_body ++ [$fabric_row] ++ [$total_row]) | str join "\n"
}

# ── eyelet-on-rod ─────────────────────────────────────────────────────────────

def gen_eyelet_md [scope curtains windows window_scopes] {
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
      label: $item.label
      width_cm: $w.size_w_cm
      drop_cm: $w.size_h_cm
      qty: $item.qty
      rod_m_per: $rod_m_per
      track_m_per: $rod_m_per   # alias so per-track-metre fittings still work
      fabric_m_per: 0           # eyelet panels don't use fabric metres
      panels_per: $panels_per
      rod_m: ($rod_m_per * $item.qty | math round --precision 2)
      panels: ($panels_per * $item.qty | into int)
    }
  })

  let total_rod_m = ($win_rows | get rod_m | math sum | math round --precision 2)
  let total_panels = ($win_rows | get panels | math sum | into int)

  let fitting_rows = ($scope.fittings | each {|fid|
    let f = ($curtains | get $fid)
    let qty = ($win_rows | each {|wr| compute_fitting_qty $f $wr} | append 0 | math sum | into int)
    {
      name: $f.name
      url: $f.url
      qty: $qty
      unit_min: $f.baht_per_unit_min
      unit_max: $f.baht_per_unit_max
      cost_min: ($qty * $f.baht_per_unit_min | into int)
      cost_max: ($qty * $f.baht_per_unit_max | into int)
    }
  })

  let rod_p = ($curtains | get $scope.rod_product)
  let rod_min = ($total_rod_m * $rod_p.baht_per_metre_min | math round | into int)
  let rod_max = ($total_rod_m * $rod_p.baht_per_metre_max | math round | into int)
  let panels_min = ($total_panels * $panel_p.baht_per_panel_min | into int)
  let panels_max = ($total_panels * $panel_p.baht_per_panel_max | into int)
  let fittings_min = ($fitting_rows | get cost_min | append 0 | math sum | into int)
  let fittings_max = ($fitting_rows | get cost_max | append 0 | math sum | into int)
  let total_min = ($rod_min + $panels_min + $fittings_min)
  let total_max = ($rod_max + $panels_max + $fittings_max)

  let win_table = (build_window_table_eyelet $win_rows $total_rod_m $total_panels $panel_w)
  let cost_table = (build_cost_table_eyelet $rod_p $panel_p $fitting_rows $total_rod_m $total_panels $rod_min $rod_max $panels_min $panels_max $total_min $total_max)

  let intro = $"Window dimensions are reused from the `($scope.from_window_scope)` scope in `scope-picks.json` — never duplicated here. Style: **eyelet-on-rod**. Rod length = window width + ($scope.rod_overlap_cm) cm overlap; panels = ceil\(window width × ($scope.fullness) fullness / ($panel_w) cm panel width\). No sewing — pre-made panels with metal eyelets thread onto the rod."

  wrap_md $scope $intro ($win_table + "\n\n" + $cost_table)
}

def build_window_table_eyelet [win_rows total_rod_m total_panels panel_w] {
  let h = [
    $"| Window | Frame size \(W × H cm\) | Qty | Rod \(m\) | Panels \(($panel_w) cm wide\) |"
    "|---|---|---|---|---|"
  ]
  let body = ($win_rows | each {|r|
    $"| ($r.label) | ($r.width_cm) × ($r.drop_cm) | ($r.qty) | ($r.rod_m) | ($r.panels) |"
  })
  let total = $"| **Total** | | | **($total_rod_m)** | **($total_panels)** |"
  ($h ++ $body ++ [$total]) | str join "\n"
}

def build_cost_table_eyelet [rod_p panel_p fitting_rows total_rod_m total_panels rod_min rod_max panels_min panels_max total_min total_max] {
  let h = [
    "| Product | Qty | Unit price | Subtotal |"
    "|---|---|---|---|"
  ]
  let rod_link = if $rod_p.url != null { $"[($rod_p.name)]\(($rod_p.url)\)" } else { $rod_p.name }
  let panel_link = if $panel_p.url != null { $"[($panel_p.name)]\(($panel_p.url)\)" } else { $panel_p.name }

  let rod_row = $"| ($rod_link) | ($total_rod_m) m | (price_label_per_m $rod_p.baht_per_metre_min $rod_p.baht_per_metre_max) | (price_label $rod_min $rod_max) |"
  let fitting_body = ($fitting_rows | each {|r|
    let link = if $r.url != null { $"[($r.name)]\(($r.url)\)" } else { $r.name }
    $"| ($link) | ($r.qty) | (price_label $r.unit_min $r.unit_max)/ea | (price_label $r.cost_min $r.cost_max) |"
  })
  let panel_row = $"| ($panel_link) | ($total_panels) panels | (price_label $panel_p.baht_per_panel_min $panel_p.baht_per_panel_max)/ea | (price_label $panels_min $panels_max) |"
  let total_row = $"| **Total** | | | **(price_label $total_min $total_max)** |"

  ($h ++ [$rod_row] ++ $fitting_body ++ [$panel_row] ++ [$total_row]) | str join "\n"
}

# ── shared ────────────────────────────────────────────────────────────────────

def wrap_md [scope intro body] {
  let frontmatter = $"---
title: ($scope.title)
status: Draft
rev: \"1\"
generated: true
---

"
  let header = $"### ($scope.title)

($intro)

"
  let footer = "

*Generated by `data/curtains.nu` from `data/*.json` — do not edit this file by hand; change the JSON and run `mise run gen`.*
"
  $frontmatter + $header + $body + $footer
}

# Compute the qty of one fitting needed for one window in scope.
# Used by both styles. Reads per_calc + optional interval_cm / covers_cm_per_unit.
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
    # Prefer curtain_width_m_per (always the hung curtain width, regardless of
    # fabric orientation). Fall back to fabric_m_per for win_rows that don't
    # carry the new field (e.g. eyelet style where fabric_m_per is 0 and
    # there are no curtain-width fittings anyway).
    let width_m = ($win_row | get -o curtain_width_m_per | default $win_row.fabric_m_per)
    let per_window = (($width_m * 100.0 / $covers) | math ceil)
    ($per_window * $win_row.qty)
  } else {
    0
  }
}

def price_label [min max] {
  if $min == $max { $"฿($min)" } else { $"฿($min)-($max)" }
}

def price_label_per_m [min max] {
  if $min == $max { $"฿($min)/m" } else { $"฿($min)-($max)/m" }
}
