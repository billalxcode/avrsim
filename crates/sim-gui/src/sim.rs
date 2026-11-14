//! Background AVR simulation session and worker protocol.

use sim_avr::{Cpu, ExternalPin, PinElectricalConfig};
use sim_board::{Board, HexImage};
use std::collections::VecDeque;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const CHUNK: usize = 100;
const POLL_MS: u64 = 2;
const FLASH_BYTES: usize = 0x8000;
const WAVE_LIMIT: usize = 2048;
/// Default probe sampling interval in CPU cycles.
pub const DEFAULT_SAMPLE_EVERY: u64 = 100;

/// One PB5/LED sample.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sample {
    pub cycle: u64,
    pub pb5_high: bool,
    pub pb5_millivolts: i32,
    pub led_on: bool,
    pub led_microamps: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SampleState {
    pb5_high: bool,
    pb5_millivolts: i32,
    led_on: bool,
    led_microamps: i32,
}

impl SampleState {
    fn of(machine: &SimMachine) -> Self {
        Self {
            pb5_high: machine.pb5_high,
            pb5_millivolts: machine.pb5_millivolts,
            led_on: machine.led_on,
            led_microamps: machine.led_microamps,
        }
    }

    const fn sample(self, cycle: u64) -> Sample {
        Sample {
            cycle,
            pb5_high: self.pb5_high,
            pb5_millivolts: self.pb5_millivolts,
            led_on: self.led_on,
            led_microamps: self.led_microamps,
        }
    }
}

struct WorkerState {
    machine: Option<SimMachine>,
    running: bool,
    last: SampleState,
    sample_every: u64,
}

impl WorkerState {
    fn new() -> Self {
        Self {
            machine: None,
            running: false,
            last: SampleState::default(),
            sample_every: DEFAULT_SAMPLE_EVERY,
        }
    }

    fn load(&mut self, words: Vec<u16>, events: &Sender<Event>) {
        load_machine(
            words,
            &mut self.machine,
            &mut self.running,
            &mut self.last,
            events,
        );
    }

    fn run(&mut self, value: bool, events: &Sender<Event>) {
        update_running(&self.machine, &mut self.running, value, events);
    }

    fn step(&mut self, events: &Sender<Event>) {
        step_once(&mut self.machine, &mut self.running, &mut self.last, events);
    }

    fn reset(&mut self, events: &Sender<Event>) {
        reset_machine(&mut self.machine, &mut self.running, &mut self.last, events);
    }

    fn queue_uart(&mut self, data: Vec<u8>) {
        queue_uart(&mut self.machine, data);
    }
}

/// Current simulation status.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Status {
    pub cycles: u64,
    pub pb5_high: bool,
    pub pb5_millivolts: i32,
    pub led_on: bool,
    pub led_microamps: i32,
    pub running: bool,
    pub loaded: bool,
}

/// Bounded transition history for the PB5 waveform.
#[derive(Default)]
pub struct Waveform {
    samples: VecDeque<Sample>,
}

impl Waveform {
    /// Removes all samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Appends one sample, dropping the oldest beyond the limit.
    pub fn push(&mut self, sample: Sample) {
        if self.samples.len() == WAVE_LIMIT {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    /// Returns the sampled transitions.
    #[must_use]
    pub fn samples(&self) -> &VecDeque<Sample> {
        &self.samples
    }

    /// Returns the visible cycle range, extended to the current cycle.
    #[must_use]
    pub fn span(&self, current: u64) -> (u64, u64) {
        let start = self.samples.front().map_or(0, |sample| sample.cycle);
        let last = self.samples.back().map_or(0, |sample| sample.cycle);
        let end = last.max(current);
        if end <= start {
            return (start, start + 1);
        }
        (start, end)
    }

    /// Returns PB5 and LED states at a cycle.
    #[must_use]
    pub fn state_at(&self, cycle: u64) -> (bool, bool) {
        let mut state = (false, false);
        for sample in &self.samples {
            if sample.cycle > cycle {
                break;
            }
            state = (sample.pb5_high, sample.led_on);
        }
        state
    }
}

/// Worker command.
pub enum Command {
    Load(Vec<u16>),
    Run,
    Pause,
    Step,
    Reset,
    Uart(Vec<u8>),
    SampleEvery(u64),
    Shutdown,
}

/// Worker event.
pub enum Event {
    Status(Status),
    Sample(Sample),
    Uart(Vec<u8>),
    UartCleared,
    Error(String),
}

/// Converts Intel HEX text into AVR instruction words.
pub fn hex_words(text: &str) -> Result<Vec<u16>, String> {
    let mut image = HexImage::new(FLASH_BYTES);
    image.load(text)?;
    Ok(image.words())
}

/// Reads and converts an Intel HEX file.
pub fn load_hex(path: &Path) -> Result<Vec<u16>, String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("read failed: {error}"))?;
    hex_words(&text)
}

