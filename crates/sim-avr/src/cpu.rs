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

    /// Sets a flag by AVR SREG index.
    pub fn set(&mut self, flag: u8, value: bool) {
        match flag {
            0 => self.carry = value,
            1 => self.zero = value,
            2 => self.negative = value,
            3 => self.overflow = value,
            4 => self.sign = value,
            5 => self.half_carry = value,
            6 => self.transfer = value,
            _ => self.interrupt = value,
        }
    }

    /// Packs the register into an AVR SREG byte.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        (self.carry as u8)
            | ((self.zero as u8) << 1)
            | ((self.negative as u8) << 2)
            | ((self.overflow as u8) << 3)
            | ((self.sign as u8) << 4)
            | ((self.half_carry as u8) << 5)
            | ((self.transfer as u8) << 6)
            | ((self.interrupt as u8) << 7)
    }

    /// Unpacks an AVR SREG byte.
    #[must_use]
    pub const fn from_byte(value: u8) -> Self {
        Self {
            carry: value & 0x01 != 0,
            zero: value & 0x02 != 0,
            negative: value & 0x04 != 0,
            overflow: value & 0x08 != 0,
            sign: value & 0x10 != 0,
            half_carry: value & 0x20 != 0,
            transfer: value & 0x40 != 0,
            interrupt: value & 0x80 != 0,
        }
    }
}

/// ATmega328P interrupt vector word addresses.
pub const VECTORS: [u16; 26] = [
    0x0000, 0x0002, 0x0004, 0x0006, 0x0008, 0x000A, 0x000C, 0x000E, 0x0010, 0x0012, 0x0014, 0x0016,
    0x0018, 0x001A, 0x001C, 0x001E, 0x0020, 0x0022, 0x0024, 0x0026, 0x0028, 0x002A, 0x002C, 0x002E,
    0x0030, 0x0032,
];

