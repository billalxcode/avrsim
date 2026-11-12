//! Virtual serial port backed by socat and the STK500 bootloader.

use crate::bootloader::{Bootloader, Frame, parse_frame};
use sim_avr::{Cpu, ExternalPin, PinElectricalConfig};
use sim_board::Board;
use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const DEFAULT_LINK: &str = "/tmp/embed-emu-tty";
const DEFAULT_SOCKET: &str = "/tmp/embed-emu-sock";

/// Runs the virtual serial port until the client closes it.
pub fn run(args: &[String]) -> Result<(), String> {
    let link = path_arg(args, "--link", DEFAULT_LINK);
    let socket = path_arg(args, "--socket", DEFAULT_SOCKET);
    let port = VirtualPort::start(&link, &socket)?;
    println!("virtual serial port: {}", link.display());
    println!("press Ctrl-C to close");
    let stream = connect(&socket)?;
    serve(stream)?;
    drop(port);
    Ok(())
}

fn path_arg(args: &[String], flag: &str, default: &str) -> PathBuf {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| PathBuf::from(&pair[1]))
        .unwrap_or_else(|| PathBuf::from(default))
}

struct VirtualPort {
    child: Child,
    link: PathBuf,
    socket: PathBuf,
}

impl VirtualPort {
    fn start(link: &Path, socket: &Path) -> Result<Self, String> {
        remove_stale(link);
        remove_stale(socket);
        let child = Command::new("socat")
            .arg(format!("PTY,raw,echo=0,ignoreeof,link={}", link.display()))
            .arg(format!("UNIX-LISTEN:{},unlink-close=1", socket.display()))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("failed to start socat: {error}"))?;
        let mut port = Self {
            child,
            link: link.to_path_buf(),
            socket: socket.to_path_buf(),
        };
        port.wait_ready()?;
        Ok(port)
    }

    fn wait_ready(&mut self) -> Result<(), String> {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if self.socket.exists() {
                return Ok(());
            }
            if let Some(status) = self.child.try_wait().map_err(|error| error.to_string())? {
                return Err(format!("socat exited with {status}"));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Err("socat did not create the virtual port".to_string())
    }
}

impl Drop for VirtualPort {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.link);
        let _ = std::fs::remove_file(&self.socket);
    }
}

fn remove_stale(path: &Path) {
    if path.symlink_metadata().is_ok() {
        let _ = std::fs::remove_file(path);
    }
}

fn connect(socket: &Path) -> Result<UnixStream, String> {
    let stream = UnixStream::connect(socket)
        .map_err(|error| format!("failed to connect virtual port: {error}"))?;
    stream
        .set_nonblocking(true)
        .map_err(|error| format!("failed to set nonblocking mode: {error}"))?;
    Ok(stream)
}

fn serve(stream: UnixStream) -> Result<(), String> {
    let mut session = Session::new(stream)?;
    session.run()
}

struct Session {
    stream: UnixStream,
    bootloader: Bootloader,
    cpu: Cpu,
    board: Board,
    pin_config: PinElectricalConfig,
    buffer: Vec<u8>,
    led_on: bool,
    toggles: usize,
    tx_reported: bool,
    rx_queue: VecDeque<u8>,
}

enum ReadStatus {
    Data,
    Empty,
    Closed,
}

impl Session {
    fn new(stream: UnixStream) -> Result<Self, String> {
        let cpu = Cpu::new(vec![0x0000]).map_err(|error| error.to_string())?;
        Ok(Self {
            stream,
            bootloader: Bootloader::new(),
            cpu,
            board: Board::new(),
            pin_config: PinElectricalConfig::default(),
            buffer: Vec::new(),
            led_on: false,
            toggles: 0,
            tx_reported: false,
            rx_queue: VecDeque::new(),
        })
    }

    fn run(&mut self) -> Result<(), String> {
        loop {
            match self.read()? {
                ReadStatus::Closed => return Ok(()),
                ReadStatus::Empty if !self.bootloader.running() => {
                    std::thread::sleep(Duration::from_millis(1));
                }
                ReadStatus::Empty | ReadStatus::Data => {}
            }
            self.handle_input()?;
            if self.bootloader.running() {
                self.step_cpu()?;
            }
        }
    }

