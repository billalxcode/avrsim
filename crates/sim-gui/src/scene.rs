//! Canvas rendering for components and placement ghosts.

use crate::library;
use crate::model::{
    ComponentDefinition, ComponentId, Document, GridPos, Mirror, NetClass, Rotation, WireId,
    body_size, pin_direction, world_pin,
};
use eframe::egui;

const BODY_FILL: egui::Color32 = egui::Color32::from_rgb(31, 35, 43);
const BODY_STROKE: egui::Color32 = egui::Color32::from_rgb(135, 145, 160);
const SELECTED_STROKE: egui::Color32 = egui::Color32::from_rgb(0, 200, 220);
const SYMBOL_STROKE: egui::Color32 = egui::Color32::from_rgb(190, 198, 210);
const PIN_COLOR: egui::Color32 = egui::Color32::from_rgb(215, 220, 230);
const TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(200, 205, 215);
const JUNCTION_COLOR: egui::Color32 = egui::Color32::from_rgb(230, 235, 245);
const GHOST_OK: egui::Color32 = egui::Color32::from_rgb(60, 210, 130);
const GHOST_BAD: egui::Color32 = egui::Color32::from_rgb(230, 80, 80);
const PREVIEW_BAD: egui::Color32 = egui::Color32::from_rgb(240, 90, 90);

/// Immutable scene used by the canvas renderer.
#[derive(Clone, Copy)]
pub struct Scene<'a> {
    pub document: &'a Document,
    pub library: &'a [ComponentDefinition],
    pub selected: Option<ComponentId>,
    pub selected_wire: Option<WireId>,
    pub placing: Option<&'a ComponentDefinition>,
    pub hover: Option<GridPos>,
    pub rotation: Rotation,
    pub mirror: Mirror,
    pub wire_preview: &'a [GridPos],
    pub wire_hover: Option<GridPos>,
    pub wire_class: NetClass,
}

/// Draws all wires, components, and the placement ghost.
pub fn draw(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: Scene<'_>,
) {
    draw_wires(painter, rect, camera, scene.document, scene.selected_wire);
    draw_junctions(painter, rect, camera, scene.document);
    draw_components(painter, rect, camera, &scene);
    draw_wire_handles(painter, rect, camera, &scene);
    draw_wire_preview(painter, rect, camera, &scene);
    if let Some(definition) = scene.placing {
        if let Some(hover) = scene.hover {
            draw_ghost(painter, rect, camera, &scene, definition, hover);
        }
    }
}

fn draw_components(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
) {
    for instance in scene.document.components() {
        let Some(definition) = library::find(instance.definition) else {
            continue;
        };
        let selected = scene.selected == Some(instance.id);
        draw_component(painter, rect, camera, definition, instance, selected);
    }
}

fn draw_wire_handles(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
) {
    let Some(id) = scene.selected_wire else {
        return;
    };
    let Some(wire) = scene.document.wires().iter().find(|wire| wire.id == id) else {
        return;
    };
    for segment in wire.points.windows(2) {
        let midpoint = segment_midpoint(segment[0], segment[1]);
        let screen = camera.world_to_screen(rect, midpoint);
        draw_handle(painter, screen);
    }
}

fn segment_midpoint(first: GridPos, second: GridPos) -> egui::Pos2 {
    egui::pos2(
        (first.x + second.x) as f32 * 0.5,
        (first.y + second.y) as f32 * 0.5,
    )
}

fn draw_handle(painter: &egui::Painter, screen: egui::Pos2) {
    let handle = egui::Rect::from_center_size(screen, egui::vec2(8.0, 8.0));
    painter.rect_filled(handle, egui::CornerRadius::ZERO, SELECTED_STROKE);
    painter.rect_stroke(
        handle,
        egui::CornerRadius::ZERO,
        egui::Stroke::new(1.0, JUNCTION_COLOR),
        egui::StrokeKind::Middle,
    );
}

fn draw_wires(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    document: &Document,
    selected: Option<WireId>,
) {
    let mut wires: Vec<_> = document.wires().iter().collect();
    wires.sort_by_key(|wire| wire.id);
    for wire in wires {
        draw_wire(painter, rect, camera, wire, selected == Some(wire.id));
    }
}

