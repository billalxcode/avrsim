//! AVR execution core with a structural datapath.

use super::alu::{ArithmeticUnit, LogicUnit, arithmetic_unit, logic_unit, select_operand};
use super::fetch::ProgramMemory;
use super::instruction::{Instruction, Mnemonic, Operand};
use super::multiply::{MultiplyUnit, multiply_unit};
use super::register_bank::RegisterBank;
use super::shift::{ShiftUnit, shift_unit};
use sim_cells::{DigitalBuilder, DigitalCircuit, NetId, RegisterPorts, read_bits};
use sim_core::SimError;

/// AVR status register state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sreg {
    /// Carry.
    pub carry: bool,
    /// Zero.
    pub zero: bool,
    /// Negative.
    pub negative: bool,
    /// Overflow.
    pub overflow: bool,
    /// Signed sign.
    pub sign: bool,
    /// Half carry.
    pub half_carry: bool,
    /// Transfer bit.
    pub transfer: bool,
    /// Interrupt enable.
    pub interrupt: bool,
}

impl Sreg {
    /// Returns a flag by AVR SREG index.
    #[must_use]
    pub fn get(self, flag: u8) -> bool {
        match flag {
            0 => self.carry,
            1 => self.zero,
            2 => self.negative,
            3 => self.overflow,
            4 => self.sign,
            5 => self.half_carry,
            6 => self.transfer,
            _ => self.interrupt,
        }
    }
}

/// One executed instruction trace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvrTrace {
    /// Program counter after the instruction.
    pub pc: u16,
    /// Executed instruction.
    pub instruction: Instruction,
    /// Register snapshot.
    pub registers: [u8; 32],
    /// Status register snapshot.
    pub sreg: Sreg,
    /// Halt state.
    pub halted: bool,
}

struct AluNets {
    first: Vec<NetId>,
    second: Vec<NetId>,
    immediate: Vec<NetId>,
    use_immediate: NetId,
    subtract: NetId,
    carry: NetId,
}

struct Datapath {
    bank: RegisterBank,
    arithmetic: ArithmeticUnit,
    logic: LogicUnit,
    shift: ShiftUnit,
    multiply: MultiplyUnit,
}

/// A small AVR core for instruction integration tests.
pub struct Cpu {
    circuit: DigitalCircuit,
    datapath: Datapath,
    program: ProgramMemory,
    pc: u16,
    sreg: Sreg,
    halted: bool,
}

impl Cpu {
    /// Builds a CPU with the provided program words.
    pub fn new(program: Vec<u16>) -> Result<Self, SimError> {
        if program.is_empty() {
            return Err(SimError::InvalidNetlist(
                "AVR program must not be empty".to_string(),
            ));
        }
        let mut builder = DigitalBuilder::new();
        let datapath = build_datapath(&mut builder)?;
        Ok(Self {
            circuit: builder.build(),
            datapath,
            program: ProgramMemory::new(program),
            pc: 0,
            sreg: Sreg::default(),
            halted: false,
        })
    }

    /// Executes one instruction and returns its trace.
    pub fn step(&mut self) -> Result<Option<AvrTrace>, SimError> {
        if self.halted {
            return Ok(None);
        }
        let fetched = self.program.fetch(self.pc)?;
        let next_pc = self.pc.wrapping_add(u16::from(fetched.words));
        self.execute(&fetched.instruction, next_pc)?;
        Ok(Some(self.trace(fetched.instruction)))
    }

    /// Executes until halt or the step limit.
    pub fn run(&mut self, max_steps: usize) -> Result<Vec<AvrTrace>, SimError> {
        let mut traces = Vec::new();
        for _ in 0..max_steps {
            match self.step()? {
                Some(trace) => traces.push(trace),
                None => return Ok(traces),
            }
        }
        Err(SimError::ConvergenceFailure {
            iteration: max_steps,
        })
    }

    /// Returns a register value.
    #[must_use]
    pub fn register(&self, index: u8) -> u8 {
        self.datapath
            .bank
            .read_byte(&self.circuit, usize::from(index))
            .unwrap_or(0)
    }

    /// Returns the program counter.
    #[must_use]
    pub const fn pc(&self) -> u16 {
        self.pc
    }

    /// Returns the status register.
    #[must_use]
    pub const fn sreg(&self) -> Sreg {
        self.sreg
    }

    /// Returns true when halted.
    #[must_use]
    pub const fn is_halted(&self) -> bool {
        self.halted
    }

