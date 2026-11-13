//! Schematic document model, grid geometry, and component transforms.

use std::collections::{BTreeMap, BTreeSet};

/// Integer position on the schematic grid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    /// Creates a grid position.
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Adds another grid offset.
    #[must_use]
    pub const fn offset(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    /// Subtracts another grid offset.
    #[must_use]
    pub const fn subtract(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

/// Size in grid cells.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GridSize {
    pub width: i32,
    pub height: i32,
}

impl GridSize {
    /// Creates a grid size.
    #[must_use]
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    /// Returns the size with width and height swapped.
    #[must_use]
    pub const fn swapped(self) -> Self {
        Self {
            width: self.height,
            height: self.width,
        }
    }
}

/// Axis-aligned rectangle on the grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridRect {
    pub min: GridPos,
    pub size: GridSize,
}

impl GridRect {
    /// Creates a rectangle from its top-left position and size.
    #[must_use]
    pub const fn new(min: GridPos, size: GridSize) -> Self {
        Self { min, size }
    }

    /// Returns the exclusive right edge.
    #[must_use]
    pub const fn max_x(self) -> i32 {
        self.min.x + self.size.width
    }

    /// Returns the exclusive bottom edge.
    #[must_use]
    pub const fn max_y(self) -> i32 {
        self.min.y + self.size.height
    }

    /// Returns true when a grid point lies inside or on the rectangle.
    #[must_use]
    pub const fn contains(self, point: GridPos) -> bool {
        point.x >= self.min.x
            && point.x <= self.max_x()
            && point.y >= self.min.y
            && point.y <= self.max_y()
    }

    /// Returns true when a grid point lies strictly inside the rectangle.
    #[must_use]
    pub const fn strictly_contains(self, point: GridPos) -> bool {
        point.x > self.min.x
            && point.x < self.max_x()
            && point.y > self.min.y
            && point.y < self.max_y()
    }

    /// Returns true when two rectangles overlap in their interiors.
    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        self.min.x < other.max_x()
            && other.min.x < self.max_x()
            && self.min.y < other.max_y()
            && other.min.y < self.max_y()
    }
}

/// Component rotation in quarter turns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Rotation {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Rotation {
    /// Returns the next clockwise rotation.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Deg0 => Self::Deg90,
            Self::Deg90 => Self::Deg180,
            Self::Deg180 => Self::Deg270,
            Self::Deg270 => Self::Deg0,
        }
    }

    /// Returns a human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Deg0 => "0",
            Self::Deg90 => "90",
            Self::Deg180 => "180",
            Self::Deg270 => "270",
        }
    }

    fn apply(self, delta: GridPos) -> GridPos {
        match self {
            Self::Deg0 => delta,
            Self::Deg90 => GridPos::new(delta.y, -delta.x),
            Self::Deg180 => GridPos::new(-delta.x, -delta.y),
            Self::Deg270 => GridPos::new(-delta.y, delta.x),
        }
    }
}

/// Component mirroring across an axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mirror {
    #[default]
    None,
    Horizontal,
    Vertical,
}

impl Mirror {
    /// Returns the next mirror mode.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::None => Self::Horizontal,
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::None,
        }
    }

    /// Returns a human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }

    fn apply(self, delta: GridPos) -> GridPos {
        match self {
            Self::None => delta,
            Self::Horizontal => GridPos::new(-delta.x, delta.y),
            Self::Vertical => GridPos::new(delta.x, -delta.y),
        }
    }
}

/// One pin on a component definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PinDefinition {
    pub name: &'static str,
    pub number: &'static str,
    pub offset: GridPos,
}

/// Symbol family used by the canvas renderer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Resistor,
    Led,
    Button,
    Vcc,
    Gnd,
    Mcu,
}

/// Library category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Board,
    Passive,
    Power,
}

impl Category {
    pub const ALL: [Self; 3] = [Self::Board, Self::Passive, Self::Power];

    /// Returns a human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Board => "Board",
            Self::Passive => "Passive",
            Self::Power => "Power",
        }
    }
}

