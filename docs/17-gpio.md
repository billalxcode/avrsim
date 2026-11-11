# 17 - GPIO

Dokumen ini menjelaskan model GPIO struktural untuk port B, C, dan D.

---

## 1. Model

Setiap port memiliki:

- Register DDR 8-bit dari D flip-flop.
- Register PORT 8-bit dari D flip-flop.
- Delapan net input pin eksternal.
- Delapan net output pin.
- Delapan sinyal pull-up aktif.

Output pin:

```
pin = DDR ? PORT : pin_input
```

Pull-up aktif saat `DDR = 0` dan `PORT = 1`.

---

## 2. Peta I/O

| Port | PIN | DDR | PORT |
|---|---|---|---|
| B | 0x03 | 0x04 | 0x05 |
| C | 0x06 | 0x07 | 0x08 |
| D | 0x09 | 0x0A | 0x0B |

CPU mengarahkan IN, OUT, SBI, CBI, SBIC, dan SBIS ke register GPIO
untuk alamat tersebut.

---

## 3. API

```rust
pub struct GpioPort {
    // ...
}

impl GpioPort {
    pub fn new(builder: &mut DigitalBuilder, prefix: &str) -> Result<Self, SimError>;
    pub fn set_ddr(&mut self, circuit: &mut DigitalCircuit, value: u8) -> Result<(), SimError>;
    pub fn set_port(&mut self, circuit: &mut DigitalCircuit, value: u8) -> Result<(), SimError>;
    pub fn set_pin_input(&mut self, circuit: &mut DigitalCircuit, value: u8) -> Result<(), SimError>;
    pub fn read_ddr(&self, circuit: &DigitalCircuit) -> u8;
    pub fn read_port(&self, circuit: &DigitalCircuit) -> u8;
    pub fn read_pin(&self, circuit: &DigitalCircuit) -> u8;
    pub fn read_pull_up(&self, circuit: &DigitalCircuit) -> u8;
}
```

CPU menyediakan `set_pin_input(port, value)` dan `pull_up(port)`.

---

## 4. Test

- Output mengikuti DDR dan PORT.
- Input mengikuti pin saat DDR rendah.
- Arah campuran menggabungkan output dan input.
- Pull-up aktif saat DDR rendah dan PORT tinggi.
- Reset mengosongkan DDR dan PORT.
- CPU menulis DDR/PORT dan membaca PIN.
- CPU membaca pin input eksternal.
- Pull-up CPU mengikuti PORT.

Total test `sim-avr`: 101.

---

## 5. Batas

- Model hanya dua-state, tanpa high-Z atau kontensi.
- Belum ada tegangan, arus, kapasitansi, atau drive strength.
- Belum ada pin-change interrupt.
- Belum ada timer, UART, SPI, I2C, ADC, atau EEPROM.
