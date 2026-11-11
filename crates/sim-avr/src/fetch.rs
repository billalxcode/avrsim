//! AVR program memory and instruction fetch.

use super::decode::{decode, uses_second_word};
use super::instruction::Instruction;
use sim_core::SimError;

/// A fetched AVR instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fetched {
    /// Decoded instruction.
    pub instruction: Instruction,
    /// First instruction word.
    pub first: u16,
    /// Second instruction word, when present.
    pub second: Option<u16>,
    /// Number of words fetched.
    pub words: u8,
}

/// Behavioral AVR program memory.
#[derive(Clone, Debug)]
pub struct ProgramMemory {
    words: Vec<u16>,
}

impl ProgramMemory {
    /// Creates program memory from instruction words.
    #[must_use]
    pub fn new(words: Vec<u16>) -> Self {
        Self { words }
    }

    /// Returns the instruction words.
    #[must_use]
    pub fn words(&self) -> &[u16] {
        &self.words
    }

    /// Reads one word at a word address.
    pub fn read(&self, address: usize) -> Result<u16, SimError> {
        self.words
            .get(address)
            .copied()
            .ok_or_else(|| SimError::Parse(format!("program word 0x{address:04X} out of range")))
    }

    /// Fetches and decodes an instruction at a word address.
    pub fn fetch(&self, pc: u16) -> Result<Fetched, SimError> {
        let address = usize::from(pc);
        let first = self.read(address)?;
        if !uses_second_word(first) {
            return self.single(first);
        }
        let second = self.read(address + 1)?;
        let instruction = decode(first, Some(second))?;
        Ok(Fetched {
            instruction,
            first,
            second: Some(second),
            words: 2,
        })
    }

    fn single(&self, first: u16) -> Result<Fetched, SimError> {
        let instruction = decode(first, None)?;
        Ok(Fetched {
            instruction,
            first,
            second: None,
            words: 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ProgramMemory;
    use crate::instruction::{Mnemonic, Operand};

    #[test]
    fn fetches_single_word_instruction() {
        let memory = ProgramMemory::new(vec![0xE115]);
        let fetched = memory.fetch(0).expect("fetch");
        assert_eq!(fetched.words, 1);
        assert_eq!(fetched.second, None);
        assert_eq!(fetched.instruction.mnemonic, Mnemonic::Ldi);
        assert_eq!(fetched.instruction.first, Operand::Register(17));
    }

    #[test]
    fn fetches_two_word_instruction() {
        let memory = ProgramMemory::new(vec![0x9100, 0x1234]);
        let fetched = memory.fetch(0).expect("fetch");
        assert_eq!(fetched.words, 2);
        assert_eq!(fetched.second, Some(0x1234));
        assert_eq!(fetched.instruction.mnemonic, Mnemonic::Lds);
        assert_eq!(fetched.instruction.second, Operand::Address(0x1234));
    }

    #[test]
    fn rejects_truncated_two_word_instruction() {
        let memory = ProgramMemory::new(vec![0x9100]);
        assert!(memory.fetch(0).is_err());
    }

    #[test]
    fn rejects_address_out_of_range() {
        let memory = ProgramMemory::new(vec![0x0000]);
        assert!(memory.fetch(1).is_err());
    }

    #[test]
    fn fetch_uses_word_addresses() {
        let memory = ProgramMemory::new(vec![0x0000, 0xE000]);
        let fetched = memory.fetch(1).expect("fetch");
        assert_eq!(fetched.instruction.mnemonic, Mnemonic::Ldi);
    }
}