/// Static component definition.
#[derive(Clone, Copy, Debug)]
pub struct ComponentDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub category: Category,
    pub reference_prefix: &'static str,
    pub default_value: &'static str,
    pub size: GridSize,
    pub pins: &'static [PinDefinition],
    pub symbol: SymbolKind,
}

/// Stable component instance identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ComponentId(pub u32);

/// Stable wire identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WireId(pub u32);

/// Net class used for wire coloring and semantics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NetClass {
    Power,
    Ground,
    #[default]
    Digital,
    Analog,
    Clock,
    Uart,
    Spi,
    I2c,
    Pwm,
    Reset,
    User,
}

impl NetClass {
    pub const ALL: [Self; 11] = [
        Self::Power,
        Self::Ground,
        Self::Digital,
        Self::Analog,
        Self::Clock,
        Self::Uart,
        Self::Spi,
        Self::I2c,
        Self::Pwm,
        Self::Reset,
        Self::User,
    ];

    /// Returns a human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Power => "Power",
            Self::Ground => "Ground",
            Self::Digital => "Digital",
            Self::Analog => "Analog",
            Self::Clock => "Clock",
            Self::Uart => "UART",
            Self::Spi => "SPI",
            Self::I2c => "I2C",
            Self::Pwm => "PWM",
            Self::Reset => "Reset",
            Self::User => "User",
        }
    }
}

/// One placed wire polyline.
#[derive(Clone, Debug)]
pub struct Wire {
    pub id: WireId,
    pub points: Vec<GridPos>,
    pub class: NetClass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct GridEdge {
    first: GridPos,
    second: GridPos,
}

impl GridEdge {
    fn new(first: GridPos, second: GridPos) -> Self {
        if first <= second {
            Self { first, second }
        } else {
            Self {
                first: second,
                second: first,
            }
        }
    }
}

/// One placed component.
#[derive(Clone, Debug)]
pub struct ComponentInstance {
    pub id: ComponentId,
    pub definition: &'static str,
    pub position: GridPos,
    pub rotation: Rotation,
    pub mirror: Mirror,
    pub label: String,
    pub value: String,
}

impl ComponentInstance {
    /// Returns the placed body rectangle.
    #[must_use]
    pub fn bounds(&self, definition: &ComponentDefinition) -> GridRect {
        GridRect::new(self.position, body_size(definition, self.rotation))
    }
}

struct TransformTarget<'a> {
    definition: &'a ComponentDefinition,
    position: GridPos,
    rotation: Rotation,
    mirror: Mirror,
}

/// Schematic document containing components and wires.
#[derive(Default)]
pub struct Document {
    components: Vec<ComponentInstance>,
    wires: Vec<Wire>,
    next_id: u32,
    next_wire_id: u32,
}

impl Document {
    /// Creates an empty document.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a document from restored components and wires.
    #[must_use]
    pub fn restore(components: Vec<ComponentInstance>, wires: Vec<Wire>) -> Self {
        let next_id = components
            .iter()
            .map(|component| component.id.0 + 1)
            .max()
            .unwrap_or(0);
        let next_wire_id = wires.iter().map(|wire| wire.id.0 + 1).max().unwrap_or(0);
        Self {
            components,
            wires,
            next_id,
            next_wire_id,
        }
    }

    /// Returns all components.
    #[must_use]
    pub fn components(&self) -> &[ComponentInstance] {
        &self.components
    }

    /// Returns all wires.
    #[must_use]
    pub fn wires(&self) -> &[Wire] {
        &self.wires
    }

    /// Adds a validated wire polyline.
    pub fn add_wire(
        &mut self,
        library: &[ComponentDefinition],
        points: Vec<GridPos>,
        class: NetClass,
    ) -> Option<WireId> {
        if !self.validate_wire(library, &points, None) {
            return None;
        }
        let id = WireId(self.next_wire_id);
        self.next_wire_id += 1;
        self.wires.push(Wire { id, points, class });
        Some(id)
    }

