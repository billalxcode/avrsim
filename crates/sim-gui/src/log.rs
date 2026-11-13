//! Bounded GUI status and error log.

use std::collections::VecDeque;

const LIMIT: usize = 200;

/// Most recent status messages.
#[derive(Default)]
pub struct Log {
    entries: VecDeque<String>,
}

impl Log {
    /// Adds an entry, dropping the oldest beyond the limit.
    pub fn push(&mut self, entry: String) {
        if self.entries.len() == LIMIT {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Removes all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns entries from oldest to newest.
    #[must_use]
    pub fn entries(&self) -> &VecDeque<String> {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::{LIMIT, Log};

    #[test]
    fn log_is_bounded() {
        let mut log = Log::default();
        for index in 0..LIMIT + 5 {
            log.push(format!("entry {index}"));
        }
        assert_eq!(log.entries().len(), LIMIT);
        assert_eq!(log.entries().front().expect("front"), "entry 5");
        log.clear();
        assert!(log.entries().is_empty());
    }
}
