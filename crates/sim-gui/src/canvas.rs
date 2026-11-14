//! Canvas camera, grid, pan, and zoom.

use eframe::egui;

const BACKGROUND: egui::Color32 = egui::Color32::from_rgb(18, 20, 24);
const MINOR_DOT: egui::Color32 = egui::Color32::from_rgb(55, 60, 70);
const MAJOR_LINE: egui::Color32 = egui::Color32::from_rgb(40, 45, 54);
const AXIS_LINE: egui::Color32 = egui::Color32::from_rgb(90, 100, 118);
const FOCUS_COLOR: egui::Color32 = egui::Color32::from_rgb(80, 215, 225);
const RULER_COLOR: egui::Color32 = egui::Color32::from_rgb(145, 155, 170);

/// Screen-space state of the grid camera.
#[derive(Clone, Debug)]
pub struct GridCamera {
    /// Screen offset of the world origin relative to the canvas top-left.
    pub pan: egui::Vec2,
    /// Zoom multiplier applied to the base grid cell size.
    pub zoom: f32,
    /// Base size of one grid cell in pixels at 100 percent zoom.
    pub cell_px: f32,
}

impl Default for GridCamera {
    fn default() -> Self {
        Self {
            pan: egui::Vec2::ZERO,
            zoom: 1.0,
            cell_px: 20.0,
        }
    }
}

impl GridCamera {
    /// Returns pixels per world grid unit.
    #[must_use]
    pub fn scale(&self) -> f32 {
        self.cell_px * self.zoom
    }

    /// Converts a world grid position to screen space.
    #[must_use]
    pub fn world_to_screen(&self, rect: egui::Rect, world: egui::Pos2) -> egui::Pos2 {
        rect.min + self.pan + world.to_vec2() * self.scale()
    }

    /// Converts a screen position to world grid space.
    #[must_use]
    pub fn screen_to_world(&self, rect: egui::Rect, screen: egui::Pos2) -> egui::Pos2 {
        ((screen - rect.min - self.pan) / self.scale()).to_pos2()
    }

    /// Zooms around a screen position while keeping that world point fixed.
    pub fn zoom_at(&mut self, rect: egui::Rect, cursor: egui::Pos2, factor: f32) {
        let world = self.screen_to_world(rect, cursor);
        self.zoom = (self.zoom * factor).clamp(0.25, 4.0);
        let screen = self.world_to_screen(rect, world);
        self.pan += cursor - screen;
    }

    /// Fits a world-space box into the canvas with padding.
    pub fn fit_world(&mut self, rect: egui::Rect, min: egui::Pos2, max: egui::Pos2, padding: f32) {
        let size = (max - min).max(egui::vec2(1.0, 1.0));
        let available =
            (rect.size() - egui::vec2(padding * 2.0, padding * 2.0)).max(egui::vec2(1.0, 1.0));
        let zoom_x = available.x / (size.x * self.cell_px);
        let zoom_y = available.y / (size.y * self.cell_px);
        self.zoom = zoom_x.min(zoom_y).clamp(0.25, 4.0);
        let center = ((min.to_vec2() + max.to_vec2()) * 0.5).to_pos2();
        self.pan = rect.size() * 0.5 - center.to_vec2() * self.scale();
    }

