# 18 - Timer0

Dokumen ini menjelaskan core Timer0.

---

## 1. Model

- Counter 8-bit dibangun dari D flip-flop dan incrementer struktural.
- Prescaler adalah divider clock behavioral dengan pilihan 1, 8, 64, 256, dan 1024.
- Mode normal membiarkan counter wrap dari 0xFF ke 0x00.
- Mode CTC mereset counter saat nilai compare tercapai.
- Flag overflow dan compare tersedia.
- Counter dapat direset.
- Timer0 terhubung ke CPU dan maju mengikuti cycle instruksi.
- Register I/O: TCCR0A 0x24, TCCR0B 0x25, TCNT0 0x26, OCR0A 0x27, OCR0B 0x28, TIFR0 0x35.
- TIMSK0 dapat ditulis pada 0x2E dan 0x6E untuk model I/O yang disederhanakan.
- Overflow memakai vektor 16; compare A memakai vektor 14.

---

## 2. API

```rust
pub struct Timer0 {
    // ...
}

impl Timer0 {
    pub fn new() -> Result<Self, SimError>;
    pub fn set_prescaler(&mut self, select: u8) -> Result<(), SimError>;
    pub fn set_compare(&mut self, value: u8);
    pub fn set_ctc(&mut self, enabled: bool);
    pub fn tick(&mut self, cycles: u32) -> Result<(), SimError>;
    pub fn counter(&self) -> u8;
    pub fn take_overflow(&mut self) -> bool;
    pub fn take_compare(&mut self) -> bool;
    pub fn reset(&mut self) -> Result<(), SimError>;
}
```

---

## 3. Test

- Prescaler 8 membagi clock dengan benar.
- Counter wrap dan flag overflow.
- Compare flag pada mode normal.
- CTC mereset counter saat compare.
- Reset mengosongkan counter dan flag.
- Prescaler tidak valid ditolak.
- Timer0 maju dari cycle instruksi CPU.
- Overflow flag dan compare flag terbaca lewat TIFR0.
- Overflow interrupt masuk ke vektor 16.

Total test `sim-avr`: 114.

---

## 4. Batas

- Prescaler masih behavioral, bukan rangkaian counter gate-level.
- Pemetaan TIMSK0 memakai alamat I/O yang disederhanakan.
- Compare interrupt sudah meminta vektor 14, tetapi belum diuji end-to-end.
- PWM belum diimplementasikan.
- Timer1 dan Timer2 belum dimulai.
