//! Runs the Timer0 overflow interrupt demo.

use sim_avr::{Cpu, Sreg};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cpu = Cpu::new(demo_program())?;
    cpu.set_sreg(Sreg {
        interrupt: true,
        ..Sreg::default()
    });
    for _ in 0..2000 {
        cpu.step()?;
        if cpu.register(17) == 7 {
            break;
        }
    }
    println!("r17={} cycles={}", cpu.register(17), cpu.cycles());
    Ok(())
}

fn demo_program() -> Vec<u16> {
    let mut program = vec![0u16; 40];
    program[0] = 0xE001;
    program[1] = 0xBD05;
    program[2] = 0xE001;
    program[3] = 0xBD0E;
    program[4] = 0xEF2F;
    program[5] = 0x952A;
    program[6] = 0xF7F1;
    program[7] = 0xCFFF;
    program[32] = 0xE017;
    program[33] = 0x9518;
    program
}