    fn visible_range(&self, rect: egui::Rect, step: i32) -> GridRange {
        let min = self.screen_to_world(rect, rect.min);
        let max = self.screen_to_world(rect, rect.max);
        let step = step as f32;
        GridRange {
            min_x: (min.x / step).floor() as i32,
            max_x: (max.x / step).ceil() as i32,
            min_y: (min.y / step).floor() as i32,
            max_y: (max.y / step).ceil() as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GridRange {
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

/// Result of one canvas frame.
#[derive(Clone, Copy, Debug)]
pub struct CanvasState {
    /// Canvas rectangle in screen space.
    pub rect: egui::Rect,
    /// Pointer position in world grid space.
    pub pointer_world: Option<egui::Pos2>,
    /// True when the primary button was clicked on the canvas.
    pub clicked: bool,
    /// World position of the primary click.
    pub click_world: Option<egui::Pos2>,
    /// True when the secondary button was clicked on the canvas.
    pub secondary_clicked: bool,
    /// True when the primary button was double-clicked.
    pub double_clicked: bool,
    /// True when a primary drag started this frame.
    pub drag_started: bool,
    /// True while a primary drag is active.
    pub dragged: bool,
    /// True when the primary drag stopped this frame.
    pub drag_stopped: bool,
    /// True while the Space key is held.
    pub space_down: bool,
    /// True when Shift or Ctrl is held for additive selection.
    pub multi_select: bool,
    /// True when the canvas has keyboard focus.
    pub focused: bool,
    /// Position of a secondary click.
    pub context_click: Option<ContextClick>,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            rect: egui::Rect::NOTHING,
            pointer_world: None,
            clicked: false,
            click_world: None,
            secondary_clicked: false,
            double_clicked: false,
            drag_started: false,
            dragged: false,
            drag_stopped: false,
            space_down: false,
            multi_select: false,
            focused: false,
            context_click: None,
        }
    }
}

/// Screen and world position of a context click.
#[derive(Clone, Copy, Debug)]
pub struct ContextClick {
    pub screen: egui::Pos2,
    pub world: egui::Pos2,
}

/// Draws the canvas and applies pan and zoom input.
pub fn show(
    ui: &mut egui::Ui,
    camera: &mut GridCamera,
    grid_visible: bool,
    initialized: &mut bool,
    draw_scene: impl FnOnce(&egui::Painter, egui::Rect, &GridCamera, Option<egui::Pos2>),
) -> CanvasState {
    let (response, painter) =
        ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
    let rect = response.rect;
    initialize_camera(rect, camera, initialized);
    handle_pan(&response, ui, camera);
    handle_zoom(&response, ui, camera, rect);
    let pointer = pointer_world(&response, rect, camera);
    focus_canvas(&response);
    let frame = CanvasFrame::new(&response, &painter, rect, camera, grid_visible, pointer);
    frame.paint(draw_scene);
    canvas_state(ui, &response, rect, camera, pointer)
}

struct CanvasFrame<'a> {
    response: &'a egui::Response,
    painter: &'a egui::Painter,
    rect: egui::Rect,
    camera: &'a GridCamera,
    grid_visible: bool,
    pointer: Option<egui::Pos2>,
}

impl<'a> CanvasFrame<'a> {
    fn new(
        response: &'a egui::Response,
        painter: &'a egui::Painter,
        rect: egui::Rect,
        camera: &'a GridCamera,
        grid_visible: bool,
        pointer: Option<egui::Pos2>,
    ) -> Self {
        Self {
            response,
            painter,
            rect,
            camera,
            grid_visible,
            pointer,
        }
    }