    /// Replaces a wire polyline when the result is valid.
    pub fn update_wire_points(
        &mut self,
        library: &[ComponentDefinition],
        id: WireId,
        points: Vec<GridPos>,
    ) -> bool {
        if !self.validate_wire(library, &points, Some(id)) {
            return false;
        }
        let Some(wire) = self.wires.iter_mut().find(|wire| wire.id == id) else {
            return false;
        };
        wire.points = points;
        true
    }

    /// Tests whether a single segment can be added.
    #[must_use]
    pub fn can_add_segment(
        &self,
        library: &[ComponentDefinition],
        start: GridPos,
        end: GridPos,
    ) -> bool {
        if !is_orthogonal(start, end) || start == end {
            return false;
        }
        if self.segment_blocked(library, start, end) {
            return false;
        }
        let Some(edges) = wire_edges(&[start, end]) else {
            return false;
        };
        !edges.iter().any(|edge| self.edge_occupied(*edge, None))
    }

    /// Returns true when a point is a pin or an existing wire vertex.
    #[must_use]
    pub fn is_connection_point(&self, library: &[ComponentDefinition], point: GridPos) -> bool {
        self.is_connection_point_excluding(library, point, None)
    }

    fn is_connection_point_excluding(
        &self,
        library: &[ComponentDefinition],
        point: GridPos,
        ignore: Option<WireId>,
    ) -> bool {
        self.has_pin_at(library, point) || self.has_wire_vertex(point, ignore)
    }

    /// Removes a wire.
    pub fn remove_wire(&mut self, id: WireId) {
        self.wires.retain(|wire| wire.id != id);
    }

    /// Returns the nearest wire within a world-space tolerance.
    #[must_use]
    pub fn wire_at(&self, point: [f32; 2], tolerance: f32) -> Option<WireId> {
        let mut best: Option<(f32, WireId)> = None;
        for wire in &self.wires {
            for segment in wire.points.windows(2) {
                let distance = segment_distance(point, segment[0], segment[1]);
                if distance > tolerance {
                    continue;
                }
                if best.is_none_or(|(current, _)| distance < current) {
                    best = Some((distance, wire.id));
                }
            }
        }
        best.map(|(_, id)| id)
    }

    /// Returns the nearest segment of one wire within a tolerance.
    #[must_use]
    pub fn wire_segment_at(&self, id: WireId, point: [f32; 2], tolerance: f32) -> Option<usize> {
        let wire = self.wires.iter().find(|wire| wire.id == id)?;
        let mut best: Option<(f32, usize)> = None;
        for (index, segment) in wire.points.windows(2).enumerate() {
            let distance = segment_distance(point, segment[0], segment[1]);
            if distance > tolerance {
                continue;
            }
            if best.is_none_or(|(current, _)| distance < current) {
                best = Some((distance, index));
            }
        }
        best.map(|(_, index)| index)
    }

    /// Returns grid points where three or more segments meet.
    #[must_use]
    pub fn junctions(&self) -> Vec<GridPos> {
        let mut counts: BTreeMap<GridPos, usize> = BTreeMap::new();
        for wire in &self.wires {
            for segment in wire.points.windows(2) {
                *counts.entry(segment[0]).or_default() += 1;
                *counts.entry(segment[1]).or_default() += 1;
            }
        }
        counts
            .into_iter()
            .filter_map(|(point, count)| (count >= 3).then_some(point))
            .collect()
    }

    /// Finds one component by ID.
    #[must_use]
    pub fn component(&self, id: ComponentId) -> Option<&ComponentInstance> {
        self.components.iter().find(|component| component.id == id)
    }

    /// Finds one component by ID for editing.
    pub fn component_mut(&mut self, id: ComponentId) -> Option<&mut ComponentInstance> {
        self.components
            .iter_mut()
            .find(|component| component.id == id)
    }