fn vector_address(index: u8) -> u16 {
    VECTORS.get(usize::from(index)).copied().unwrap_or(0)
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

#[derive(Clone, Copy)]
struct MemoryMode {
    base: u8,
    post: bool,
    pre: bool,
    load: bool,
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
    io: [u8; 64],
    data: Vec<u8>,
    sp: u16,
    pending: u32,
    sei_delay: u8,
    sleeping: bool,
}

impl Cpu {
    /// Builds a CPU with the provided program words.
    pub fn new(program: Vec<u16>) -> Result<Self, SimError> {
        if program.is_empty() {
            return Err(SimError::InvalidNetlist(
                "AVR program must not be empty".to_string(),
            ));
        }
        Self::build(program)
    }

    fn build(program: Vec<u16>) -> Result<Self, SimError> {
        let mut builder = DigitalBuilder::new();
        let datapath = build_datapath(&mut builder)?;
        Ok(Self {
            circuit: builder.build(),
            datapath,
            program: ProgramMemory::new(program),
            pc: 0,
            sreg: Sreg::default(),
            halted: false,
            io: [0; 64],
            data: vec![0; 0x1000],
            sp: 0x08FF,
            pending: 0,
            sei_delay: 0,
            sleeping: false,
        })
    }

    /// Executes one instruction and returns its trace.
    pub fn step(&mut self) -> Result<Option<AvrTrace>, SimError> {
        if self.halted {
            return Ok(None);
        }
        if self.sleeping && !self.wake_ready() {
            return Ok(None);
        }
        self.sleeping = false;
        self.service_interrupt()?;
        let fetched = self.program.fetch(self.pc)?;
        let next_pc = self.pc.wrapping_add(u16::from(fetched.words));
        self.execute(&fetched.instruction, next_pc)?;
        self.finish_sei();
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

    /// Returns an I/O register value.
    #[must_use]
    pub fn io(&self, address: u8) -> u8 {
        self.io_read(address)
    }

    /// Returns a data space value.
    #[must_use]
    pub fn data(&self, address: u16) -> u8 {
        self.data[usize::from(address) & 0x0FFF]
    }

    /// Returns the stack pointer.
    #[must_use]
    pub const fn sp(&self) -> u16 {
        self.sp
    }

    /// Requests an interrupt by vector index.
    pub fn request_interrupt(&mut self, index: u8) -> Result<(), SimError> {
        if usize::from(index) >= VECTORS.len() {
            return Err(SimError::InvalidNetlist(
                "interrupt vector out of range".to_string(),
            ));
        }
        self.pending |= 1 << index;
        Ok(())
    }

    /// Replaces the status register.
    pub fn set_sreg(&mut self, sreg: Sreg) {
        self.sreg = sreg;
    }

    fn service_interrupt(&mut self) -> Result<(), SimError> {
        if !self.sreg.interrupt || self.pending == 0 {
            return Ok(());
        }
        let index = self.pending.trailing_zeros() as u8;
        self.pending &= !(1u32 << index);
        self.push_word(self.pc);
        self.sreg.interrupt = false;
        self.pc = vector_address(index);
        Ok(())
    }

    fn finish_sei(&mut self) {
        if self.sei_delay == 0 {
            return;
        }
        self.sei_delay -= 1;
        if self.sei_delay == 0 {
            self.sreg.interrupt = true;
        }
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

    /// Returns true when sleeping.
    #[must_use]
    pub const fn is_sleeping(&self) -> bool {
        self.sleeping
    }

    fn wake_ready(&self) -> bool {
        self.sreg.interrupt && self.pending != 0
    }

    fn sleep(&mut self) -> Result<(), SimError> {
        self.sleeping = true;
        Ok(())
    }

    /// Resets the core to its initial state.
    pub fn reset(&mut self) -> Result<(), SimError> {
        self.circuit.reset()?;
        self.pc = 0;
        self.sreg = Sreg::default();
        self.halted = false;
        self.sp = 0x08FF;
        self.io.fill(0);
        self.data.fill(0);
        self.pending = 0;
        self.sei_delay = 0;
        self.sleeping = false;
        Ok(())
    }

    fn execute(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::Nop | Mnemonic::Wdr => return self.advance(next_pc),
            Mnemonic::Break => return self.halt(),
            Mnemonic::Sleep => return self.sleep(),
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
        self.execute_control(instruction, next_pc)
    }

    fn execute_control(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        let mnemonic = instruction.mnemonic;
        if is_flow(mnemonic) {
            return self.execute_flow(instruction, next_pc);
        }
        if is_stack(mnemonic) {
            return self.execute_stack(instruction, next_pc);
        }
        if is_call(mnemonic) {
            return self.execute_call(instruction, next_pc);
        }
        self.execute_io_flag(instruction, next_pc)
    }

    fn execute_stack(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::Push => {
                let source = register(instruction.first)?;
                self.push_byte(self.register(source));
            }
            Mnemonic::Pop => {
                let destination = register(instruction.first)?;
                let value = self.pop_byte();
                self.write_register(destination, value)?;
            }
            _ => return Err(unsupported(instruction.mnemonic)),
        }
        self.advance(next_pc)
    }

    fn execute_call(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::Rcall => self.call_relative(instruction, next_pc),
            Mnemonic::Call => self.call_absolute(instruction, next_pc),
            Mnemonic::Icall | Mnemonic::Eicall => self.call_indirect(next_pc),
            Mnemonic::Ijmp | Mnemonic::Eijmp => {
                self.pc = self.read_pointer(30);
                Ok(())
            }
            Mnemonic::Ret => self.return_from(false),
            Mnemonic::Reti => self.return_from(true),
            _ => Err(unsupported(instruction.mnemonic)),
        }
    }

    fn return_from(&mut self, interrupt: bool) -> Result<(), SimError> {
        self.pc = self.pop_word();
        if interrupt {
            self.sreg.interrupt = true;
        }
        Ok(())
    }

    fn call_relative(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        let offset = offset(instruction.first)?;
        self.push_word(next_pc);
        self.pc = next_pc.wrapping_add(offset as u16);
        Ok(())
    }

    fn call_absolute(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        let address = long_address(instruction.first)? as u16;
        self.push_word(next_pc);
        self.pc = address;
        Ok(())
    }

    fn call_indirect(&mut self, next_pc: u16) -> Result<(), SimError> {
        let address = self.read_pointer(30);
        self.push_word(next_pc);
        self.pc = address;
        Ok(())
    }

    fn push_byte(&mut self, value: u8) {
        self.write_data(self.sp, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop_byte(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.read_data(self.sp)
    }

    fn push_word(&mut self, value: u16) {
        self.push_byte(value as u8);
        self.push_byte((value >> 8) as u8);
    }

    fn pop_word(&mut self) -> u16 {
        let high = self.pop_byte();
        let low = self.pop_byte();
        u16::from(low) | (u16::from(high) << 8)
    }

    fn execute_io_flag(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        let mnemonic = instruction.mnemonic;
        if is_memory(mnemonic) {
            return self.execute_memory(instruction, next_pc);
        }
        if is_skip(mnemonic) {
            return self.execute_skip(instruction, next_pc);
        }
        if is_io(mnemonic) {
            return self.execute_io(instruction, next_pc);
        }
        if is_flag(mnemonic) {
            return self.execute_flag(instruction, next_pc);
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

    fn execute_skip(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        if self.skip_condition(instruction)? {
            self.pc = self.skipped_pc(next_pc)?;
        } else {
            self.pc = next_pc;
        }
        Ok(())
    }

    fn skip_condition(&self, instruction: &Instruction) -> Result<bool, SimError> {
        match instruction.mnemonic {
            Mnemonic::Cpse => Ok(self.register(register(instruction.first)?)
                == self.register(register(instruction.second)?)),
            Mnemonic::Sbrc => Ok(self.bit_value(instruction)? == 0),
            Mnemonic::Sbrs => Ok(self.bit_value(instruction)? != 0),
            Mnemonic::Sbic => Ok(self.io_bit(instruction)? == 0),
            Mnemonic::Sbis => Ok(self.io_bit(instruction)? != 0),
            _ => Err(unsupported(instruction.mnemonic)),
        }
    }

    fn bit_value(&self, instruction: &Instruction) -> Result<u8, SimError> {
        let value = self.register(register(instruction.first)?);
        let bit = bit(instruction.second)?;
        Ok((value >> bit) & 1)
    }

    fn io_bit(&self, instruction: &Instruction) -> Result<u8, SimError> {
        let value = self.io(io_address(instruction.first)?);
        let bit = bit(instruction.second)?;
        Ok((value >> bit) & 1)
    }

    fn skipped_pc(&self, next_pc: u16) -> Result<u16, SimError> {
        let fetched = self.program.fetch(next_pc)?;
        Ok(next_pc.wrapping_add(u16::from(fetched.words)))
    }

    fn execute_io(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::Sbi => self.set_io_bit(instruction, true)?,
            Mnemonic::Cbi => self.set_io_bit(instruction, false)?,
            Mnemonic::In => self.execute_in(instruction)?,
            Mnemonic::Out => self.execute_out(instruction)?,
            _ => return Err(unsupported(instruction.mnemonic)),
        }
        self.advance(next_pc)
    }

    fn set_io_bit(&mut self, instruction: &Instruction, set: bool) -> Result<(), SimError> {
        let address = io_address(instruction.first)?;
        let bit = bit(instruction.second)?;
        let mut value = self.io_read(address);
        if set {
            value |= 1 << bit;
        } else {
            value &= !(1 << bit);
        }
        self.io_write(address, value);
        Ok(())
    }

    fn execute_in(&mut self, instruction: &Instruction) -> Result<(), SimError> {
        let destination = register(instruction.first)?;
        let address = io_address(instruction.second)?;
        self.write_register(destination, self.io(address))
    }

    fn execute_out(&mut self, instruction: &Instruction) -> Result<(), SimError> {
        let address = io_address(instruction.first)?;
        let source = register(instruction.second)?;
        self.io_write(address, self.register(source));
        Ok(())
    }

    fn execute_flag(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::Bset => self.set_flag(flag(instruction.first)?, true),
            Mnemonic::Bclr => self.set_flag(flag(instruction.first)?, false),
            Mnemonic::Bst => self.execute_bst(instruction)?,
            Mnemonic::Bld => return self.execute_bld(instruction, next_pc),
            _ => return Err(unsupported(instruction.mnemonic)),
        }
        self.advance(next_pc)
    }

    fn set_flag(&mut self, flag: u8, value: bool) {
        if flag == 7 {
            if value {
                self.sei_delay = 2;
            } else {
                self.sreg.interrupt = false;
                self.sei_delay = 0;
            }
        } else {
            self.sreg.set(flag, value);
        }
    }

    fn execute_bst(&mut self, instruction: &Instruction) -> Result<(), SimError> {
        let value = self.register(register(instruction.first)?);
        let bit = bit(instruction.second)?;
        self.sreg.transfer = (value >> bit) & 1 != 0;
        Ok(())
    }

    fn execute_bld(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        let destination = register(instruction.first)?;
        let bit = bit(instruction.second)?;
        let mut value = self.register(destination);
        if self.sreg.transfer {
            value |= 1 << bit;
        } else {
            value &= !(1 << bit);
        }
        self.write_register(destination, value)?;
        self.advance(next_pc)
    }

    fn execute_memory(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::LddY | Mnemonic::LddZ => {
                self.execute_displacement(instruction, next_pc, true)
            }
            Mnemonic::StdY | Mnemonic::StdZ => {
                self.execute_displacement(instruction, next_pc, false)
            }
            Mnemonic::Lds => self.execute_direct(instruction, next_pc, true),
            Mnemonic::Sts => self.execute_direct(instruction, next_pc, false),
            Mnemonic::LpmZ | Mnemonic::LpmZPost | Mnemonic::ElpmZ | Mnemonic::ElpmZPost => {
                self.execute_lpm(instruction, next_pc)
            }
            _ => self.execute_indirect(instruction, next_pc),
        }
    }

    fn execute_indirect(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let mode =
            indirect_mode(instruction.mnemonic).ok_or_else(|| unsupported(instruction.mnemonic))?;
        let mut address = self.read_pointer(mode.base);
        if mode.pre {
            address = address.wrapping_sub(1);
            self.write_pointer(mode.base, address)?;
        }
        self.access_indirect(instruction.first, mode.load, address)?;
        if mode.post {
            address = address.wrapping_add(1);
            self.write_pointer(mode.base, address)?;
        }
        self.advance(next_pc)
    }

    fn access_indirect(
        &mut self,
        operand: Operand,
        load: bool,
        address: u16,
    ) -> Result<(), SimError> {
        if load {
            let destination = register(operand)?;
            let value = self.read_data(address);
            self.write_register(destination, value)
        } else {
            let source = register(operand)?;
            let value = self.register(source);
            self.write_data(address, value);
            Ok(())
        }
    }

    fn execute_displacement(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
        load: bool,
    ) -> Result<(), SimError> {
        let base = pointer_base(instruction.mnemonic);
        let address = self.displacement_address(instruction, load, base)?;
        let operand = if load {
            instruction.first
        } else {
            instruction.second
        };
        self.access_indirect(operand, load, address)?;
        self.advance(next_pc)
    }

    fn displacement_address(
        &self,
        instruction: &Instruction,
        load: bool,
        base: u8,
    ) -> Result<u16, SimError> {
        let displacement = if load {
            displacement(instruction.second)?
        } else {
            displacement(instruction.first)?
        };
        Ok(self
            .read_pointer(base)
            .wrapping_add(u16::from(displacement)))
    }

    fn execute_direct(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
        load: bool,
    ) -> Result<(), SimError> {
        let address = data_address(instruction.second)?;
        if load {
            let destination = register(instruction.first)?;
            let value = self.read_data(address);
            self.write_register(destination, value)?;
        } else {
            let source = register(instruction.first)?;
            let value = self.register(source);
            self.write_data(address, value);
        }
        self.advance(next_pc)
    }

    fn execute_lpm(&mut self, instruction: &Instruction, next_pc: u16) -> Result<(), SimError> {
        let destination = register(instruction.first)?;
        let address = self.read_pointer(30);
        let value = self.read_program_byte(address);
        self.write_register(destination, value)?;
        if matches!(
            instruction.mnemonic,
            Mnemonic::LpmZPost | Mnemonic::ElpmZPost
        ) {
            self.write_pointer(30, address.wrapping_add(1))?;
        }
        self.advance(next_pc)
    }

    fn read_pointer(&self, low: u8) -> u16 {
        u16::from(self.register(low)) | (u16::from(self.register(low + 1)) << 8)
    }

    fn write_pointer(&mut self, low: u8, value: u16) -> Result<(), SimError> {
        self.write_register(low, value as u8)?;
        self.write_register(low + 1, (value >> 8) as u8)
    }

    fn read_data(&self, address: u16) -> u8 {
        self.data[usize::from(address) & 0x0FFF]
    }

    fn io_read(&self, address: u8) -> u8 {
        if address == 0x3F {
            return self.sreg.to_byte();
        }
        self.io.get(usize::from(address)).copied().unwrap_or(0)
    }

    fn io_write(&mut self, address: u8, value: u8) {
        if address == 0x3F {
            self.sreg = Sreg::from_byte(value);
        } else if let Some(slot) = self.io.get_mut(usize::from(address)) {
            *slot = value;
        }
    }

    fn write_data(&mut self, address: u16, value: u8) {
        self.data[usize::from(address) & 0x0FFF] = value;
    }

    fn read_program_byte(&self, address: u16) -> u8 {
        let word = self.program.read(usize::from(address) >> 1).unwrap_or(0);
        if address & 1 == 0 {
            word as u8
        } else {
            (word >> 8) as u8
        }
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

fn is_stack(mnemonic: Mnemonic) -> bool {
    matches!(mnemonic, Mnemonic::Push | Mnemonic::Pop)
}

fn is_call(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Rcall
            | Mnemonic::Call
            | Mnemonic::Icall
            | Mnemonic::Eicall
            | Mnemonic::Ijmp
            | Mnemonic::Eijmp
            | Mnemonic::Ret
            | Mnemonic::Reti
    )
}

fn is_skip(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Cpse | Mnemonic::Sbrc | Mnemonic::Sbrs | Mnemonic::Sbic | Mnemonic::Sbis
    )
}

fn is_io(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Sbi | Mnemonic::Cbi | Mnemonic::In | Mnemonic::Out
    )
}

fn is_flag(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Bset | Mnemonic::Bclr | Mnemonic::Bst | Mnemonic::Bld
    )
}