/// Handle to the simulation worker.
pub struct SimHandle {
    commands: Sender<Command>,
    events: Receiver<Event>,
    join: Option<JoinHandle<()>>,
}

impl SimHandle {
    /// Starts the simulation worker.
    #[must_use]
    pub fn spawn() -> Self {
        let (commands, command_rx) = mpsc::channel();
        let (event_tx, events) = mpsc::channel();
        let join = thread::spawn(move || worker(command_rx, event_tx));
        Self {
            commands,
            events,
            join: Some(join),
        }
    }

    /// Loads a program and pauses the worker.
    pub fn load(&self, words: Vec<u16>) -> Result<(), String> {
        self.send(Command::Load(words))
    }

    /// Starts continuous execution.
    pub fn run(&self) -> Result<(), String> {
        self.send(Command::Run)
    }

    /// Pauses continuous execution.
    pub fn pause(&self) -> Result<(), String> {
        self.send(Command::Pause)
    }

    /// Executes one instruction.
    pub fn step(&self) -> Result<(), String> {
        self.send(Command::Step)
    }

    /// Resets the current program.
    pub fn reset(&self) -> Result<(), String> {
        self.send(Command::Reset)
    }

    /// Queues bytes for the simulated UART receiver.
    pub fn send_uart(&self, data: Vec<u8>) -> Result<(), String> {
        self.send(Command::Uart(data))
    }

    /// Sets the probe sampling interval in CPU cycles.
    pub fn set_sample_every(&self, every: u64) -> Result<(), String> {
        self.send(Command::SampleEvery(every))
    }

    /// Drains pending events.
    #[must_use]
    pub fn poll(&self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            events.push(event);
            if events.len() >= 4096 {
                break;
            }
        }
        events
    }

    #[cfg(test)]
    fn wait_status(&self) -> Option<Status> {
        loop {
            match self.events.recv_timeout(Duration::from_secs(1)) {
                Ok(Event::Status(status)) => return Some(status),
                Ok(_) => {}
                Err(_) => return None,
            }
        }
    }

    #[cfg(test)]
    fn wait_until(&self, predicate: impl Fn(&Status) -> bool) -> Option<Status> {
        loop {
            let status = self.wait_status()?;
            if predicate(&status) {
                return Some(status);
            }
        }
    }

    fn send(&self, command: Command) -> Result<(), String> {
        self.commands
            .send(command)
            .map_err(|_| "simulation worker stopped".to_string())
    }
}

impl Drop for SimHandle {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

struct SimMachine {
    cpu: Cpu,
    board: Board,
    config: PinElectricalConfig,
    led_on: bool,
    pb5_high: bool,
    pb5_millivolts: i32,
    led_microamps: i32,
    uart_bytes: Vec<u8>,
    uart_rx: VecDeque<u8>,
}

impl SimMachine {
    fn new(words: Vec<u16>) -> Result<Self, String> {
        let cpu = Cpu::new(words).map_err(|error| error.to_string())?;
        Ok(Self {
            cpu,
            board: Board::new(),
            config: PinElectricalConfig::default(),
            led_on: false,
            pb5_high: false,
            pb5_millivolts: 0,
            led_microamps: 0,
            uart_bytes: Vec::new(),
            uart_rx: VecDeque::new(),
        })
    }

    fn step(&mut self) -> Result<bool, String> {
        self.pump_uart_rx();
        let trace = self.cpu.step().map_err(|error| error.to_string())?;
        if trace.is_none() {
            return Ok(false);
        }
        self.update_board()?;
        self.update_uart();
        Ok(true)
    }