fn draw_wire(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    wire: &crate::model::Wire,
    selected: bool,
) {
    if selected {
        let halo = SELECTED_STROKE.gamma_multiply(0.6);
        draw_polyline(painter, rect, camera, &wire.points, halo, 6.0);
    }
    let color = net_color(wire.class);
    draw_polyline(painter, rect, camera, &wire.points, color, 2.0);
}

fn draw_junctions(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    document: &Document,
) {
    for point in document.junctions() {
        painter.circle_filled(screen_pos(rect, camera, point), 3.2, JUNCTION_COLOR);
    }
}

fn draw_wire_preview(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
) {
    if scene.wire_preview.is_empty() {
        return;
    }
    let color = net_color(scene.wire_class).gamma_multiply(0.8);
    draw_polyline(painter, rect, camera, scene.wire_preview, color, 2.0);
    let Some(hover) = scene.wire_hover else {
        return;
    };
    let Some(&last) = scene.wire_preview.last() else {
        return;
    };
    if last == hover {
        return;
    }
    let color = if scene.document.can_add_segment(scene.library, last, hover) {
        color
    } else {
        PREVIEW_BAD
    };
    draw_segment(painter, rect, camera, last, hover, color, 2.0);
}

fn draw_polyline(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    points: &[GridPos],
    color: egui::Color32,
    width: f32,
) {
    for segment in points.windows(2) {
        draw_segment(painter, rect, camera, segment[0], segment[1], color, width);
    }
}

fn draw_segment(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    start: GridPos,
    end: GridPos,
    color: egui::Color32,
    width: f32,
) {
    let start = screen_pos(rect, camera, start);
    let end = screen_pos(rect, camera, end);
    painter.line_segment([start, end], egui::Stroke::new(width, color));
}

fn net_color(class: NetClass) -> egui::Color32 {
    match class {
        NetClass::Power => egui::Color32::from_rgb(225, 85, 85),
        NetClass::Ground => egui::Color32::from_rgb(125, 130, 140),
        NetClass::Digital => egui::Color32::from_rgb(85, 205, 125),
        NetClass::Analog => egui::Color32::from_rgb(90, 155, 240),
        NetClass::Clock => egui::Color32::from_rgb(240, 165, 75),
        NetClass::Uart => egui::Color32::from_rgb(80, 215, 225),
        NetClass::Spi => egui::Color32::from_rgb(185, 125, 235),
        NetClass::I2c => egui::Color32::from_rgb(230, 210, 95),
        NetClass::Pwm => egui::Color32::from_rgb(230, 115, 195),
        NetClass::Reset => egui::Color32::from_rgb(185, 135, 95),
        NetClass::User => egui::Color32::from_rgb(195, 200, 210),
    }
}

fn draw_component(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    definition: &ComponentDefinition,
    instance: &crate::model::ComponentInstance,
    selected: bool,
) {
    let body = component_rect(rect, camera, instance, definition);
    draw_body(painter, body, component_stroke(selected));
    draw_symbol(painter, body, definition.symbol);
    draw_pins(painter, rect, camera, definition, instance);
    draw_labels(painter, body, instance);
}

fn component_stroke(selected: bool) -> egui::Stroke {
    if selected {
        egui::Stroke::new(2.0, SELECTED_STROKE)
    } else {
        egui::Stroke::new(1.0, BODY_STROKE)
    }
}

fn draw_body(painter: &egui::Painter, body: egui::Rect, stroke: egui::Stroke) {
    painter.rect_filled(body, egui::CornerRadius::ZERO, BODY_FILL);
    painter.rect_stroke(
        body,
        egui::CornerRadius::ZERO,
        stroke,
        egui::StrokeKind::Middle,
    );
}

fn draw_symbol(painter: &egui::Painter, body: egui::Rect, symbol: crate::model::SymbolKind) {
    let stroke = egui::Stroke::new(1.5, SYMBOL_STROKE);
    match symbol {
        crate::model::SymbolKind::Resistor => draw_resistor(painter, body, stroke),
        crate::model::SymbolKind::Led => draw_led(painter, body, stroke),
        crate::model::SymbolKind::Button => draw_button(painter, body, stroke),
        crate::model::SymbolKind::Vcc => draw_vcc(painter, body, stroke),
        crate::model::SymbolKind::Gnd => draw_gnd(painter, body, stroke),
        crate::model::SymbolKind::Mcu => draw_mcu(painter, body, stroke),
    }
}

