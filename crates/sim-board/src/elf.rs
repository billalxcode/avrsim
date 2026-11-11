//! Minimal ELF32 loader and symbol reader.

use sim_core::SimError;

const ELF_HEADER_SIZE: usize = 52;
const PROGRAM_HEADER_SIZE: usize = 32;
const SECTION_HEADER_SIZE: usize = 40;
const SYMBOL_SIZE: usize = 16;
const MACHINE_AVR: u16 = 83;
const PT_LOAD: u32 = 1;
const SHT_SYMTAB: u32 = 2;
const AVR_DATA_BASE: u32 = 0x0080_0000;

/// A loadable ELF segment.
#[derive(Clone, Debug, PartialEq)]
pub struct ElfSegment {
    /// Virtual address.
    pub address: u32,
    /// File bytes.
    pub data: Vec<u8>,
    /// Memory size including zero fill.
    pub memory_size: u32,
}

/// Symbol kind derived from `st_info`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElfSymbolKind {
    /// Function symbol.
    Function,
    /// Object symbol.
    Object,
    /// Other symbol.
    Other,
}

/// A named ELF symbol.
#[derive(Clone, Debug, PartialEq)]
pub struct ElfSymbol {
    /// Symbol name.
    pub name: String,
    /// Symbol address.
    pub address: u32,
    /// Symbol size.
    pub size: u32,
    /// Symbol kind.
    pub kind: ElfSymbolKind,
}

/// Parsed ELF32 image.
#[derive(Clone, Debug, PartialEq)]
pub struct ElfImage {
    entry: u32,
    segments: Vec<ElfSegment>,
    symbols: Vec<ElfSymbol>,
}

#[derive(Clone, Copy)]
struct ElfHeader {
    entry: u32,
    program_offset: u32,
    section_offset: u32,
    program_count: u16,
    section_count: u16,
}

#[derive(Clone, Copy)]
struct Section {
    kind: u32,
    offset: u32,
    size: u32,
    link: u32,
    entry_size: u32,
}

impl ElfImage {
    /// Parses an ELF32 AVR image.
    pub fn parse(data: &[u8]) -> Result<Self, SimError> {
        let header = parse_header(data)?;
        let segments = parse_segments(data, header)?;
        let symbols = parse_symbols(data, header)?;
        Ok(Self {
            entry: header.entry,
            segments,
            symbols,
        })
    }

    /// Returns the entry point.
    #[must_use]
    pub const fn entry(&self) -> u32 {
        self.entry
    }

    /// Returns loadable segments.
    #[must_use]
    pub fn segments(&self) -> &[ElfSegment] {
        &self.segments
    }

    /// Returns parsed symbols.
    #[must_use]
    pub fn symbols(&self) -> &[ElfSymbol] {
        &self.symbols
    }

    /// Writes flash segments into a byte image.
    pub fn load_flash(&self, image: &mut [u8]) -> Result<(), SimError> {
        for segment in &self.segments {
            if segment.address >= AVR_DATA_BASE {
                continue;
            }
            let start = segment.address as usize;
            let end = start
                .checked_add(segment.data.len())
                .ok_or_else(|| SimError::Parse("ELF segment overflow".to_string()))?;
            if end > image.len() {
                return Err(SimError::Parse("ELF segment exceeds image".to_string()));
            }
            image[start..end].copy_from_slice(&segment.data);
        }
        Ok(())
    }

    /// Returns flash bytes as little-endian program words.
    pub fn program_words(&self) -> Result<Vec<u16>, SimError> {
        let size = self
            .segments
            .iter()
            .filter(|segment| segment.address < AVR_DATA_BASE)
            .map(|segment| segment.address as usize + segment.data.len())
            .max()
            .unwrap_or(0);
        let size = size + (size % 2);
        let mut bytes = vec![0u8; size];
        self.load_flash(&mut bytes)?;
        Ok(bytes
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect())
    }
}

fn parse_header(data: &[u8]) -> Result<ElfHeader, SimError> {
    validate_ident(data)?;
    validate_machine(data)?;
    let program_count = read_u16(data, 44)?;
    let section_count = read_u16(data, 48)?;
    validate_section_names(data, section_count)?;
    validate_tables(data, program_count, section_count)?;
    Ok(ElfHeader {
        entry: read_u32(data, 24)?,
        program_offset: read_u32(data, 28)?,
        section_offset: read_u32(data, 32)?,
        program_count,
        section_count,
    })
}

fn validate_ident(data: &[u8]) -> Result<(), SimError> {
    if data.len() < ELF_HEADER_SIZE {
        return Err(SimError::Parse("ELF header is truncated".to_string()));
    }
    if &data[0..4] != b"\x7fELF" {
        return Err(SimError::Parse("invalid ELF magic".to_string()));
    }
    if data[4] != 1 || data[5] != 1 {
        return Err(SimError::Parse(
            "ELF must be 32-bit little endian".to_string(),
        ));
    }
    Ok(())
}