    fn execute(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::Nop => return self.advance(next_pc),
            Mnemonic::Break => return self.halt(),
            _ => {}
        }
        self.execute_complex(instruction, next_pc)
    }

    fn execute_complex(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        let mnemonic = instruction.mnemonic;
        if is_move(mnemonic) {
            return self.execute_move(instruction, next_pc);
        }
        if is_arithmetic(mnemonic) {
            return self.execute_arithmetic(instruction, next_pc);
        }
        if is_logic(mnemonic) {
            return self.execute_logic(instruction, next_pc);
        }
        self.execute_special(instruction, next_pc)
    }

    fn execute_special(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        let mnemonic = instruction.mnemonic;
        if is_shift(mnemonic) {
            return self.execute_shift(instruction, next_pc);
        }
        if mnemonic == Mnemonic::Mul {
            return self.execute_multiply(instruction, next_pc);
        }
        if is_flow(mnemonic) {
            return self.execute_flow(instruction, next_pc);
        }
        Err(unsupported(mnemonic))
    }

    fn execute_move(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::Ldi => {
                let destination = register(instruction.first)?;
                let value = immediate(instruction.second)?;
                self.write_register(destination, value)?;
            }
            Mnemonic::Mov => {
                let destination = register(instruction.first)?;
                let source = register(instruction.second)?;
                self.write_register(destination, self.register(source))?;
            }
            Mnemonic::Movw => self.execute_movw(instruction)?,
            _ => return Err(unsupported(instruction.mnemonic)),
        }
        self.advance(next_pc)
    }

    fn execute_movw(&mut self, instruction: &Instruction) -> Result<(), SimError> {
        let destination = register(instruction.first)?;
        let source = register(instruction.second)?;
        let low = self.register(source);
        let high = self.register(source + 1);
        self.write_register(destination, low)?;
        self.write_register(destination + 1, high)
    }

    fn execute_arithmetic(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        if let Some((destination, value)) = update_operand(instruction)? {
            return self.execute_update(instruction.mnemonic, destination, value, next_pc);
        }
        self.execute_binary_arithmetic(instruction, next_pc)
    }

    fn execute_binary_arithmetic(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let destination = register(instruction.first)?;
        let (source, immediate_value) = second_operand(instruction.second)?;
        let (subtract, use_immediate, carry, write) = arithmetic_control(instruction.mnemonic)?;
        self.load_arithmetic(destination, source, immediate_value, use_immediate)?;
        self.set_bool("sub", subtract)?;
        self.set_bool("cin", carry && self.sreg.carry)?;
        self.circuit.settle()?;
        let result = read_bits(&self.circuit, &self.datapath.arithmetic.result) as u8;
        self.update_full_flags();
        if write {
            self.write_register(destination, result)?;
        }
        self.advance(next_pc)
    }

    fn execute_update(
        &mut self,
        mnemonic: Mnemonic,
        destination: u8,
        value: u8,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let (first, second, subtract) = update_values(mnemonic, self.register(destination), value);
        self.set_byte("a", first)?;
        self.set_byte("b", second)?;
        self.set_bool("imm", false)?;
        self.set_bool("sub", subtract)?;
        self.set_bool("cin", false)?;
        self.circuit.settle()?;
        let result = read_bits(&self.circuit, &self.datapath.arithmetic.result) as u8;
        self.update_partial_flags();
        self.write_register(destination, result)?;
        self.advance(next_pc)
    }

    fn execute_logic(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        let (op, use_immediate) = logic_control(instruction.mnemonic)?;
        let destination = register(instruction.first)?;
        let (source, immediate_value) = if instruction.mnemonic == Mnemonic::Com {
            (0, 0)
        } else {
            second_operand(instruction.second)?
        };
        self.load_arithmetic(destination, source, immediate_value, use_immediate)?;
        self.set_bits("lop", u64::from(op), 2)?;
        self.circuit.settle()?;
        let result = read_bits(&self.circuit, &self.datapath.logic.result) as u8;
        self.update_logic_flags(instruction.mnemonic);
        self.write_register(destination, result)?;
        self.advance(next_pc)
    }

    fn execute_shift(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        let destination = register(instruction.first)?;
        let op = shift_op(instruction.mnemonic);
        self.set_byte("a", self.register(destination))?;
        self.set_bool("cin", self.sreg.carry)?;
        self.set_bits("sop", u64::from(op), 2)?;
        self.circuit.settle()?;
        let result = read_bits(&self.circuit, &self.datapath.shift.result) as u8;
        self.update_shift_flags();
        self.write_register(destination, result)?;
        self.advance(next_pc)
    }

    fn execute_multiply(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let first = register(instruction.first)?;
        let second = register(instruction.second)?;
        self.set_byte("a", self.register(first))?;
        self.set_byte("b", self.register(second))?;
        self.circuit.settle()?;
        let result = read_bits(&self.circuit, &self.datapath.multiply.result);
        self.sreg.zero = self.circuit.value(self.datapath.multiply.zero);
        self.sreg.carry = self.circuit.value(self.datapath.multiply.carry);
        self.write_register(0, result as u8)?;
        self.write_register(1, (result >> 8) as u8)?;
        self.advance(next_pc)
    }

    fn execute_flow(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::Rjmp => {
                let offset = offset(instruction.first)?;
                self.pc = next_pc.wrapping_add(offset as u16);
            }
            Mnemonic::Jmp => self.pc = long_address(instruction.first)? as u16,
            Mnemonic::Brbs => self.branch(instruction, next_pc, true)?,
            Mnemonic::Brbc => self.branch(instruction, next_pc, false)?,
            _ => return Err(unsupported(instruction.mnemonic)),
        }
        Ok(())
    }

    fn branch(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
        set: bool,
    ) -> Result<(), SimError> {
        let flag = flag(instruction.first)?;
        let offset = offset(instruction.second)?;
        if self.sreg.get(flag) == set {
            self.pc = next_pc.wrapping_add(offset as u16);
        } else {
            self.pc = next_pc;
        }
        Ok(())
    }

    fn load_arithmetic(
        &mut self,
        destination: u8,
        source: u8,
        immediate_value: u8,
        use_immediate: bool,
    ) -> Result<(), SimError> {
        self.set_byte("a", self.register(destination))?;
        self.set_byte(
            "b",
            if use_immediate {
                0
            } else {
                self.register(source)
            },
        )?;
        self.set_byte("k", immediate_value)?;
        self.set_bool("imm", use_immediate)
    }

    fn update_full_flags(&mut self) {
        let flags = self.arithmetic_flags();
        self.sreg.carry = flags.0;
        self.sreg.zero = flags.1;
        self.sreg.negative = flags.2;
        self.sreg.overflow = flags.3;
        self.sreg.sign = flags.4;
        self.sreg.half_carry = flags.5;
    }

    fn update_partial_flags(&mut self) {
        let flags = self.arithmetic_flags();
        self.sreg.zero = flags.1;
        self.sreg.negative = flags.2;
        self.sreg.overflow = flags.3;
        self.sreg.sign = flags.4;
    }

    fn update_logic_flags(&mut self, mnemonic: Mnemonic) {
        let unit = &self.datapath.logic;
        self.sreg.zero = self.circuit.value(unit.zero);
        self.sreg.negative = self.circuit.value(unit.negative);
        self.sreg.overflow = false;
        self.sreg.sign = self.circuit.value(unit.sign);
        if mnemonic == Mnemonic::Com {
            self.sreg.carry = true;
        }
    }

    fn update_shift_flags(&mut self) {
        let unit = &self.datapath.shift;
        self.sreg.carry = self.circuit.value(unit.carry);
        self.sreg.zero = self.circuit.value(unit.zero);
        self.sreg.negative = self.circuit.value(unit.negative);
        self.sreg.overflow = self.circuit.value(unit.overflow);
        self.sreg.sign = self.circuit.value(unit.sign);
    }

    fn arithmetic_flags(&self) -> (bool, bool, bool, bool, bool, bool) {
        let unit = &self.datapath.arithmetic;
        (
            self.circuit.value(unit.carry),
            self.circuit.value(unit.zero),
            self.circuit.value(unit.negative),
            self.circuit.value(unit.overflow),
            self.circuit.value(unit.sign),
            self.circuit.value(unit.half_carry),
        )
    }

    fn write_register(&mut self, address: u8, value: u8) -> Result<(), SimError> {
        self.set_bits("wa", u64::from(address), 5)?;
        self.set_byte("wd", value)?;
        self.set_bool("we", true)?;
        self.circuit.tick()?;
        self.set_bool("we", false)?;
        self.circuit.settle()
    }

    fn set_byte(&mut self, prefix: &str, value: u8) -> Result<(), SimError> {
        self.set_bits(prefix, u64::from(value), 8)
    }

    fn set_bits(&mut self, prefix: &str, value: u64, width: usize) -> Result<(), SimError> {
        for index in 0..width {
            let bit = value & (1 << index) != 0;
            self.circuit.set_input(&format!("{prefix}{index}"), bit)?;
        }
        Ok(())
    }

    fn set_bool(&mut self, name: &str, value: bool) -> Result<(), SimError> {
        self.circuit.set_input(name, value)
    }

    fn advance(&mut self, next_pc: u16) -> Result<(), SimError> {
        self.pc = next_pc;
        Ok(())
    }

    fn halt(&mut self) -> Result<(), SimError> {
        self.halted = true;
        Ok(())
    }

    fn trace(&self, instruction: Instruction) -> AvrTrace {
        let mut registers = [0u8; 32];
        for (index, slot) in registers.iter_mut().enumerate() {
            *slot = self.register(index as u8);
        }
        AvrTrace {
            pc: self.pc,
            instruction,
            registers,
            sreg: self.sreg,
            halted: self.halted,
        }
    }
}

