# Examples

Contoh runnable berada di masing-masing crate:

```bash
cargo run -p sim-analysis --example rc_analysis
cargo run -p sim-analysis --example inverter_vtc
cargo run -p sim-cells --example ring_oscillator
cargo run -p sim-avr --example cpu_program
cargo run -p sim-board --example blink
cargo run -p sim-avr --example uart_hello
cargo run -p sim-avr --example timer_interrupt
```

`rc_analysis` menunjukkan DC operating point, transient, dan AC pada satu
rangkaian RC.

Contoh firmware ada di `../firmware/`.

Proyek GUI:

- `led.emu.json`: resistor dan LED.
- `uart-hello.emu.json`: resistor, LED, dan tombol.

Buka melalui tombol `Open` di `embed-emu-gui`.

Panduan lengkap ada di `../docs/43-user-guide.md` dan
`../docs/58-gui-user-guide.md`.
