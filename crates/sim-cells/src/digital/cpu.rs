//! Small multi-cycle CPU for digital integration tests.

use super::alu::{DigitalAlu, digital_alu};
use super::blocks::{ProgramCounter, decoder, or_all, read_bits, register};
use super::circuit::{DigitalBuilder, DigitalCircuit, NetId};
use super::registers::{RegisterFile, RegisterPorts, register_file};
use sim_core::SimError;

/// No operation.
pub const OP_NOP: u16 = 0;
/// Load immediate 6-bit value.
pub const OP_LDI: u16 = 1;
/// Add register.
pub const OP_ADD: u16 = 2;
/// Subtract register.
pub const OP_SUB: u16 = 3;
/// Bitwise AND.
pub const OP_AND: u16 = 4;
/// Bitwise OR.
pub const OP_OR: u16 = 5;
/// Bitwise XOR.
pub const OP_XOR: u16 = 6;
/// Move register.
pub const OP_MOV: u16 = 7;
/// Unconditional jump.
pub const OP_JMP: u16 = 8;
/// Jump if zero.
pub const OP_JZ: u16 = 9;
/// Jump if not zero.
pub const OP_JNZ: u16 = 10;
/// Halt.
pub const OP_HLT: u16 = 31;

const WRITE_OPCODES: [usize; 7] = [1, 2, 3, 4, 5, 6, 7];
const FLAG_OPCODES: [usize; 5] = [2, 3, 4, 5, 6];
const ALU_BIT0: [usize; 4] = [1, 3, 5, 7];
const ALU_BIT1: [usize; 3] = [4, 5, 6];
const ALU_BIT2: [usize; 3] = [1, 6, 7];

/// Encodes a load-immediate instruction.
#[must_use]
pub fn encode_ldi(register: u16, value: u16) -> u16 {
    (OP_LDI << 11) | ((register & 0x1F) << 6) | (value & 0x3F)
}

/// Encodes an ALU register instruction.
#[must_use]
pub fn encode_alu(opcode: u16, destination: u16, source: u16) -> u16 {
    (opcode << 11) | ((destination & 0x1F) << 6) | (source & 0x1F)
}

/// Encodes a jump instruction.
#[must_use]
pub fn encode_branch(opcode: u16, target: u16) -> u16 {
    (opcode << 11) | (target & 0x7FF)
}

/// Encodes a halt instruction.
#[must_use]
pub const fn encode_hlt() -> u16 {
    OP_HLT << 11
}

/// One instruction trace entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuTrace {
    /// Program counter after the instruction.
    pub pc: u64,
    /// Instruction word in the register.
    pub instruction: u16,
    /// Halt state.
    pub halted: bool,
    /// Register snapshot after the instruction.
    pub registers: [u8; 32],
}

struct Fields {
    opcode: Vec<NetId>,
    rd: Vec<NetId>,
    rs: Vec<NetId>,
    imm: Vec<NetId>,
    target: Vec<NetId>,
}

struct Decode {
    fields: Fields,
    lines: Vec<NetId>,
    fetch: NetId,
    execute: NetId,
    signals: DecodeSignals,
}

struct DecodeSignals {
    reg_write: NetId,
    flag_write: NetId,
    use_imm: NetId,
    alu_op: Vec<NetId>,
}

struct Branch {
    pc_load: NetId,
    halt: NetId,
}

struct BuiltCpu {
    registers: RegisterFile,
    pc: ProgramCounter,
    ir: Vec<NetId>,
    halted: NetId,
}

/// A small CPU with 32 registers, 8-bit ALU, and 16-bit PC.
pub struct Cpu {
    circuit: DigitalCircuit,
    registers: RegisterFile,
    pc: ProgramCounter,
    ir: Vec<NetId>,
    halted: NetId,
    instruction: Vec<NetId>,
    program: Vec<u16>,
}

impl Cpu {
    /// Builds a CPU with the provided program.
    pub fn new(program: Vec<u16>) -> Result<Self, SimError> {
        if program.is_empty() || program.len() > 2048 {
            return Err(SimError::InvalidNetlist(
                "program must contain 1..=2048 words".to_string(),
            ));
        }
        let mut builder = DigitalBuilder::new();
        let instruction = instruction_inputs(&mut builder);
        let built = build_circuit(&mut builder, &instruction)?;
        Ok(Self {
            circuit: builder.build(),
            registers: built.registers,
            pc: built.pc,
            ir: built.ir,
            halted: built.halted,
            instruction,
            program,
        })
    }

