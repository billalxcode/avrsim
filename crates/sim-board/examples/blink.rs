//! Runs the blink program and drives the board LED through the pin model.

use sim_avr::{Cpu, ExternalPin, PinElectricalConfig};
use sim_board::Board;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = vec![
        0xE200, 0xB904, 0xB905, 0xEF1F, 0x951A, 0xF7F1, 0xE220, 0x2702, 0xCFF9,
    ];
    let mut cpu = Cpu::new(program)?;
    let mut board = Board::new();
    let config = PinElectricalConfig::default();
    let (toggles, led_on) = run_blink(&mut cpu, &mut board, &config)?;
    println!("toggles={toggles} led_on={led_on}");
    Ok(())
}

fn run_blink(
    cpu: &mut Cpu,
    board: &mut Board,
    config: &PinElectricalConfig,
) -> Result<(usize, bool), Box<dyn std::error::Error>> {
    let mut led_on = false;
    let mut toggles = 0;
    for _ in 0..2000 {
        cpu.step()?;
        let state = cpu.pin_electrical(0, 5, ExternalPin::HighZ, config)?;
        board.set_pb5_voltage(state.voltage);
        if board.led().is_on() != led_on {
            led_on = !led_on;
            toggles += 1;
        }
    }
    Ok((toggles, led_on))
}