    fn read(&mut self) -> Result<ReadStatus, String> {
        let mut bytes = [0u8; 1024];
        match self.stream.read(&mut bytes) {
            Ok(0) => Ok(ReadStatus::Closed),
            Ok(count) => {
                self.buffer.extend_from_slice(&bytes[..count]);
                Ok(ReadStatus::Data)
            }
            Err(error)
                if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::Interrupted) =>
            {
                Ok(ReadStatus::Empty)
            }
            Err(error) => Err(error.to_string()),
        }
    }

    fn handle_input(&mut self) -> Result<(), String> {
        if self.bootloader.running() {
            return self.handle_running_input();
        }
        self.handle_frames()
    }

    fn handle_running_input(&mut self) -> Result<(), String> {
        let Some(index) = self.buffer.iter().position(|byte| *byte == 0x1B) else {
            self.feed_uart();
            return Ok(());
        };
        if index > 0 {
            let prefix: Vec<u8> = self.buffer.drain(..index).collect();
            self.feed_bytes(&prefix);
            return Ok(());
        }
        match parse_frame(&mut self.buffer) {
            Ok(Some(frame)) => self.handle_frame(&frame),
            Ok(None) => Ok(()),
            Err(_) => {
                self.feed_uart();
                Ok(())
            }
        }
    }

    fn feed_uart(&mut self) {
        let bytes = std::mem::take(&mut self.buffer);
        self.feed_bytes(&bytes);
    }

    fn feed_bytes(&mut self, bytes: &[u8]) {
        self.rx_queue.extend(bytes.iter().copied());
    }

    fn pump_uart(&mut self) {
        if self.cpu.uart().rx_ready() {
            return;
        }
        if let Some(byte) = self.rx_queue.pop_front() {
            self.cpu.uart().receive(byte);
        }
    }

    fn handle_frames(&mut self) -> Result<(), String> {
        loop {
            match parse_frame(&mut self.buffer) {
                Ok(Some(frame)) => self.handle_frame(&frame)?,
                Ok(None) => return Ok(()),
                Err(_) => continue,
            }
        }
    }

    fn handle_frame(&mut self, frame: &Frame) -> Result<(), String> {
        let was_running = self.bootloader.running();
        let response = self.bootloader.handle(frame);
        self.flush(&response)?;
        if !was_running && self.bootloader.running() {
            self.load_program()?;
        }
        Ok(())
    }

    fn flush(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.stream
            .write_all(bytes)
            .map_err(|error| error.to_string())
    }

    fn load_program(&mut self) -> Result<(), String> {
        let words = self.bootloader.program_words();
        self.cpu
            .load_program(words)
            .map_err(|error| error.to_string())?;
        self.led_on = false;
        self.toggles = 0;
        self.rx_queue.clear();
        Ok(())
    }

    fn step_cpu(&mut self) -> Result<(), String> {
        self.pump_uart();
        for _ in 0..32 {
            if self
                .cpu
                .step()
                .map_err(|error| error.to_string())?
                .is_none()
            {
                break;
            }
            self.update_board()?;
            self.forward_uart()?;
        }
        Ok(())
    }

    fn forward_uart(&mut self) -> Result<(), String> {
        let complete = self.cpu.uart().tx_complete();
        if !complete {
            self.tx_reported = false;
            return Ok(());
        }
        if self.tx_reported {
            return Ok(());
        }
        let byte = self.cpu.uart().tx_byte();
        match self.stream.write(&[byte]) {
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::WouldBlock => {}
            Err(error) => return Err(error.to_string()),
        }
        self.tx_reported = true;
        Ok(())
    }

    fn update_board(&mut self) -> Result<(), String> {
        let state = self
            .cpu
            .pin_electrical(0, 5, ExternalPin::HighZ, &self.pin_config)
            .map_err(|error| error.to_string())?;
        self.board.set_pb5_voltage(state.voltage);
        let on = self.board.led().is_on();
        if on != self.led_on {
            self.led_on = on;
            self.toggles += 1;
            println!("led_on={on} toggles={}", self.toggles);
            let _ = std::io::stdout().flush();
        }
        Ok(())
    }
}