    /// Executes one instruction and returns its trace.
    pub fn step(&mut self) -> Result<Option<CpuTrace>, SimError> {
        if self.is_halted() {
            return Ok(None);
        }
        self.fetch()?;
        self.circuit.tick()?;
        self.circuit.tick()?;
        Ok(Some(self.trace()?))
    }

    /// Executes until halt or until the step limit is reached.
    pub fn run(&mut self, max_steps: usize) -> Result<Vec<CpuTrace>, SimError> {
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
    pub fn register(&self, index: usize) -> Option<u8> {
        let bits = self.registers.bits().get(index)?;
        Some(read_bits(&self.circuit, bits) as u8)
    }

    /// Returns the program counter.
    #[must_use]
    pub fn pc(&self) -> u64 {
        self.pc.read(&self.circuit)
    }

    /// Returns the instruction register value.
    #[must_use]
    pub fn instruction(&self) -> u16 {
        read_bits(&self.circuit, &self.ir) as u16
    }

    /// Returns true when halted.
    #[must_use]
    pub fn is_halted(&self) -> bool {
        self.circuit.value(self.halted)
    }

    /// Resets the CPU to its initial state.
    pub fn reset(&mut self) -> Result<(), SimError> {
        self.circuit.reset()?;
        self.set_instruction(0)?;
        self.circuit.settle()
    }

    fn fetch(&mut self) -> Result<(), SimError> {
        let address = (self.pc.read(&self.circuit) as usize) & 0x7FF;
        let word = self.program.get(address).copied().unwrap_or(0);
        self.set_instruction(word)?;
        self.circuit.settle()
    }

    fn set_instruction(&mut self, word: u16) -> Result<(), SimError> {
        for index in 0..self.instruction.len() {
            let bit = word & (1 << index) != 0;
            self.circuit.set_input(&format!("instr{index}"), bit)?;
        }
        Ok(())
    }

    fn trace(&self) -> Result<CpuTrace, SimError> {
        let mut registers = [0u8; 32];
        for (index, bits) in self.registers.bits().iter().enumerate() {
            registers[index] = read_bits(&self.circuit, bits) as u8;
        }
        Ok(CpuTrace {
            pc: self.pc.read(&self.circuit),
            instruction: read_bits(&self.circuit, &self.ir) as u16,
            halted: self.is_halted(),
            registers,
        })
    }
}

fn instruction_inputs(builder: &mut DigitalBuilder) -> Vec<NetId> {
    (0..16)
        .map(|index| builder.input(&format!("instr{index}")))
        .collect()
}

fn build_circuit(
    builder: &mut DigitalBuilder,
    instruction: &[NetId],
) -> Result<BuiltCpu, SimError> {
    let fields = extract_fields(instruction)?;
    let phase = builder.net();
    let decode = build_decode(builder, fields, phase)?;
    let (registers, alu, write_data) = build_datapath(builder, &decode)?;
    connect_write_data(builder, &alu.result, &write_data);
    let zero = latch_zero(builder, alu.zero, decode.signals.flag_write);
    let branch = build_branch(builder, &decode, zero);
    let (pc, ir, halted) = build_state(builder, &decode, instruction, branch)?;
    Ok(BuiltCpu {
        registers,
        pc,
        ir,
        halted,
    })
}

fn build_state(
    builder: &mut DigitalBuilder,
    decode: &Decode,
    instruction: &[NetId],
    branch: Branch,
) -> Result<(ProgramCounter, Vec<NetId>, NetId), SimError> {
    let target = extended_target(builder, &decode.fields.target);
    let pc = ProgramCounter::new(builder, 16, branch.pc_load, &target, decode.fetch)?;
    let ir = register(builder, 16, instruction, decode.fetch)?;
    let halted = builder.net();
    let halt_next = or_all(builder, &[halted, branch.halt]);
    builder.dff_into(halt_next, halted);
    builder.dff_into(decode.fetch, decode.execute);
    Ok((pc, ir, halted))
}

fn extract_fields(instruction: &[NetId]) -> Result<Fields, SimError> {
    if instruction.len() != 16 {
        return Err(SimError::InvalidNetlist(
            "instruction must be 16 bits".to_string(),
        ));
    }
    Ok(Fields {
        opcode: instruction[11..16].to_vec(),
        rd: instruction[6..11].to_vec(),
        rs: instruction[0..5].to_vec(),
        imm: instruction[0..6].to_vec(),
        target: instruction[0..11].to_vec(),
    })
}

fn build_decode(
    builder: &mut DigitalBuilder,
    fields: Fields,
    phase: NetId,
) -> Result<Decode, SimError> {
    let lines = decoder(builder, &fields.opcode)?;
    let fetch = builder.not(phase);
    let signals = decode_signals(builder, &lines, phase);
    Ok(Decode {
        fields,
        lines,
        fetch,
        execute: phase,
        signals,
    })
}

fn decode_signals(builder: &mut DigitalBuilder, lines: &[NetId], phase: NetId) -> DecodeSignals {
    DecodeSignals {
        reg_write: write_enable(builder, lines, phase),
        flag_write: flag_enable(builder, lines, phase),
        use_imm: lines[OP_LDI as usize],
        alu_op: alu_op(builder, lines),
    }
}

fn write_enable(builder: &mut DigitalBuilder, lines: &[NetId], execute: NetId) -> NetId {
    let terms: Vec<NetId> = WRITE_OPCODES.iter().map(|&op| lines[op]).collect();
    let any = or_all(builder, &terms);
    builder.and(execute, any)
}

fn flag_enable(builder: &mut DigitalBuilder, lines: &[NetId], execute: NetId) -> NetId {
    let terms: Vec<NetId> = FLAG_OPCODES.iter().map(|&op| lines[op]).collect();
    let any = or_all(builder, &terms);
    builder.and(execute, any)
}

fn alu_op(builder: &mut DigitalBuilder, lines: &[NetId]) -> Vec<NetId> {
    vec![
        encoded_signal(builder, lines, &ALU_BIT0),
        encoded_signal(builder, lines, &ALU_BIT1),
        encoded_signal(builder, lines, &ALU_BIT2),
    ]
}

fn encoded_signal(builder: &mut DigitalBuilder, lines: &[NetId], opcodes: &[usize]) -> NetId {
    let terms: Vec<NetId> = opcodes.iter().map(|&op| lines[op]).collect();
    or_all(builder, &terms)
}

fn build_datapath(
    builder: &mut DigitalBuilder,
    decode: &Decode,
) -> Result<(RegisterFile, DigitalAlu, Vec<NetId>), SimError> {
    let write_data: Vec<NetId> = (0..8).map(|_| builder.net()).collect();
    let ports = register_ports(decode, write_data.clone());
    let registers = register_file(builder, 32, 8, &ports)?;
    let second = effective_second(
        builder,
        registers.read_b(),
        &decode.fields,
        decode.signals.use_imm,
    );
    let alu = digital_alu(builder, registers.read_a(), &second, &decode.signals.alu_op)?;
    Ok((registers, alu, write_data))
}

fn register_ports(decode: &Decode, write_data: Vec<NetId>) -> RegisterPorts {
    RegisterPorts {
        write_address: decode.fields.rd.clone(),
        write_data,
        write_enable: decode.signals.reg_write,
        read_address_a: decode.fields.rd.clone(),
        read_address_b: decode.fields.rs.clone(),
    }
}

fn effective_second(
    builder: &mut DigitalBuilder,
    register_second: &[NetId],
    fields: &Fields,
    use_imm: NetId,
) -> Vec<NetId> {
    let zero = builder.constant(false);
    let mut values = Vec::with_capacity(8);
    for (index, &value) in register_second.iter().enumerate().take(5) {
        values.push(builder.mux(value, fields.imm[index], use_imm));
    }
    values.push(builder.mux(zero, fields.imm[5], use_imm));
    values.push(zero);
    values.push(zero);
    values
}

fn connect_write_data(builder: &mut DigitalBuilder, source: &[NetId], target: &[NetId]) {
    for (input, output) in source.iter().zip(target) {
        builder.buf_into(*input, *output);
    }
}

fn latch_zero(builder: &mut DigitalBuilder, value: NetId, enable: NetId) -> NetId {
    let zero = builder.net();
    let next = builder.mux(zero, value, enable);
    builder.dff_into(next, zero);
    zero
}

fn build_branch(builder: &mut DigitalBuilder, decode: &Decode, zero: NetId) -> Branch {
    let not_zero = builder.not(zero);
    let jz_taken = builder.and(decode.lines[OP_JZ as usize], zero);
    let jnz_taken = builder.and(decode.lines[OP_JNZ as usize], not_zero);
    let any = or_all(
        builder,
        &[decode.lines[OP_JMP as usize], jz_taken, jnz_taken],
    );
    let pc_load = builder.and(decode.execute, any);
    let halt = builder.and(decode.execute, decode.lines[OP_HLT as usize]);
    Branch { pc_load, halt }
}

fn extended_target(builder: &mut DigitalBuilder, target: &[NetId]) -> Vec<NetId> {
    let zero = builder.constant(false);
    let mut value = target.to_vec();
    value.resize(16, zero);
    value
}

#[cfg(test)]
mod tests {
    use super::{
        Cpu, OP_ADD, OP_HLT, OP_JNZ, OP_JZ, OP_LDI, OP_SUB, encode_alu, encode_branch, encode_hlt,
        encode_ldi,
    };

