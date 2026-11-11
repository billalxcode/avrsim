//! Simple AVR monitor used by the CLI.

use sim_avr::{AvrTrace, Cpu, ProgramMemory, disassemble};

/// A monitor around one AVR core instance.
pub struct Monitor {
    cpu: Cpu,
    program: Vec<u16>,
}

impl Monitor {
    /// Creates a monitor for a program.
    pub fn new(program: Vec<u16>) -> Result<Self, String> {
        let cpu = Cpu::new(program.clone()).map_err(|error| error.to_string())?;
        Ok(Self { cpu, program })
    }

    /// Executes one instruction.
    pub fn step(&mut self) -> Result<String, String> {
        let trace = self.cpu.step().map_err(|error| error.to_string())?;
        Ok(match trace {
            Some(trace) => self.trace_text(&trace),
            None => "halted".to_string(),
        })
    }

    /// Executes until halt or the step limit.
    pub fn run(&mut self, limit: usize) -> Result<String, String> {
        let traces = self.cpu.run(limit).map_err(|error| error.to_string())?;
        Ok(format!(
            "{} instructions, {} cycles",
            traces.len(),
            self.cpu.cycles()
        ))
    }

    /// Returns a register dump.
    #[must_use]
    pub fn registers(&self) -> String {
        let mut text = String::new();
        for index in 0..32 {
            if index % 4 == 0 {
                if index > 0 {
                    text.push('\n');
                }
                text.push_str(&format!("r{index:02}"));
            }
            text.push_str(&format!("={:02X} ", self.cpu.register(index)));
        }
        text
    }

    /// Returns a SREG dump.
    #[must_use]
    pub fn sreg(&self) -> String {
        let sreg = self.cpu.sreg();
        format!(
            "C={} Z={} N={} V={} S={} H={} T={} I={}",
            u8::from(sreg.carry),
            u8::from(sreg.zero),
            u8::from(sreg.negative),
            u8::from(sreg.overflow),
            u8::from(sreg.sign),
            u8::from(sreg.half_carry),
            u8::from(sreg.transfer),
            u8::from(sreg.interrupt)
        )
    }

    /// Disassembles instructions starting at a word address.
    pub fn disassemble(&self, address: u16, count: usize) -> Result<String, String> {
        let memory = ProgramMemory::new(self.program.clone());
        let mut pc = address;
        let mut lines = Vec::new();
        for _ in 0..count {
            let fetched = memory.fetch(pc).map_err(|error| error.to_string())?;
            lines.push(format!("0x{pc:04X}: {}", disassemble(fetched.instruction)));
            pc = pc.wrapping_add(u16::from(fetched.words));
        }
        Ok(lines.join("\n"))
    }

    /// Returns a hexadecimal memory dump.
    #[must_use]
    pub fn memory(&self, address: u16, length: usize) -> String {
        (0..length)
            .map(|offset| {
                let current = address.wrapping_add(offset as u16);
                format!("{:04X}={:02X}", current, self.cpu.data(current))
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Returns an I/O register value.
    #[must_use]
    pub fn io(&self, address: u8) -> String {
        format!("0x{address:02X}={:02X}", self.cpu.io(address))
    }

    /// Returns the current program counter.
    #[must_use]
    pub fn pc(&self) -> u16 {
        self.cpu.pc()
    }

    /// Returns the current cycle count.
    #[must_use]
    pub fn cycles(&self) -> u64 {
        self.cpu.cycles()
    }

    fn trace_text(&self, trace: &AvrTrace) -> String {
        format!(
            "pc=0x{:04X} cycles={} {}",
            trace.pc,
            trace.cycles,
            disassemble(trace.instruction)
        )
    }
}

/// Parses one hexadecimal instruction word.
pub fn parse_word(raw: &str) -> Result<u16, String> {
    let text = raw.trim_start_matches("0x");
    u16::from_str_radix(text, 16).map_err(|_| format!("invalid word: {raw}"))
}

/// Parses a decimal or hexadecimal number.
pub fn parse_number(raw: &str) -> Result<usize, String> {
    if let Some(hex) = raw.strip_prefix("0x") {
        return usize::from_str_radix(hex, 16).map_err(|_| format!("invalid number: {raw}"));
    }
    raw.parse().map_err(|_| format!("invalid number: {raw}"))
}

#[cfg(test)]
mod tests {
    use super::{Monitor, parse_number, parse_word};

    fn ldi(register: u16, value: u16) -> u16 {
        0xE000 | ((value & 0xF0) << 4) | ((register - 16) << 4) | (value & 0x0F)
    }

    #[test]
    fn parses_words_and_numbers() {
        assert_eq!(parse_word("0xE005"), Ok(0xE005));
        assert_eq!(parse_word("e005"), Ok(0xE005));
        assert_eq!(parse_number("16"), Ok(16));
        assert_eq!(parse_number("0x10"), Ok(16));
        assert!(parse_word("zz").is_err());
    }

    #[test]
    fn monitor_steps_and_dumps_registers() {
        let mut monitor = Monitor::new(vec![ldi(16, 5), 0x9598]).expect("monitor");
        let step = monitor.step().expect("step");
        assert!(step.contains("ldi r16, 0x05"));
        assert!(monitor.registers().contains("r16=05"));
        assert!(monitor.step().expect("step").contains("break"));
        assert_eq!(monitor.step().expect("step"), "halted");
    }

    #[test]
    fn monitor_disassembles_program() {
        let monitor = Monitor::new(vec![ldi(16, 5), 0x9598]).expect("monitor");
        let text = monitor.disassemble(0, 2).expect("disasm");
        assert!(text.contains("0x0000: ldi r16, 0x05"));
        assert!(text.contains("0x0001: break"));
    }

    #[test]
    fn monitor_reads_data_memory() {
        let mut monitor =
            Monitor::new(vec![ldi(16, 0x55), 0x9200 | (16 << 4), 0x0100, 0x9598]).expect("monitor");
        monitor.run(10).expect("run");
        assert!(monitor.memory(0x0100, 1).contains("55"));
    }
}
