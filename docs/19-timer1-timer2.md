# 19 - Timer1 dan Timer2

Dokumen ini menjelaskan core Timer1 16-bit dan Timer2 8-bit.

---

## 1. Timer1

- Counter 16-bit dari D flip-flop dan incrementer struktural.
- Prescaler behavioral 1, 8, 64, 256, dan 1024.
- Compare A dan compare B.
- Input capture dengan nilai tangkapan software.
- CTC memakai compare A.
- Fast PWM pada compare A.
- Flag overflow, compare A, compare B, dan capture.

API:

```rust
pub struct Timer1 { /* ... */ }

impl Timer1 {
    pub fn new() -> Result<Self, SimError>;
    pub fn set_prescaler(&mut self, select: u8) -> Result<(), SimError>;
    pub fn set_compare_a(&mut self, value: u16);
    pub fn set_compare_b(&mut self, value: u16);
    pub fn capture(&mut self, value: u16);
    pub fn set_ctc(&mut self, enabled: bool);
    pub fn set_pwm(&mut self, inverting: bool);
    pub fn tick(&mut self, cycles: u32) -> Result<(), SimError>;
    pub fn counter(&self) -> u16;
}
```

---

## 2. Timer2

- Memakai core Timer0 8-bit.
- Prescaler, compare, CTC, PWM, overflow, dan compare flag.
- Async mode disimpan sebagai flag.
- Async clock belum dimodelkan.

API:

```rust
pub struct Timer2 { /* ... */ }

impl Timer2 {
    pub fn new() -> Result<Self, SimError>;
    pub fn set_async(&mut self, enabled: bool);
    pub fn is_async(&self) -> bool;
    pub fn tick(&mut self, cycles: u32) -> Result<(), SimError>;
}
```

---

## 3. Test

- Timer1 prescaler membagi clock.
- Compare A dan B men-set flag.
- Capture menyimpan nilai.
- CTC mereset counter.
- PWM mengikuti compare A.
- Timer2 menghitung, wrap, dan set overflow.
- Timer2 async flag tersimpan.
- Timer2 PWM mengikuti compare.

Total test `sim-avr`: 205.

---

## 4. Integrasi Interrupt

- CPU menyediakan `poll_timer1(mask)` dan `poll_timer2(mask)`.
- Timer1 capture memakai vektor 10.
- Timer1 compare A dan B memakai vektor 11 dan 12.
- Timer1 overflow memakai vektor 13.
- Timer2 compare memakai vektor 7.
- Timer2 overflow memakai vektor 9.

## 5. Batas

- Timer1 dan Timer2 belum dihubungkan ke register I/O CPU.
- Capture masih berupa pemanggilan software, bukan edge pin.
- Timer1 PWM baru fast PWM compare A.
- Timer2 async hanya flag, belum clock async.
- Interrupt memakai polling method, belum otomatis dari tick CPU.