    #[test]
    fn cpu_adds_register_values() {
        let program = vec![
            encode_ldi(1, 5),
            encode_ldi(2, 3),
            encode_alu(super::OP_MOV, 3, 1),
            encode_alu(OP_ADD, 3, 2),
            encode_hlt(),
        ];
        let mut cpu = Cpu::new(program).expect("cpu");
        let traces = cpu.run(20).expect("run");
        assert_eq!(traces.len(), 5);
        assert_eq!(cpu.register(3), Some(8), "{traces:?}");
        assert!(cpu.is_halted());
    }

    #[test]
    fn cpu_branches_on_zero() {
        let program = vec![
            encode_ldi(1, 0),
            encode_alu(OP_SUB, 1, 1),
            encode_branch(OP_JZ, 5),
            encode_ldi(2, 1),
            encode_ldi(2, 2),
            encode_ldi(3, 7),
            encode_hlt(),
        ];
        let mut cpu = Cpu::new(program).expect("cpu");
        let traces = cpu.run(30).expect("run");
        assert_eq!(cpu.register(2), Some(0));
        assert_eq!(cpu.register(3), Some(7));
        assert_eq!(traces[2].pc, 5);
    }

    #[test]
    fn cpu_loops_until_zero() {
        let program = vec![
            encode_ldi(1, 0),
            encode_ldi(2, 3),
            encode_ldi(3, 1),
            encode_alu(OP_ADD, 1, 3),
            encode_alu(OP_SUB, 2, 3),
            encode_branch(OP_JNZ, 3),
            encode_hlt(),
        ];
        let mut cpu = Cpu::new(program).expect("cpu");
        cpu.run(100).expect("run");
        assert_eq!(cpu.register(1), Some(3));
        assert_eq!(cpu.register(2), Some(0));
        assert!(cpu.is_halted());
    }