fn is_memory(mnemonic: Mnemonic) -> bool {
    is_load(mnemonic) || is_store(mnemonic)
}

fn is_load(mnemonic: Mnemonic) -> bool {
    is_indirect_load(mnemonic)
        || matches!(
            mnemonic,
            Mnemonic::LddY
                | Mnemonic::LddZ
                | Mnemonic::Lds
                | Mnemonic::LpmZ
                | Mnemonic::LpmZPost
                | Mnemonic::ElpmZ
                | Mnemonic::ElpmZPost
        )
}

fn is_indirect_load(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::LdX
            | Mnemonic::LdXPost
            | Mnemonic::LdXPre
            | Mnemonic::LdY
            | Mnemonic::LdYPost
            | Mnemonic::LdYPre
            | Mnemonic::LdZ
            | Mnemonic::LdZPost
            | Mnemonic::LdZPre
    )
}

fn is_store(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::StX
            | Mnemonic::StXPost
            | Mnemonic::StXPre
            | Mnemonic::StY
            | Mnemonic::StYPost
            | Mnemonic::StYPre
            | Mnemonic::StZ
            | Mnemonic::StZPost
            | Mnemonic::StZPre
            | Mnemonic::StdY
            | Mnemonic::StdZ
            | Mnemonic::Sts
    )
}

