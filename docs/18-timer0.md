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

Total test `sim-avr`: 110.

---

## 4. Batas

- Prescaler masih behavioral, bukan rangkaian counter gate-level.
- Timer belum dihubungkan ke I/O register CPU.
- Overflow dan compare belum masuk ke interrupt controller.
- PWM belum diimplementasikan.
- Timer1 dan Timer2 belum dimulai.
