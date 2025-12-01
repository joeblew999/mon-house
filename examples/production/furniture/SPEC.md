---
leaf_folders: [shopping, images, assets]
---
# Furniture Specifications - Master Index

This document serves as the index for all furniture and fixture specifications.

Each category has its own SPEC.md file in its respective folder.

---

## Categories

| Section | Category | Items | Status |
|---------|----------|-------|--------|
| 1.x | [Living Room](living-room/SPEC.md) | Chaise lounge, coffee table, TV stand | In Progress |
| 2.x | [Bathroom](bathroom/SPEC.md) | Toilet, basin, shower, tiles, etc. (18 items) | Active |
| 3.x | [Dining](dining/SPEC.md) | Dining table, chairs, pendant light | TBD |
| 4.x | [Lighting](lighting/SPEC.md) | Ceiling lights, floor lamps, reading lamps | TBD |
| 5.x | [Outdoor](outdoor/SPEC.md) | Plants, planters, outdoor furniture | TBD |

---

## Section Numbering Convention

- **1.x** = Living Room (1.1 Chaise Lounge, 1.2 Coffee Table, etc.)
- **2.x** = Bathroom (2.1 Toilet, 2.2 Basin, etc.)
- **3.x** = Dining (3.1 Table, 3.2 Chairs, etc.)
- **4.x** = Lighting (4.1 Living Room Light, 4.2 Bedroom Light, etc.)
- **5.x** = Outdoor (5.1 Plants, 5.2 Outdoor Furniture, etc.)

---

## Folder Structure

```
furniture/
├── SPEC.md              # This index file
├── SELLERS.md           # Shared list of Thailand retailers
├── README.md            # Naming conventions and guidelines
│
├── living-room/         # Section 1.x
│   ├── SPEC.md
│   └── shopping/
│       └── 1.1-chaise-lounge.md
│
├── bathroom/            # Section 2.x
│   ├── SPEC.md
│   └── shopping/
│       ├── 2.1-toilet.md
│       └── ...
│
├── dining/              # Section 3.x
│   ├── SPEC.md
│   └── shopping/
│
├── lighting/            # Section 4.x
│   ├── SPEC.md
│   └── shopping/
│
└── outdoor/             # Section 5.x
    ├── SPEC.md
    └── shopping/
```

---

## Quick Links

### Living Room (Section 1)
- [1.1 Chaise Lounge](living-room/SPEC.md#11-chaise-lounge--reclining-chair) - ฿2,999 (Molesun)

### Bathroom (Section 2)
- [2.1 Toilet](bathroom/SPEC.md#21-toilet) - COTTO SC19517(T) ฿32,253
- [2.2 Basin](bathroom/SPEC.md#22-sinkbasin) - COTTO C0107 ฿2,090
- [2.5 Shower Taps](bathroom/SPEC.md#25-shower-tapsfaucet) - COTTO CT2326A ฿2,990
- [2.6 Basin Taps](bathroom/SPEC.md#26-basin-tapsfaucet) - COTTO CT1160AN ฿1,090
- [Full list...](bathroom/SPEC.md)

### Dining (Section 3)
- *TBD*

### Lighting (Section 4)
- *TBD*

### Outdoor (Section 5)
- *TBD*

---

## Adding New Items

1. Determine the category (living-room, bathroom, dining, lighting, outdoor)
2. Add to the category's `SPEC.md` with next section number
3. Create shopping file: `{category}/shopping/{section}-{name}.md`
4. Research products using sellers in [SELLERS.md](SELLERS.md)