    fn paint(
        self,
        draw_scene: impl FnOnce(&egui::Painter, egui::Rect, &GridCamera, Option<egui::Pos2>),
    ) {
        let painter = self.painter;
        painter.rect_filled(self.rect, egui::CornerRadius::ZERO, BACKGROUND);
        if self.grid_visible {
            draw_grid(painter, self.rect, self.camera);
        }
        draw_scene(painter, self.rect, self.camera, self.pointer);
        paint_focus_ring(self.response, painter, self.rect);
    }
}

fn focus_canvas(response: &egui::Response) {
    if response.clicked() || response.drag_started() {
        response.request_focus();
    }
}

fn paint_focus_ring(response: &egui::Response, painter: &egui::Painter, rect: egui::Rect) {
    if !response.has_focus() {
        return;
    }
    painter.rect_stroke(
        rect.shrink(1.0),
        0.0,
        egui::Stroke::new(2.0, FOCUS_COLOR),
        egui::StrokeKind::Inside,
    );
}

fn canvas_state(
    ui: &egui::Ui,
    response: &egui::Response,
    rect: egui::Rect,
    camera: &GridCamera,
    pointer_world: Option<egui::Pos2>,
) -> CanvasState {
    let space_down = ui.input(|input| input.key_down(egui::Key::Space));
    let mut state = CanvasState {
        rect,
        pointer_world,
        space_down,
        focused: response.has_focus(),
        ..CanvasState::default()
    };
    fill_interaction(&mut state, response, rect, camera);
    state
}

fn fill_interaction(
    state: &mut CanvasState,
    response: &egui::Response,
    rect: egui::Rect,
    camera: &GridCamera,
) {
    state.clicked = response.clicked();
    state.click_world = click_world(response, rect, camera);
    state.secondary_clicked = response.secondary_clicked();
    state.double_clicked = response.double_clicked();
    state.drag_started = response.drag_started_by(egui::PointerButton::Primary);
    state.dragged = response.dragged_by(egui::PointerButton::Primary);
    state.drag_stopped = response.drag_stopped_by(egui::PointerButton::Primary);
    state.multi_select = multi_select(response);
    state.context_click = context_click(response, rect, camera);
}

fn context_click(
    response: &egui::Response,
    rect: egui::Rect,
    camera: &GridCamera,
) -> Option<ContextClick> {
    let screen = interaction_screen(response)?;
    Some(ContextClick {
        screen,
        world: camera.screen_to_world(rect, screen),
    })
}

fn interaction_screen(response: &egui::Response) -> Option<egui::Pos2> {
    response
        .interact_pointer_pos()
        .or_else(|| response.hover_pos())
}

fn click_world(
    response: &egui::Response,
    rect: egui::Rect,
    camera: &GridCamera,
) -> Option<egui::Pos2> {
    response
        .interact_pointer_pos()
        .map(|position| camera.screen_to_world(rect, position))
}

fn multi_select(response: &egui::Response) -> bool {
    response
        .ctx
        .input(|input| input.modifiers.shift || input.modifiers.command)
}

fn initialize_camera(rect: egui::Rect, camera: &mut GridCamera, initialized: &mut bool) {
    if *initialized {
        return;
    }
    camera.pan = rect.size() * 0.5;
    *initialized = true;
}

fn pointer_world(
    response: &egui::Response,
    rect: egui::Rect,
    camera: &GridCamera,
) -> Option<egui::Pos2> {
    response
        .hover_pos()
        .map(|position| camera.screen_to_world(rect, position))
}

fn handle_pan(response: &egui::Response, ui: &egui::Ui, camera: &mut GridCamera) {
    let space = ui.input(|input| input.key_down(egui::Key::Space));
    let dragging = response.dragged_by(egui::PointerButton::Middle)
        || (space && response.dragged_by(egui::PointerButton::Primary));
    if dragging {
        camera.pan += response.drag_delta();
    }
}

fn handle_zoom(
    response: &egui::Response,
    ui: &egui::Ui,
    camera: &mut GridCamera,
    rect: egui::Rect,
) {
    if !response.hovered() {
        return;
    }
    let scroll = ui.input(|input| input.smooth_scroll_delta.y);
    let Some(cursor) = response.hover_pos() else {
        return;
    };
    if scroll != 0.0 {
        camera.zoom_at(rect, cursor, (scroll * 0.002).exp());
    }
}

fn draw_grid(painter: &egui::Painter, rect: egui::Rect, camera: &GridCamera) {
    let step = grid_step(camera.scale());
    let range = camera.visible_range(rect, step);
    draw_minor_dots(painter, rect, camera, range, step);
    draw_major_lines(painter, rect, camera, range, step);
    draw_axes(painter, rect, camera);
    draw_rulers(painter, rect, camera, range, step);
}

fn draw_rulers(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
) {
    draw_x_ruler(painter, rect, camera, range, step);
    draw_y_ruler(painter, rect, camera, range, step);
}

fn draw_x_ruler(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
) {
    let major = step * 5;
    let font = egui::FontId::monospace(9.0);
    for index in (range.min_x..=range.max_x).filter(|index| index * step % major == 0) {
        let world = (index * step) as f32;
        let screen = camera.world_to_screen(rect, egui::pos2(world, 0.0));
        let pos = egui::pos2(screen.x + 2.0, rect.top() + 2.0);
        paint_ruler_text(painter, pos, world, &font);
    }
}

fn draw_y_ruler(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
) {
    let major = step * 5;
    let font = egui::FontId::monospace(9.0);
    for index in (range.min_y..=range.max_y).filter(|index| index * step % major == 0) {
        let world = (index * step) as f32;
        let screen = camera.world_to_screen(rect, egui::pos2(0.0, world));
        let pos = egui::pos2(rect.left() + 2.0, screen.y + 2.0);
        paint_ruler_text(painter, pos, world, &font);
    }
}

fn paint_ruler_text(painter: &egui::Painter, pos: egui::Pos2, world: f32, font: &egui::FontId) {
    painter.text(
        pos,
        egui::Align2::LEFT_TOP,
        format!("{world:.0}"),
        font.clone(),
        RULER_COLOR,
    );
}

fn draw_minor_dots(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
) {
    for x in range.min_x..=range.max_x {
        for y in range.min_y..=range.max_y {
            let world = egui::pos2((x * step) as f32, (y * step) as f32);
            let screen = camera.world_to_screen(rect, world);
            painter.circle_filled(screen, 1.0, MINOR_DOT);
        }
    }
}

fn draw_major_lines(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
) {
    let major = step * 5;
    let stroke = egui::Stroke::new(1.0, MAJOR_LINE);
    for index in (range.min_x..=range.max_x).filter(|index| index * step % major == 0) {
        draw_vertical_line(painter, rect, camera, (index * step) as f32, stroke);
    }
    for index in (range.min_y..=range.max_y).filter(|index| index * step % major == 0) {
        draw_horizontal_line(painter, rect, camera, (index * step) as f32, stroke);
    }
}

fn draw_vertical_line(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    world_x: f32,
    stroke: egui::Stroke,
) {
    let screen = camera.world_to_screen(rect, egui::pos2(world_x, 0.0));
    painter.line_segment(
        [
            egui::pos2(screen.x, rect.top()),
            egui::pos2(screen.x, rect.bottom()),
        ],
        stroke,
    );
}

fn draw_horizontal_line(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    world_y: f32,
    stroke: egui::Stroke,
) {
    let screen = camera.world_to_screen(rect, egui::pos2(0.0, world_y));
    painter.line_segment(
        [
            egui::pos2(rect.left(), screen.y),
            egui::pos2(rect.right(), screen.y),
        ],
        stroke,
    );
}

fn draw_axes(painter: &egui::Painter, rect: egui::Rect, camera: &GridCamera) {
    let origin = camera.world_to_screen(rect, egui::Pos2::ZERO);
    let stroke = egui::Stroke::new(1.0, AXIS_LINE);
    painter.line_segment(
        [
            egui::pos2(origin.x, rect.top()),
            egui::pos2(origin.x, rect.bottom()),
        ],
        stroke,
    );
    painter.line_segment(
        [
            egui::pos2(rect.left(), origin.y),
            egui::pos2(rect.right(), origin.y),
        ],
        stroke,
    );
}

fn grid_step(scale: f32) -> i32 {
    if scale >= 12.0 {
        1
    } else if scale >= 6.0 {
        2
    } else {
        5
    }
}

#[cfg(test)]
mod tests {
    use super::{GridCamera, grid_step};
    use eframe::egui;