fn draw_resistor(painter: &egui::Painter, body: egui::Rect, stroke: egui::Stroke) {
    let inset = body.shrink2(egui::vec2(body.width() * 0.18, body.height() * 0.22));
    painter.rect_stroke(
        inset,
        egui::CornerRadius::ZERO,
        stroke,
        egui::StrokeKind::Middle,
    );
}

fn draw_led(painter: &egui::Painter, body: egui::Rect, stroke: egui::Stroke) {
    let center = body.center();
    let top = center + egui::vec2(0.0, -body.height() * 0.22);
    let bottom = center + egui::vec2(0.0, body.height() * 0.22);
    let left = center + egui::vec2(-body.width() * 0.16, 0.0);
    let right = center + egui::vec2(body.width() * 0.16, 0.0);
    painter.add(egui::Shape::convex_polygon(
        vec![left, right, bottom],
        SYMBOL_STROKE,
        stroke,
    ));
    painter.line_segment([top, bottom], stroke);
}

fn draw_button(painter: &egui::Painter, body: egui::Rect, stroke: egui::Stroke) {
    let center = body.center();
    let left = center + egui::vec2(-body.width() * 0.18, 0.0);
    let right = center + egui::vec2(body.width() * 0.18, 0.0);
    let top = center + egui::vec2(0.0, -body.height() * 0.2);
    painter.line_segment([left, right], stroke);
    painter.line_segment(
        [top + egui::vec2(-6.0, 0.0), top + egui::vec2(6.0, 0.0)],
        stroke,
    );
}

fn draw_vcc(painter: &egui::Painter, body: egui::Rect, stroke: egui::Stroke) {
    let center = body.center();
    painter.circle_stroke(center, body.height() * 0.18, stroke);
    painter.line_segment(
        [
            center + egui::vec2(0.0, body.height() * 0.18),
            center + egui::vec2(0.0, body.height() * 0.4),
        ],
        stroke,
    );
}

fn draw_gnd(painter: &egui::Painter, body: egui::Rect, stroke: egui::Stroke) {
    let center = body.center();
    for (index, width) in [0.35, 0.24, 0.13].iter().enumerate() {
        let y = center.y + index as f32 * 6.0 - 6.0;
        painter.line_segment(
            [
                egui::pos2(center.x - body.width() * width, y),
                egui::pos2(center.x + body.width() * width, y),
            ],
            stroke,
        );
    }
}

fn draw_mcu(painter: &egui::Painter, body: egui::Rect, stroke: egui::Stroke) {
    let notch = body.center_top() + egui::vec2(-body.width() * 0.12, 0.0);
    painter.circle_stroke(notch, body.width() * 0.08, stroke);
    painter.line_segment(
        [
            body.center_top() + egui::vec2(0.0, body.height() * 0.08),
            body.center_bottom() - egui::vec2(0.0, body.height() * 0.08),
        ],
        egui::Stroke::new(1.0, BODY_STROKE),
    );
}

fn draw_pins(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    definition: &ComponentDefinition,
    instance: &crate::model::ComponentInstance,
) {
    let stroke = egui::Stroke::new(1.5, PIN_COLOR);
    for pin in definition.pins {
        draw_pin(painter, rect, camera, definition, instance, pin, stroke);
    }
}

fn draw_pin(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    definition: &ComponentDefinition,
    instance: &crate::model::ComponentInstance,
    pin: &crate::model::PinDefinition,
    stroke: egui::Stroke,
) {
    let world = world_pin(instance, definition, pin);
    let screen = screen_pos(rect, camera, world);
    let direction = pin_direction(definition, pin, instance.rotation, instance.mirror);
    let stub = pin_stub(rect, camera, world, direction);
    painter.line_segment([screen, stub], stroke);
    painter.circle_filled(stub, 2.0, PIN_COLOR);
    if camera.zoom >= 0.75 {
        draw_pin_label(painter, stub, pin, direction);
    }
}

fn pin_stub(
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    world: GridPos,
    direction: GridPos,
) -> egui::Pos2 {
    let stub = GridPos::new(world.x + direction.x, world.y + direction.y);
    screen_pos(rect, camera, stub)
}

