//! Canvas rendering for components and placement ghosts.

use crate::library;
use crate::model::{
    ComponentDefinition, Document, GridPos, Mirror, NetClass, Rotation, WireId, body_size,
    pin_direction, world_pin,
};
use crate::selection::Selection;
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

#[derive(Clone, Copy)]
struct WireStyle {
    color: egui::Color32,
    width: f32,
    dash: Option<(f32, f32)>,
    flow_phase: Option<f32>,
}

/// Immutable scene used by the canvas renderer.
#[derive(Clone, Copy)]
pub struct Scene<'a> {
    pub document: &'a Document,
    pub library: &'a [ComponentDefinition],
    pub selection: &'a Selection,
    pub hover_wire: Option<WireId>,
    pub placing: Option<&'a ComponentDefinition>,
    pub rotation: Rotation,
    pub mirror: Mirror,
    pub wire_preview: &'a [GridPos],
    pub wire_class: NetClass,
    pub selection_box: Option<(GridPos, GridPos)>,
    pub minimap: bool,
    pub flow_phase: Option<f32>,
}

/// Draws all wires, components, and the placement ghost.
pub fn draw(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: Scene<'_>,
    hover: Option<GridPos>,
) {
    draw_wire_layer(painter, rect, camera, &scene);
    draw_junctions(painter, rect, camera, scene.document);
    draw_components(painter, rect, camera, &scene, hover);
    draw_wire_handles(painter, rect, camera, &scene);
    draw_wire_preview(painter, rect, camera, &scene, hover);
    draw_ghost_layer(painter, rect, camera, &scene, hover);
    draw_selection_box(painter, rect, camera, scene.selection_box);
    if scene.minimap {
        draw_minimap(painter, rect, camera, &scene);
    }
}

fn draw_wire_layer(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
) {
    draw_wires(
        painter,
        rect,
        camera,
        scene.document,
        scene.selection,
        scene.hover_wire,
        scene.flow_phase,
    );
}

fn draw_ghost_layer(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
    hover: Option<GridPos>,
) {
    let Some(definition) = scene.placing else {
        return;
    };
    let Some(hover) = hover else {
        return;
    };
    draw_ghost(painter, rect, camera, scene, definition, hover);
}

fn draw_selection_box(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    corners: Option<(GridPos, GridPos)>,
) {
    let Some((start, end)) = corners else {
        return;
    };
    let box_rect = screen_box(rect, camera, start, end);
    let fill = egui::Color32::from_rgba_unmultiplied(0, 200, 220, 28);
    painter.rect_filled(box_rect, 0.0, fill);
    painter.rect_stroke(
        box_rect,
        0.0,
        egui::Stroke::new(1.0, SELECTED_STROKE),
        egui::StrokeKind::Middle,
    );
}

#[derive(Clone, Copy)]
struct MinimapMap {
    origin: egui::Pos2,
    scale: f32,
}

impl MinimapMap {
    fn point(self, point: GridPos) -> egui::Pos2 {
        self.origin + egui::vec2(point.x as f32, point.y as f32) * self.scale
    }
}

fn draw_minimap(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
) {
    if rect.width() < 360.0 || rect.height() < 260.0 {
        return;
    }
    let area = minimap_area(rect);
    paint_minimap_frame(painter, area);
    let Some(map) = minimap_for(area, scene) else {
        return;
    };
    draw_minimap_wires(painter, map, scene.document);
    draw_minimap_components(painter, map, scene.document, scene.library);
    draw_minimap_viewport(painter, map, rect, camera);
}

fn minimap_area(rect: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(rect.right() - 172.0, rect.bottom() - 122.0),
        egui::vec2(160.0, 110.0),
    )
}

fn paint_minimap_frame(painter: &egui::Painter, area: egui::Rect) {
    painter.rect_filled(area, 4.0, egui::Color32::from_black_alpha(205));
    painter.rect_stroke(
        area,
        4.0,
        egui::Stroke::new(1.0, JUNCTION_COLOR),
        egui::StrokeKind::Middle,
    );
}

fn minimap_for(area: egui::Rect, scene: &Scene<'_>) -> Option<MinimapMap> {
    let (min, max) = scene.document.bounds(scene.library)?;
    minimap_map(area, min, max)
}