fn build_datapath(builder: &mut DigitalBuilder) -> Result<Datapath, SimError> {
    let nets = build_alu_nets(builder);
    let selected = select_operand(builder, &nets.second, &nets.immediate, nets.use_immediate)?;
    let arithmetic = arithmetic_unit(builder, &nets.first, &selected, nets.subtract, nets.carry)?;
    let logic_op = named_inputs(builder, "lop", 2);
    let logic = logic_unit(builder, &nets.first, &selected, &logic_op)?;
    let shift_op = named_inputs(builder, "sop", 2);
    let shift = shift_unit(builder, &nets.first, nets.carry, &shift_op)?;
    let multiply = multiply_unit(builder, &nets.first, &nets.second)?;
    let bank = build_bank(builder)?;
    Ok(Datapath {
        bank,
        arithmetic,
        logic,
        shift,
        multiply,
    })
}

fn build_alu_nets(builder: &mut DigitalBuilder) -> AluNets {
    AluNets {
        first: named_inputs(builder, "a", 8),
        second: named_inputs(builder, "b", 8),
        immediate: named_inputs(builder, "k", 8),
        use_immediate: builder.input("imm"),
        subtract: builder.input("sub"),
        carry: builder.input("cin"),
    }
}

fn build_bank(builder: &mut DigitalBuilder) -> Result<RegisterBank, SimError> {
    let ports = RegisterPorts {
        write_address: named_inputs(builder, "wa", 5),
        write_data: named_inputs(builder, "wd", 8),
        write_enable: builder.input("we"),
        read_address_a: named_inputs(builder, "ra", 5),
        read_address_b: named_inputs(builder, "rb", 5),
    };
    RegisterBank::new(builder, &ports)
}

