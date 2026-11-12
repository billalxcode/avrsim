//! STK500/Optiboot bootloader protocol for the virtual serial port.

const FLASH_SIZE: usize = 32 * 1024;
const SIGNATURE: [u8; 3] = [0x1E, 0x95, 0x0F];

const CMD_GET_SYNC: u8 = 0x30;
const CMD_GET_PARAMETER: u8 = 0x41;
const CMD_SET_DEVICE: u8 = 0x42;
const CMD_SET_DEVICE_EXT: u8 = 0x45;
const CMD_ENTER_PROGMODE: u8 = 0x50;
const CMD_LEAVE_PROGMODE: u8 = 0x51;
const CMD_CHIP_ERASE: u8 = 0x52;
const CMD_CHECK_AUTOINC: u8 = 0x53;
const CMD_LOAD_ADDRESS: u8 = 0x55;
const CMD_PROG_PAGE: u8 = 0x64;
const CMD_READ_OSCCAL: u8 = 0x66;
const CMD_READ_PAGE: u8 = 0x74;
const CMD_READ_SIGN: u8 = 0x75;

const RESP_INSYNC: u8 = 0x14;
const RESP_OK: u8 = 0x10;
const RESP_NOSYNC: u8 = 0x00;

/// One STK500 frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Command byte.
    pub command: u8,
    /// Sequence byte.
    pub sequence: u8,
    /// Command payload.
    pub data: Vec<u8>,
}

#[derive(Clone, Copy)]
struct Header {
    command: u8,
    sequence: u8,
    complement: u8,
}

/// Frame parsing error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameError {
    /// Checksum or sequence complement is invalid.
    InvalidChecksum,
}

/// Parses one frame from a byte buffer.
pub fn parse_frame(buffer: &mut Vec<u8>) -> Result<Option<Frame>, FrameError> {
    resync(buffer);
    if buffer.len() < 4 {
        return Ok(None);
    }
    let header = parse_header(buffer)?;
    let Some(total) = frame_length(header.command, &buffer[4..]) else {
        return Ok(None);
    };
    if buffer.len() < total {
        return Ok(None);
    }
    finish_frame(buffer, header, total)
}

fn parse_header(buffer: &mut Vec<u8>) -> Result<Header, FrameError> {
    let header = Header {
        command: buffer[1],
        sequence: buffer[2],
        complement: buffer[3],
    };
    if header.sequence ^ header.complement != 0xFF {
        buffer.remove(0);
        return Err(FrameError::InvalidChecksum);
    }
    Ok(header)
}

fn frame_length(command: u8, available: &[u8]) -> Option<usize> {
    let payload = payload_length(command, available)?;
    Some(4 + payload + 1)
}

fn finish_frame(
    buffer: &mut Vec<u8>,
    header: Header,
    total: usize,
) -> Result<Option<Frame>, FrameError> {
    let data = buffer[4..total - 1].to_vec();
    let checksum = buffer[total - 1];
    buffer.drain(..total);
    if checksum != frame_checksum(header.command, header.sequence, header.complement, &data) {
        return Err(FrameError::InvalidChecksum);
    }
    Ok(Some(Frame {
        command: header.command,
        sequence: header.sequence,
        data,
    }))
}

fn resync(buffer: &mut Vec<u8>) {
    match buffer.iter().position(|byte| *byte == 0x1B) {
        Some(0) => {}
        Some(index) => {
            buffer.drain(..index);
        }
        None => buffer.clear(),
    }
}

fn payload_length(command: u8, available: &[u8]) -> Option<usize> {
    match command {
        CMD_GET_PARAMETER => Some(1),
        CMD_SET_DEVICE => Some(20),
        CMD_SET_DEVICE_EXT => Some(5),
        CMD_LOAD_ADDRESS => Some(2),
        CMD_PROG_PAGE => variable_length(available),
        CMD_READ_PAGE => Some(3),
        _ => Some(0),
    }
}

fn variable_length(available: &[u8]) -> Option<usize> {
    if available.len() < 3 {
        return None;
    }
    let length = usize::from(u16::from_be_bytes([available[0], available[1]]));
    Some(3 + length)
}

fn frame_checksum(command: u8, sequence: u8, complement: u8, data: &[u8]) -> u8 {
    data.iter()
        .fold(command ^ sequence ^ complement, |value, byte| value ^ byte)
}

/// A bootloader that writes an ATmega328P flash image.
pub struct Bootloader {
    flash: Vec<u8>,
    address: usize,
    running: bool,
}