    fn rect() -> egui::Rect {
        egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(800.0, 600.0))
    }

    #[test]
    fn screen_world_round_trip() {
        let camera = GridCamera::default();
        let world = egui::pos2(3.0, -2.0);
        let screen = camera.world_to_screen(rect(), world);
        let result = camera.screen_to_world(rect(), screen);
        assert!((result.x - world.x).abs() < 1e-5);
        assert!((result.y - world.y).abs() < 1e-5);
    }

    #[test]
    fn zoom_keeps_cursor_world_point() {
        let mut camera = GridCamera::default();
        let cursor = egui::pos2(200.0, 180.0);
        let before = camera.screen_to_world(rect(), cursor);
        camera.zoom_at(rect(), cursor, 1.5);
        let after = camera.screen_to_world(rect(), cursor);
        assert!((before.x - after.x).abs() < 1e-5);
        assert!((before.y - after.y).abs() < 1e-5);
    }

    #[test]
    fn fit_world_centers_and_scales() {
        let mut camera = crate::canvas::GridCamera::default();
        let rect = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(0.0, 0.0),
            eframe::egui::vec2(200.0, 100.0),
        );
        camera.fit_world(
            rect,
            eframe::egui::pos2(0.0, 0.0),
            eframe::egui::pos2(10.0, 10.0),
            0.0,
        );
        assert!((camera.zoom - 0.5).abs() < 1e-6);
        let center = camera.world_to_screen(rect, eframe::egui::pos2(5.0, 5.0));
        assert!((center - rect.center()).length() < 1e-4);
    }

    #[test]
    fn zoom_is_clamped() {
        let mut camera = GridCamera::default();
        camera.zoom_at(rect(), egui::pos2(100.0, 100.0), 100.0);
        assert_eq!(camera.zoom, 4.0);
        camera.zoom_at(rect(), egui::pos2(100.0, 100.0), 0.001);
        assert_eq!(camera.zoom, 0.25);
    }

    #[test]
    fn grid_step_keeps_readable_spacing() {
        assert_eq!(grid_step(20.0), 1);
        assert_eq!(grid_step(8.0), 2);
        assert_eq!(grid_step(5.0), 5);
    }
}
