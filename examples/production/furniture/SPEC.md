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
| 4.x | [Electrics](electrics/SPEC.md) | Track lights, floor lamps, reading lamps | Active |
| 5.x | [Outdoor](outdoor/SPEC.md) | Plants, planters, power outlets | Active |
| 6.x | [Transport](transport/SPEC.md) | Electric scooter | Active |
| 7.x | [Dogs](dogs/SPEC.md) | Dog bed | Active |
| 8.x | [Bedroom](bedroom/SPEC.md) | Double bed | Active |

---

## Section Numbering Convention

- **1.x** = Living Room (1.1 Chaise Lounge, 1.2 Coffee Table, etc.)
- **2.x** = Bathroom (2.1 Toilet, 2.2 Basin, etc.)
- **3.x** = Dining (3.1 Table, 3.2 Chairs, etc.)
- **4.x** = Electrics (4.1 Track Lights, 4.2 Floor Lamps, etc.)
- **5.x** = Outdoor (5.1 Plants, 5.2 Outdoor Furniture, etc.)
- **6.x** = Transport (6.1 Electric Scooter, etc.)
- **7.x** = Dogs (7.1 Dog Bed, etc.)
- **8.x** = Bedroom (8.1 Double Bed, etc.)

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
├── electrics/           # Section 4.x
│   ├── SPEC.md
│   └── shopping/
│
├── outdoor/             # Section 5.x
│   ├── SPEC.md
│   └── shopping/
│
├── transport/           # Section 6.x
│   ├── SPEC.md
│   └── shopping/
│       └── 6.1-electric-scooter.md
│
├── dogs/                # Section 7.x
│   ├── SPEC.md
│   └── shopping/
│       └── 7.1-dog-bed.md
│
└── bedroom/             # Section 8.x
    ├── SPEC.md
    └── shopping/
        └── 8.1-double-bed.md
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

### Electrics (Section 4)
- [4.1 Strip Spot Lights](electrics/SPEC.md#41-strip-spot-lights-track-lighting) - Track lighting with dimming options
- [4.5 Light Switches](electrics/SPEC.md#45-light-switches) - Dimmable and standard switches
- [4.6 Outdoor Power Outlets](electrics/SPEC.md#46-outdoor-power-outlets) - Weatherproof outlets for scooter charging

### Outdoor (Section 5)
- [5.4 External Power Outlets](outdoor/SPEC.md#54-external-power-outlets) - Builder requirement for scooter charging

### Transport (Section 6)
- [6.1 Electric Scooter](transport/SPEC.md#61-electric-scooter) - Yadea OVA (~฿9,000-12,000)

### Dogs (Section 7)
- [7.1 Dog Bed](dogs/SPEC.md#71-dog-bed-2-dogs--เตียงสุนัข-2-ตัว) - Large bed for 2 dogs

### Bedroom (Section 8)
- [8.1 Double Bed](bedroom/SPEC.md#81-double-bed) - Queen size bed (IKEA MALM ~฿6,990)

---

## Adding New Items

1. Determine the category (living-room, bathroom, dining, electrics, outdoor, transport)
2. Add to the category's `SPEC.md` with next section number
3. Create shopping file: `{category}/shopping/{section}-{name}.md`
4. Research products using sellers in [SELLERS.md](SELLERS.md)