fn validate_machine(data: &[u8]) -> Result<(), SimError> {
    if read_u16(data, 18)? != MACHINE_AVR {
        return Err(SimError::Parse("ELF machine is not AVR".to_string()));
    }
    Ok(())
}

fn validate_section_names(data: &[u8], count: u16) -> Result<(), SimError> {
    let index = read_u16(data, 50)?;
    if index != 0 && index >= count {
        return Err(SimError::Parse(
            "ELF section name index is invalid".to_string(),
        ));
    }
    Ok(())
}

fn validate_tables(data: &[u8], programs: u16, sections: u16) -> Result<(), SimError> {
    if programs > 0 {
        check_table(data, read_u32(data, 28)?, programs, PROGRAM_HEADER_SIZE)?;
    }
    if sections > 0 {
        check_table(data, read_u32(data, 32)?, sections, SECTION_HEADER_SIZE)?;
    }
    Ok(())
}

fn parse_segments(data: &[u8], header: ElfHeader) -> Result<Vec<ElfSegment>, SimError> {
    let mut segments = Vec::new();
    for index in 0..usize::from(header.program_count) {
        let offset = header.program_offset as usize + index * PROGRAM_HEADER_SIZE;
        if read_u32(data, offset)? != PT_LOAD {
            continue;
        }
        let file_offset = read_u32(data, offset + 4)? as usize;
        let address = read_u32(data, offset + 8)?;
        let file_size = read_u32(data, offset + 16)? as usize;
        let memory_size = read_u32(data, offset + 20)?;
        let bytes = slice(data, file_offset, file_size)?;
        segments.push(ElfSegment {
            address,
            data: bytes.to_vec(),
            memory_size,
        });
    }
    Ok(segments)
}

fn parse_symbols(data: &[u8], header: ElfHeader) -> Result<Vec<ElfSymbol>, SimError> {
    let sections = parse_sections(data, header)?;
    let mut symbols = Vec::new();
    for section in &sections {
        if section.kind == SHT_SYMTAB {
            symbols.extend(parse_symbol_table(data, section, &sections)?);
        }
    }
    Ok(symbols)
}

fn parse_symbol_table(
    data: &[u8],
    section: &Section,
    sections: &[Section],
) -> Result<Vec<ElfSymbol>, SimError> {
    let names = sections
        .get(section.link as usize)
        .ok_or_else(|| SimError::Parse("ELF symtab link is invalid".to_string()))?;
    let string_table = slice(data, names.offset as usize, names.size as usize)?;
    let entry_size = symbol_entry_size(section);
    let mut symbols = Vec::new();
    for entry in (0..section.size as usize).step_by(entry_size) {
        let offset = section.offset as usize + entry;
        append_symbol(&mut symbols, data, offset, string_table)?;
    }
    Ok(symbols)
}

fn symbol_entry_size(section: &Section) -> usize {
    if section.entry_size == 0 {
        SYMBOL_SIZE
    } else {
        section.entry_size as usize
    }
}

fn parse_sections(data: &[u8], header: ElfHeader) -> Result<Vec<Section>, SimError> {
    let mut sections = Vec::new();
    for index in 0..usize::from(header.section_count) {
        let offset = header.section_offset as usize + index * SECTION_HEADER_SIZE;
        sections.push(Section {
            kind: read_u32(data, offset + 4)?,
            offset: read_u32(data, offset + 16)?,
            size: read_u32(data, offset + 20)?,
            link: read_u32(data, offset + 24)?,
            entry_size: read_u32(data, offset + 36)?,
        });
    }
    Ok(sections)
}
fn append_symbol(
    symbols: &mut Vec<ElfSymbol>,
    data: &[u8],
    offset: usize,
    names: &[u8],
) -> Result<(), SimError> {
    let (name_offset, address, size, info, section) = symbol_fields(data, offset)?;
    if section == 0 {
        return Ok(());
    }
    let name = read_string(names, name_offset)?;
    if name.is_empty() {
        return Ok(());
    }
    symbols.push(make_symbol(name.to_string(), address, size, info));
    Ok(())
}

fn make_symbol(name: String, address: u32, size: u32, info: u8) -> ElfSymbol {
    ElfSymbol {
        name,
        address,
        size,
        kind: symbol_kind(info),
    }
}

fn symbol_fields(data: &[u8], offset: usize) -> Result<(usize, u32, u32, u8, u16), SimError> {
    let name_offset = read_u32(data, offset)? as usize;
    let address = read_u32(data, offset + 4)?;
    let size = read_u32(data, offset + 8)?;
    let info = data
        .get(offset + 12)
        .copied()
        .ok_or_else(|| SimError::Parse("ELF symbol is truncated".to_string()))?;
    let section = read_u16(data, offset + 14)?;
    Ok((name_offset, address, size, info, section))
}

fn symbol_kind(info: u8) -> ElfSymbolKind {
    match info & 0x0F {
        1 => ElfSymbolKind::Object,
        2 => ElfSymbolKind::Function,
        _ => ElfSymbolKind::Other,
    }
}

