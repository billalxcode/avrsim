//! Component and wire selection set.

use crate::model::{ComponentId, WireId};
use std::collections::BTreeSet;

/// Components and wires currently selected.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    components: BTreeSet<ComponentId>,
    wires: BTreeSet<WireId>,
}

impl Selection {
    /// Removes all selected items.
    pub fn clear(&mut self) {
        self.components.clear();
        self.wires.clear();
    }

    /// Returns true when nothing is selected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.components.is_empty() && self.wires.is_empty()
    }

    /// Returns the number of selected items.
    #[must_use]
    pub fn len(&self) -> usize {
        self.components.len() + self.wires.len()
    }

    /// Returns selected components.
    #[must_use]
    pub fn components(&self) -> &BTreeSet<ComponentId> {
        &self.components
    }

    /// Returns selected wires.
    #[must_use]
    pub fn wires(&self) -> &BTreeSet<WireId> {
        &self.wires
    }

    /// Returns true when a component is selected.
    #[must_use]
    pub fn contains_component(&self, id: ComponentId) -> bool {
        self.components.contains(&id)
    }

    /// Returns true when a wire is selected.
    #[must_use]
    pub fn contains_wire(&self, id: WireId) -> bool {
        self.wires.contains(&id)
    }

    /// Returns the component when it is the only selected item.
    #[must_use]
    pub fn single_component(&self) -> Option<ComponentId> {
        if self.wires.is_empty() && self.components.len() == 1 {
            return self.components.iter().copied().next();
        }
        None
    }

    /// Returns the wire when it is the only selected item.
    #[must_use]
    pub fn single_wire(&self) -> Option<WireId> {
        if self.components.is_empty() && self.wires.len() == 1 {
            return self.wires.iter().copied().next();
        }
        None
    }

    /// Replaces the selection with one component.
    pub fn select_component(&mut self, id: ComponentId) {
        self.clear();
        self.components.insert(id);
    }

    /// Replaces the selection with one wire.
    pub fn select_wire(&mut self, id: WireId) {
        self.clear();
        self.wires.insert(id);
    }

    /// Adds or removes a component.
    pub fn toggle_component(&mut self, id: ComponentId) {
        if !self.components.remove(&id) {
            self.components.insert(id);
        }
    }

    /// Adds or removes a wire.
    pub fn toggle_wire(&mut self, id: WireId) {
        if !self.wires.remove(&id) {
            self.wires.insert(id);
        }
    }

    /// Adds one component without clearing the rest.
    pub fn add_component(&mut self, id: ComponentId) {
        self.components.insert(id);
    }

    /// Replaces or extends the selection with many items.
    pub fn apply(
        &mut self,
        components: impl IntoIterator<Item = ComponentId>,
        wires: impl IntoIterator<Item = WireId>,
        additive: bool,
    ) {
        if !additive {
            self.clear();
        }
        self.components.extend(components);
        self.wires.extend(wires);
    }
}

#[cfg(test)]
mod tests {
    use super::Selection;
    use crate::model::{ComponentId, WireId};

    #[test]
    fn single_selection_clears_other_kind() {
        let mut selection = Selection::default();
        selection.add_component(ComponentId(1));
        selection.select_wire(WireId(2));
        assert!(!selection.contains_component(ComponentId(1)));
        assert_eq!(selection.single_wire(), Some(WireId(2)));
        selection.select_component(ComponentId(3));
        assert_eq!(selection.single_component(), Some(ComponentId(3)));
        assert!(selection.wires().is_empty());
    }

    #[test]
    fn toggle_adds_and_removes() {
        let mut selection = Selection::default();
        selection.toggle_component(ComponentId(1));
        selection.toggle_wire(WireId(2));
        assert_eq!(selection.len(), 2);
        selection.toggle_component(ComponentId(1));
        assert!(!selection.contains_component(ComponentId(1)));
    }

    #[test]
    fn apply_replaces_or_extends() {
        let mut selection = Selection::default();
        selection.add_component(ComponentId(1));
        selection.apply([ComponentId(2)], [WireId(3)], false);
        assert!(!selection.contains_component(ComponentId(1)));
        selection.apply([ComponentId(4)], [], true);
        assert_eq!(selection.components().len(), 2);
        assert_eq!(selection.wires().len(), 1);
    }
}