fn named_inputs(builder: &mut DigitalBuilder, prefix: &str, width: usize) -> Vec<NetId> {
    (0..width)
        .map(|index| builder.input(&format!("{prefix}{index}")))
        .collect()
}

fn is_move(mnemonic: Mnemonic) -> bool {
    matches!(mnemonic, Mnemonic::Ldi | Mnemonic::Mov | Mnemonic::Movw)
}

fn is_arithmetic(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Add
            | Mnemonic::Adc
            | Mnemonic::Sub
            | Mnemonic::Sbc
            | Mnemonic::Subi
            | Mnemonic::Sbci
            | Mnemonic::Cp
            | Mnemonic::Cpc
            | Mnemonic::Cpi
            | Mnemonic::Neg
            | Mnemonic::Inc
            | Mnemonic::Dec
    )
}

fn is_logic(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::And
            | Mnemonic::Andi
            | Mnemonic::Or
            | Mnemonic::Ori
            | Mnemonic::Eor
            | Mnemonic::Com
    )
}

fn is_shift(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Lsr | Mnemonic::Asr | Mnemonic::Ror | Mnemonic::Swap
    )
}

fn is_flow(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Rjmp | Mnemonic::Jmp | Mnemonic::Brbs | Mnemonic::Brbc
    )
}

fn arithmetic_control(mnemonic: Mnemonic) -> Result<(bool, bool, bool, bool), SimError> {
    match mnemonic {
        Mnemonic::Add => Ok((false, false, false, true)),
        Mnemonic::Adc => Ok((false, false, true, true)),
        Mnemonic::Sub => Ok((true, false, false, true)),
        Mnemonic::Sbc => Ok((true, false, true, true)),
        Mnemonic::Subi => Ok((true, true, false, true)),
        Mnemonic::Sbci => Ok((true, true, true, true)),
        Mnemonic::Cp => Ok((true, false, false, false)),
        Mnemonic::Cpc => Ok((true, false, true, false)),
        Mnemonic::Cpi => Ok((true, true, false, false)),
        _ => Err(unsupported(mnemonic)),
    }
}

