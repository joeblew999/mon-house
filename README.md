# House Renovation Project / โครงการรีโนเวทบ้าน

**English**: Complete architectural documentation for converting a 2-bedroom house into an open-plan living space with loft bedroom.

**ภาษาไทย**: เอกสารสถาปัตยกรรมสมบูรณ์สำหรับการเปลี่ยนบ้าน 2 ห้องนอนเป็นพื้นที่นั่งเล่นแบบเปิดพร้อมห้องนอนลอฟท์

---

## 🏗️ For Builders / สำหรับช่างก่อสร้าง

### **English Speakers** → [drawings/en/](drawings/en/)
**START HERE**: [📖 Read English Documentation](drawings/en/README.md)

Your drawings:
- **Current layout**: [drawings/en/existing/](drawings/en/existing/) (floor plan + section)
- **New design**: [drawings/en/proposed/](drawings/en/proposed/) (open-plan + loft)

---

### **Thai Speakers / ผู้พูดภาษาไทย** → [drawings/th/](drawings/th/)
**เริ่มที่นี่**: [📖 อ่านเอกสารภาษาไทย](drawings/th/README.th.md)

แบบวาดของคุณ:
- **ผังปัจจุบัน**: [drawings/th/existing/](drawings/th/existing/) (แบบผัง + หน้าตัด)
- **แบบใหม่**: [drawings/th/proposed/](drawings/th/proposed/) (แบบเปิด + ลอฟท์)

---

## ⚠️ Critical Information / ข้อมูลสำคัญมาก

### What to Build / สิ่งที่ต้องสร้าง

| Action / การกระทำ | English | ไทย |
|---|---|---|
| **REMOVE / รื้อ** | Interior bedroom walls<br>(red dashed lines) | ผนังภายในห้องนอน<br>(เส้นประสีแดง) |
| **KEEP / เก็บไว้** | All exterior walls<br>(black solid lines) | ผนังภายนอกทั้งหมด<br>(เส้นทึบสีดำ) |
| **KEEP / เก็บไว้** | Bathroom - NO changes | ห้องน้ำ - ไม่เปลี่ยนแปลง |
| **ADD / เพิ่ม** | Loft bedroom at 3.0m height | ห้องนอนลอฟท์ที่ความสูง 3.0 ม. |

### Wall Types / ประเภทผนัง

**On the drawings, walls have different colors:**

| Type | Appearance | English | ไทย |
|------|-----------|---------|-----|
| **EXTERIOR** | ⬛ Black solid (thick) | Building perimeter<br>**MUST KEEP** | เส้นรอบอาคาร<br>**ต้องเก็บไว้** |
| **INTERIOR** | 🔴 Red dashed (thin) | Internal partitions<br>**CAN REMOVE** | ผนังกั้นภายใน<br>**รื้อได้** |

---

## 📐 Technical Specifications / ข้อกำหนดทางเทคนิค

| Item | Specification |
|------|--------------|
| **Wall height / ความสูงผนัง** | 3.0 meters / เมตร |
| **Roof ridge / สันหลังคา** | 5.0 meters / เมตร |
| **Loft beam / คานลอฟท์** | 3.0m high, 100mm deep<br>ความสูง 3.0 ม., หนา 100 มม. |
| **Scale / มาตราส่วน** | 1 meter = 100 pixels in drawings<br>1 เมตร = 100 พิกเซล |

---

## 📂 Project Structure / โครงสร้างโปรเจค

```
mon-house/
├── README.md                    ← You are here / คุณอยู่ที่นี่
│
├── drawings/                    # All architectural drawings
│   │
│   ├── en/                      # 🇬🇧 English version
│   │   ├── README.md            # Full specifications in English
│   │   ├── existing/            # Current house layout
│   │   │   ├── plan.svg         # Floor plan
│   │   │   └── section.svg      # Building section A-A
│   │   └── proposed/            # Renovation design
│   │       ├── plan.svg         # New open-plan layout
│   │       └── section.svg      # Section showing loft
│   │
│   └── th/                      # 🇹🇭 Thai version / ฉบับภาษาไทย
│       ├── README.th.md         # ข้อกำหนดฉบับสมบูรณ์
│       ├── existing/            # ผังบ้านปัจจุบัน
│       │   ├── plan.svg         # แบบผัง
│       │   └── section.svg      # หน้าตัด A-A
│       └── proposed/            # แบบรีโนเวท
│           ├── plan.svg         # ผังแบบเปิดใหม่
│           └── section.svg      # หน้าตัดแสดงลอฟท์
│
├── photos/                      # Reference images
│   ├── paint.png                # Paint/color reference
│   └── vision.png               # Design vision
│
└── CLAUDE.md                    # Development context (for developers)
```

---

## 🖥️ Viewing the Drawings / การดูแบบวาด

**All drawings are SVG format** - open in any web browser or SVG editor.

**แบบวาดทั้งหมดเป็น SVG** - เปิดในเว็บเบราว์เซอร์หรือโปรแกรมแก้ไข SVG ได้

### How to View / วิธีดู:
1. **In browser / ในเบราว์เซอร์**: Double-click any `.svg` file
2. **Print / พิมพ์**: Open in browser → Print (scales automatically)
3. **Edit / แก้ไข**: Use Inkscape (free) or Adobe Illustrator

---

## 🎯 Design Intent / เป้าหมายการออกแบบ

### English Version:
**Convert** the existing 2-bedroom house into a modern open-plan living space:
- Remove interior bedroom walls to create large open living area (3.0m × 6.4m)
- Add loft bedroom above at 3.0m level on structural beam
- Maintain existing bathroom, living room, and kitchen
- Maximize vertical space while keeping ground floor open

**Result**: More spacious feel, modern open design, bedroom privacy maintained on loft level.

### Thai Version / ฉบับภาษาไทย:
**เปลี่ยน** บ้าน 2 ห้องนอนเป็นพื้นที่นั่งเล่นแบบเปิดสมัยใหม่:
- รื้อผนังภายในห้องนอนเพื่อสร้างพื้นที่นั่งเล่นขนาดใหญ่ (3.0 ม. × 6.4 ม.)
- เพิ่มห้องนอนลอฟท์ด้านบนที่ระดับ 3.0 ม. บนคานโครงสร้าง
- รักษาห้องน้ำ ห้องนั่งเล่น และห้องครัวที่มีอยู่
- ใช้พื้นที่แนวตั้งสูงสุดในขณะที่เก็บชั้นล่างเป็นแบบเปิด

**ผลลัพธ์**: รู้สึกกว้างขึ้น ดีไซน์เปิดสมัยใหม่ ความเป็นส่วนตัวของห้องนอนยังคงอยู่บนระดับลอฟท์

---

## 📞 Questions? / มีคำถาม?

- **English documentation**: [drawings/en/README.md](drawings/en/README.md)
- **Thai documentation**: [drawings/th/README.th.md](drawings/th/README.th.md)

---

**Last updated / อัปเดตล่าสุด**: October 2025
