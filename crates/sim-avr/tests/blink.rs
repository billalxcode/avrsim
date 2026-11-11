use sim_avr::Cpu;

fn blink_program() -> Vec<u16> {
    vec![
        0xE200, 0xB904, 0xB905, 0xEF1F, 0x951A, 0xF7F1, 0xE220, 0x2702, 0xCFF9,
    ]
}

#[test]
fn blink_toggles_portb_bit_five() {
    let mut cpu = Cpu::new(blink_program()).expect("cpu");
    let mut low = false;
    let mut high = false;
    for _ in 0..2000 {
        cpu.step().expect("step");
        let portb = cpu.io(0x05);
        high |= portb & 0x20 != 0;
        low |= portb & 0x20 == 0;
    }
    assert!(high, "PB5 never went high");
    assert!(low, "PB5 never went low");
}
