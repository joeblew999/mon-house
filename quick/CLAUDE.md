# CLAUDE

This is a **reusable bilingual document generation system** for construction specifications.

**Platform requirement**: This system MUST work on all desktop platforms — Windows, macOS, and Linux. All scripts, tools, and tasks must be cross-platform compatible. Avoid platform-specific shell syntax, paths, or commands.

---

## THE THREE NON-NEGOTIABLE RULES

### Rule 1 — Watch is the primary workflow. Always.

```bash
mise run watch
```

This is the only command needed during normal work. It watches `specs/*.md` and
`scripts/theme.typ`, and on every save it runs `fonts` then `all` — in the
right order, skipping everything that hasn't changed.

**Never run `mise run all` manually during development.** Watch does it.
**Never run `mise run fonts` manually during development.** Watch does it.

### Rule 2 — Every task MUST be idempotent.

If nothing changed, nothing runs. No exceptions.

Every task must have EITHER:
- mise `sources`/`outputs` — mise skips the task if outputs are newer than sources
- Script-level hash/file check — script exits immediately if state matches expected

**If you add a new task and it is not idempotent, you have broken the system.**

Test idempotency by running a task twice. The second run must produce no output
and do no work. If it does work, fix it before committing.

### Rule 3 — No hardcoded paths. Ever.

Every directory the Rust code reads from or writes to MUST go through a `Config`
field backed by an env var. This allows any path to be redirected to an S3 mount,
NFS share, or alternate local location without touching source code.

**If you add code that opens, globs, or writes a file using a literal path string
(`"specs/"`, `"out/"`, `"scripts/"`, etc.) you have broken the system.**

Use `cfg.specs_dir`, `cfg.out_dir`, `cfg.scripts_dir`, etc. instead.

---

## Directory Layout — All Paths Are Configurable

Every directory is an env var. Override any of them in `mise.local.toml`
(gitignored) without touching `mise.toml` or the Rust source.

| Env var | Default | Purpose | S3-ready? |
|---|---|---|---|
| `QUICK_RESOURCES_DIR` | `resources` | Parent for all input assets | — |
| `QUICK_FONT_DIR` | `resources/fonts` | Downloaded `.ttf` files | ✓ |
| `QUICK_IMAGES_DIR` | `resources/images` | Spec images | ✓ |
| `QUICK_OUT_DIR` | `out` | Generated PDFs | ✓ |
| `QUICK_SPECS_DIR` | `specs` | EN source markdown + TEMPLATE.md | ✓ |
| `QUICK_SCRIPTS_DIR` | `scripts` | Typst theme code | — |
| `QUICK_THEME_FILE` | `scripts/theme.typ` | Active theme wrapper | — |

**To point `out/` and assets at S3 mounts** (example `mise.local.toml`):
```toml
[env]
QUICK_OUT_DIR       = "/mnt/s3-specs-output"
QUICK_RESOURCES_DIR = "/mnt/s3-assets"
```
Nothing else changes — the binary reads these at startup.

**`QUICK_FONT_DIR` and `QUICK_IMAGES_DIR`** derive from `QUICK_RESOURCES_DIR`
automatically but can be overridden independently.

**`QUICK_THEME_FILE`** derives from `QUICK_SCRIPTS_DIR` automatically but can
be overridden independently.

Scripts (`scripts/`) are Typst source code — versioned, committed, not S3 assets.

---

## Why these rules matter

Watch + idempotency + path abstraction together mean:
- Save a file → only that file's outputs rebuild. Nothing else runs.
- Save the same file again → nothing runs at all.
- Add a font to theme.typ → font downloads, all PDFs rebuild. One save.
- Redirect `out/` to S3 → CI writes PDFs directly to S3 with zero code change.
- 267ms for a full no-op pass. Zero wasted work.

Break any rule and the system degrades into a manual, slow, fragile build.

---

## Primary Workflow

```bash
mise install          # first time only — installs all tools
mise run fonts        # first time only — downloads fonts
mise run watch        # ← start this. leave it running. edit files.
```

`mise run watch` calls the Rust pipeline directly (no mise subprocess):
1. `fonts::cmd_download()` — downloads newly declared fonts (hash + per-file skip)
2. `translate::cmd_translate()` — SHA-256 hash skip; calls claude CLI only if changed
3. `build::cmd_build()` — only runs if `out/.build-stamp` is older than any `.th.md` or `theme.typ`

