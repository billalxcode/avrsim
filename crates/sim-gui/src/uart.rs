//! Bounded UART receive log for the GUI monitor.

const LIMIT: usize = 16 * 1024;

/// UTF-8 lossy UART text with a size limit.
#[derive(Default)]
pub struct UartLog {
    text: String,
}

impl UartLog {
    /// Appends bytes and trims the oldest text beyond the limit.
    pub fn push(&mut self, data: &[u8]) {
        self.text.push_str(&String::from_utf8_lossy(data));
        self.trim();
    }

    /// Removes all text.
    pub fn clear(&mut self) {
        self.text.clear();
    }

    /// Returns the current text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the text length in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    fn trim(&mut self) {
        if self.text.len() <= LIMIT {
            return;
        }
        let excess = self.text.len() - LIMIT;
        let split = (excess..self.text.len())
            .find(|index| self.text.is_char_boundary(*index))
            .unwrap_or(self.text.len());
        self.text.drain(..split);
    }
}

#[cfg(test)]
mod tests {
    use super::UartLog;

    #[test]
    fn push_and_clear_text() {
        let mut log = UartLog::default();
        log.push(b"Hello");
        log.push(b" world\n");
        assert_eq!(log.text(), "Hello world\n");
        assert_eq!(log.len(), 12);
        log.clear();
        assert!(log.text().is_empty());
    }

    #[test]
    fn trims_oldest_text_to_limit() {
        let mut log = UartLog::default();
        log.push(&vec![b'a'; super::LIMIT + 100]);
        assert_eq!(log.len(), super::LIMIT);
        assert!(log.text().ends_with('a'));
    }
}