fn indirect_mode(mnemonic: Mnemonic) -> Option<MemoryMode> {
    load_mode(mnemonic).or_else(|| store_mode(mnemonic))
}

fn load_mode(mnemonic: Mnemonic) -> Option<MemoryMode> {
    let (base, post, pre) = match mnemonic {
        Mnemonic::LdX => (26, false, false),
        Mnemonic::LdXPost => (26, true, false),
        Mnemonic::LdXPre => (26, false, true),
        Mnemonic::LdY => (28, false, false),
        Mnemonic::LdYPost => (28, true, false),
        Mnemonic::LdYPre => (28, false, true),
        Mnemonic::LdZ => (30, false, false),
        Mnemonic::LdZPost => (30, true, false),
        Mnemonic::LdZPre => (30, false, true),
        _ => return None,
    };
    Some(MemoryMode {
        base,
        post,
        pre,
        load: true,
    })
}

fn store_mode(mnemonic: Mnemonic) -> Option<MemoryMode> {
    let (base, post, pre) = match mnemonic {
        Mnemonic::StX => (26, false, false),
        Mnemonic::StXPost => (26, true, false),
        Mnemonic::StXPre => (26, false, true),
        Mnemonic::StY => (28, false, false),
        Mnemonic::StYPost => (28, true, false),
        Mnemonic::StYPre => (28, false, true),
        Mnemonic::StZ => (30, false, false),
        Mnemonic::StZPost => (30, true, false),
        Mnemonic::StZPre => (30, false, true),
        _ => return None,
    };
    Some(MemoryMode {
        base,
        post,
        pre,
        load: false,
    })
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

fn bit(operand: Operand) -> Result<u8, SimError> {
    match operand {
        Operand::Bit(value) => Ok(value),
        _ => Err(SimError::Parse("expected bit index".to_string())),
    }
}

fn io_address(operand: Operand) -> Result<u8, SimError> {
    match operand {
        Operand::Io(value) => Ok(value),
        _ => Err(SimError::Parse("expected I/O address".to_string())),
    }
}

fn displacement(operand: Operand) -> Result<u8, SimError> {
    match operand {
        Operand::Displacement(value) => Ok(value),
        _ => Err(SimError::Parse("expected displacement".to_string())),
    }
}

fn data_address(operand: Operand) -> Result<u16, SimError> {
    match operand {
        Operand::Address(value) => Ok(value),
        _ => Err(SimError::Parse("expected data address".to_string())),
    }
}

fn pointer_base(mnemonic: Mnemonic) -> u8 {
    if matches!(mnemonic, Mnemonic::LddY | Mnemonic::StdY) {
        28
    } else {
        30
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

    fn in_out(base: u16, register: u16, address: u16) -> u16 {
        base | (((address >> 4) & 3) << 9) | ((register & 0x1F) << 4) | (address & 0x0F)
    }

    fn io_bit_op(base: u16, address: u16, bit: u16) -> u16 {
        base | ((address & 0x1F) << 3) | (bit & 0x07)
    }

    fn bit_register(base: u16, register: u16, bit: u16) -> u16 {
        base | ((register & 0x1F) << 4) | (bit & 0x07)
    }

    fn sreg_flag(base: u16, flag: u16) -> u16 {
        base | ((flag & 0x07) << 4)
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

    #[test]
    fn cpse_skips_equal_registers() {
        let cpu = run(vec![
            ldi(16, 4),
            ldi(17, 4),
            reg_op(0x1000, 16, 17),
            ldi(18, 9),
            ldi(19, 3),
            0x9598,
        ]);
        assert_eq!(cpu.register(18), 0);
        assert_eq!(cpu.register(19), 3);
    }

    #[test]
    fn sbrc_and_sbrs_skip() {
        let cpu = run(vec![
            ldi(16, 4),
            bit_register(0xFE00, 16, 2),
            ldi(17, 9),
            bit_register(0xFC00, 16, 0),
            ldi(18, 9),
            ldi(19, 3),
            0x9598,
        ]);
        assert_eq!(cpu.register(17), 0);
        assert_eq!(cpu.register(18), 0);
        assert_eq!(cpu.register(19), 3);
    }

    #[test]
    fn io_in_out_and_bit_ops() {
        let cpu = run(vec![
            ldi(16, 0xAA),
            in_out(0xB800, 16, 0x10),
            in_out(0xB000, 17, 0x10),
            io_bit_op(0x9A00, 0x10, 0),
            io_bit_op(0x9800, 0x10, 1),
            in_out(0xB000, 18, 0x10),
            0x9598,
        ]);
        assert_eq!(cpu.register(17), 0xAA);
        assert_eq!(cpu.register(18), 0xA9);
        assert_eq!(cpu.io(0x10), 0xA9);
    }

    #[test]
    fn sbic_and_sbis_skip() {
        let cpu = run(vec![
            io_bit_op(0x9A00, 0x10, 2),
            io_bit_op(0x9B00, 0x10, 2),
            ldi(16, 9),
            io_bit_op(0x9800, 0x10, 2),
            io_bit_op(0x9900, 0x10, 2),
            ldi(17, 9),
            ldi(18, 4),
            0x9598,
        ]);
        assert_eq!(cpu.register(16), 0);
        assert_eq!(cpu.register(17), 0);
        assert_eq!(cpu.register(18), 4);
    }

    #[test]
    fn sreg_flag_and_transfer_bits() {
        let cpu = run(vec![
            sreg_flag(0x9408, 0),
            sreg_flag(0x9488, 0),
            ldi(16, 1),
            bit_register(0xFA00, 16, 0),
            ldi(17, 0),
            bit_register(0xF800, 17, 3),
            0x9598,
        ]);
        assert_eq!(cpu.register(17), 0x08);
        assert!(cpu.sreg().transfer);
        assert!(!cpu.sreg().carry);
    }

    #[test]
    fn indirect_load_store_with_post_and_pre() {
        let cpu = run(vec![
            ldi(26, 0x00),
            ldi(27, 0x01),
            ldi(16, 0x55),
            0x920D | (16 << 4),
            0x900E | (17 << 4),
            0x9598,
        ]);
        assert_eq!(cpu.data(0x0100), 0x55);
        assert_eq!(cpu.register(17), 0x55);
        assert_eq!(cpu.register(26), 0x00);
        assert_eq!(cpu.register(27), 0x01);
    }

    #[test]
    fn ldd_and_std_displacement() {
        let cpu = run(vec![
            ldi(28, 0x00),
            ldi(29, 0x02),
            ldi(16, 0x66),
            0x8208 | (16 << 4) | 5,
            0x8008 | (17 << 4) | 5,
            0x9598,
        ]);
        assert_eq!(cpu.data(0x0205), 0x66);
        assert_eq!(cpu.register(17), 0x66);
    }

    #[test]
    fn lds_and_sts_direct() {
        let cpu = run(vec![
            ldi(16, 0x77),
            0x9200 | (16 << 4),
            0x0300,
            0x9000 | (17 << 4),
            0x0300,
            0x9598,
        ]);
        assert_eq!(cpu.data(0x0300), 0x77);
        assert_eq!(cpu.register(17), 0x77);
    }

    #[test]
    fn lpm_reads_program_byte() {
        let first = ldi(30, 0);
        let cpu = run(vec![first, ldi(16, 0), 0x9104, 0x9598]);
        assert_eq!(cpu.register(16), first as u8);
    }

    #[test]
    fn lpm_post_increment_advances_z() {
        let first = ldi(30, 0);
        let cpu = run(vec![first, 0x9105, 0x9005 | (17 << 4), 0x9598]);
        assert_eq!(cpu.register(16), first as u8);
        assert_eq!(cpu.register(17), (first >> 8) as u8);
        assert_eq!(cpu.register(30), 2);
    }

    #[test]
    fn push_and_pop_use_stack() {
        let cpu = run(vec![
            ldi(16, 0x5A),
            0x920F | (16 << 4),
            ldi(16, 0),
            0x900F | (17 << 4),
            0x9598,
        ]);
        assert_eq!(cpu.register(17), 0x5A);
        assert_eq!(cpu.sp(), 0x08FF);
    }

    #[test]
    fn rcall_and_ret_return_to_caller() {
        let cpu = run(vec![
            0xD003,
            ldi(16, 1),
            0x9598,
            ldi(17, 9),
            ldi(17, 7),
            0x9508,
        ]);
        assert_eq!(cpu.register(16), 1);
        assert_eq!(cpu.register(17), 7);
        assert_eq!(cpu.sp(), 0x08FF);
    }

    #[test]
    fn call_and_ret_use_two_word_target() {
        let cpu = run(vec![0x940E, 0x0004, ldi(16, 1), 0x9598, ldi(17, 7), 0x9508]);
        assert_eq!(cpu.register(16), 1);
        assert_eq!(cpu.register(17), 7);
    }

    #[test]
    fn reti_sets_interrupt_flag() {
        let cpu = run(vec![0xD002, 0x9598, 0x0000, 0x9518]);
        assert!(cpu.sreg().interrupt);
    }

    #[test]
    fn ijmp_uses_z_pointer() {
        let cpu = run(vec![
            ldi(30, 4),
            ldi(31, 0),
            0x9409,
            ldi(16, 9),
            ldi(17, 4),
            0x9598,
        ]);
        assert_eq!(cpu.register(16), 0);
        assert_eq!(cpu.register(17), 4);
    }

    #[test]
    fn sreg_is_visible_through_io() {
        let cpu = run(vec![
            ldi(16, 0xFF),
            ldi(17, 1),
            reg_op(0x0C00, 16, 17),
            in_out(0xB000, 18, 0x3F),
            ldi(19, 0x80),
            in_out(0xB800, 19, 0x3F),
            0x9598,
        ]);
        assert_eq!(cpu.register(18) & 0x03, 0x03);
        assert!(cpu.sreg().interrupt);
    }

    #[test]
    fn sleep_sets_sleeping_state() {
        let mut cpu = Cpu::new(vec![0x9588, ldi(16, 5), 0x9598]).expect("cpu");
        let trace = cpu.step().expect("step").expect("trace");
        assert!(!trace.halted);
        assert!(cpu.is_sleeping());
        assert!(cpu.step().expect("step").is_none());
    }

    #[test]
    fn sleep_wakes_on_interrupt() {
        let mut cpu =
            Cpu::new(vec![0x9588, 0xCFFF, 0x0000, 0x0000, ldi(17, 7), 0x9518]).expect("cpu");
        cpu.request_interrupt(2).expect("interrupt");
        cpu.step().expect("step");
        assert!(cpu.is_sleeping());
        cpu.set_sreg(super::Sreg {
            interrupt: true,
            ..super::Sreg::default()
        });
        let trace = cpu.step().expect("step").expect("trace");
        assert_eq!(trace.pc, 5);
        assert_eq!(cpu.register(17), 7);
        assert!(!cpu.is_sleeping());
    }

    #[test]
    fn wdr_is_noop() {
        let cpu = run(vec![0x95A8, ldi(16, 5), 0x9598]);
        assert_eq!(cpu.register(16), 5);
    }

    #[test]
    fn reset_clears_core_state() {
        let mut cpu = Cpu::new(vec![ldi(16, 9), 0x9598]).expect("cpu");
        cpu.step().expect("step");
        cpu.reset().expect("reset");
        assert_eq!(cpu.pc(), 0);
        assert_eq!(cpu.register(16), 0);
        assert_eq!(cpu.sreg(), super::Sreg::default());
        assert_eq!(cpu.sp(), 0x08FF);
        assert!(!cpu.is_halted());
    }

    #[test]
    fn sreg_byte_round_trip() {
        let sreg = super::Sreg {
            carry: true,
            zero: true,
            negative: true,
            overflow: true,
            sign: true,
            half_carry: true,
            transfer: true,
            interrupt: true,
        };
        assert_eq!(super::Sreg::from_byte(sreg.to_byte()), sreg);
    }

    #[test]
    fn vector_table_matches_datasheet() {
        assert_eq!(super::VECTORS.len(), 26);
        assert_eq!(super::VECTORS[0], 0x0000);
        assert_eq!(super::VECTORS[1], 0x0002);
        assert_eq!(super::VECTORS[25], 0x0032);
    }

    #[test]
    fn interrupt_entry_and_reti() {
        let mut cpu = Cpu::new(vec![ldi(16, 1), 0x9598, ldi(17, 7), 0x9518]).expect("cpu");
        cpu.step().expect("step");
        cpu.set_sreg(super::Sreg {
            interrupt: true,
            ..super::Sreg::default()
        });
        cpu.request_interrupt(1).expect("interrupt");
        let trace = cpu.step().expect("step").expect("trace");
        assert_eq!(trace.pc, 3);
        assert_eq!(cpu.register(17), 7);
        assert!(!cpu.sreg().interrupt);
        assert_eq!(cpu.sp(), 0x08FD);
        let trace = cpu.step().expect("step").expect("trace");
        assert_eq!(trace.pc, 1);
        assert!(cpu.sreg().interrupt);
    }

    #[test]
    fn sei_delays_interrupt_until_after_next_instruction() {
        let mut cpu = Cpu::new(vec![
            sreg_flag(0x9408, 7),
            ldi(16, 1),
            0xCFFF,
            0x0000,
            ldi(17, 7),
            0x9518,
        ])
        .expect("cpu");
        cpu.request_interrupt(2).expect("interrupt");
        cpu.step().expect("step");
        assert!(!cpu.sreg().interrupt);
        cpu.step().expect("step");
        assert!(cpu.sreg().interrupt);
        let trace = cpu.step().expect("step").expect("trace");
        assert_eq!(trace.pc, 5);
        assert_eq!(cpu.register(17), 7);
    }

    #[test]
    fn interrupt_rejects_bad_vector() {
        let mut cpu = Cpu::new(vec![0x9598]).expect("cpu");
        assert!(cpu.request_interrupt(26).is_err());
    }
}