    /// Adds a component and generates its reference label.
    pub fn add(
        &mut self,
        definition: &ComponentDefinition,
        position: GridPos,
        rotation: Rotation,
        mirror: Mirror,
    ) -> ComponentId {
        let id = self.allocate_id();
        let label = format!(
            "{}{}",
            definition.reference_prefix,
            self.next_reference(definition.reference_prefix)
        );
        let instance = new_instance(id, definition, position, rotation, mirror, label);
        self.components.push(instance);
        id
    }

    /// Removes a component.
    pub fn remove(&mut self, id: ComponentId) {
        self.components.retain(|component| component.id != id);
    }

    /// Tests whether a definition can be placed without overlap.
    #[must_use]
    pub fn can_place(
        &self,
        library: &[ComponentDefinition],
        definition: &ComponentDefinition,
        position: GridPos,
        rotation: Rotation,
        mirror: Mirror,
    ) -> bool {
        !self.overlaps(library, definition, position, rotation, mirror, None)
    }

    /// Moves a component when the target does not overlap.
    pub fn move_component(
        &mut self,
        library: &[ComponentDefinition],
        id: ComponentId,
        position: GridPos,
    ) -> bool {
        let Some(instance) = self.component(id) else {
            return false;
        };
        let Some(definition) = find_definition(library, instance.definition) else {
            return false;
        };
        let rotation = instance.rotation;
        let mirror = instance.mirror;
        if self.overlaps(library, definition, position, rotation, mirror, Some(id)) {
            return false;
        }
        if let Some(instance) = self.component_mut(id) {
            instance.position = position;
        }
        true
    }

    /// Rotates a component when the target does not overlap.
    pub fn set_rotation(
        &mut self,
        library: &[ComponentDefinition],
        id: ComponentId,
        rotation: Rotation,
    ) -> bool {
        self.update_transform(library, id, rotation, None)
    }

    /// Mirrors a component when the target does not overlap.
    pub fn set_mirror(
        &mut self,
        library: &[ComponentDefinition],
        id: ComponentId,
        mirror: Mirror,
    ) -> bool {
        self.update_transform(library, id, Rotation::Deg0, Some(mirror))
    }

    /// Returns the topmost component at a grid point.
    #[must_use]
    pub fn hit_test(&self, library: &[ComponentDefinition], point: GridPos) -> Option<ComponentId> {
        self.components.iter().rev().find_map(|instance| {
            let definition = find_definition(library, instance.definition)?;
            instance
                .bounds(definition)
                .contains(point)
                .then_some(instance.id)
        })
    }

    fn update_transform(
        &mut self,
        library: &[ComponentDefinition],
        id: ComponentId,
        rotation: Rotation,
        mirror: Option<Mirror>,
    ) -> bool {
        let Some(target) = self.transform_target(library, id, rotation, mirror) else {
            return false;
        };
        if !self.transform_allowed(library, &target, id) {
            return false;
        }
        self.apply_transform(id, target.rotation, target.mirror)
    }

    fn transform_allowed(
        &self,
        library: &[ComponentDefinition],
        target: &TransformTarget<'_>,
        id: ComponentId,
    ) -> bool {
        !self.overlaps(
            library,
            target.definition,
            target.position,
            target.rotation,
            target.mirror,
            Some(id),
        )
    }