    fn update_board(&mut self) -> Result<(), String> {
        let state = self
            .cpu
            .pin_electrical(0, 5, ExternalPin::HighZ, &self.config)
            .map_err(|error| error.to_string())?;
        self.board.set_pb5_voltage(state.voltage);
        self.led_on = self.board.led().is_on();
        self.pb5_high = state.voltage >= self.config.input_threshold;
        self.pb5_millivolts = (state.voltage * 1000.0).round() as i32;
        self.led_microamps = (self.board.led().current() * 1_000_000.0).round() as i32;
        Ok(())
    }

    fn queue_uart(&mut self, data: &[u8]) {
        let space = 4096usize.saturating_sub(self.uart_rx.len());
        self.uart_rx.extend(data.iter().take(space).copied());
    }

    fn pump_uart_rx(&mut self) {
        if self.cpu.uart().rx_ready() {
            return;
        }
        let Some(byte) = self.uart_rx.pop_front() else {
            return;
        };
        self.cpu.uart().receive(byte);
    }

    fn update_uart(&mut self) {
        while let Some(byte) = self.cpu.uart().take_tx_byte() {
            self.uart_bytes.push(byte);
        }
    }

    fn take_uart(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.uart_bytes)
    }

    fn reset(&mut self) -> Result<(), String> {
        self.cpu.reset().map_err(|error| error.to_string())?;
        self.board.set_pb5(false);
        self.led_on = false;
        self.pb5_high = false;
        self.pb5_millivolts = 0;
        self.led_microamps = 0;
        self.uart_bytes.clear();
        self.uart_rx.clear();
        Ok(())
    }

    fn cycles(&self) -> u64 {
        self.cpu.cycles()
    }
}

fn worker(commands: Receiver<Command>, events: Sender<Event>) {
    let mut state = WorkerState::new();
    loop {
        if state.running {
            step_chunk(&mut state, &events);
            if !drain_commands(&commands, &mut state, &events) {
                break;
            }
            continue;
        }
        if !wait_command(&commands, &mut state, &events) {
            break;
        }
    }
}

fn wait_command(
    commands: &Receiver<Command>,
    state: &mut WorkerState,
    events: &Sender<Event>,
) -> bool {
    match commands.recv_timeout(Duration::from_millis(POLL_MS)) {
        Ok(Command::Shutdown) => false,
        Ok(command) => {
            handle_command(command, state, events);
            true
        }
        Err(RecvTimeoutError::Timeout) => true,
        Err(RecvTimeoutError::Disconnected) => false,
    }
}

fn drain_commands(
    commands: &Receiver<Command>,
    state: &mut WorkerState,
    events: &Sender<Event>,
) -> bool {
    loop {
        match commands.try_recv() {
            Ok(Command::Shutdown) => return false,
            Ok(command) => handle_command(command, state, events),
            Err(TryRecvError::Empty) => return true,
            Err(TryRecvError::Disconnected) => return false,
        }
    }
}

fn handle_command(command: Command, state: &mut WorkerState, events: &Sender<Event>) {
    match command {
        Command::Load(words) => state.load(words, events),
        Command::Run => state.run(true, events),
        Command::Pause => state.run(false, events),
        Command::Step => state.step(events),
        Command::Reset => state.reset(events),
        Command::Uart(data) => state.queue_uart(data),
        Command::SampleEvery(every) => state.sample_every = every.max(1),
        Command::Shutdown => {}
    }
}

fn update_running(
    machine: &Option<SimMachine>,
    running: &mut bool,
    value: bool,
    events: &Sender<Event>,
) {
    *running = value && machine.is_some();
    send_status(machine, *running, events);
}

fn queue_uart(machine: &mut Option<SimMachine>, data: Vec<u8>) {
    if let Some(machine) = machine {
        machine.queue_uart(&data);
    }
}

fn load_machine(
    words: Vec<u16>,
    machine: &mut Option<SimMachine>,
    running: &mut bool,
    last: &mut SampleState,
    events: &Sender<Event>,
) {
    *running = false;
    match SimMachine::new(words) {
        Ok(next) => {
            *last = SampleState::of(&next);
            *machine = Some(next);
            let _ = events.send(Event::UartCleared);
            send_sample(0, *last, events);
        }
        Err(error) => {
            *machine = None;
            let _ = events.send(Event::Error(error));
        }
    }
    send_status(machine, false, events);
}

