//! Runs a small AVR program that adds two registers.

use sim_avr::Cpu;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = vec![0xE005, 0xE013, 0x0F01, 0x9598];
    let mut cpu = Cpu::new(program)?;
    cpu.run(10)?;
    println!(
        "r16={} cycles={} halted={}",
        cpu.register(16),
        cpu.cycles(),
        cpu.is_halted()
    );
    Ok(())
}