    fn transform_target<'a>(
        &'a self,
        library: &'a [ComponentDefinition],
        id: ComponentId,
        rotation: Rotation,
        mirror: Option<Mirror>,
    ) -> Option<TransformTarget<'a>> {
        let instance = self.component(id)?;
        let definition = find_definition(library, instance.definition)?;
        let next_rotation = mirror.map_or(rotation, |_| instance.rotation);
        Some(TransformTarget {
            definition,
            position: instance.position,
            rotation: next_rotation,
            mirror: mirror.unwrap_or(instance.mirror),
        })
    }

    fn apply_transform(&mut self, id: ComponentId, rotation: Rotation, mirror: Mirror) -> bool {
        let Some(instance) = self.component_mut(id) else {
            return false;
        };
        instance.rotation = rotation;
        instance.mirror = mirror;
        true
    }

    fn overlaps(
        &self,
        library: &[ComponentDefinition],
        definition: &ComponentDefinition,
        position: GridPos,
        rotation: Rotation,
        mirror: Mirror,
        ignore: Option<ComponentId>,
    ) -> bool {
        let _ = mirror;
        let candidate = GridRect::new(position, body_size(definition, rotation));
        self.components.iter().any(|instance| {
            if Some(instance.id) == ignore {
                return false;
            }
            let Some(existing) = find_definition(library, instance.definition) else {
                return false;
            };
            candidate.intersects(instance.bounds(existing))
        })
    }

    fn allocate_id(&mut self) -> ComponentId {
        let id = ComponentId(self.next_id);
        self.next_id += 1;
        id
    }

    fn validate_wire(
        &self,
        library: &[ComponentDefinition],
        points: &[GridPos],
        ignore: Option<WireId>,
    ) -> bool {
        let Some(&first) = points.first() else {
            return false;
        };
        let Some(&last) = points.last() else {
            return false;
        };
        if points.len() < 2 || !self.is_connection_point_excluding(library, first, ignore) {
            return false;
        }
        if !self.is_connection_point_excluding(library, last, ignore) {
            return false;
        }
        let Some(edges) = wire_edges(points) else {
            return false;
        };
        if edges.iter().any(|edge| self.edge_occupied(*edge, ignore)) {
            return false;
        }
        !self.path_blocked(library, points)
    }

    fn edge_occupied(&self, edge: GridEdge, ignore: Option<WireId>) -> bool {
        self.wires.iter().any(|wire| {
            if Some(wire.id) == ignore {
                return false;
            }
            wire.points
                .windows(2)
                .any(|segment| segment_edges(segment[0], segment[1]).contains(&edge))
        })
    }

    fn path_blocked(&self, library: &[ComponentDefinition], points: &[GridPos]) -> bool {
        points
            .windows(2)
            .any(|segment| self.segment_blocked(library, segment[0], segment[1]))
    }

    fn segment_blocked(
        &self,
        library: &[ComponentDefinition],
        start: GridPos,
        end: GridPos,
    ) -> bool {
        let length = segment_length(start, end);
        (1..length).any(|step| self.point_in_body(library, segment_step(start, end, step)))
    }

    fn point_in_body(&self, library: &[ComponentDefinition], point: GridPos) -> bool {
        self.components.iter().any(|instance| {
            let Some(definition) = find_definition(library, instance.definition) else {
                return false;
            };
            instance.bounds(definition).strictly_contains(point)
        })
    }

    fn has_pin_at(&self, library: &[ComponentDefinition], point: GridPos) -> bool {
        self.components.iter().any(|instance| {
            let Some(definition) = find_definition(library, instance.definition) else {
                return false;
            };
            definition
                .pins
                .iter()
                .any(|pin| world_pin(instance, definition, pin) == point)
        })
    }

    fn has_wire_vertex(&self, point: GridPos, ignore: Option<WireId>) -> bool {
        self.wires
            .iter()
            .any(|wire| Some(wire.id) != ignore && wire.points.contains(&point))
    }

    fn next_reference(&self, prefix: &str) -> usize {
        self.components
            .iter()
            .filter(|component| component.label.starts_with(prefix))
            .count()
            + 1
    }
}

fn new_instance(
    id: ComponentId,
    definition: &ComponentDefinition,
    position: GridPos,
    rotation: Rotation,
    mirror: Mirror,
    label: String,
) -> ComponentInstance {
    ComponentInstance {
        id,
        definition: definition.id,
        position,
        rotation,
        mirror,
        label,
        value: definition.default_value.to_string(),
    }
}

/// Returns the body size after rotation.
#[must_use]
pub fn body_size(definition: &ComponentDefinition, rotation: Rotation) -> GridSize {
    match rotation {
        Rotation::Deg0 | Rotation::Deg180 => definition.size,
        Rotation::Deg90 | Rotation::Deg270 => definition.size.swapped(),
    }
}

