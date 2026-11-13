//! Intel HEX loader and writer.

/// Formats bytes as Intel HEX records up to 64 KiB.
#[must_use]
pub fn to_intel_hex(bytes: &[u8]) -> String {
    let mut text = String::new();
    for (index, chunk) in bytes.chunks(16).enumerate() {
        text.push_str(&format_record((index * 16) as u16, chunk));
    }
    text.push_str(":00000001FF\n");
    text
}

fn format_record(address: u16, data: &[u8]) -> String {
    let mut record = Vec::with_capacity(data.len() + 5);
    record.push(data.len() as u8);
    record.push((address >> 8) as u8);
    record.push(address as u8);
    record.push(0);
    record.extend_from_slice(data);
    let sum = record
        .iter()
        .fold(0u8, |acc, value| acc.wrapping_add(*value));
    record.push(0u8.wrapping_sub(sum));
    let hex: String = record.iter().map(|byte| format!("{byte:02X}")).collect();
    format!(":{hex}\n")
}

/// A loaded Intel HEX image.
pub struct HexImage {
    bytes: Vec<u8>,
}

impl HexImage {
    /// Creates an erased image.
    #[must_use]
    pub fn new(size: usize) -> Self {
        Self {
            bytes: vec![0xFF; size],
        }
    }

    /// Returns the image bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns little-endian instruction words.
    #[must_use]
    pub fn words(&self) -> Vec<u16> {
        self.bytes
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect()
    }

    /// Loads Intel HEX text.
    pub fn load(&mut self, text: &str) -> Result<usize, String> {
        let mut base = 0u32;
        let mut written = 0;
        for (line_number, line) in text.lines().enumerate() {
            let record = parse_record(line, line_number + 1)?;
            if record.kind == 1 {
                return Ok(written);
            }
            if record.kind == 4 {
                base = extended_address(&record)?;
                continue;
            }
            if record.kind != 0 {
                continue;
            }
            written += self.store(base, &record)?;
        }
        Ok(written)
    }

    fn store(&mut self, base: u32, record: &Record) -> Result<usize, String> {
        let start = base + u32::from(record.address);
        for (offset, value) in record.data.iter().enumerate() {
            let address = start as usize + offset;
            if let Some(slot) = self.bytes.get_mut(address) {
                *slot = *value;
            }
        }
        Ok(record.data.len())
    }
}

struct Record {
    address: u16,
    kind: u8,
    data: Vec<u8>,
}

fn parse_record(line: &str, line_number: usize) -> Result<Record, String> {
    let text = line.trim();
    if !text.starts_with(':') {
        return Err(format!("line {line_number}: missing colon"));
    }
    let bytes = decode_hex(&text[1..], line_number)?;
    validate_record(&bytes, line_number)?;
    Ok(build_record(&bytes))
}

fn validate_record(bytes: &[u8], line_number: usize) -> Result<(), String> {
    if bytes.len() < 5 {
        return Err(format!("line {line_number}: record too short"));
    }
    let length = bytes[0] as usize;
    if bytes.len() != length + 5 {
        return Err(format!("line {line_number}: length mismatch"));
    }
    if checksum(bytes) != 0 {
        return Err(format!("line {line_number}: checksum mismatch"));
    }
    Ok(())
}

fn build_record(bytes: &[u8]) -> Record {
    let length = bytes[0] as usize;
    Record {
        address: u16::from(bytes[1]) << 8 | u16::from(bytes[2]),
        kind: bytes[3],
        data: bytes[4..4 + length].to_vec(),
    }
}

fn decode_hex(text: &str, line_number: usize) -> Result<Vec<u8>, String> {
    if text.len() % 2 != 0 {
        return Err(format!("line {line_number}: odd hex length"));
    }
    (0..text.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&text[index..index + 2], 16)
                .map_err(|_| format!("line {line_number}: invalid hex"))
        })
        .collect()
}

fn checksum(bytes: &[u8]) -> u8 {
    bytes
        .iter()
        .fold(0u8, |sum, value| sum.wrapping_add(*value))
}

fn extended_address(record: &Record) -> Result<u32, String> {
    if record.data.len() != 2 {
        return Err("extended address must have two bytes".to_string());
    }
    Ok((u32::from(record.data[0]) << 8 | u32::from(record.data[1])) << 16)
}

#[cfg(test)]
mod tests {
    use super::{HexImage, to_intel_hex};

    #[test]
    fn writes_intel_hex() {
        assert_eq!(
            to_intel_hex(&[1, 2, 3, 4]),
            ":0400000001020304F2\n:00000001FF\n"
        );
    }

    #[test]
    fn loads_data_record() {
        let mut image = HexImage::new(32);
        let count = image
            .load(":0400000001020304F2\n:00000001FF\n")
            .expect("hex");
        assert_eq!(count, 4);
        assert_eq!(&image.bytes()[..4], &[1, 2, 3, 4]);
    }

    #[test]
    fn converts_bytes_to_little_endian_words() {
        let mut image = HexImage::new(4);
        image
            .load(":0400000001020304F2\n:00000001FF\n")
            .expect("hex");
        assert_eq!(image.words(), vec![0x0201, 0x0403]);
    }

    #[test]
    fn rejects_bad_checksum() {
        let mut image = HexImage::new(32);
        assert!(image.load(":0400000001020304F3\n").is_err());
    }

    #[test]
    fn rejects_malformed_records() {
        let mut image = HexImage::new(16);
        assert!(image.load("00000001FF\n").is_err());
        assert!(image.load(":01000000AA\n").is_err());
        assert!(image.load(":ZZ\n").is_err());
        assert!(image.load(":0100000401FA\n").is_err());
    }

    #[test]
    fn supports_extended_address() {
        let mut image = HexImage::new(0x20000);
        let count = image
            .load(":020000040001F9\n:01000000AA55\n:00000001FF\n")
            .expect("hex");
        assert_eq!(count, 1);
        assert_eq!(image.bytes()[0x10000], 0xAA);
    }
}