Watch monitors `QUICK_SPECS_DIR`, `QUICK_SCRIPTS_DIR`, and `QUICK_SCRIPTS_DIR/themes/`.

---

## What triggers a rebuild

| File changed | What happens |
|---|---|
| Any `specs/*.md` spec | translates if changed → rebuilds EN + Thai PDFs for that spec |
| `scripts/theme.typ` layout | rebuilds all PDFs |
| `scripts/theme.typ` font added | downloads new font → rebuilds all PDFs |
| Nothing changed | nothing runs (267ms no-op) |

---

## Idempotency — how each task achieves it

| Task | Layer 1 (mise stamp) | Layer 2 (Rust hash/exists) | Layer 3 (per item) |
|---|---|---|---|
| `fonts` | stamp newer than theme.typ → skip | hash + all files present → exit | skip existing .ttf files |
| `translate` | — | SHA-256 in `.th.md.hash` → skip | — |
| `all` | `out/.build-stamp` newer than `.th.md` + `theme.typ` → skip | — | — |
| `watch` | n/a (watch never exits) | same as fonts + translate | `needs_build_in()` stamp check |

**Verify idempotency:**
```bash
mise run fonts:test      # all 3 layers: health + hash + per-file
mise run test:unit       # 4 unit tests for needs_build timestamp logic
mise run watch           # save an unchanged file — nothing should rebuild
```

---

## Single Source of Truth

**Only edit `specs/*.md` files (English source).** Everything else is generated or derived.

| File | Status | Regenerated by |
|---|---|---|
| `specs/*.md` EN specs | **committed — edit these** | — |
| `resources/images/` | **committed** | — |
| `specs/*.th.md` Thai translations | **committed** (so CI builds without claude) | `mise run translate` after editing EN |
| `specs/*.th.md.hash` | **committed** (idempotency stamps) | `mise run translate` |
| `out/*.pdf` | gitignored — generated | `mise run all` or CI |
| `resources/fonts/` | gitignored — downloaded | `mise run fonts` |

**After editing an EN spec:** `mise run watch` auto-translates and rebuilds. Commit both the `.md` and the new `.th.md` + `.th.md.hash`.

---

## CI — GitHub Actions

Every push that changes specs or the CLI triggers `.github/workflows/build-specs.yml`:

1. Builds `quick-tool` from source
2. Downloads fonts
3. Runs `mise run all` (translate skips — hashes match committed files)
4. Uploads PDFs as a workflow artifact (90 day retention)
5. Creates/updates the **`specs-latest`** GitHub Release with all PDFs as direct download links

**Contractors download from:** `https://github.com/joeblew999/mon-house/releases/tag/specs-latest`

No login required. Trigger a manual rebuild anytime from the GitHub Actions tab.

---

## Fonts

**Single source of truth: `scripts/theme.typ` font stack.**

```typ
font: ("Inter", "Noto Sans", "Noto Sans Thai"),
```

Add a family name here → save → watch downloads it and rebuilds PDFs. No config file.

```bash
mise run "fonts:search" -- "sarabun"   # discover a font name
mise run "fonts:test"                  # verify health + all 3 idempotency layers
```

---

## Frontmatter — MANDATORY on every spec file

Every `specs/*.md` file MUST start with YAML frontmatter:

```markdown
---
title: Gate
status: Draft
rev: "1"
---
```

| Field | Values | Effect if missing |
|---|---|---|
| `title` | any text | Cover bar, header, footer all blank |
| `status` | `Draft` / `For Review` / `Approved` | Footer shows no status |
| `rev` | `"1"`, `"2"` etc. | Footer shows blank rev |

Use `specs/TEMPLATE.md` as the starting point.

---

## Adding a New Spec

```bash
mise run new -- SPECNAME   # creates specs/SPECNAME.md with frontmatter
# edit specs/SPECNAME.md
# mise run watch picks it up on save
```

Or manually: copy `specs/TEMPLATE.md` → `specs/NEWSPEC.md`, fill in content, save.

---

## Adding a New Task

1. Add to `mise.toml`
2. Use `{{env.QUICK_*}}` variables for all paths — never hardcode `specs/`, `out/`, etc.
3. Add `sources` + `outputs` for mise-level idempotency
4. Add script-level hash/file check if sources/outputs are not sufficient
5. Run the task twice — second run must do nothing
6. Confirm `mise run watch` triggers it correctly when relevant files change