fn minimap_map(area: egui::Rect, min: GridPos, max: GridPos) -> Option<MinimapMap> {
    let inner = area.shrink(6.0);
    let width = (max.x - min.x).max(1) as f32;
    let height = (max.y - min.y).max(1) as f32;
    let scale = (inner.width() / width).min(inner.height() / height);
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    let center = egui::vec2((min.x + max.x) as f32 * 0.5, (min.y + max.y) as f32 * 0.5);
    Some(MinimapMap {
        origin: inner.center() - center * scale,
        scale,
    })
}

fn draw_minimap_wires(painter: &egui::Painter, map: MinimapMap, document: &Document) {
    for wire in document.wires() {
        let color = wire_color(wire).gamma_multiply(0.8);
        for segment in wire.points.windows(2) {
            painter.line_segment(
                [map.point(segment[0]), map.point(segment[1])],
                egui::Stroke::new(1.0, color),
            );
        }
    }
}

fn draw_minimap_components(
    painter: &egui::Painter,
    map: MinimapMap,
    document: &Document,
    definitions: &[ComponentDefinition],
) {
    for instance in document.components() {
        let Some(definition) = definitions
            .iter()
            .find(|definition| definition.id == instance.definition)
        else {
            continue;
        };
        painter.rect_filled(
            minimap_component_rect(map, instance, definition),
            1.0,
            egui::Color32::from_rgb(120, 130, 150),
        );
    }
}

fn minimap_component_rect(
    map: MinimapMap,
    instance: &crate::model::ComponentInstance,
    definition: &ComponentDefinition,
) -> egui::Rect {
    let bounds = instance.bounds(definition);
    let min = map.point(bounds.min);
    let max = map.point(GridPos::new(bounds.max_x(), bounds.max_y()));
    egui::Rect::from_min_max(min, max)
}

fn draw_minimap_viewport(
    painter: &egui::Painter,
    map: MinimapMap,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
) {
    let min = camera.screen_to_world(rect, rect.min);
    let max = camera.screen_to_world(rect, rect.max);
    let min = map.point(GridPos::new(min.x.round() as i32, min.y.round() as i32));
    let max = map.point(GridPos::new(max.x.round() as i32, max.y.round() as i32));
    painter.rect_stroke(
        egui::Rect::from_two_pos(min, max),
        0.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(240, 220, 120)),
        egui::StrokeKind::Middle,
    );
}

fn screen_box(
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    start: GridPos,
    end: GridPos,
) -> egui::Rect {
    let first = screen_pos(rect, camera, start);
    let second = screen_pos(rect, camera, end);
    egui::Rect::from_two_pos(first, second)
}

fn draw_components(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
    hover: Option<GridPos>,
) {
    for instance in scene.document.components() {
        draw_component_instance(painter, rect, camera, scene, instance, hover);
    }
}

fn draw_component_instance(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
    instance: &crate::model::ComponentInstance,
    hover: Option<GridPos>,
) {
    let Some(definition) = library::find(instance.definition) else {
        return;
    };
    let body = component_rect(rect, camera, instance, definition);
    if !rect.intersects(body) {
        return;
    }
    let selected = scene.selection.contains_component(instance.id);
    draw_component(painter, rect, camera, definition, instance, selected, hover);
}

