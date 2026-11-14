//! Uniform-grid index for wire segment hit testing.

use crate::model::{GridPos, Wire, WireId};
use std::collections::{BTreeSet, HashMap};

/// One wire segment stored in an indexed cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SegmentRef {
    pub(crate) wire: WireId,
    pub(crate) index: usize,
    pub(crate) start: GridPos,
    pub(crate) end: GridPos,
}

/// Grid cell index over all wire segments.
#[derive(Clone, Debug, Default)]
pub(crate) struct SegmentIndex {
    cells: HashMap<(i32, i32), Vec<SegmentRef>>,
}

impl SegmentIndex {
    /// Builds an index from all wires.
    pub(crate) fn rebuild(wires: &[Wire]) -> Self {
        let mut index = Self::default();
        for wire in wires {
            index.insert(wire);
        }
        index
    }

    /// Adds every segment of one wire.
    pub(crate) fn insert(&mut self, wire: &Wire) {
        for segment in segments_of(wire) {
            for cell in segment_cells(segment.start, segment.end) {
                self.cells.entry(cell).or_default().push(segment);
            }
        }
    }

    /// Removes every segment of one wire.
    pub(crate) fn remove(&mut self, wire: &Wire) {
        for segment in segments_of(wire) {
            for cell in segment_cells(segment.start, segment.end) {
                if let Some(entries) = self.cells.get_mut(&cell) {
                    entries.retain(|entry| entry.wire != wire.id);
                }
            }
        }
        self.cells.retain(|_, entries| !entries.is_empty());
    }

    /// Returns candidate segments near a world-space point.
    pub(crate) fn candidates(&self, point: [f32; 2]) -> Vec<SegmentRef> {
        let cell = (point[0].floor() as i32, point[1].floor() as i32);
        let mut found = BTreeSet::new();
        for x in (cell.0 - 1)..=(cell.0 + 1) {
            for y in (cell.1 - 1)..=(cell.1 + 1) {
                if let Some(entries) = self.cells.get(&(x, y)) {
                    found.extend(entries.iter().copied());
                }
            }
        }
        found.into_iter().collect()
    }
}

fn segments_of(wire: &Wire) -> impl Iterator<Item = SegmentRef> + '_ {
    let id = wire.id;
    wire.points
        .windows(2)
        .enumerate()
        .map(move |(index, points)| SegmentRef {
            wire: id,
            index,
            start: points[0],
            end: points[1],
        })
}

fn segment_cells(start: GridPos, end: GridPos) -> Vec<(i32, i32)> {
    if start.y == end.y {
        return axis_cells(start.x, end.x, start.y, true);
    }
    if start.x == end.x {
        return axis_cells(start.y, end.y, start.x, false);
    }
    bbox_cells(start, end)
}

fn axis_cells(first: i32, second: i32, fixed: i32, horizontal: bool) -> Vec<(i32, i32)> {
    let (min, max) = ordered(first, second);
    (min..=max)
        .map(|value| {
            if horizontal {
                (value, fixed)
            } else {
                (fixed, value)
            }
        })
        .collect()
}

fn bbox_cells(start: GridPos, end: GridPos) -> Vec<(i32, i32)> {
    let (min_x, max_x) = ordered(start.x, end.x);
    let (min_y, max_y) = ordered(start.y, end.y);
    let mut cells = Vec::new();
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            cells.push((x, y));
        }
    }
    cells
}

fn ordered(first: i32, second: i32) -> (i32, i32) {
    if first <= second {
        (first, second)
    } else {
        (second, first)
    }
}

#[cfg(test)]
mod tests {
    use super::{SegmentIndex, SegmentRef};
    use crate::model::{GridPos, NetClass, Wire, WireId};

    fn wire(id: u32, points: &[(i32, i32)]) -> Wire {
        Wire {
            id: WireId(id),
            points: points.iter().map(|(x, y)| GridPos::new(*x, *y)).collect(),
            class: NetClass::Digital,
            color_override: None,
            label: None,
        }
    }

    #[test]
    fn index_finds_horizontal_segment() {
        let cable = wire(1, &[(0, 0), (4, 0)]);
        let index = SegmentIndex::rebuild(&[cable]);
        let found = index.candidates([2.0, 0.2]);
        assert_eq!(
            found,
            vec![SegmentRef {
                wire: WireId(1),
                index: 0,
                start: GridPos::new(0, 0),
                end: GridPos::new(4, 0),
            }]
        );
    }

    #[test]
    fn index_ignores_distant_points() {
        let cable = wire(1, &[(0, 0), (4, 0)]);
        let index = SegmentIndex::rebuild(&[cable]);
        assert!(index.candidates([2.0, 3.0]).is_empty());
    }

    #[test]
    fn index_removes_old_segments() {
        let cable = wire(1, &[(0, 0), (4, 0)]);
        let mut index = SegmentIndex::rebuild(std::slice::from_ref(&cable));
        index.remove(&cable);
        assert!(index.candidates([2.0, 0.2]).is_empty());
    }

    #[test]
    fn index_handles_vertical_segments() {
        let cable = wire(2, &[(1, 0), (1, 5)]);
        let index = SegmentIndex::rebuild(&[cable]);
        assert_eq!(index.candidates([1.2, 3.0]).len(), 1);
    }
}
