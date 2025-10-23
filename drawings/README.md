# House Renovation Drawings

This folder contains bilingual architectural drawings for the house renovation project.

## Folder Structure

```
drawings/
├── en/              # English documentation and drawings
│   ├── README.md    # Complete English documentation
│   ├── existing/    # Current house layout
│   │   ├── plan.svg     # Floor plan with semantic walls
│   │   └── section.svg  # Section A-A through building
│   └── proposed/    # Proposed renovation
│       ├── plan.svg     # Open-plan layout with loft
│       └── section.svg  # Section showing loft bedroom
│
└── th/              # Thai documentation and drawings (ภาษาไทย)
    ├── README.th.md # Complete Thai documentation
    ├── existing/    # Current house layout (ผังปัจจุบัน)
    │   ├── plan.svg     # Floor plan with Thai labels
    │   └── section.svg  # Section with Thai labels
    └── proposed/    # Proposed renovation (ผังที่เสนอ)
        ├── plan.svg     # Open-plan with Thai labels
        └── section.svg  # Section with Thai labels
```

## For Builders / สำหรับช่างก่อสร้าง

### English Speakers
👉 Go to [en/](en/) folder
- Read [en/README.md](en/README.md) for complete documentation
- View drawings in [en/existing/](en/existing/) and [en/proposed/](en/proposed/)

### Thai Speakers / ผู้พูดภาษาไทย
👉 ไปที่โฟลเดอร์ [th/](th/)
- อ่าน [th/README.th.md](th/README.th.md) สำหรับเอกสารฉบับสมบูรณ์
- ดูแบบวาดใน [th/existing/](th/existing/) และ [th/proposed/](th/proposed/)

## Key Information / ข้อมูลสำคัญ

### Design Intent / เป้าหมายการออกแบบ
- **English**: Convert 2-bedroom house to open-plan living space with loft bedroom above
- **ไทย**: เปลี่ยนบ้าน 2 ห้องนอนเป็นพื้นที่นั่งเล่นแบบเปิดพร้อมห้องนอนลอฟท์ด้านบน

### Important / สำคัญ
- **REMOVE**: Interior bedroom walls (red dashed lines / เส้นประสีแดง)
- **KEEP**: All exterior walls (black solid lines / เส้นทึบสีดำ)
- **KEEP**: Bathroom unchanged (ห้องน้ำไม่เปลี่ยนแปลง)

### Wall Types / ประเภทผนัง
- **EXTERIOR / ภายนอก**: Black solid (8px) / ทึบสีดำ - Building perimeter, must maintain
- **INTERIOR / ภายใน**: Red dashed (4px) / เส้นประสีแดง - Can be removed

## Technical Details / รายละเอียดทางเทคนิค

- **Scale / มาตราส่วน**: 1 meter = 100 pixels in SVG
- **Wall height / ความสูงผนัง**: 3.0 m / ม.
- **Ridge height / ความสูงสันหลังคา**: 5.0 m / ม.
- **Loft beam / คานลอฟท์**: At 3.0 m height, 100mm deep / ที่ความสูง 3.0 ม., ลึก 100 มม.

## Viewing the Drawings / การดูแบบวาด

All drawings are in SVG format and can be:
- Opened in any web browser
- Edited in Inkscape, Adobe Illustrator, or any SVG editor
- Printed at any scale

แบบวาดทั้งหมดเป็นรูปแบบ SVG และสามารถ:
- เปิดในเว็บเบราว์เซอร์ใดก็ได้
- แก้ไขใน Inkscape, Adobe Illustrator หรือโปรแกรมแก้ไข SVG ใดก็ได้
- พิมพ์ในขนาดใดก็ได้