impl Bootloader {
    /// Creates a bootloader with erased flash.
    #[must_use]
    pub fn new() -> Self {
        Self {
            flash: vec![0xFF; FLASH_SIZE],
            address: 0,
            running: false,
        }
    }

    /// Returns true after the uploader leaves programming mode.
    #[must_use]
    pub fn running(&self) -> bool {
        self.running
    }

    /// Handles one frame and returns the response bytes.
    pub fn handle(&mut self, frame: &Frame) -> Vec<u8> {
        if let Some(response) = fixed_response(frame) {
            return response;
        }
        match frame.command {
            CMD_GET_SYNC => self.enter_programming(),
            CMD_LEAVE_PROGMODE => self.leave_programming(),
            CMD_CHIP_ERASE => self.erase(),
            CMD_LOAD_ADDRESS => self.load_address(&frame.data),
            CMD_PROG_PAGE => self.program_page(&frame.data),
            CMD_READ_PAGE => self.read_page(&frame.data),
            _ => error(),
        }
    }

    fn enter_programming(&mut self) -> Vec<u8> {
        self.running = false;
        ok(&[])
    }

    fn leave_programming(&mut self) -> Vec<u8> {
        self.running = true;
        ok(&[])
    }

    /// Returns the uploaded flash image as instruction words.
    #[must_use]
    pub fn program_words(&self) -> Vec<u16> {
        let end = self.used_end();
        let mut words: Vec<u16> = self.flash[..end]
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        if words.is_empty() {
            words.push(0x0000);
        }
        words
    }

    fn used_end(&self) -> usize {
        let mut end = self.flash.len();
        while end >= 2 && self.flash[end - 2] == 0xFF && self.flash[end - 1] == 0xFF {
            end -= 2;
        }
        end
    }

    fn erase(&mut self) -> Vec<u8> {
        self.flash.fill(0xFF);
        self.address = 0;
        ok(&[])
    }

    fn load_address(&mut self, data: &[u8]) -> Vec<u8> {
        if data.len() < 2 {
            return error();
        }
        self.address = usize::from(u16::from_le_bytes([data[0], data[1]]));
        ok(&[])
    }

    fn program_page(&mut self, data: &[u8]) -> Vec<u8> {
        let Some((length, memory)) = page_header(data) else {
            return error();
        };
        if data.len() < 3 + length {
            return error();
        }
        if memory == b'F' {
            self.write_flash(&data[3..3 + length]);
        }
        ok(&[])
    }

    fn write_flash(&mut self, data: &[u8]) {
        let start = self.address.saturating_mul(2);
        let end = start.saturating_add(data.len()).min(self.flash.len());
        if start >= end {
            return;
        }
        let available = &data[..end - start];
        self.flash[start..end].copy_from_slice(available);
        self.address = self.address.saturating_add(data.len() / 2);
    }

    fn read_page(&self, data: &[u8]) -> Vec<u8> {
        let Some((length, memory)) = page_header(data) else {
            return error();
        };
        if memory != b'F' {
            return ok(&[]);
        }
        let start = self.address.saturating_mul(2);
        let end = start.saturating_add(length).min(self.flash.len());
        if start >= end {
            return error();
        }
        ok(&self.flash[start..end])
    }
}

fn page_header(data: &[u8]) -> Option<(usize, u8)> {
    if data.len() < 3 {
        return None;
    }
    let length = usize::from(u16::from_be_bytes([data[0], data[1]]));
    Some((length, data[2]))
}

fn fixed_response(frame: &Frame) -> Option<Vec<u8>> {
    match frame.command {
        CMD_GET_PARAMETER => Some(ok(&[parameter(frame.data.first().copied().unwrap_or(0))])),
        CMD_SET_DEVICE | CMD_SET_DEVICE_EXT | CMD_ENTER_PROGMODE | CMD_CHECK_AUTOINC => {
            Some(ok(&[]))
        }
        CMD_READ_SIGN => Some(ok(&SIGNATURE)),
        CMD_READ_OSCCAL => Some(ok(&[0x00])),
        _ => None,
    }
}

fn parameter(code: u8) -> u8 {
    match code {
        0x80 => 0x03,
        0x81 => 0x04,
        0x82 => 0x04,
        0x93 => 0x03,
        _ => 0x00,
    }
}

fn ok(data: &[u8]) -> Vec<u8> {
    let mut response = Vec::with_capacity(data.len() + 2);
    response.push(RESP_INSYNC);
    response.extend_from_slice(data);
    response.push(RESP_OK);
    response
}

fn error() -> Vec<u8> {
    vec![RESP_INSYNC, RESP_NOSYNC]
}