---

## Adding a New Configurable Path

If a new directory is needed (e.g. `QUICK_ARCHIVE_DIR`):

1. Add the field to `Config` in `cli/src/main.rs` with `#[arg(env = "QUICK_ARCHIVE_DIR", default_value = "archive")]`
2. Add a resolver method if it derives from another dir (e.g. `resolved_archive_dir()`)
3. Add the env var to `mise.toml [env]` with a comment explaining what it points at
4. Use `cfg.archive_dir` (or `cfg.resolved_archive_dir()`) everywhere in Rust — never a string literal

---

## Tools (installed by mise)

- `typst` — compiles `.typ` → `.pdf`
- `pandoc` — converts `.md` → `.typ`
- `quick-tool` — built from `cli/` by `mise run build-cli`; handles fonts, translate, build, watch, new, clean

---

## Themes

Switch between built-in themes without touching any spec files:

```bash
mise run themes:list                    # list all themes; shows active ▶
mise run themes:switch -- minimal       # switch active theme
mise run themes:switch -- compact       # compact layout, 9pt, tighter margins
mise run themes:switch -- default       # restore default blue theme
mise run themes:test-all                # compile test PDF for every theme
mise run themes:check                   # full health: wrapper + compile round-trip
```

Adding a new theme: create `scripts/themes/newname.typ` with the same `conf()` signature,
add an entry to `scripts/themes/registry.toml`, then `mise run themes:check`.

---

## Testing

```bash
# Fast — no tools needed; runs the 4 needs_build timestamp unit tests
mise run test:unit

# Full E2E — requires pandoc, typst, fonts, all PDFs built
# Verifies actual side effects: PDFs exist and are non-empty, .th.md files present,
# fonts/.done stamp valid, theme wrapper contains the correct import
mise run test:e2e

# E2E with a specific theme
QUICK_TEST_THEME=compact mise run test:e2e
QUICK_TEST_THEME=minimal mise run test:e2e

# Both tiers
mise run test
```

**Watch as the primary test workflow:** `mise run watch` is the most important test
of the full system. When you save an unchanged file and NOTHING runs, all three
idempotency layers are working. When you save a changed file and ONLY the affected
spec rebuilds, the pipeline is correct.

**E2E test coverage:**
| Test | What side effect is verified |
|---|---|
| `tier2_translate_produces_th_md_files` | Every spec has a `.th.md`, non-empty |
| `tier2_translate_is_idempotent` | Second translate produces `0 translated` |
| `tier2_build_produces_pdfs_for_all_specs` | All PDFs exist and are > 1 KB |
| `tier2_themes_switch_updates_wrapper_file` | `scripts/theme.typ` imports the active theme |
| `tier2_themes_test_all_every_theme_compiles` | All 3 themes produce valid PDFs |
| `tier2_fonts_download_all_files_present` | All `.ttf` files present, `resources/fonts/.done` exists |
| `tier2_full_watch_pipeline_side_effects` | Complete fonts → translate → build → check |

---

## Reference Commands

These are for CI, debugging, or first-time setup only.

```bash
mise run fonts                          # download fonts from theme.typ
mise run "fonts:test"                   # verify fonts: health + all 3 idempotency layers
mise run "fonts:search" -- "sarabun"    # search Google Fonts registry
mise run new -- DECK                    # scaffold a new spec from specs/TEMPLATE.md
mise run one -- GATE                    # translate + build single spec
mise run all                            # translate + build all specs
mise run clean                          # remove out/
```

---

## Image Grids in Specs

To show two images side-by-side in a spec, use a raw Typst block:

````markdown
```{=typst}
#grid-images(
  image("resources/images/gate/before.jpg"),
  image("resources/images/gate/after.jpg"),
)
```
````

`grid-images` is defined in `scripts/theme.typ`. Note: `image()` paths are relative
to the working directory (`quick/`), which is where `_tmp.typ` is compiled.

---

## Fresh Machine Setup

```bash
git clone ...
cd quick
mise install            # installs pandoc, typst, quick-tool (built from cli/)
mise run fonts          # downloads fonts
mise run "fonts:test"   # verify fonts healthy
mise run watch          # start working
```