fn draw_wire_handles(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
) {
    for wire in scene.document.wires() {
        if !scene.selection.contains_wire(wire.id) {
            continue;
        }
        for segment in wire.points.windows(2) {
            let midpoint = segment_midpoint(segment[0], segment[1]);
            let screen = camera.world_to_screen(rect, midpoint);
            draw_handle(painter, screen);
        }
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
    selection: &Selection,
    hovered: Option<WireId>,
    flow_phase: Option<f32>,
) {
    draw_wire_lines(painter, rect, camera, document, selection, flow_phase);
    draw_wire_labels(painter, rect, camera, document, selection, hovered);
}

fn draw_wire_lines(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    document: &Document,
    selection: &Selection,
    flow_phase: Option<f32>,
) {
    for wire in document.wires() {
        if !wire_visible(rect, camera, wire) {
            continue;
        }
        let selected = selection.contains_wire(wire.id);
        draw_wire(painter, rect, camera, wire, selected, flow_phase);
    }
}

fn draw_wire_labels(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    document: &Document,
    selection: &Selection,
    hovered: Option<WireId>,
) {
    for wire in document.wires() {
        if !wire_visible(rect, camera, wire) {
            continue;
        }
        let selected = selection.contains_wire(wire.id);
        if show_wire_label(selected, hovered == Some(wire.id), camera.zoom) {
            draw_net_label(painter, rect, camera, wire);
        }
    }
}

fn show_wire_label(selected: bool, hovered: bool, zoom: f32) -> bool {
    selected || hovered || zoom >= 1.5
}

fn draw_net_label(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    wire: &crate::model::Wire,
) {
    let Some(midpoint) = wire_midpoint(wire) else {
        return;
    };
    let pos = camera.world_to_screen(rect, midpoint) + egui::vec2(4.0, -16.0);
    let color = wire_color(wire);
    let galley = painter.layout_no_wrap(
        wire_label_text(wire).to_string(),
        egui::FontId::monospace(11.0),
        color,
    );
    let background = egui::Rect::from_min_size(pos, galley.size()).expand(3.0);
    painter.rect_filled(background, 3.0, egui::Color32::from_black_alpha(190));
    painter.galley(pos, galley, color);
}

fn wire_midpoint(wire: &crate::model::Wire) -> Option<egui::Pos2> {
    let index = wire.points.len().saturating_sub(1) / 2;
    let segment = wire.points.windows(2).nth(index)?;
    Some(segment_midpoint(segment[0], segment[1]))
}

fn draw_wire(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    wire: &crate::model::Wire,
    selected: bool,
    flow_phase: Option<f32>,
) {
    if selected {
        let halo = SELECTED_STROKE.gamma_multiply(0.6);
        draw_polyline(painter, rect, camera, &wire.points, halo, 6.0);
        draw_wire_ends(painter, rect, camera, wire);
    }
    let style = wire_style(wire_color(wire), wire.class, flow_phase);
    draw_patterned_polyline(painter, rect, camera, &wire.points, style);
}

fn wire_style(color: egui::Color32, class: NetClass, flow_phase: Option<f32>) -> WireStyle {
    WireStyle {
        color,
        width: 2.0,
        dash: wire_dash(class),
        flow_phase,
    }
}

fn draw_wire_ends(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    wire: &crate::model::Wire,
) {
    for point in [wire.points.first(), wire.points.last()]
        .into_iter()
        .flatten()
    {
        let screen = screen_pos(rect, camera, *point);
        painter.circle_stroke(screen, 5.0, egui::Stroke::new(1.5, SELECTED_STROKE));
    }
}

fn draw_junctions(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    document: &Document,
) {
    for point in document.junctions() {
        let screen = screen_pos(rect, camera, point);
        if !rect.contains(screen) {
            continue;
        }
        painter.circle_filled(screen, 3.2, JUNCTION_COLOR);
    }
}

fn draw_wire_preview(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
    hover: Option<GridPos>,
) {
    if scene.wire_preview.is_empty() {
        return;
    }
    let color = net_color(scene.wire_class).gamma_multiply(0.8);
    let style = wire_style(color, scene.wire_class, None);
    draw_patterned_polyline(painter, rect, camera, scene.wire_preview, style);
    draw_preview_extension(painter, rect, camera, scene, hover, style);
}

fn draw_preview_extension(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
    hover: Option<GridPos>,
    mut style: WireStyle,
) {
    let Some((last, hover)) = preview_segment(scene, hover) else {
        return;
    };
    if !scene.document.can_add_segment(scene.library, last, hover) {
        style.color = PREVIEW_BAD;
    }
    draw_patterned_segment(painter, rect, camera, last, hover, style);
}

fn preview_segment(scene: &Scene<'_>, hover: Option<GridPos>) -> Option<(GridPos, GridPos)> {
    let hover = hover?;
    let &last = scene.wire_preview.last()?;
    if last == hover {
        return None;
    }
    Some((last, hover))
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

fn draw_patterned_polyline(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    points: &[GridPos],
    style: WireStyle,
) {
    for segment in points.windows(2) {
        draw_patterned_segment(painter, rect, camera, segment[0], segment[1], style);
    }
}

fn draw_patterned_segment(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    start: GridPos,
    end: GridPos,
    style: WireStyle,
) {
    draw_wire_base(painter, rect, camera, start, end, style);
    let Some(phase) = style.flow_phase else {
        return;
    };
    let [start, end] = screen_segment(rect, camera, start, end);
    paint_flow_overlay(painter, start, end, phase);
}

fn draw_wire_base(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    start: GridPos,
    end: GridPos,
    style: WireStyle,
) {
    let Some((dash_length, gap_length)) = style.dash else {
        draw_segment(painter, rect, camera, start, end, style.color, style.width);
        return;
    };
    let [start, end] = screen_segment(rect, camera, start, end);
    painter.extend(egui::Shape::dashed_line(
        &[start, end],
        egui::Stroke::new(style.width, style.color),
        dash_length,
        gap_length,
    ));
}

fn paint_flow_overlay(painter: &egui::Painter, start: egui::Pos2, end: egui::Pos2, phase: f32) {
    painter.extend(egui::Shape::dashed_line_with_offset(
        &[start, end],
        egui::Stroke::new(1.5, egui::Color32::from_white_alpha(90)),
        &[6.0],
        &[10.0],
        phase,
    ));
}

fn screen_segment(
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    start: GridPos,
    end: GridPos,
) -> [egui::Pos2; 2] {
    [
        screen_pos(rect, camera, start),
        screen_pos(rect, camera, end),
    ]
}

pub(crate) fn wire_dash(class: NetClass) -> Option<(f32, f32)> {
    match class {
        NetClass::Power | NetClass::Ground | NetClass::Digital => None,
        NetClass::Analog => Some((6.0, 4.0)),
        NetClass::Clock => Some((10.0, 3.0)),
        NetClass::Uart => Some((2.0, 3.0)),
        NetClass::Spi => Some((8.0, 3.0)),
        NetClass::I2c => Some((4.0, 4.0)),
        NetClass::Pwm => Some((12.0, 3.0)),
        NetClass::Reset => Some((3.0, 5.0)),
        NetClass::User => Some((5.0, 5.0)),
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

pub(crate) fn wire_color(wire: &crate::model::Wire) -> egui::Color32 {
    wire.color_override.map_or_else(
        || net_color(wire.class),
        |[red, green, blue]| egui::Color32::from_rgb(red, green, blue),
    )
}

pub(crate) fn wire_label_text(wire: &crate::model::Wire) -> &str {
    wire.label
        .as_deref()
        .filter(|label| !label.is_empty())
        .unwrap_or(wire.class.label())
}

pub(crate) fn net_color(class: NetClass) -> egui::Color32 {
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

pub(crate) fn draw_library_preview(
    painter: &egui::Painter,
    area: egui::Rect,
    definition: &ComponentDefinition,
) {
    let body = preview_rect(area, definition);
    draw_body(painter, body, egui::Stroke::new(1.0, BODY_STROKE));
    draw_symbol(painter, body, definition.symbol);
}

pub(crate) fn preview_rect(area: egui::Rect, definition: &ComponentDefinition) -> egui::Rect {
    let width = definition.size.width as f32 + 2.0;
    let height = definition.size.height as f32 + 2.0;
    let scale = (area.width() / width)
        .min(area.height() / height)
        .clamp(1.0, 24.0);
    let size = egui::vec2(
        definition.size.width as f32 * scale,
        definition.size.height as f32 * scale,
    );
    egui::Rect::from_center_size(area.center(), size)
}

fn draw_component(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    definition: &ComponentDefinition,
    instance: &crate::model::ComponentInstance,
    selected: bool,
    hover: Option<GridPos>,
) {
    let body = component_rect(rect, camera, instance, definition);
    draw_body(painter, body, component_stroke(selected));
    draw_symbol(painter, body, definition.symbol);
    draw_pins(painter, rect, camera, definition, instance, hover);
    if show_component_labels(selected, camera.zoom) {
        draw_labels(painter, body, instance);
    }
}

fn show_component_labels(selected: bool, zoom: f32) -> bool {
    selected || zoom >= 0.7
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
    hover: Option<GridPos>,
) {
    for pin in definition.pins {
        draw_pin(painter, rect, camera, definition, instance, pin, hover);
    }
}

fn draw_pin(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    definition: &ComponentDefinition,
    instance: &crate::model::ComponentInstance,
    pin: &crate::model::PinDefinition,
    hover: Option<GridPos>,
) {
    let world = world_pin(instance, definition, pin);
    let screen = screen_pos(rect, camera, world);
    let direction = pin_direction(definition, pin, instance.rotation, instance.mirror);
    let stub = pin_stub(rect, camera, world, direction);
    painter.line_segment([screen, stub], egui::Stroke::new(1.5, PIN_COLOR));
    painter.circle_filled(stub, 2.0, PIN_COLOR);
    let hovered = hover == Some(world) || hover == Some(world.offset(direction));
    draw_pin_details(painter, stub, pin, direction, camera.zoom, hovered);
}

fn draw_pin_details(
    painter: &egui::Painter,
    stub: egui::Pos2,
    pin: &crate::model::PinDefinition,
    direction: GridPos,
    zoom: f32,
    hovered: bool,
) {
    if zoom >= 0.75 {
        draw_pin_label(painter, stub, pin, direction);
    }
    if hovered {
        draw_pin_tooltip(painter, stub, pin, direction);
    }
}

fn draw_pin_tooltip(
    painter: &egui::Painter,
    position: egui::Pos2,
    pin: &crate::model::PinDefinition,
    direction: GridPos,
) {
    let text = pin_tooltip(pin, direction);
    let pos = position + egui::vec2(8.0, -22.0);
    let galley = painter.layout_no_wrap(text, egui::FontId::monospace(10.0), TEXT_COLOR);
    let background = egui::Rect::from_min_size(pos, galley.size()).expand(3.0);
    painter.rect_filled(background, 3.0, egui::Color32::from_black_alpha(210));
    painter.galley(pos, galley, TEXT_COLOR);
}

fn pin_tooltip(pin: &crate::model::PinDefinition, direction: GridPos) -> String {
    let side = match (direction.x, direction.y) {
        (-1, 0) => "left",
        (1, 0) => "right",
        (0, -1) => "up",
        (0, 1) => "down",
        _ => "pin",
    };
    format!("{} ({}) {side}", pin.name, pin.number)
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
        pin_label_text(pin),
        egui::FontId::monospace(9.0),
        TEXT_COLOR,
    );
}

fn pin_label_text(pin: &crate::model::PinDefinition) -> String {
    if pin.name == pin.number {
        return pin.name.to_string();
    }
    format!("{}/{}", pin.name, pin.number)
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

fn wire_visible(
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    wire: &crate::model::Wire,
) -> bool {
    wire.points
        .windows(2)
        .any(|segment| segment_visible(rect, camera, segment[0], segment[1]))
}

fn segment_visible(
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    start: GridPos,
    end: GridPos,
) -> bool {
    let start = screen_pos(rect, camera, start);
    let end = screen_pos(rect, camera, end);
    egui::Rect::from_two_pos(start, end).intersects(rect)
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
    use super::{
        minimap_map, net_color, pin_label_text, pin_tooltip, placement_position, preview_rect,
        segment_visible, show_component_labels, show_wire_label, wire_midpoint,
    };
    use crate::library;
    use crate::model::{Document, GridPos, Mirror, NetClass, Rotation, Wire, WireId};
    use crate::selection::Selection;

    #[test]
    fn component_labels_use_lod_except_selection() {
        assert!(show_component_labels(true, 0.4));
        assert!(show_component_labels(false, 0.7));
        assert!(!show_component_labels(false, 0.69));
    }

    #[test]
    fn pin_tooltip_reports_name_and_side() {
        let pin = crate::model::PinDefinition {
            name: "A",
            number: "1",
            offset: GridPos::new(0, 1),
        };
        assert_eq!(pin_tooltip(&pin, GridPos::new(-1, 0)), "A (1) left");
        assert_eq!(pin_label_text(&pin), "A/1");
    }

    #[test]
    fn preview_rect_fits_library_area() {
        let definition = library::find("resistor").expect("resistor");
        let area = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(0.0, 0.0),
            eframe::egui::vec2(100.0, 60.0),
        );
        let body = preview_rect(area, definition);
        assert!(area.contains_rect(body));
        assert!((body.width() / body.height() - 2.0).abs() < 0.01);
    }

    #[test]
    fn minimap_centers_world_bounds() {
        let area = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(0.0, 0.0),
            eframe::egui::vec2(160.0, 110.0),
        );
        let map = minimap_map(area, GridPos::new(0, 0), GridPos::new(10, 10)).expect("map");
        let center = map.point(GridPos::new(5, 5));
        assert!((center - area.center()).length() < 1e-4);
        assert!(map.scale > 0.0);
    }

    #[test]
    fn scene_renders_in_egui_context() {
        let output = sample_render_with(None);
        assert!(!output.shapes.is_empty());
    }

    #[test]
    fn flow_overlay_renders_in_egui_context() {
        let output = sample_render_with(Some(12.0));
        assert!(!output.shapes.is_empty());
    }

    fn sample_render_with(flow_phase: Option<f32>) -> eframe::egui::FullOutput {
        let library = library::builtin();
        let resistor = crate::library::find("resistor").expect("resistor");
        let mut document = Document::new();
        document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        let selection = Selection::default();
        let camera = crate::canvas::GridCamera::default();
        let mut scene = sample_scene(&document, library, &selection);
        scene.flow_phase = flow_phase;
        let input = test_input();
        let mut output = eframe::egui::Context::default().run_ui(input, |ui| {
            render_into(ui, scene, &camera);
        });
        output.textures_delta.clear();
        output
    }

    fn test_input() -> eframe::egui::RawInput {
        eframe::egui::RawInput {
            screen_rect: Some(eframe::egui::Rect::from_min_size(
                eframe::egui::Pos2::ZERO,
                eframe::egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        }
    }

    fn render_into(
        ui: &mut eframe::egui::Ui,
        scene: super::Scene<'_>,
        camera: &crate::canvas::GridCamera,
    ) {
        let (response, painter) = ui.allocate_painter(
            eframe::egui::vec2(400.0, 300.0),
            eframe::egui::Sense::hover(),
        );
        super::draw(&painter, response.rect, camera, scene, None);
    }

    fn sample_scene<'a>(
        document: &'a Document,
        library: &'a [crate::model::ComponentDefinition],
        selection: &'a Selection,
    ) -> super::Scene<'a> {
        super::Scene {
            document,
            library,
            selection,
            hover_wire: None,
            placing: None,
            rotation: Rotation::Deg0,
            mirror: Mirror::None,
            wire_preview: &[],
            wire_class: NetClass::Digital,
            selection_box: None,
            minimap: true,
            flow_phase: None,
        }
    }

    #[test]
    fn segment_visibility_rejects_offscreen_wires() {
        let camera = crate::canvas::GridCamera::default();
        let rect = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(0.0, 0.0),
            eframe::egui::vec2(200.0, 200.0),
        );
        assert!(segment_visible(
            rect,
            &camera,
            GridPos::new(0, 0),
            GridPos::new(2, 0)
        ));
        assert!(!segment_visible(
            rect,
            &camera,
            GridPos::new(100, 0),
            GridPos::new(102, 0)
        ));
    }

    #[test]
    fn wire_label_visible_on_hover_selection_or_zoom() {
        assert!(show_wire_label(true, false, 0.5));
        assert!(show_wire_label(false, true, 0.5));
        assert!(show_wire_label(false, false, 1.5));
        assert!(!show_wire_label(false, false, 1.0));
    }

    #[test]
    fn wire_midpoint_uses_middle_segment() {
        let wire = Wire {
            id: WireId(1),
            points: vec![GridPos::new(0, 0), GridPos::new(4, 0), GridPos::new(4, 4)],
            class: NetClass::Digital,
            color_override: None,
            label: None,
        };
        assert_eq!(wire_midpoint(&wire), Some(eframe::egui::pos2(4.0, 2.0)));
        let short = Wire {
            id: WireId(2),
            points: vec![GridPos::new(0, 0)],
            class: NetClass::Digital,
            color_override: None,
            label: None,
        };
        assert_eq!(wire_midpoint(&short), None);
    }

    #[test]
    fn net_colors_are_distinct() {
        let colors: Vec<_> = NetClass::ALL.into_iter().map(net_color).collect();
        for (index, color) in colors.iter().enumerate() {
            for other in &colors[index + 1..] {
                assert_ne!(color, other);
            }
        }
    }

    #[test]
    fn wire_color_uses_override() {
        let wire = Wire {
            id: WireId(1),
            points: vec![GridPos::new(0, 0), GridPos::new(2, 0)],
            class: NetClass::Digital,
            color_override: Some([1, 2, 3]),
            label: Some("LED_A".to_string()),
        };
        assert_eq!(
            super::wire_color(&wire),
            eframe::egui::Color32::from_rgb(1, 2, 3)
        );
        assert_eq!(super::wire_label_text(&wire), "LED_A");
    }

    #[test]
    fn net_colors_meet_wcag_aa_on_canvas() {
        let background = luminance(eframe::egui::Color32::from_rgb(18, 22, 26));
        for class in NetClass::ALL {
            let foreground = luminance(net_color(class));
            let ratio = (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05);
            assert!(ratio >= 4.5, "{} contrast {ratio:.2}", class.label());
        }
    }

    fn luminance(color: eframe::egui::Color32) -> f64 {
        0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
    }

    fn channel(value: u8) -> f64 {
        let value = f64::from(value) / 255.0;
        if value <= 0.04045 {
            return value / 12.92;
        }
        ((value + 0.055) / 1.055).powf(2.4)
    }

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