#[cfg(test)]
mod tests {
    use super::{Bootloader, Frame, FrameError, parse_frame};

    fn frame(command: u8, sequence: u8, data: &[u8]) -> Vec<u8> {
        let complement = !sequence;
        let mut bytes = vec![0x1B, command, sequence, complement];
        bytes.extend_from_slice(data);
        let checksum = data
            .iter()
            .fold(command ^ sequence ^ complement, |value, byte| value ^ byte);
        bytes.push(checksum);
        bytes
    }

    fn parse(bytes: &[u8]) -> Frame {
        let mut buffer = bytes.to_vec();
        parse_frame(&mut buffer).expect("parse").expect("frame")
    }

    #[test]
    fn parses_get_sync() {
        let frame = parse(&frame(0x30, 1, &[]));
        assert_eq!(frame.command, 0x30);
        assert_eq!(frame.sequence, 1);
        assert!(frame.data.is_empty());
    }

    #[test]
    fn rejects_bad_checksum() {
        let mut bytes = frame(0x30, 1, &[]);
        *bytes.last_mut().expect("checksum") ^= 0xFF;
        let mut buffer = bytes;
        assert_eq!(parse_frame(&mut buffer), Err(FrameError::InvalidChecksum));
    }

    #[test]
    fn handles_read_signature() {
        let mut bootloader = Bootloader::new();
        let response = bootloader.handle(&parse(&frame(0x75, 1, &[])));
        assert_eq!(response, vec![0x14, 0x1E, 0x95, 0x0F, 0x10]);
    }

    #[test]
    fn programs_flash_page() {
        let mut bootloader = Bootloader::new();
        let address = frame(0x55, 1, &[0x00, 0x00]);
        bootloader.handle(&parse(&address));
        let page = frame(0x64, 2, &[0x00, 0x04, b'F', 0x05, 0xE0, 0x00, 0x00]);
        bootloader.handle(&parse(&page));
        assert_eq!(bootloader.program_words(), vec![0xE005, 0x0000]);
    }

    #[test]
    fn reads_flash_page() {
        let mut bootloader = Bootloader::new();
        bootloader.handle(&parse(&frame(0x55, 1, &[0x00, 0x00])));
        let page = frame(0x64, 2, &[0x00, 0x02, b'F', 0x34, 0x12]);
        bootloader.handle(&parse(&page));
        bootloader.handle(&parse(&frame(0x55, 3, &[0x00, 0x00])));
        let read = frame(0x74, 4, &[0x00, 0x02, b'F']);
        assert_eq!(
            bootloader.handle(&parse(&read)),
            vec![0x14, 0x34, 0x12, 0x10]
        );
    }

    #[test]
    fn chip_erase_clears_flash() {
        let mut bootloader = Bootloader::new();
        bootloader.handle(&parse(&frame(0x55, 1, &[0x00, 0x00])));
        bootloader.handle(&parse(&frame(0x64, 2, &[0x00, 0x02, b'F', 0x34, 0x12])));
        bootloader.handle(&parse(&frame(0x52, 3, &[])));
        assert_eq!(bootloader.program_words(), vec![0x0000]);
    }

    #[test]
    fn leave_program_mode_marks_running() {
        let mut bootloader = Bootloader::new();
        bootloader.handle(&parse(&frame(0x51, 1, &[])));
        assert!(bootloader.running());
        bootloader.handle(&parse(&frame(0x30, 2, &[])));
        assert!(!bootloader.running());
    }

    #[test]
    fn trims_trailing_erased_words() {
        let mut bootloader = Bootloader::new();
        bootloader.handle(&parse(&frame(0x55, 1, &[0x00, 0x00])));
        bootloader.handle(&parse(&frame(0x64, 2, &[0x00, 0x02, b'F', 0x34, 0x12])));
        assert_eq!(bootloader.program_words(), vec![0x1234]);
    }

    #[test]
    fn round_trips_blink_program() {
        let program: [u16; 9] = [
            0xE200, 0xB904, 0xB905, 0xEF1F, 0x951A, 0xF7F1, 0xE220, 0x2702, 0xCFF9,
        ];
        let mut payload = vec![0x00, 0x12, b'F'];
        for word in program {
            payload.extend_from_slice(&word.to_le_bytes());
        }
        let mut bootloader = Bootloader::new();
        bootloader.handle(&parse(&frame(0x55, 1, &[0x00, 0x00])));
        bootloader.handle(&parse(&frame(0x64, 2, &payload)));
        assert_eq!(bootloader.program_words(), program.to_vec());
    }
}