fn logic_control(mnemonic: Mnemonic) -> Result<(u8, bool), SimError> {
    match mnemonic {
        Mnemonic::And => Ok((0, false)),
        Mnemonic::Andi => Ok((0, true)),
        Mnemonic::Or => Ok((1, false)),
        Mnemonic::Ori => Ok((1, true)),
        Mnemonic::Eor => Ok((2, false)),
        Mnemonic::Com => Ok((3, false)),
        _ => Err(unsupported(mnemonic)),
    }
}

fn shift_op(mnemonic: Mnemonic) -> u8 {
    match mnemonic {
        Mnemonic::Lsr => 0,
        Mnemonic::Asr => 1,
        Mnemonic::Ror => 2,
        _ => 3,
    }
}

fn update_operand(instruction: &Instruction) -> Result<Option<(u8, u8)>, SimError> {
    let destination = register(instruction.first)?;
    Ok(match instruction.mnemonic {
        Mnemonic::Neg => Some((destination, 0)),
        Mnemonic::Inc | Mnemonic::Dec => Some((destination, 1)),
        _ => None,
    })
}

fn update_values(mnemonic: Mnemonic, current: u8, value: u8) -> (u8, u8, bool) {
    if mnemonic == Mnemonic::Neg {
        (0, current, true)
    } else {
        (current, value, mnemonic == Mnemonic::Dec)
    }
}

fn second_operand(operand: Operand) -> Result<(u8, u8), SimError> {
    match operand {
        Operand::Register(value) => Ok((value, 0)),
        Operand::Immediate(value) => Ok((0, value)),
        _ => Err(SimError::Parse(
            "expected register or immediate".to_string(),
        )),
    }
}

fn register(operand: Operand) -> Result<u8, SimError> {
    match operand {
        Operand::Register(value) => Ok(value),
        _ => Err(SimError::Parse("expected register".to_string())),
    }
}

fn immediate(operand: Operand) -> Result<u8, SimError> {
    match operand {
        Operand::Immediate(value) => Ok(value),
        _ => Err(SimError::Parse("expected immediate".to_string())),
    }
}

fn flag(operand: Operand) -> Result<u8, SimError> {
    match operand {
        Operand::Flag(value) => Ok(value),
        _ => Err(SimError::Parse("expected flag".to_string())),
    }
}

fn offset(operand: Operand) -> Result<i16, SimError> {
    match operand {
        Operand::Offset(value) => Ok(value),
        _ => Err(SimError::Parse("expected offset".to_string())),
    }
}

fn long_address(operand: Operand) -> Result<u32, SimError> {
    match operand {
        Operand::LongAddress(value) => Ok(value),
        _ => Err(SimError::Parse("expected address".to_string())),
    }
}

fn unsupported(mnemonic: Mnemonic) -> SimError {
    SimError::Parse(format!("unsupported AVR instruction: {mnemonic:?}"))
}

#[cfg(test)]
mod tests {
    use super::Cpu;

    fn ldi(register: u16, value: u16) -> u16 {
        0xE000 | ((value & 0xF0) << 4) | ((register - 16) << 4) | (value & 0x0F)
    }

    fn reg_op(base: u16, destination: u16, source: u16) -> u16 {
        base | ((source & 0x10) << 5) | ((destination & 0x1F) << 4) | (source & 0x0F)
    }

    fn one_reg(base: u16, register: u16) -> u16 {
        base | ((register & 0x1F) << 4)
    }

    fn branch(flag: u16, offset: i8) -> u16 {
        0xF000 | (((offset as u16) & 0x7F) << 3) | flag
    }

    fn branch_clear(flag: u16, offset: i8) -> u16 {
        0xF400 | (((offset as u16) & 0x7F) << 3) | flag
    }

    fn immediate(base: u16, register: u16, value: u16) -> u16 {
        base | ((value & 0xF0) << 4) | ((register - 16) << 4) | (value & 0x0F)
    }

    fn run(program: Vec<u16>) -> Cpu {
        let mut cpu = Cpu::new(program).expect("cpu");
        cpu.run(100).expect("run");
        cpu
    }

