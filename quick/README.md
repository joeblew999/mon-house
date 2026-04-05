# Quick Specs — Laem Chabang House

Construction specification documents. Each spec is written in English, auto-translated to Thai, and compiled to PDF.

---

## Spec Files

| File | Description |
|---|---|
| [GATE.md](GATE.md) | Sliding gate replacement |
| [METAL.md](METAL.md) | Roof sheeting and steelwork |
| [CONCRETE.md](CONCRETE.md) | Concrete works |
| [CEILING.md](CEILING.md) | Ceiling works |
| [PAINT.md](PAINT.md) | Painting |
| [WINDOWS.md](WINDOWS.md) | Windows |

---

## Common Commands

### Build a single spec
```
mise run one -- GATE
mise run one -- METAL
```
Translates to Thai + builds EN and Thai PDFs into `out/`.

### Build all specs
```
mise run all
```

### Translate only (no PDF)
```
mise run translate
```

### Watch for changes (auto-rebuild)
```
mise run watch
```

### Clean output folder
```
mise run clean
```

---

## Output

PDFs are written to `out/`:
- `out/GATE.pdf` — English
- `out/GATE.th.pdf` — Thai

---

## Adding a New Spec

1. Create `NEWSPEC.md` with frontmatter:
```markdown
---
title: My Spec
status: Draft
rev: "1"
---
```
2. Run `mise run one -- NEWSPEC`

---

## Frontmatter Fields

| Field | Values | Notes |
|---|---|---|
| `title` | any text | Shown in header and footer |
| `status` | `Draft` / `For Review` / `Approved` | Shown in footer |
| `rev` | `"1"`, `"2"` etc. | Shown in footer |

---

## Source of Truth

**Only edit `*.md` files (English source).** Everything else is generated:

| What | How to regenerate |
|---|---|
| `*.th.md` Thai translations | `mise run all` (auto-overwritten every build) |
| `out/*.pdf` PDFs | `mise run all` |
| `fonts/` | `mise run fonts` |

`*.th.md` and `out/` are gitignored — never commit them, never edit them manually.

---

## Folder Structure

```
quick/
├── mise.toml          # Task runner (all commands defined here)
├── font_config.toml   # Font declarations
├── README.md          # This file
├── GATE.md            # ← edit these (English source only)
├── METAL.md
├── images/            # Photos referenced by specs
├── scripts/
│   ├── translate_markdown.py   # AI translation (Claude CLI)
│   ├── download_fonts.py
│   └── theme.typ      # PDF theme
└── out/               # Generated PDFs (gitignored)
```
