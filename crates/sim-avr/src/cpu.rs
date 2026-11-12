//! AVR execution core with a structural datapath.

use super::adc::Adc;
use super::alu::{ArithmeticUnit, LogicUnit, arithmetic_unit, logic_unit, select_operand};
use super::fetch::ProgramMemory;
use super::gpio::GpioPort;
use super::instruction::{Instruction, Mnemonic, Operand};
use super::multiply::{MultiplyUnit, multiply_unit};
use super::pin::{ExternalPin, PinElectrical, PinElectricalConfig, measure_pin};
use super::register_bank::RegisterBank;
use super::shift::{ShiftUnit, shift_unit};
use super::spi::Spi;
use super::timer0::Timer0;
use super::timer1::Timer1;
use super::timer2::Timer2;
use super::twi::Twi;
use super::uart::Uart;
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
    /// Cycles used by this instruction.
    pub cycles: u8,
    /// Total cycles since reset.
    pub total_cycles: u64,
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

struct Gpio {
    b: GpioPort,
    c: GpioPort,
    d: GpioPort,
}

struct Timers {
    timer0: Timer0,
    timer1: Timer1,
    timer2: Timer2,
}

struct Serial {
    spi: Spi,
    twi: Twi,
    adc: Adc,
}

impl Serial {
    fn new() -> Self {
        Self {
            spi: Spi::new(),
            twi: Twi::new(),
            adc: Adc::new(),
        }
    }
}

struct Datapath {
    bank: RegisterBank,
    arithmetic: ArithmeticUnit,
    logic: LogicUnit,
    shift: ShiftUnit,
    multiply: MultiplyUnit,
    gpio: Gpio,
    timers: Timers,
    serial: Serial,
    uart: Uart,
}

/// A small AVR core for instruction integration tests.
pub struct Cpu {
    circuit: DigitalCircuit,
    datapath: Datapath,
    program: ProgramMemory,
    pc: u16,
    sreg: Sreg,
    halted: bool,
    io: [u8; 256],
    data: Vec<u8>,
    sp: u16,
    pending: u32,
    sei_delay: u8,
    sleeping: bool,
    cycles: u64,
    pin_change: PinChange,
}