fn step_once(
    machine: &mut Option<SimMachine>,
    running: &mut bool,
    last: &mut SampleState,
    events: &Sender<Event>,
) {
    *running = false;
    if let Some(machine) = machine {
        match machine.step() {
            Ok(true) => {
                let state = SampleState::of(machine);
                if state != *last {
                    send_sample(machine.cycles(), state, events);
                    *last = state;
                }
            }
            Ok(false) => {}
            Err(error) => {
                let _ = events.send(Event::Error(error));
            }
        }
        send_uart(machine, events);
    }
    send_status(machine, false, events);
}

fn reset_machine(
    machine: &mut Option<SimMachine>,
    running: &mut bool,
    last: &mut SampleState,
    events: &Sender<Event>,
) {
    *running = false;
    if let Some(machine) = machine {
        match machine.reset() {
            Ok(()) => {
                *last = SampleState::default();
                let _ = events.send(Event::UartCleared);
                send_sample(0, *last, events);
            }
            Err(error) => {
                let _ = events.send(Event::Error(error));
            }
        }
    }
    send_status(machine, false, events);
}

fn step_chunk(state: &mut WorkerState, events: &Sender<Event>) {
    let Some(current) = state.machine.as_mut() else {
        state.running = false;
        return;
    };
    advance(
        current,
        &mut state.running,
        &mut state.last,
        state.sample_every,
        events,
    );
    send_uart(current, events);
    send_status(&state.machine, state.running, events);
}

fn advance(
    machine: &mut SimMachine,
    running: &mut bool,
    last: &mut SampleState,
    sample_every: u64,
    events: &Sender<Event>,
) {
    for _ in 0..CHUNK {
        if !step_one(machine, running, events) {
            break;
        }
        sample_step(machine, last, sample_every, events);
    }
}

fn step_one(machine: &mut SimMachine, running: &mut bool, events: &Sender<Event>) -> bool {
    match machine.step() {
        Ok(true) => true,
        Ok(false) => {
            *running = false;
            false
        }
        Err(error) => {
            let _ = events.send(Event::Error(error));
            *running = false;
            false
        }
    }
}

fn sample_step(
    machine: &SimMachine,
    last: &mut SampleState,
    sample_every: u64,
    events: &Sender<Event>,
) {
    let state = SampleState::of(machine);
    let cycle = machine.cycles();
    let every = sample_every.max(1);
    if state == *last && cycle % every != 0 {
        return;
    }
    *last = state;
    send_sample(cycle, state, events);
}

fn send_uart(machine: &mut SimMachine, events: &Sender<Event>) {
    let bytes = machine.take_uart();
    if !bytes.is_empty() {
        let _ = events.send(Event::Uart(bytes));
    }
}

fn send_status(machine: &Option<SimMachine>, running: bool, events: &Sender<Event>) {
    let status = match machine {
        Some(machine) => Status {
            cycles: machine.cycles(),
            pb5_high: machine.pb5_high,
            pb5_millivolts: machine.pb5_millivolts,
            led_on: machine.led_on,
            led_microamps: machine.led_microamps,
            running,
            loaded: true,
        },
        None => Status::default(),
    };
    let _ = events.send(Event::Status(status));
}

fn send_sample(cycle: u64, state: SampleState, events: &Sender<Event>) {
    let _ = events.send(Event::Sample(state.sample(cycle)));
}

#[cfg(test)]
mod tests {
    use super::{Event, Sample, SampleState, SimHandle, SimMachine, Waveform, hex_words};

    fn blink_program() -> Vec<u16> {
        vec![
            0xE200, 0xB904, 0xB905, 0xEF1F, 0x951A, 0xF7F1, 0xE220, 0x2702, 0xCFF9,
        ]
    }

    #[test]
    fn uart_tx_bytes_are_collected() {
        let mut machine = SimMachine::new(blink_program()).expect("machine");
        machine.cpu.uart().write_control(1, 0x08);
        machine.cpu.uart().write_tx(b'A');
        machine.cpu.uart().tick(200);
        machine.update_uart();
        assert_eq!(machine.take_uart(), vec![b'A']);
        assert!(machine.take_uart().is_empty());
    }