fn draw_pin_label(
    painter: &egui::Painter,
    position: egui::Pos2,
    pin: &crate::model::PinDefinition,
    direction: GridPos,
) {
    let offset = egui::vec2(direction.x as f32 * 5.0, direction.y as f32 * 5.0);
    painter.text(
        position + offset,
        pin_anchor(direction),
        pin.name,
        egui::FontId::monospace(9.0),
        TEXT_COLOR,
    );
}

fn pin_anchor(direction: GridPos) -> egui::Align2 {
    match (direction.x, direction.y) {
        (-1, _) => egui::Align2::RIGHT_CENTER,
        (1, _) => egui::Align2::LEFT_CENTER,
        (_, -1) => egui::Align2::CENTER_BOTTOM,
        _ => egui::Align2::CENTER_TOP,
    }
}

fn draw_labels(
    painter: &egui::Painter,
    body: egui::Rect,
    instance: &crate::model::ComponentInstance,
) {
    let text = format!("{} {}", instance.label, instance.value);
    painter.text(
        body.center_bottom() + egui::vec2(0.0, 4.0),
        egui::Align2::CENTER_TOP,
        text,
        egui::FontId::monospace(10.0),
        TEXT_COLOR,
    );
}

fn draw_ghost(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
    definition: &ComponentDefinition,
    hover: GridPos,
) {
    let position = placement_position(definition, hover, scene.rotation);
    let color = ghost_color(scene, definition, position);
    let body = rect_for(
        rect,
        camera,
        position,
        body_size(definition, scene.rotation),
    );
    draw_ghost_body(painter, body, color);
}

fn ghost_color(
    scene: &Scene<'_>,
    definition: &ComponentDefinition,
    position: GridPos,
) -> egui::Color32 {
    let valid = scene.document.can_place(
        scene.library,
        definition,
        position,
        scene.rotation,
        scene.mirror,
    );
    if valid { GHOST_OK } else { GHOST_BAD }
}

fn draw_ghost_body(painter: &egui::Painter, body: egui::Rect, color: egui::Color32) {
    painter.rect_filled(body, egui::CornerRadius::ZERO, color.gamma_multiply(0.18));
    painter.rect_stroke(
        body,
        egui::CornerRadius::ZERO,
        egui::Stroke::new(2.0, color),
        egui::StrokeKind::Middle,
    );
}

fn component_rect(
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    instance: &crate::model::ComponentInstance,
    definition: &ComponentDefinition,
) -> egui::Rect {
    rect_for(
        rect,
        camera,
        instance.position,
        body_size(definition, instance.rotation),
    )
}

fn rect_for(
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    position: GridPos,
    size: crate::model::GridSize,
) -> egui::Rect {
    let min = camera.world_to_screen(rect, egui::pos2(position.x as f32, position.y as f32));
    let max = camera.world_to_screen(
        rect,
        egui::pos2(
            (position.x + size.width) as f32,
            (position.y + size.height) as f32,
        ),
    );
    egui::Rect::from_min_max(min, max)
}

fn screen_pos(
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    position: GridPos,
) -> egui::Pos2 {
    camera.world_to_screen(rect, egui::pos2(position.x as f32, position.y as f32))
}

/// Centers a component on the hovered grid point.
#[must_use]
pub fn placement_position(
    definition: &ComponentDefinition,
    hover: GridPos,
    rotation: Rotation,
) -> GridPos {
    let size = body_size(definition, rotation);
    hover.subtract(GridPos::new(size.width / 2, size.height / 2))
}

#[cfg(test)]
mod tests {
    use super::placement_position;
    use crate::library;
    use crate::model::{GridPos, Rotation};

    #[test]
    fn placement_centers_component() {
        let definition = library::find("resistor").expect("resistor");
        let position = placement_position(definition, GridPos::new(10, 10), Rotation::Deg0);
        assert_eq!(position, GridPos::new(8, 9));
    }

    #[test]
    fn placement_uses_rotated_size() {
        let definition = library::find("resistor").expect("resistor");
        let position = placement_position(definition, GridPos::new(10, 10), Rotation::Deg90);
        assert_eq!(position, GridPos::new(9, 8));
    }
}
