//! Bounded undo and redo history for the schematic document.

use crate::model::Document;

const LIMIT: usize = 100;

/// Snapshot-based undo and redo history.
#[derive(Default)]
pub struct History {
    undo: Vec<Document>,
    redo: Vec<Document>,
}

impl History {
    /// Records the state before a change.
    pub fn record(&mut self, document: &Document) {
        if self.undo.len() == LIMIT {
            self.undo.remove(0);
        }
        self.undo.push(document.clone());
        self.redo.clear();
    }

    /// Restores the previous state.
    pub fn undo(&mut self, current: &Document) -> Option<Document> {
        let previous = self.undo.pop()?;
        self.redo.push(current.clone());
        Some(previous)
    }

    /// Reapplies the next state.
    pub fn redo(&mut self, current: &Document) -> Option<Document> {
        let next = self.redo.pop()?;
        self.undo.push(current.clone());
        Some(next)
    }

    /// Clears both stacks.
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    /// Returns true when an undo step is available.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Returns true when a redo step is available.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::History;
    use crate::library;
    use crate::model::{GridPos, Mirror, Rotation};

    #[test]
    fn undo_and_redo_restore_document() {
        let resistor = library::find("resistor").expect("resistor");
        let mut document = crate::model::Document::new();
        let mut history = History::default();
        history.record(&document);
        document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        let undone = history.undo(&document).expect("undo");
        assert_eq!(undone.components().len(), 0);
        let redone = history.redo(&undone).expect("redo");
        assert_eq!(redone.components().len(), 1);
    }

    #[test]
    fn record_clears_redo() {
        let mut history = History::default();
        let document = crate::model::Document::new();
        history.record(&document);
        history.undo(&document).expect("undo");
        assert!(history.can_redo());
        history.record(&document);
        assert!(!history.can_redo());
    }
}