/// Returns a pin position in world grid coordinates.
#[must_use]
pub fn world_pin(
    instance: &ComponentInstance,
    definition: &ComponentDefinition,
    pin: &PinDefinition,
) -> GridPos {
    instance.position.offset(transform_point(
        definition,
        pin.offset,
        instance.rotation,
        instance.mirror,
    ))
}

/// Returns the outward direction of a pin in world grid units.
#[must_use]
pub fn pin_direction(
    definition: &ComponentDefinition,
    pin: &PinDefinition,
    rotation: Rotation,
    mirror: Mirror,
) -> GridPos {
    let delta = outward_delta(definition, pin.offset);
    let mirrored = mirror.apply(delta);
    rotation.apply(mirrored)
}

fn outward_delta(definition: &ComponentDefinition, offset: GridPos) -> GridPos {
    if offset.x == 0 {
        GridPos::new(-1, 0)
    } else if offset.x == definition.size.width {
        GridPos::new(1, 0)
    } else if offset.y == 0 {
        GridPos::new(0, -1)
    } else {
        GridPos::new(0, 1)
    }
}

fn transform_point(
    definition: &ComponentDefinition,
    point: GridPos,
    rotation: Rotation,
    mirror: Mirror,
) -> GridPos {
    let min = transform_min(definition, rotation, mirror);
    let raw = transform_raw(definition, point, rotation, mirror);
    raw.subtract(min)
}

fn transform_min(definition: &ComponentDefinition, rotation: Rotation, mirror: Mirror) -> GridPos {
    let width = definition.size.width;
    let height = definition.size.height;
    let corners = [
        GridPos::new(0, 0),
        GridPos::new(width, 0),
        GridPos::new(0, height),
        GridPos::new(width, height),
    ];
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    for corner in corners {
        let raw = transform_raw(definition, corner, rotation, mirror);
        min_x = min_x.min(raw.x);
        min_y = min_y.min(raw.y);
    }
    GridPos::new(min_x, min_y)
}

fn transform_raw(
    definition: &ComponentDefinition,
    point: GridPos,
    rotation: Rotation,
    mirror: Mirror,
) -> GridPos {
    let center = GridPos::new(definition.size.width / 2, definition.size.height / 2);
    let delta = point.subtract(center);
    let mirrored = mirror.apply(delta);
    let rotated = rotation.apply(mirrored);
    center.offset(rotated)
}

/// Looks up a definition by ID.
#[must_use]
pub fn find_definition<'a>(
    library: &'a [ComponentDefinition],
    id: &str,
) -> Option<&'a ComponentDefinition> {
    library.iter().find(|definition| definition.id == id)
}

fn is_orthogonal(start: GridPos, end: GridPos) -> bool {
    start.x == end.x || start.y == end.y
}

fn wire_edges(points: &[GridPos]) -> Option<BTreeSet<GridEdge>> {
    if points.len() < 2 {
        return None;
    }
    let mut edges = BTreeSet::new();
    for segment in points.windows(2) {
        let start = segment[0];
        let end = segment[1];
        if start == end || !is_orthogonal(start, end) {
            return None;
        }
        for edge in segment_edges(start, end) {
            if !edges.insert(edge) {
                return None;
            }
        }
    }
    Some(edges)
}

fn segment_edges(start: GridPos, end: GridPos) -> Vec<GridEdge> {
    let length = segment_length(start, end);
    (0..length)
        .map(|step| {
            GridEdge::new(
                segment_step(start, end, step),
                segment_step(start, end, step + 1),
            )
        })
        .collect()
}

fn segment_distance(point: [f32; 2], start: GridPos, end: GridPos) -> f32 {
    let x = point[0];
    let y = point[1];
    if start.y == end.y {
        let min = start.x.min(end.x) as f32;
        let max = start.x.max(end.x) as f32;
        let nearest = x.clamp(min, max);
        return ((x - nearest).powi(2) + (y - start.y as f32).powi(2)).sqrt();
    }
    let min = start.y.min(end.y) as f32;
    let max = start.y.max(end.y) as f32;
    let nearest = y.clamp(min, max);
    ((x - start.x as f32).powi(2) + (y - nearest).powi(2)).sqrt()
}

