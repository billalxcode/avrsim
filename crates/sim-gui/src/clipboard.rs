//! In-memory clipboard for schematic components and wires.

use crate::model::{
    ComponentDefinition, ComponentId, Document, GridPos, Mirror, NetClass, Rotation, Wire, WireId,
};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
struct ClipComponent {
    definition: &'static str,
    position: GridPos,
    rotation: Rotation,
    mirror: Mirror,
    value: String,
}

#[derive(Clone, Debug)]
struct ClipWire {
    points: Vec<GridPos>,
    class: NetClass,
    color: Option<[u8; 3]>,
}

/// Copied components and wires with a common origin.
#[derive(Default)]
pub struct Clipboard {
    components: Vec<ClipComponent>,
    wires: Vec<ClipWire>,
    origin: GridPos,
}

/// IDs created by one paste operation.
pub struct PasteResult {
    pub components: Vec<ComponentId>,
    pub wires: Vec<WireId>,
}

impl Clipboard {
    /// Captures the selected components and wires.
    #[must_use]
    pub fn capture(
        document: &Document,
        components: &BTreeSet<ComponentId>,
        wires: &BTreeSet<WireId>,
    ) -> Self {
        let components = capture_components(document, components);
        let origin = origin_of(&components);
        let wires = capture_wires(document, wires);
        Self {
            components,
            wires,
            origin,
        }
    }

    /// Returns the top-left origin of the copied components.
    #[must_use]
    pub const fn origin(&self) -> GridPos {
        self.origin
    }

    /// Returns true when no component has been copied.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Pastes copied components and wires with the given offset.
    pub fn paste(
        &self,
        document: &mut Document,
        library: &[ComponentDefinition],
        offset: GridPos,
    ) -> Result<PasteResult, String> {
        if self.is_empty() {
            return Err("Clipboard is empty".to_string());
        }
        let mut candidate = document.clone();
        let components = paste_components(&mut candidate, library, &self.components, offset)?;
        let wires = paste_wires(&mut candidate, library, &self.wires, offset);
        *document = candidate;
        Ok(PasteResult { components, wires })
    }
}

fn capture_components(document: &Document, ids: &BTreeSet<ComponentId>) -> Vec<ClipComponent> {
    ids.iter()
        .filter_map(|id| document.component(*id))
        .map(ClipComponent::from_instance)
        .collect()
}

fn capture_wires(document: &Document, ids: &BTreeSet<WireId>) -> Vec<ClipWire> {
    document
        .wires()
        .iter()
        .filter(|wire| ids.contains(&wire.id))
        .map(ClipWire::from_wire)
        .collect()
}

impl ClipComponent {
    fn from_instance(instance: &crate::model::ComponentInstance) -> Self {
        Self {
            definition: instance.definition,
            position: instance.position,
            rotation: instance.rotation,
            mirror: instance.mirror,
            value: instance.value.clone(),
        }
    }
}

impl ClipWire {
    fn from_wire(wire: &Wire) -> Self {
        Self {
            points: wire.points.clone(),
            class: wire.class,
            color: wire.color_override,
        }
    }
}

fn origin_of(components: &[ClipComponent]) -> GridPos {
    let x = components
        .iter()
        .map(|component| component.position.x)
        .min()
        .unwrap_or(0);
    let y = components
        .iter()
        .map(|component| component.position.y)
        .min()
        .unwrap_or(0);
    GridPos::new(x, y)
}

fn paste_components(
    document: &mut Document,
    library: &[ComponentDefinition],
    components: &[ClipComponent],
    offset: GridPos,
) -> Result<Vec<ComponentId>, String> {
    let mut ids = Vec::new();
    for clip in components {
        let Some(definition) = library.iter().find(|item| item.id == clip.definition) else {
            return Err(format!("Unknown component: {}", clip.definition));
        };
        let position = clip.position.offset(offset);
        if !document.can_place(library, definition, position, clip.rotation, clip.mirror) {
            return Err("Paste blocked by overlap".to_string());
        }
        let id = document.add_with_value(
            definition,
            position,
            clip.rotation,
            clip.mirror,
            &clip.value,
        );
        ids.push(id);
    }
    Ok(ids)
}