    #[test]
    fn uart_rx_bytes_are_delivered() {
        let mut machine = SimMachine::new(blink_program()).expect("machine");
        machine.queue_uart(b"Hi");
        machine.pump_uart_rx();
        assert!(machine.cpu.uart().rx_ready());
        assert_eq!(machine.cpu.uart().read_rx(), b'H');
        machine.pump_uart_rx();
        assert_eq!(machine.cpu.uart().read_rx(), b'i');
    }

    #[test]
    fn worker_loads_program_and_reports_status() {
        let handle = SimHandle::spawn();
        handle.load(blink_program()).expect("load");
        let status = handle.wait_status().expect("status");
        assert!(status.loaded);
        assert!(!status.running);
        assert_eq!(status.cycles, 0);
    }

    #[test]
    fn worker_runs_and_pauses() {
        let handle = SimHandle::spawn();
        handle.load(blink_program()).expect("load");
        handle.wait_status().expect("load status");
        handle.run().expect("run");
        let running = handle
            .wait_until(|status| status.running && status.cycles > 0)
            .expect("running status");
        assert!(running.cycles > 0);
        handle.pause().expect("pause");
        let paused = handle
            .wait_until(|status| !status.running)
            .expect("paused status");
        assert!(!paused.running);
    }

    #[test]
    fn blink_program_toggles_board_led() {
        let mut machine = SimMachine::new(blink_program()).expect("machine");
        let mut seen = false;
        let mut high_voltage = false;
        for _ in 0..5000 {
            if !machine.step().expect("step") {
                break;
            }
            seen |= machine.led_on;
            high_voltage |= machine.led_on && machine.pb5_millivolts > 0;
        }
        assert!(seen);
        assert!(high_voltage);
    }

    #[test]
    fn sampling_respects_interval() {
        let mut machine = SimMachine::new(blink_program()).expect("machine");
        let mut last = SampleState::default();
        let (events, received) = std::sync::mpsc::channel();
        for _ in 0..10 {
            machine.step().expect("step");
            super::sample_step(&machine, &mut last, 10, &events);
        }
        let cycles: Vec<_> = received
            .try_iter()
            .filter_map(|event| match event {
                Event::Sample(sample) => Some(sample.cycle),
                _ => None,
            })
            .collect();
        assert!(cycles.contains(&10));
    }

    #[test]
    fn reset_clears_cycles_and_led() {
        let mut machine = SimMachine::new(blink_program()).expect("machine");
        for _ in 0..200 {
            machine.step().expect("step");
        }
        machine.reset().expect("reset");
        assert_eq!(machine.cycles(), 0);
        assert!(!machine.led_on);
    }

    #[test]
    fn waveform_span_extends_to_current_cycle() {
        let mut waveform = Waveform::default();
        waveform.push(sample(10, true));
        waveform.push(sample(40, false));
        assert_eq!(waveform.span(100), (10, 100));
        assert_eq!(waveform.span(20), (10, 40));
        assert_eq!(waveform.state_at(0), (false, false));
        assert_eq!(waveform.state_at(10), (true, true));
        assert_eq!(waveform.state_at(40), (false, false));
    }

    fn sample(cycle: u64, high: bool) -> Sample {
        Sample {
            cycle,
            pb5_high: high,
            pb5_millivolts: if high { 5000 } else { 0 },
            led_on: high,
            led_microamps: if high { 14_000 } else { 0 },
        }
    }

    #[test]
    fn waveform_is_bounded() {
        let mut waveform = Waveform::default();
        for cycle in 0..super::WAVE_LIMIT as u64 + 5 {
            waveform.push(Sample {
                cycle,
                pb5_high: cycle % 2 == 0,
                pb5_millivolts: if cycle % 2 == 0 { 5000 } else { 0 },
                led_on: cycle % 2 == 0,
                led_microamps: if cycle % 2 == 0 { 14_000 } else { 0 },
            });
        }
        assert_eq!(waveform.samples().len(), super::WAVE_LIMIT);
        assert_eq!(waveform.span(9999), (5, 9999));
    }

    #[test]
    fn hex_text_becomes_words() {
        let text = sim_board::to_intel_hex(&[0x01, 0x02, 0x03, 0x04]);
        let words = hex_words(&text).expect("hex");
        assert_eq!(&words[..2], &[0x0201, 0x0403]);
        assert_eq!(words.len(), super::FLASH_BYTES / 2);
    }
}
