//! Alignment and distribution moves for selected components.

use crate::model::{ComponentDefinition, ComponentId, Document, GridPos, GridRect};
use crate::selection::Selection;

/// Edge or center used for alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
    Top,
    Bottom,
    CenterX,
    CenterY,
}

/// Axis used for distribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Distribute {
    Horizontal,
    Vertical,
}

struct Bounds {
    id: ComponentId,
    rect: GridRect,
}

struct Target {
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
    center_x: i32,
    center_y: i32,
}

/// Returns moves that align selected components.
#[must_use]
pub fn align_moves(
    document: &Document,
    definitions: &[ComponentDefinition],
    selection: &Selection,
    align: Align,
) -> Vec<(ComponentId, GridPos)> {
    let bounds = selected_bounds(document, definitions, selection);
    if bounds.len() < 2 {
        return Vec::new();
    }
    let target = target_of(&bounds);
    bounds
        .iter()
        .map(|item| (item.id, align_position(item.rect, align, &target)))
        .collect()
}

/// Returns moves that distribute selected components.
#[must_use]
pub fn distribute_moves(
    document: &Document,
    definitions: &[ComponentDefinition],
    selection: &Selection,
    axis: Distribute,
) -> Vec<(ComponentId, GridPos)> {
    let mut bounds = selected_bounds(document, definitions, selection);
    if bounds.len() < 3 {
        return Vec::new();
    }
    sort_by_axis(&mut bounds, axis);
    distribute_bounds(&bounds, axis)
}

fn distribute_bounds(bounds: &[Bounds], axis: Distribute) -> Vec<(ComponentId, GridPos)> {
    let first = center(&bounds[0].rect, axis);
    let last = center(&bounds[bounds.len() - 1].rect, axis);
    let step = (last - first) / (bounds.len() as i32 - 1);
    bounds
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let value = first + step * index as i32;
            (item.id, distributed_position(item.rect, axis, value))
        })
        .collect()
}

fn selected_bounds(
    document: &Document,
    definitions: &[ComponentDefinition],
    selection: &Selection,
) -> Vec<Bounds> {
    selection
        .components()
        .iter()
        .filter_map(|id| {
            let instance = document.component(*id)?;
            let definition = definitions
                .iter()
                .find(|definition| definition.id == instance.definition)?;
            Some(Bounds {
                id: *id,
                rect: instance.bounds(definition),
            })
        })
        .collect()
}

fn target_of(bounds: &[Bounds]) -> Target {
    let (left, right, top, bottom) = outer_edges(bounds);
    Target {
        left,
        right,
        top,
        bottom,
        center_x: average_center(bounds, Distribute::Horizontal),
        center_y: average_center(bounds, Distribute::Vertical),
    }
}

fn outer_edges(bounds: &[Bounds]) -> (i32, i32, i32, i32) {
    let left = bounds.iter().map(|item| item.rect.min.x).min().unwrap_or(0);
    let right = bounds
        .iter()
        .map(|item| item.rect.max_x())
        .max()
        .unwrap_or(0);
    let top = bounds.iter().map(|item| item.rect.min.y).min().unwrap_or(0);
    let bottom = bounds
        .iter()
        .map(|item| item.rect.max_y())
        .max()
        .unwrap_or(0);
    (left, right, top, bottom)
}

fn average_center(bounds: &[Bounds], axis: Distribute) -> i32 {
    let sum: i32 = bounds.iter().map(|item| center(&item.rect, axis)).sum();
    sum / bounds.len() as i32
}

fn align_position(rect: GridRect, align: Align, target: &Target) -> GridPos {
    match align {
        Align::Left => GridPos::new(target.left, rect.min.y),
        Align::Right => GridPos::new(target.right - rect.size.width, rect.min.y),
        Align::Top => GridPos::new(rect.min.x, target.top),
        Align::Bottom => GridPos::new(rect.min.x, target.bottom - rect.size.height),
        Align::CenterX => GridPos::new(target.center_x - rect.size.width / 2, rect.min.y),
        Align::CenterY => GridPos::new(rect.min.x, target.center_y - rect.size.height / 2),
    }
}

fn sort_by_axis(bounds: &mut [Bounds], axis: Distribute) {
    bounds.sort_by_key(|item| center(&item.rect, axis));
}

fn center(rect: &GridRect, axis: Distribute) -> i32 {
    match axis {
        Distribute::Horizontal => (rect.min.x + rect.max_x()) / 2,
        Distribute::Vertical => (rect.min.y + rect.max_y()) / 2,
    }
}

fn distributed_position(rect: GridRect, axis: Distribute, value: i32) -> GridPos {
    match axis {
        Distribute::Horizontal => GridPos::new(value - rect.size.width / 2, rect.min.y),
        Distribute::Vertical => GridPos::new(rect.min.x, value - rect.size.height / 2),
    }
}

#[cfg(test)]
mod tests {
    use super::{Align, Distribute, align_moves, distribute_moves};
    use crate::library;
    use crate::model::{Document, GridPos, Mirror, Rotation};
    use crate::selection::Selection;

    fn setup() -> (Document, Selection) {
        let resistor = crate::library::find("resistor").expect("resistor");
        let mut document = Document::new();
        let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        let second = document.add(resistor, GridPos::new(10, 4), Rotation::Deg0, Mirror::None);
        let mut selection = Selection::default();
        selection.select_component(first);
        selection.add_component(second);
        (document, selection)
    }

    #[test]
    fn align_moves_use_outer_edges() {
        let (document, selection) = setup();
        let library = library::builtin();
        let moves = align_moves(&document, library, &selection, Align::Left);
        assert_eq!(moves[1].1, GridPos::new(0, 4));
        let moves = align_moves(&document, library, &selection, Align::Top);
        assert_eq!(moves[1].1, GridPos::new(10, 0));
    }

    #[test]
    fn distribute_spaces_three_components() {
        let (mut document, selection) = setup();
        let library = library::builtin();
        let resistor = crate::library::find("resistor").expect("resistor");
        let third = document.add(resistor, GridPos::new(20, 0), Rotation::Deg0, Mirror::None);
        let mut selection = selection;
        selection.add_component(third);
        let moves = distribute_moves(&document, library, &selection, Distribute::Horizontal);
        assert_eq!(moves.len(), 3);
        assert_eq!(moves[1].1.x, 10);
    }
}