#[derive(Default)]
struct PinChange {
    pcicr: u8,
    pcifr: u8,
    pcmsk: [u8; 3],
    last_pin: [u8; 3],
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
            io: [0; 256],
            data: vec![0; 0x1000],
            sp: 0x08FF,
            pending: 0,
            sei_delay: 0,
            sleeping: false,
            cycles: 0,
            pin_change: PinChange::default(),
        })
    }

    /// Replaces program memory and resets the core.
    pub fn load_program(&mut self, program: Vec<u16>) -> Result<(), SimError> {
        if program.is_empty() {
            return Err(SimError::InvalidNetlist(
                "AVR program must not be empty".to_string(),
            ));
        }
        self.program = ProgramMemory::new(program);
        self.reset()
    }

    /// Returns the current program words.
    #[must_use]
    pub fn program(&self) -> &[u16] {
        self.program.words()
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
        let interrupt = self.service_interrupt()?;
        let fetched = self.program.fetch(self.pc)?;
        let next_pc = self.pc.wrapping_add(u16::from(fetched.words));
        self.execute(&fetched.instruction, next_pc)?;
        self.finish_sei();
        let cycles = self.instruction_cycles(fetched.instruction.mnemonic, next_pc);
        let total = u64::from(cycles) + if interrupt { 4 } else { 0 };
        self.cycles += total;
        self.advance_timer0(total as u32)?;
        self.advance_serial(total as u32);
        Ok(Some(self.trace(fetched.instruction, cycles)))
    }

    fn advance_serial(&mut self, cycles: u32) {
        self.datapath.uart.tick(cycles);
        self.datapath.serial.spi.tick(cycles);
        self.datapath.serial.twi.tick(cycles);
        self.datapath.serial.adc.tick(cycles);
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

    /// Drives external pin inputs for port B, C, or D.
    pub fn set_pin_input(&mut self, port: u8, value: u8) -> Result<(), SimError> {
        self.drive_pin(port, value)?;
        let pin = self.gpio_pin(port)?;
        self.handle_pin_change(port, pin)
    }

    /// Returns the electrical state of one pin.
    pub fn pin_electrical(
        &self,
        port: u8,
        bit: u8,
        external: ExternalPin,
        config: &PinElectricalConfig,
    ) -> Result<PinElectrical, SimError> {
        let mode = self.gpio_ref(port)?.pin_mode(&self.circuit, bit)?;
        measure_pin(mode, external, config)
    }

    /// Applies an external voltage and updates the digital input when needed.
    pub fn set_pin_voltage(
        &mut self,
        port: u8,
        bit: u8,
        external: ExternalPin,
        config: &PinElectricalConfig,
    ) -> Result<PinElectrical, SimError> {
        let mode = self.gpio_ref(port)?.pin_mode(&self.circuit, bit)?;
        let state = measure_pin(mode, external, config)?;
        if mode.is_input() {
            self.set_pin_input_bit(port, bit, state.logic_level(config))?;
            let pin = self.gpio_pin(port)?;
            self.handle_pin_change(port, pin)?;
        }
        Ok(state)
    }

    /// Sets the pin-change mask for port B, C, or D.
    pub fn set_pin_change_mask(&mut self, port: u8, mask: u8) -> Result<(), SimError> {
        if port > 2 {
            return Err(SimError::InvalidNetlist(
                "port index out of range".to_string(),
            ));
        }
        self.pin_change.pcmsk[usize::from(port)] = mask;
        Ok(())
    }

    /// Enables or disables pin-change interrupt for a port.
    pub fn enable_pin_change(&mut self, port: u8, enabled: bool) -> Result<(), SimError> {
        if port > 2 {
            return Err(SimError::InvalidNetlist(
                "port index out of range".to_string(),
            ));
        }
        if enabled {
            self.pin_change.pcicr |= 1 << port;
        } else {
            self.pin_change.pcicr &= !(1 << port);
        }
        Ok(())
    }

    /// Returns the pin-change flag register.
    #[must_use]
    pub const fn pin_change_flags(&self) -> u8 {
        self.pin_change.pcifr
    }

    /// Clears one pin-change flag.
    pub fn clear_pin_change_flag(&mut self, port: u8) {
        if port <= 2 {
            self.pin_change.pcifr &= !(1 << port);
        }
    }

    fn drive_pin(&mut self, port: u8, value: u8) -> Result<(), SimError> {
        match port {
            0 => self.datapath.gpio.b.set_pin_input(&mut self.circuit, value),
            1 => self.datapath.gpio.c.set_pin_input(&mut self.circuit, value),
            2 => self.datapath.gpio.d.set_pin_input(&mut self.circuit, value),
            _ => Err(SimError::InvalidNetlist(
                "port index out of range".to_string(),
            )),
        }
    }

    fn gpio_pin(&self, port: u8) -> Result<u8, SimError> {
        Ok(self.gpio_ref(port)?.read_pin(&self.circuit))
    }

    fn gpio_ref(&self, port: u8) -> Result<&GpioPort, SimError> {
        match port {
            0 => Ok(&self.datapath.gpio.b),
            1 => Ok(&self.datapath.gpio.c),
            2 => Ok(&self.datapath.gpio.d),
            _ => Err(SimError::InvalidNetlist(
                "port index out of range".to_string(),
            )),
        }
    }

    fn set_pin_input_bit(&mut self, port: u8, bit: u8, value: bool) -> Result<(), SimError> {
        let Self {
            circuit, datapath, ..
        } = self;
        match port {
            0 => datapath.gpio.b.set_pin_input_bit(circuit, bit, value),
            1 => datapath.gpio.c.set_pin_input_bit(circuit, bit, value),
            2 => datapath.gpio.d.set_pin_input_bit(circuit, bit, value),
            _ => Err(SimError::InvalidNetlist(
                "port index out of range".to_string(),
            )),
        }
    }

    fn handle_pin_change(&mut self, port: u8, pin: u8) -> Result<(), SimError> {
        let index = usize::from(port);
        let changed = self.pin_change.last_pin[index] ^ pin;
        self.pin_change.last_pin[index] = pin;
        if changed & self.pin_change.pcmsk[index] == 0 || self.pin_change.pcicr & (1 << port) == 0 {
            return Ok(());
        }
        self.pin_change.pcifr |= 1 << port;
        self.request_interrupt(3 + port)
    }

    /// Returns active pull-up bits for port B, C, or D.
    #[must_use]
    pub fn pull_up(&self, port: u8) -> u8 {
        match port {
            0 => self.datapath.gpio.b.read_pull_up(&self.circuit),
            1 => self.datapath.gpio.c.read_pull_up(&self.circuit),
            2 => self.datapath.gpio.d.read_pull_up(&self.circuit),
            _ => 0,
        }
    }

    /// Returns the Timer0 PWM output compare A signal.
    #[must_use]
    pub const fn pwm_a(&self) -> bool {
        self.datapath.timers.timer0.pwm_output()
    }

    /// Returns the Timer1 core for configuration.
    pub fn timer1(&mut self) -> &mut Timer1 {
        &mut self.datapath.timers.timer1
    }

    /// Returns the Timer2 core for configuration.
    pub fn timer2(&mut self) -> &mut Timer2 {
        &mut self.datapath.timers.timer2
    }

    /// Returns the UART core for configuration.
    pub fn uart(&mut self) -> &mut Uart {
        &mut self.datapath.uart
    }

    /// Returns the SPI core for configuration.
    pub fn spi(&mut self) -> &mut Spi {
        &mut self.datapath.serial.spi
    }

    /// Returns the TWI core for configuration.
    pub fn twi(&mut self) -> &mut Twi {
        &mut self.datapath.serial.twi
    }

    /// Returns the ADC core for configuration.
    pub fn adc(&mut self) -> &mut Adc {
        &mut self.datapath.serial.adc
    }

    /// Requests SPI interrupt when SPIF is set.
    pub fn poll_spi(&mut self) -> Result<(), SimError> {
        if self.datapath.serial.spi.spif() {
            self.request_interrupt(17)?;
        }
        Ok(())
    }

    /// Requests TWI interrupt when TWINT is set.
    pub fn poll_twi(&mut self) -> Result<(), SimError> {
        if self.datapath.serial.twi.twint() {
            self.request_interrupt(24)?;
        }
        Ok(())
    }

    /// Requests ADC interrupt when ADIF is set.
    pub fn poll_adc(&mut self) -> Result<(), SimError> {
        if self.datapath.serial.adc.flag() {
            self.request_interrupt(21)?;
        }
        Ok(())
    }

    /// Requests UART interrupts for the provided flag mask.
    pub fn poll_uart(&mut self, mask: u8) -> Result<(), SimError> {
        let uart = &self.datapath.uart;
        let rx = uart.rx_ready();
        let tx = uart.tx_complete();
        let udre = uart.tx_ready();
        if rx && mask & 0x01 != 0 {
            self.request_interrupt(18)?;
        }
        if tx && mask & 0x02 != 0 {
            self.request_interrupt(20)?;
        }
        if udre && mask & 0x04 != 0 {
            self.request_interrupt(19)?;
        }
        Ok(())
    }

    /// Requests Timer1 interrupts for the provided flag mask.
    pub fn poll_timer1(&mut self, mask: u8) -> Result<(), SimError> {
        let timer = &self.datapath.timers.timer1;
        let capture = timer.capture_flag();
        let compare_a = timer.compare_a_flag();
        let compare_b = timer.compare_b_flag();
        let overflow = timer.overflow_flag();
        if capture && mask & 0x01 != 0 {
            self.request_interrupt(10)?;
        }
        if compare_a && mask & 0x02 != 0 {
            self.request_interrupt(11)?;
        }
        if compare_b && mask & 0x04 != 0 {
            self.request_interrupt(12)?;
        }
        if overflow && mask & 0x08 != 0 {
            self.request_interrupt(13)?;
        }
        Ok(())
    }

    /// Requests Timer2 interrupts for the provided flag mask.
    pub fn poll_timer2(&mut self, mask: u8) -> Result<(), SimError> {
        let timer = &self.datapath.timers.timer2;
        let overflow = timer.overflow_flag();
        let compare = timer.compare_flag();
        if overflow && mask & 0x01 != 0 {
            self.request_interrupt(9)?;
        }
        if compare && mask & 0x02 != 0 {
            self.request_interrupt(7)?;
        }
        Ok(())
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

    fn service_interrupt(&mut self) -> Result<bool, SimError> {
        if !self.sreg.interrupt || self.pending == 0 {
            return Ok(false);
        }
        let index = self.pending.trailing_zeros() as u8;
        self.pending &= !(1u32 << index);
        if (3..=5).contains(&index) {
            self.pin_change.pcifr &= !(1 << (index - 3));
        }
        self.push_word(self.pc);
        self.sreg.interrupt = false;
        self.pc = vector_address(index);
        Ok(true)
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

    fn instruction_cycles(&self, mnemonic: Mnemonic, next_pc: u16) -> u8 {
        if matches!(mnemonic, Mnemonic::Brbs | Mnemonic::Brbc) {
            return if self.pc != next_pc { 2 } else { 1 };
        }
        if is_skip(mnemonic) {
            if self.pc == next_pc {
                return 1;
            }
            let skipped = self.pc.wrapping_sub(next_pc).min(2);
            return 1 + skipped as u8;
        }
        base_cycles(mnemonic)
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

    /// Returns total cycles since reset.
    #[must_use]
    pub const fn cycles(&self) -> u64 {
        self.cycles
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
        self.cycles = 0;
        self.pin_change = PinChange::default();
        self.datapath.timers.timer0.reset()?;
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
        self.io_write(address, value)?;
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
        self.io_write(address, self.register(source))?;
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
        if let Some(value) = self.timer0_read(address) {
            return value;
        }
        if let Some(value) = self.gpio_read(address) {
            return value;
        }
        if address == 0x3F {
            return self.sreg.to_byte();
        }
        self.io.get(usize::from(address)).copied().unwrap_or(0)
    }

    fn io_write(&mut self, address: u8, value: u8) -> Result<(), SimError> {
        if self.timer0_write(address, value)? {
            return Ok(());
        }
        if self.gpio_write(address, value)? {
            return Ok(());
        }
        if address == 0x3F {
            self.sreg = Sreg::from_byte(value);
        } else if let Some(slot) = self.io.get_mut(usize::from(address)) {
            *slot = value;
        }
        Ok(())
    }

    fn timer0_read(&self, address: u8) -> Option<u8> {
        match address {
            0x26 => Some(self.datapath.timers.timer0.counter()),
            0x35 => Some(self.io[0x35]),
            _ => None,
        }
    }

    fn timer0_write(&mut self, address: u8, value: u8) -> Result<bool, SimError> {
        match address {
            0x24 => self.write_timer0_control(value),
            0x25 => {
                self.io[0x25] = value;
                self.apply_timer0_clock()?;
            }
            0x27 => {
                self.io[0x27] = value;
                self.datapath.timers.timer0.set_compare(value);
            }
            0x28 | 0x2E | 0x6E => self.io[usize::from(address)] = value,
            0x35 => self.write_timer0_flags(value),
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn write_timer0_control(&mut self, value: u8) {
        self.io[0x24] = value;
        let wave = value & 0x03;
        self.datapath.timers.timer0.set_ctc(wave == 0x02);
        if wave == 0x03 {
            self.datapath.timers.timer0.set_pwm(value & 0xC0 == 0xC0);
        }
    }

    fn write_timer0_flags(&mut self, value: u8) {
        if value & 0x01 != 0 {
            self.datapath.timers.timer0.take_overflow();
        }
        if value & 0x02 != 0 {
            self.datapath.timers.timer0.take_compare();
        }
        self.io[0x35] &= !(value & 0x03);
    }

    fn apply_timer0_clock(&mut self) -> Result<(), SimError> {
        let select = match self.io[0x25] & 0x07 {
            1 => 0,
            2 => 1,
            3 => 2,
            4 => 3,
            5 => 4,
            _ => return Ok(()),
        };
        self.datapath.timers.timer0.set_prescaler(select)
    }

    fn advance_timer0(&mut self, cycles: u32) -> Result<(), SimError> {
        if !(1..=5).contains(&(self.io[0x25] & 0x07)) {
            return Ok(());
        }
        self.datapath.timers.timer0.tick(cycles)?;
        self.update_timer0_flags()
    }

    fn update_timer0_flags(&mut self) -> Result<(), SimError> {
        let mut flags = self.io[0x35] & 0xF8;
        if self.datapath.timers.timer0.overflow_flag() {
            flags |= 0x01;
        }
        if self.datapath.timers.timer0.compare_flag() {
            flags |= 0x02;
        }
        self.io[0x35] = flags;
        let mask = self.io[0x2E] | self.io[0x6E];
        if flags & 0x01 != 0 && mask & 0x01 != 0 {
            self.request_interrupt(16)?;
        }
        if flags & 0x02 != 0 && mask & 0x02 != 0 {
            self.request_interrupt(14)?;
        }
        Ok(())
    }

    fn gpio_read(&self, address: u8) -> Option<u8> {
        Some(match address {
            0x03 => self.datapath.gpio.b.read_pin(&self.circuit),
            0x04 => self.datapath.gpio.b.read_ddr(&self.circuit),
            0x05 => self.datapath.gpio.b.read_port(&self.circuit),
            0x06 => self.datapath.gpio.c.read_pin(&self.circuit),
            0x07 => self.datapath.gpio.c.read_ddr(&self.circuit),
            0x08 => self.datapath.gpio.c.read_port(&self.circuit),
            0x09 => self.datapath.gpio.d.read_pin(&self.circuit),
            0x0A => self.datapath.gpio.d.read_ddr(&self.circuit),
            0x0B => self.datapath.gpio.d.read_port(&self.circuit),
            _ => return None,
        })
    }

    fn gpio_write(&mut self, address: u8, value: u8) -> Result<bool, SimError> {
        match address {
            0x04 => self.datapath.gpio.b.set_ddr(&mut self.circuit, value)?,
            0x05 => self.datapath.gpio.b.set_port(&mut self.circuit, value)?,
            0x07 => self.datapath.gpio.c.set_ddr(&mut self.circuit, value)?,
            0x08 => self.datapath.gpio.c.set_port(&mut self.circuit, value)?,
            0x0A => self.datapath.gpio.d.set_ddr(&mut self.circuit, value)?,
            0x0B => self.datapath.gpio.d.set_port(&mut self.circuit, value)?,
            _ => return Ok(false),
        }
        Ok(true)
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

    fn trace(&self, instruction: Instruction, cycles: u8) -> AvrTrace {
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
            cycles,
            total_cycles: self.cycles,
        }
    }
}

struct Units {
    arithmetic: ArithmeticUnit,
    logic: LogicUnit,
    shift: ShiftUnit,
    multiply: MultiplyUnit,
}

fn build_datapath(builder: &mut DigitalBuilder) -> Result<Datapath, SimError> {
    let nets = build_alu_nets(builder);
    let selected = select_operand(builder, &nets.second, &nets.immediate, nets.use_immediate)?;
    let units = build_units(builder, &nets, &selected)?;
    let gpio = build_gpio(builder)?;
    let (bank, timers) = build_io(builder)?;
    let serial = Serial::new();
    let uart = Uart::new();
    Ok(Datapath {
        bank,
        arithmetic: units.arithmetic,
        logic: units.logic,
        shift: units.shift,
        multiply: units.multiply,
        gpio,
        timers,
        serial,
        uart,
    })
}

fn build_io(builder: &mut DigitalBuilder) -> Result<(RegisterBank, Timers), SimError> {
    Ok((
        build_bank(builder)?,
        Timers {
            timer0: Timer0::new()?,
            timer1: Timer1::new()?,
            timer2: Timer2::new()?,
        },
    ))
}

fn build_units(
    builder: &mut DigitalBuilder,
    nets: &AluNets,
    selected: &[NetId],
) -> Result<Units, SimError> {
    let arithmetic = arithmetic_unit(builder, &nets.first, selected, nets.subtract, nets.carry)?;
    let logic_op = named_inputs(builder, "lop", 2);
    let logic = logic_unit(builder, &nets.first, selected, &logic_op)?;
    let shift_op = named_inputs(builder, "sop", 2);
    let shift = shift_unit(builder, &nets.first, nets.carry, &shift_op)?;
    let multiply = multiply_unit(builder, &nets.first, &nets.second)?;
    Ok(Units {
        arithmetic,
        logic,
        shift,
        multiply,
    })
}

fn build_gpio(builder: &mut DigitalBuilder) -> Result<Gpio, SimError> {
    Ok(Gpio {
        b: GpioPort::new(builder, "pb")?,
        c: GpioPort::new(builder, "pc")?,
        d: GpioPort::new(builder, "pd")?,
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

fn base_cycles(mnemonic: Mnemonic) -> u8 {
    if is_four_cycle(mnemonic) {
        4
    } else if is_three_cycle(mnemonic) {
        3
    } else if is_two_cycle(mnemonic) {
        2
    } else {
        1
    }
}

fn is_two_cycle(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Mul
            | Mnemonic::Muls
            | Mnemonic::Mulsu
            | Mnemonic::Push
            | Mnemonic::Pop
            | Mnemonic::Rjmp
            | Mnemonic::Ijmp
            | Mnemonic::Adiw
            | Mnemonic::Sbiw
            | Mnemonic::Sbi
            | Mnemonic::Cbi
    ) || is_indirect_load(mnemonic)
        || matches!(mnemonic, Mnemonic::LddY | Mnemonic::LddZ | Mnemonic::Lds)
        || is_store(mnemonic)
}

fn is_three_cycle(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Jmp
            | Mnemonic::Rcall
            | Mnemonic::Icall
            | Mnemonic::Eicall
            | Mnemonic::LpmZ
            | Mnemonic::LpmZPost
            | Mnemonic::ElpmZ
            | Mnemonic::ElpmZPost
    )
}

fn is_four_cycle(mnemonic: Mnemonic) -> bool {
    matches!(mnemonic, Mnemonic::Call | Mnemonic::Ret | Mnemonic::Reti)
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
    use crate::pin::{ExternalPin, PinElectricalConfig};

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
    fn load_program_replaces_flash_and_resets() {
        let mut cpu = Cpu::new(vec![ldi(16, 1), 0x9598]).expect("cpu");
        cpu.run(10).expect("run");
        cpu.load_program(vec![ldi(16, 7), 0x9598]).expect("load");
        assert_eq!(cpu.program()[0], ldi(16, 7));
        cpu.run(10).expect("run");
        assert_eq!(cpu.register(16), 7);
        assert!(cpu.is_halted());
    }

    #[test]
    fn load_program_rejects_empty() {
        let mut cpu = Cpu::new(vec![0x9598]).expect("cpu");
        assert!(cpu.load_program(Vec::new()).is_err());
    }

    #[test]
    fn load_program_runs_blink_pin() {
        let program = vec![
            0xE200, 0xB904, 0xB905, 0xEF1F, 0x951A, 0xF7F1, 0xE220, 0x2702, 0xCFF9,
        ];
        let mut cpu = Cpu::new(vec![0x0000]).expect("cpu");
        cpu.load_program(program).expect("load");
        let config = PinElectricalConfig::default();
        let mut high = false;
        for _ in 0..2000 {
            cpu.step().expect("step");
            let state = cpu
                .pin_electrical(0, 5, ExternalPin::HighZ, &config)
                .expect("pin");
            high |= state.voltage > 2.5;
        }
        assert!(high);
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

    #[test]
    fn instruction_and_total_cycles() {
        let mut cpu = Cpu::new(vec![
            ldi(16, 1),
            ldi(17, 2),
            reg_op(0x0C00, 16, 17),
            0xC001,
            ldi(18, 9),
            0x9598,
        ])
        .expect("cpu");
        let traces = cpu.run(10).expect("run");
        assert_eq!(traces.len(), 5);
        assert_eq!(cpu.cycles(), 6);
        assert_eq!(traces[4].total_cycles, 6);
    }

    #[test]
    fn taken_branch_costs_two_cycles() {
        let mut cpu = Cpu::new(vec![
            ldi(16, 0),
            reg_op(0x1800, 16, 16),
            branch(1, 1),
            ldi(17, 9),
            ldi(18, 7),
            0x9598,
        ])
        .expect("cpu");
        let traces = cpu.run(10).expect("run");
        assert_eq!(traces[2].cycles, 2);
    }

    #[test]
    fn skip_over_two_word_instruction_costs_three() {
        let mut cpu = Cpu::new(vec![
            ldi(16, 1),
            ldi(17, 1),
            reg_op(0x1000, 16, 17),
            0x9000 | (18 << 4),
            0x0100,
            0x9598,
        ])
        .expect("cpu");
        let traces = cpu.run(10).expect("run");
        assert_eq!(traces[2].cycles, 3);
    }

    #[test]
    fn interrupt_entry_adds_four_cycles() {
        let mut cpu = Cpu::new(vec![ldi(16, 1), 0x9598, ldi(17, 7), 0x9518]).expect("cpu");
        cpu.step().expect("step");
        cpu.set_sreg(super::Sreg {
            interrupt: true,
            ..super::Sreg::default()
        });
        cpu.request_interrupt(1).expect("interrupt");
        let before = cpu.cycles();
        let trace = cpu.step().expect("step").expect("trace");
        assert_eq!(trace.cycles, 1);
        assert_eq!(cpu.cycles() - before, 5);
    }

    #[test]
    fn gpio_output_and_pin_read() {
        let cpu = run(vec![
            ldi(16, 0x20),
            in_out(0xB800, 16, 0x04),
            in_out(0xB800, 16, 0x05),
            in_out(0xB000, 17, 0x03),
            0x9598,
        ]);
        assert_eq!(cpu.register(17), 0x20);
        assert_eq!(cpu.io(0x04), 0x20);
        assert_eq!(cpu.io(0x05), 0x20);
    }

    #[test]
    fn gpio_input_reads_external_pins() {
        let mut cpu = Cpu::new(vec![in_out(0xB000, 17, 0x03), 0x9598]).expect("cpu");
        cpu.set_pin_input(0, 0xAA).expect("pin");
        cpu.run(10).expect("run");
        assert_eq!(cpu.register(17), 0xAA);
    }

    #[test]
    fn gpio_pull_up_tracks_port_register() {
        let cpu = run(vec![ldi(16, 0xFF), in_out(0xB800, 16, 0x05), 0x9598]);
        assert_eq!(cpu.pull_up(0), 0xFF);
    }

    fn pin_change_cpu() -> Cpu {
        Cpu::new(vec![
            ldi(16, 1),
            0xCFFF,
            0x0000,
            0x0000,
            0x0000,
            0x0000,
            ldi(17, 7),
            0x9518,
        ])
        .expect("cpu")
    }

    #[test]
    fn pin_change_requests_interrupt() {
        let mut cpu = pin_change_cpu();
        cpu.set_sreg(super::Sreg {
            interrupt: true,
            ..super::Sreg::default()
        });
        cpu.set_pin_change_mask(0, 0x01).expect("mask");
        cpu.enable_pin_change(0, true).expect("enable");
        cpu.set_pin_input(0, 0x01).expect("pin");
        let trace = cpu.step().expect("step").expect("trace");
        assert_eq!(trace.pc, 7);
        assert_eq!(cpu.register(17), 7);
        assert_eq!(cpu.pin_change_flags(), 0);
    }

    #[test]
    fn pin_change_ignores_masked_and_disabled_bits() {
        let mut cpu = Cpu::new(vec![0x9598]).expect("cpu");
        cpu.set_pin_change_mask(0, 0x02).expect("mask");
        cpu.enable_pin_change(0, true).expect("enable");
        cpu.set_pin_input(0, 0x01).expect("pin");
        assert_eq!(cpu.pin_change_flags(), 0);
        cpu.set_pin_change_mask(0, 0x01).expect("mask");
        cpu.enable_pin_change(0, false).expect("disable");
        cpu.set_pin_input(0, 0x00).expect("pin");
        assert_eq!(cpu.pin_change_flags(), 0);
    }

    #[test]
    fn pin_change_flag_can_be_cleared() {
        let mut cpu = Cpu::new(vec![0x9598]).expect("cpu");
        cpu.set_pin_change_mask(0, 0x01).expect("mask");
        cpu.enable_pin_change(0, true).expect("enable");
        cpu.set_pin_input(0, 0x01).expect("pin");
        assert_eq!(cpu.pin_change_flags(), 1);
        cpu.clear_pin_change_flag(0);
        assert_eq!(cpu.pin_change_flags(), 0);
    }

    #[test]
    fn timer0_counts_cpu_cycles() {
        let cpu = run(vec![
            ldi(16, 1),
            in_out(0xB800, 16, 0x25),
            0x0000,
            0x0000,
            0x0000,
            0x0000,
            0x0000,
            in_out(0xB000, 17, 0x26),
            0x9598,
        ]);
        assert_eq!(cpu.register(17), 6);
    }

    #[test]
    fn timer0_sets_overflow_flag() {
        let mut cpu = Cpu::new(vec![
            ldi(16, 1),
            in_out(0xB800, 16, 0x25),
            ldi(18, 255),
            one_reg(0x940A, 18),
            0xF7F1,
            in_out(0xB000, 17, 0x35),
            0x9598,
        ])
        .expect("cpu");
        cpu.run(2000).expect("run");
        assert_eq!(cpu.register(17) & 0x01, 0x01);
    }

    #[test]
    fn timer0_sets_compare_flag_in_ctc() {
        let cpu = run(vec![
            ldi(16, 3),
            in_out(0xB800, 16, 0x27),
            ldi(16, 2),
            in_out(0xB800, 16, 0x24),
            ldi(16, 1),
            in_out(0xB800, 16, 0x25),
            0x0000,
            0x0000,
            0x0000,
            in_out(0xB000, 17, 0x35),
            0x9598,
        ]);
        assert_eq!(cpu.register(17) & 0x02, 0x02);
    }

    fn timer0_overflow_program() -> Vec<u16> {
        let mut program = vec![0u16; 40];
        program[0] = ldi(16, 1);
        program[1] = in_out(0xB800, 16, 0x25);
        program[2] = ldi(16, 1);
        program[3] = in_out(0xB800, 16, 0x2E);
        program[4] = ldi(18, 255);
        program[5] = one_reg(0x940A, 18);
        program[6] = 0xF7F1;
        program[7] = 0xCFFF;
        program[32] = ldi(17, 7);
        program[33] = 0x9518;
        program
    }

    #[test]
    fn timer0_overflow_interrupt_enters_vector() {
        let mut cpu = Cpu::new(timer0_overflow_program()).expect("cpu");
        cpu.set_sreg(super::Sreg {
            interrupt: true,
            ..super::Sreg::default()
        });
        for _ in 0..2000 {
            cpu.step().expect("step");
            if cpu.register(17) == 7 {
                break;
            }
        }
        assert_eq!(cpu.register(17), 7);
    }
    #[test]
    fn timer0_pwm_output_from_cpu() {
        let mut cpu = Cpu::new(vec![
            ldi(16, 2),
            in_out(0xB800, 16, 0x27),
            ldi(16, 0x83),
            in_out(0xB800, 16, 0x24),
            ldi(16, 1),
            in_out(0xB800, 16, 0x25),
            0x0000,
            0x9598,
        ])
        .expect("cpu");
        for _ in 0..6 {
            cpu.step().expect("step");
        }
        assert!(cpu.pwm_a());
        cpu.step().expect("nop");
        assert!(!cpu.pwm_a());
    }

    #[test]
    fn uart_transmits_from_cpu_cycles() {
        let mut cpu = Cpu::new(vec![0xCFFF]).expect("cpu");
        cpu.uart().write_control(1, 0x08);
        cpu.uart().write_tx(0x55);
        for _ in 0..120 {
            cpu.step().expect("step");
        }
        assert!(cpu.uart().tx_complete());
        assert!(cpu.uart().tx_ready());
        assert!(cpu.uart().tx_line());
    }

    #[test]
    fn uart_frame_decodes_from_cpu_cycles() {
        let mut cpu = Cpu::new(vec![0xCFFF]).expect("cpu");
        cpu.uart().write_control(1, 0x08);
        cpu.uart().write_tx(0xA5);
        assert_eq!(decode_uart(&mut cpu), 0xA5);
    }

    fn decode_uart(cpu: &mut Cpu) -> u8 {
        let mut value = 0u8;
        for bit in 0..10 {
            step_cpu(cpu, 4);
            let level = cpu.uart().tx_line();
            if (1..=8).contains(&bit) && level {
                value |= 1 << (bit - 1);
            }
            step_cpu(cpu, 4);
        }
        value
    }

    fn step_cpu(cpu: &mut Cpu, count: usize) {
        for _ in 0..count {
            cpu.step().expect("step");
        }
    }

    #[test]
    fn uart_rx_interrupt_enters_vector() {
        let mut program = vec![0u16; 40];
        program[0] = 0xCFFF;
        program[36] = ldi(17, 7);
        program[37] = 0x9518;
        let mut cpu = Cpu::new(program).expect("cpu");
        cpu.set_sreg(super::Sreg {
            interrupt: true,
            ..super::Sreg::default()
        });
        cpu.uart().receive(0x41);
        cpu.poll_uart(0x01).expect("poll");
        let trace = cpu.step().expect("step").expect("trace");
        assert_eq!(trace.pc, 37);
        assert_eq!(cpu.register(17), 7);
    }

    #[test]
    fn spi_transfer_advances_from_cpu_cycles() {
        let mut cpu = Cpu::new(vec![0xCFFF]).expect("cpu");
        cpu.spi().write_control(0x50);
        cpu.spi().set_received(0xA5);
        cpu.spi().write_data(0x5A);
        for _ in 0..40 {
            cpu.step().expect("step");
        }
        assert!(cpu.spi().spif());
        assert_eq!(cpu.spi().read_data(), 0xA5);
    }

    #[test]
    fn adc_conversion_advances_from_cpu_cycles() {
        let mut cpu = Cpu::new(vec![0xCFFF]).expect("cpu");
        cpu.adc().set_input(0, 0x2AA);
        cpu.adc().write_control_a(0);
        cpu.adc().write_control_b(0xC0);
        for _ in 0..20 {
            cpu.step().expect("step");
        }
        assert!(cpu.adc().flag());
        assert_eq!(cpu.adc().read_data_low(), 0xAA);
        assert_eq!(cpu.adc().read_data_high(), 0x02);
    }

    #[test]
    fn twi_start_completes_from_cpu_cycles() {
        let mut cpu = Cpu::new(vec![0xCFFF]).expect("cpu");
        cpu.twi().write_control(0x04);
        cpu.twi().start();
        for _ in 0..100 {
            cpu.step().expect("step");
        }
        assert!(cpu.twi().twint());
        assert_eq!(cpu.twi().status(), 0x08);
    }

    #[test]
    fn pin_voltage_updates_digital_input() {
        let mut cpu = Cpu::new(vec![0x0000]).expect("cpu");
        let config = PinElectricalConfig::default();
        let external = ExternalPin::Thevenin {
            voltage: 5.0,
            resistance: 1000.0,
        };
        let state = cpu.set_pin_voltage(0, 5, external, &config).expect("pin");
        assert!(state.high_z);
        assert!(state.logic_level(&config));
        assert_eq!(cpu.io(0x03) & 0x20, 0x20);
    }

    fn pb5_output_cpu() -> Cpu {
        let program = vec![
            io_bit_op(0x9A00, 0x04, 5),
            io_bit_op(0x9A00, 0x05, 5),
            0x0000,
        ];
        let mut cpu = Cpu::new(program).expect("cpu");
        cpu.step().expect("ddr");
        cpu.step().expect("port");
        cpu
    }

    #[test]
    fn output_contention_is_detected() {
        let cpu = pb5_output_cpu();
        let config = PinElectricalConfig::default();
        let open = cpu
            .pin_electrical(0, 5, ExternalPin::HighZ, &config)
            .expect("open");
        assert!((open.voltage - 5.0).abs() < 1e-12, "v={}", open.voltage);
        let external = ExternalPin::Thevenin {
            voltage: 0.0,
            resistance: 10.0,
        };
        let state = cpu.pin_electrical(0, 5, external, &config).expect("pin");
        assert!(state.contention);
        assert!(!state.high_z);
    }
}