fn read_string(table: &[u8], offset: usize) -> Result<&str, SimError> {
    let tail = table
        .get(offset..)
        .ok_or_else(|| SimError::Parse("ELF string offset is invalid".to_string()))?;
    let end = tail
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| SimError::Parse("ELF string is unterminated".to_string()))?;
    std::str::from_utf8(&tail[..end])
        .map_err(|_| SimError::Parse("ELF string is not UTF-8".to_string()))
}

fn read_u16(data: &[u8], offset: usize) -> Result<u16, SimError> {
    let bytes = slice(data, offset, 2)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32, SimError> {
    let bytes = slice(data, offset, 4)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn slice(data: &[u8], offset: usize, length: usize) -> Result<&[u8], SimError> {
    let end = offset
        .checked_add(length)
        .ok_or_else(|| SimError::Parse("ELF offset overflow".to_string()))?;
    data.get(offset..end)
        .ok_or_else(|| SimError::Parse("ELF offset is out of bounds".to_string()))
}

fn check_table(data: &[u8], offset: u32, count: u16, size: usize) -> Result<(), SimError> {
    let length = usize::from(count)
        .checked_mul(size)
        .ok_or_else(|| SimError::Parse("ELF table overflow".to_string()))?;
    slice(data, offset as usize, length).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::{ElfImage, ElfSymbolKind};

    fn put_u16(data: &mut [u8], offset: usize, value: u16) {
        data[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn put_u32(data: &mut [u8], offset: usize, value: u32) {
        data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn header(data: &mut [u8]) {
        data[0..4].copy_from_slice(b"\x7fELF");
        data[4] = 1;
        data[5] = 1;
        data[6] = 1;
        put_u16(data, 16, 2);
        put_u16(data, 18, 83);
        put_u32(data, 20, 1);
        put_u32(data, 28, 52);
        put_u32(data, 32, 0x200);
        put_u16(data, 42, 32);
        put_u16(data, 44, 1);
        put_u16(data, 46, 40);
        put_u16(data, 48, 4);
        put_u16(data, 50, 3);
    }

    fn program_header(data: &mut [u8]) {
        put_u32(data, 52, 1);
        put_u32(data, 56, 0x100);
        put_u32(data, 68, 4);
        put_u32(data, 72, 4);
        put_u32(data, 76, 5);
        data[0x100..0x104].copy_from_slice(&[1, 2, 3, 4]);
    }

    fn section(data: &mut [u8], index: usize, spec: (u32, u32, u32, u32, u32, u32)) {
        let base = 0x200 + index * 40;
        put_u32(data, base, spec.0);
        put_u32(data, base + 4, spec.1);
        put_u32(data, base + 16, spec.2);
        put_u32(data, base + 20, spec.3);
        put_u32(data, base + 24, spec.4);
        put_u32(data, base + 36, spec.5);
    }

    fn symbols(data: &mut [u8]) {
        data[0x300..0x306].copy_from_slice(b"\0main\0");
        put_u32(data, 0x330, 1);
        put_u32(data, 0x338, 4);
        data[0x33C] = 0x12;
        put_u16(data, 0x33E, 1);
        data[0x340..0x35B].copy_from_slice(b"\0.symtab\0.strtab\0.shstrtab\0");
    }

    fn fixture() -> Vec<u8> {
        let mut data = vec![0u8; 0x400];
        header(&mut data);
        program_header(&mut data);
        section(&mut data, 1, (1, 2, 0x320, 32, 2, 16));
        section(&mut data, 2, (9, 3, 0x300, 6, 0, 0));
        section(&mut data, 3, (17, 3, 0x340, 27, 0, 0));
        symbols(&mut data);
        data
    }

    #[test]
    fn parses_segments_and_symbols() {
        let image = ElfImage::parse(&fixture()).expect("elf");
        assert_eq!(image.entry(), 0);
        assert_eq!(image.segments().len(), 1);
        assert_eq!(image.segments()[0].data, vec![1, 2, 3, 4]);
        assert_eq!(image.symbols().len(), 1);
        assert_eq!(image.symbols()[0].name, "main");
        assert_eq!(image.symbols()[0].kind, ElfSymbolKind::Function);
        assert_eq!(image.program_words().expect("words"), vec![0x0201, 0x0403]);
    }

    #[test]
    fn loads_flash_segment() {
        let image = ElfImage::parse(&fixture()).expect("elf");
        let mut flash = vec![0u8; 8];
        image.load_flash(&mut flash).expect("load");
        assert_eq!(flash, vec![1, 2, 3, 4, 0, 0, 0, 0]);
    }

    #[test]
    fn rejects_invalid_headers() {
        let mut data = fixture();
        data[0] = 0;
        assert!(ElfImage::parse(&data).is_err());
        assert!(ElfImage::parse(&[0u8; 10]).is_err());
        let mut data = fixture();
        put_u16(&mut data, 18, 1);
        assert!(ElfImage::parse(&data).is_err());
    }
}