fn segment_length(start: GridPos, end: GridPos) -> i32 {
    (end.x - start.x).abs() + (end.y - start.y).abs()
}

fn segment_step(start: GridPos, end: GridPos, step: i32) -> GridPos {
    let dx = (end.x - start.x).signum();
    let dy = (end.y - start.y).signum();
    GridPos::new(start.x + dx * step, start.y + dy * step)
}

#[cfg(test)]
mod tests {
    use super::{
        ComponentDefinition, Document, GridPos, GridSize, Mirror, NetClass, PinDefinition,
        Rotation, SymbolKind, body_size, world_pin,
    };
    use crate::library;

    const PINS: &[PinDefinition] = &[
        PinDefinition {
            name: "1",
            number: "1",
            offset: GridPos::new(0, 1),
        },
        PinDefinition {
            name: "2",
            number: "2",
            offset: GridPos::new(4, 1),
        },
    ];

    const RESISTOR: ComponentDefinition = ComponentDefinition {
        id: "resistor",
        name: "Resistor",
        category: super::Category::Passive,
        reference_prefix: "R",
        default_value: "220",
        size: GridSize::new(4, 2),
        pins: PINS,
        symbol: SymbolKind::Resistor,
    };

    const LIBRARY: &[ComponentDefinition] = &[RESISTOR];

    #[test]
    fn rotation_swaps_body_size() {
        assert_eq!(body_size(&RESISTOR, Rotation::Deg0).width, 4);
        assert_eq!(body_size(&RESISTOR, Rotation::Deg90).width, 2);
        assert_eq!(body_size(&RESISTOR, Rotation::Deg90).height, 4);
    }

    #[test]
    fn pins_stay_on_grid_after_transform() {
        for rotation in [
            Rotation::Deg0,
            Rotation::Deg90,
            Rotation::Deg180,
            Rotation::Deg270,
        ] {
            for mirror in [Mirror::None, Mirror::Horizontal, Mirror::Vertical] {
                let mut document = Document::new();
                let id = document.add(&RESISTOR, GridPos::new(10, 10), rotation, mirror);
                let instance = document.component(id).expect("instance");
                for pin in PINS {
                    let position = world_pin(instance, &RESISTOR, pin);
                    assert!(position.x >= 10 && position.x <= 14);
                    assert!(position.y >= 10 && position.y <= 14);
                }
            }
        }
    }