    #[test]
    fn cpu_reset_clears_state() {
        let program = vec![encode_ldi(1, 9), encode_hlt()];
        let mut cpu = Cpu::new(program).expect("cpu");
        cpu.run(10).expect("run");
        cpu.reset().expect("reset");
        assert_eq!(cpu.pc(), 0);
        assert_eq!(cpu.instruction(), 0);
        assert!(!cpu.is_halted());
        assert_eq!(cpu.register(1), Some(0));
    }

    #[test]
    fn cpu_jumps_unconditionally() {
        let program = vec![
            encode_branch(super::OP_JMP, 2),
            encode_ldi(1, 9),
            encode_ldi(2, 4),
            encode_hlt(),
        ];
        let mut cpu = Cpu::new(program).expect("cpu");
        cpu.run(20).expect("run");
        assert_eq!(cpu.register(1), Some(0));
        assert_eq!(cpu.register(2), Some(4));
    }

    fn trace_cpu() -> Cpu {
        let program = vec![
            encode_ldi(1, 1),
            encode_ldi(2, 2),
            encode_alu(OP_ADD, 1, 2),
            encode_hlt(),
        ];
        Cpu::new(program).expect("cpu")
    }

    #[test]
    fn cpu_trace_matches_expected_sequence() {
        let mut cpu = trace_cpu();
        let traces = cpu.run(10).expect("run");
        let expected = [
            (1, encode_ldi(1, 1)),
            (2, encode_ldi(2, 2)),
            (3, encode_alu(OP_ADD, 1, 2)),
            (4, encode_hlt()),
        ];
        assert_eq!(traces.len(), expected.len());
        for (trace, (pc, instruction)) in traces.iter().zip(expected) {
            assert_eq!(trace.pc, pc);
            assert_eq!(trace.instruction, instruction);
        }
        assert_eq!(traces[2].registers[1], 3);
        assert!(traces[3].halted);
    }

    #[test]
    fn cpu_rejects_empty_program() {
        assert!(Cpu::new(Vec::new()).is_err());
    }

    #[test]
    fn cpu_after_halt_returns_no_trace() {
        let program = vec![encode_hlt()];
        let mut cpu = Cpu::new(program).expect("cpu");
        assert!(cpu.step().expect("step").is_some());
        assert!(cpu.step().expect("step").is_none());
    }

    #[test]
    fn opcode_encoding_round_trips() {
        assert_eq!(encode_ldi(31, 63) >> 11, OP_LDI);
        assert_eq!(encode_alu(OP_ADD, 5, 6) & 0x7FF, (5 << 6) | 6);
        assert_eq!(encode_branch(OP_JZ, 0x7FF) & 0x7FF, 0x7FF);
        assert_eq!(encode_hlt() >> 11, OP_HLT);
    }
}