    #[test]
    fn runs_arithmetic_program() {
        let cpu = run(vec![ldi(16, 5), ldi(17, 3), reg_op(0x0C00, 16, 17), 0x9598]);
        assert_eq!(cpu.register(16), 8);
        assert!(cpu.is_halted());
    }

    #[test]
    fn adc_uses_carry_flag() {
        let cpu = run(vec![
            ldi(16, 0xFF),
            ldi(17, 1),
            reg_op(0x0C00, 16, 17),
            ldi(18, 1),
            ldi(19, 0),
            reg_op(0x1C00, 18, 19),
            0x9598,
        ]);
        assert_eq!(cpu.register(18), 2);
    }

    #[test]
    fn compare_and_branch_on_zero() {
        let cpu = run(vec![
            ldi(16, 0),
            reg_op(0x1800, 16, 16),
            branch(1, 1),
            ldi(17, 9),
            ldi(18, 7),
            0x9598,
        ]);
        assert_eq!(cpu.register(17), 0);
        assert_eq!(cpu.register(18), 7);
    }

    #[test]
    fn movw_shift_and_rotate() {
        let cpu = run(vec![
            ldi(26, 0x34),
            ldi(27, 0x12),
            0x01FD,
            one_reg(0x9406, 30),
            one_reg(0x9405, 31),
            one_reg(0x9407, 30),
            one_reg(0x9402, 30),
            0x9598,
        ]);
        assert_eq!(cpu.register(30), 0xD0);
        assert_eq!(cpu.register(31), 0x09);
    }

    #[test]
    fn multiply_program() {
        let cpu = run(vec![ldi(16, 7), ldi(17, 6), reg_op(0x9C00, 16, 17), 0x9598]);
        assert_eq!(cpu.register(0), 42);
        assert_eq!(cpu.register(1), 0);
    }

    #[test]
    fn jump_skips_instruction() {
        let cpu = run(vec![0x940C, 0x0003, ldi(16, 9), ldi(17, 4), 0x9598]);
        assert_eq!(cpu.register(16), 0);
        assert_eq!(cpu.register(17), 4);
    }

    #[test]
    fn trace_records_pc_and_registers() {
        let mut cpu = Cpu::new(vec![ldi(16, 5), 0x9598]).expect("cpu");
        let trace = cpu.step().expect("step").expect("trace");
        assert_eq!(trace.pc, 1);
        assert_eq!(trace.registers[16], 5);
        assert!(!trace.halted);
        assert!(cpu.step().expect("step").expect("trace").halted);
    }

    #[test]
    fn halted_core_returns_no_trace() {
        let mut cpu = Cpu::new(vec![0x9598]).expect("cpu");
        assert!(cpu.step().expect("step").is_some());
        assert!(cpu.step().expect("step").is_none());
    }

    #[test]
    fn logic_program() {
        let cpu = run(vec![
            ldi(16, 0xAA),
            ldi(17, 0xCC),
            reg_op(0x2000, 16, 17),
            immediate(0x6000, 16, 0x01),
            reg_op(0x2400, 16, 17),
            one_reg(0x9400, 16),
            0x9598,
        ]);
        assert_eq!(cpu.register(16), 0xBA);
    }

    #[test]
    fn neg_inc_dec_program() {
        let cpu = run(vec![
            ldi(16, 5),
            one_reg(0x9401, 16),
            one_reg(0x9403, 16),
            one_reg(0x940A, 16),
            0x9598,
        ]);
        assert_eq!(cpu.register(16), 0xFB);
    }

    #[test]
    fn subi_and_cpi_branch() {
        let cpu = run(vec![
            ldi(16, 10),
            immediate(0x5000, 16, 3),
            immediate(0x3000, 16, 7),
            branch(1, 1),
            ldi(17, 9),
            ldi(18, 4),
            0x9598,
        ]);
        assert_eq!(cpu.register(16), 7);
        assert_eq!(cpu.register(17), 0);
        assert_eq!(cpu.register(18), 4);
    }

    #[test]
    fn rjmp_and_brbc() {
        let cpu = run(vec![
            ldi(16, 1),
            0xC001,
            ldi(17, 9),
            ldi(18, 4),
            branch_clear(1, 1),
            ldi(19, 9),
            0x9598,
        ]);
        assert_eq!(cpu.register(17), 0);
        assert_eq!(cpu.register(18), 4);
        assert_eq!(cpu.register(19), 0);
    }
}