    #[test]
    fn overlap_rejects_interior_and_allows_touching_edge() {
        let mut document = Document::new();
        document.add(&RESISTOR, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        assert!(!document.can_place(
            LIBRARY,
            &RESISTOR,
            GridPos::new(3, 0),
            Rotation::Deg0,
            Mirror::None
        ));
        assert!(document.can_place(
            LIBRARY,
            &RESISTOR,
            GridPos::new(4, 0),
            Rotation::Deg0,
            Mirror::None
        ));
    }

    #[test]
    fn add_generates_unique_labels() {
        let mut document = Document::new();
        let first = document.add(&RESISTOR, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        let second = document.add(&RESISTOR, GridPos::new(5, 0), Rotation::Deg0, Mirror::None);
        assert_eq!(document.component(first).expect("first").label, "R1");
        assert_eq!(document.component(second).expect("second").label, "R2");
    }

    #[test]
    fn hit_test_finds_topmost_component() {
        let mut document = Document::new();
        let id = document.add(&RESISTOR, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        assert_eq!(document.hit_test(LIBRARY, GridPos::new(2, 1)), Some(id));
        assert_eq!(document.hit_test(LIBRARY, GridPos::new(10, 10)), None);
    }

    #[test]
    fn accepts_wire_between_pins() {
        let library = library::builtin();
        let resistor = library::find("resistor").expect("resistor");
        let mut document = Document::new();
        document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
        let wire = document.add_wire(
            library,
            vec![GridPos::new(4, 1), GridPos::new(6, 1)],
            NetClass::Digital,
        );
        assert!(wire.is_some());
    }

    #[test]
    fn rejects_wire_through_body() {
        let library = library::builtin();
        let resistor = library::find("resistor").expect("resistor");
        let mut document = Document::new();
        document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        let wire = document.add_wire(
            library,
            vec![GridPos::new(0, 1), GridPos::new(4, 1)],
            NetClass::Digital,
        );
        assert!(wire.is_none());
    }

    #[test]
    fn rejects_diagonal_segment() {
        let document = Document::new();
        assert!(!document.can_add_segment(
            library::builtin(),
            GridPos::new(4, 1),
            GridPos::new(6, 2)
        ));
    }

    #[test]
    fn rejects_duplicate_edge() {
        let library = library::builtin();
        let resistor = library::find("resistor").expect("resistor");
        let mut document = Document::new();
        document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
        let points = vec![GridPos::new(4, 1), GridPos::new(6, 1)];
        assert!(
            document
                .add_wire(library, points.clone(), NetClass::Digital)
                .is_some()
        );
        assert!(
            document
                .add_wire(library, points, NetClass::Digital)
                .is_none()
        );
    }

    #[test]
    fn wire_hit_test_uses_tolerance() {
        let library = library::builtin();
        let resistor = library::find("resistor").expect("resistor");
        let mut document = Document::new();
        document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
        let wire = document
            .add_wire(
                library,
                vec![GridPos::new(4, 1), GridPos::new(6, 1)],
                NetClass::Digital,
            )
            .expect("wire");
        assert_eq!(document.wire_at([5.0, 1.2], 0.3), Some(wire));
        assert_eq!(document.wire_at([5.0, 2.0], 0.3), None);
    }

    fn segment_document() -> (Document, super::WireId) {
        let library = library::builtin();
        let resistor = library::find("resistor").expect("resistor");
        let mut document = Document::new();
        document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
        let points = vec![
            GridPos::new(4, 1),
            GridPos::new(4, 0),
            GridPos::new(6, 0),
            GridPos::new(6, 1),
        ];
        let wire = document
            .add_wire(library, points, NetClass::Digital)
            .expect("wire");
        (document, wire)
    }

    #[test]
    fn update_wire_points_moves_segment() {
        let library = library::builtin();
        let (mut document, wire) = segment_document();
        let moved = vec![
            GridPos::new(4, 1),
            GridPos::new(4, 2),
            GridPos::new(6, 2),
            GridPos::new(6, 1),
        ];
        assert!(document.update_wire_points(library, wire, moved.clone()));
        assert_eq!(document.wires()[0].points, moved);
        let invalid = vec![
            GridPos::new(4, 1),
            GridPos::new(4, 1),
            GridPos::new(6, 1),
            GridPos::new(6, 1),
        ];
        assert!(!document.update_wire_points(library, wire, invalid));
    }

    fn junction_document() -> Document {
        let library = library::builtin();
        let resistor = library::find("resistor").expect("resistor");
        let vcc = library::find("vcc").expect("vcc");
        let mut document = Document::new();
        document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
        document.add(vcc, GridPos::new(4, -2), Rotation::Deg0, Mirror::None);
        add_junction_wires(&mut document, library);
        document
    }

    fn add_junction_wires(document: &mut Document, library: &[super::ComponentDefinition]) {
        let horizontal = vec![GridPos::new(4, 1), GridPos::new(5, 1), GridPos::new(6, 1)];
        let vertical = vec![GridPos::new(5, 1), GridPos::new(5, 0)];
        assert!(
            document
                .add_wire(library, horizontal, NetClass::Digital)
                .is_some()
        );
        assert!(
            document
                .add_wire(library, vertical, NetClass::Power)
                .is_some()
        );
    }

    #[test]
    fn junction_detected_at_three_segments() {
        let document = junction_document();
        assert_eq!(document.junctions(), vec![GridPos::new(5, 1)]);
    }
}