fn paste_wires(
    document: &mut Document,
    library: &[ComponentDefinition],
    wires: &[ClipWire],
    offset: GridPos,
) -> Vec<WireId> {
    wires
        .iter()
        .filter_map(|clip| {
            let points = clip
                .points
                .iter()
                .map(|point| point.offset(offset))
                .collect();
            let id = document.add_wire(library, points, clip.class)?;
            document.set_wire_color(id, clip.color);
            Some(id)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::Clipboard;
    use crate::library;
    use crate::model::{ComponentId, Document, GridPos, Mirror, NetClass, Rotation, WireId};
    use std::collections::BTreeSet;

    fn selection(components: &[u32], wires: &[u32]) -> (BTreeSet<ComponentId>, BTreeSet<WireId>) {
        (
            components.iter().map(|id| ComponentId(*id)).collect(),
            wires.iter().map(|id| WireId(*id)).collect(),
        )
    }

    fn value_clipboard() -> (Document, Clipboard) {
        let resistor = library::find("resistor").expect("resistor");
        let mut document = Document::new();
        let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        let second = document.add_with_value(
            resistor,
            GridPos::new(6, 0),
            Rotation::Deg90,
            Mirror::Horizontal,
            "10k",
        );
        let (components, wires) = selection(&[first.0, second.0], &[]);
        let clipboard = Clipboard::capture(&document, &components, &wires);
        (document, clipboard)
    }

    fn wire_clipboard() -> (Document, Clipboard) {
        let library = library::builtin();
        let resistor = library::find("resistor").expect("resistor");
        let mut document = Document::new();
        let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        let second = document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
        let wire = document
            .add_wire(
                library,
                vec![GridPos::new(4, 1), GridPos::new(6, 1)],
                NetClass::Digital,
            )
            .expect("wire");
        document.set_wire_color(wire, Some([9, 8, 7]));
        let (components, wires) = selection(&[first.0, second.0], &[wire.0]);
        let clipboard = Clipboard::capture(&document, &components, &wires);
        (document, clipboard)
    }

    #[test]
    fn paste_preserves_relative_positions_and_values() {
        let (mut document, clipboard) = value_clipboard();
        let library = library::builtin();
        let pasted = clipboard
            .paste(&mut document, library, GridPos::new(2, 2))
            .expect("paste");
        assert_eq!(pasted.components.len(), 2);
        assert_eq!(document.components().len(), 4);
        let copied = document.component(pasted.components[1]).expect("copied");
        assert_eq!(copied.position, GridPos::new(8, 2));
        assert_eq!(copied.value, "10k");
    }

    #[test]
    fn paste_is_atomic_on_overlap() {
        let library = library::builtin();
        let resistor = library::find("resistor").expect("resistor");
        let mut document = Document::new();
        let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        let (components, wires) = selection(&[first.0], &[]);
        let clipboard = Clipboard::capture(&document, &components, &wires);
        assert!(
            clipboard
                .paste(&mut document, library, GridPos::new(0, 0))
                .is_err()
        );
        assert_eq!(document.components().len(), 1);
    }

    #[test]
    fn paste_copies_internal_wire() {
        let (mut document, clipboard) = wire_clipboard();
        let library = library::builtin();
        let pasted = clipboard
            .paste(&mut document, library, GridPos::new(0, 4))
            .expect("paste");
        assert_eq!(pasted.components.len(), 2);
        assert_eq!(pasted.wires.len(), 1);
        assert_eq!(document.wires().len(), 2);
        assert_eq!(document.wires()[1].color_override, Some([9, 8, 7]));
    }
}
