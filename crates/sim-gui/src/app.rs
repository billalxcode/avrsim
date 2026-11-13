//! GUI application shell, library sheet, inspector, and canvas integration.

use crate::canvas::{self, GridCamera};
use crate::library;
use crate::model::{
    Category, ComponentDefinition, ComponentId, ComponentInstance, Document, GridPos, Mirror,
    NetClass, Rotation, WireId,
};
use crate::persist;
use crate::scene::{self, Scene};
use eframe::egui;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Tool {
    #[default]
    Select,
    Place,
    Wire,
    Probe,
}

impl Tool {
    const ALL: [Self; 4] = [Self::Select, Self::Place, Self::Wire, Self::Probe];

    const fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Place => "Place",
            Self::Wire => "Wire",
            Self::Probe => "Probe",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InspectorAction {
    Rotate,
    Mirror,
    Delete,
}

#[derive(Clone, Copy, Debug)]
struct SegmentDrag {
    wire: WireId,
    index: usize,
}

#[derive(Clone, Copy, Debug)]
struct DragState {
    id: ComponentId,
    offset: GridPos,
}

#[derive(Clone, Copy)]
struct SceneConfig<'a> {
    selected: Option<ComponentId>,
    selected_wire: Option<WireId>,
    placing: Option<&'static ComponentDefinition>,
    rotation: Rotation,
    mirror: Mirror,
    wire_preview: &'a [GridPos],
    wire_class: NetClass,
}

#[derive(Default)]
pub struct GuiApp {
    camera: GridCamera,
    grid_visible: bool,
    tool: Tool,
    pointer_world: Option<egui::Pos2>,
    initialized: bool,
    library: &'static [ComponentDefinition],
    document: Document,
    selected: Option<ComponentId>,
    selected_wire: Option<WireId>,
    dragging: Option<DragState>,
    dragging_segment: Option<SegmentDrag>,
    placing: Option<&'static ComponentDefinition>,
    placement_rotation: Rotation,
    placement_mirror: Mirror,
    library_open: bool,
    inspector_open: bool,
    search: String,
    category: Option<Category>,
    wire_points: Vec<GridPos>,
    wire_class: NetClass,
    current_path: Option<PathBuf>,
    message: String,
}

impl GuiApp {
    pub fn new(context: &eframe::CreationContext<'_>) -> Self {
        context.egui_ctx.set_theme(egui::ThemePreference::Dark);
        context.egui_ctx.style_mut_of(egui::Theme::Dark, |style| {
            for font in style.text_styles.values_mut() {
                font.family = egui::FontFamily::Monospace;
            }
            style.visuals.selection.bg_fill = egui::Color32::from_rgb(0, 150, 170);
        });
        Self {
            library: library::builtin(),
            document: Document::new(),
            grid_visible: true,
            inspector_open: true,
            ..Self::default()
        }
    }
}

impl eframe::App for GuiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.toolbar(ui);
        self.status_bar(ui);
        let state = self.canvas(ui);
        self.pointer_world = state.pointer_world;
        self.handle_canvas(state);
        self.handle_keys(ui);
        self.library_window(ui);
        self.inspector_window(ui);
    }
}

impl GuiApp {
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                self.toolbar_tools(ui);
                ui.separator();
                self.toolbar_view(ui);
            });
        });
    }

    fn toolbar_tools(&mut self, ui: &mut egui::Ui) {
        ui.strong("embed-emu");
        ui.separator();
        self.file_buttons(ui);
        ui.separator();
        self.tool_buttons(ui);
        ui.separator();
        self.sheet_buttons(ui);
    }

    fn file_buttons(&mut self, ui: &mut egui::Ui) {
        if ui.button("Open").clicked() {
            self.open_project();
        }
        if ui.button("Save").clicked() {
            self.save_project();
        }
        if !self.message.is_empty() {
            ui.label(&self.message);
        }
    }

    fn open_project(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("embed-emu project", &["json"])
            .pick_file()
        else {
            self.message = "Open cancelled".to_string();
            return;
        };
        match persist::load(&path, self.library) {
            Ok(project) => self.apply_project(project, path),
            Err(error) => self.message = error,
        }
    }

    fn save_project(&mut self) {
        let path = self.current_path.clone().or_else(|| {
            rfd::FileDialog::new()
                .add_filter("embed-emu project", &["json"])
                .set_file_name("project.emu.json")
                .save_file()
        });
        let Some(path) = path else {
            self.message = "Save cancelled".to_string();
            return;
        };
        match persist::save(&path, &self.document, &self.camera, self.grid_visible) {
            Ok(()) => {
                self.current_path = Some(path);
                self.message = "Saved".to_string();
            }
            Err(error) => self.message = error,
        }
    }

    fn apply_project(&mut self, project: persist::Project, path: PathBuf) {
        self.document = project.document;
        self.camera = project.camera;
        self.grid_visible = project.grid_visible;
        self.selected = None;
        self.selected_wire = None;
        self.dragging = None;
        self.dragging_segment = None;
        self.placing = None;
        self.wire_points.clear();
        self.initialized = true;
        self.current_path = Some(path);
        self.message = "Loaded".to_string();
    }

    fn tool_buttons(&mut self, ui: &mut egui::Ui) {
        let previous = self.tool;
        for tool in Tool::ALL {
            ui.selectable_value(&mut self.tool, tool, tool.label());
        }
        if previous != self.tool {
            self.wire_points.clear();
        }
    }

    fn sheet_buttons(&mut self, ui: &mut egui::Ui) {
        if ui.selectable_label(self.library_open, "Library").clicked() {
            self.library_open = true;
        }
        if ui
            .selectable_label(self.inspector_open, "Inspector")
            .clicked()
        {
            self.inspector_open = true;
        }
    }

    fn toolbar_view(&mut self, ui: &mut egui::Ui) {
        if ui.button("100%").clicked() {
            self.camera.zoom = 1.0;
        }
        ui.checkbox(&mut self.grid_visible, "Grid");
        ui.separator();
        if self.tool == Tool::Wire {
            self.wire_controls(ui);
        } else {
            self.placement_controls(ui);
        }
    }

    fn wire_controls(&mut self, ui: &mut egui::Ui) {
        ui.label("Wire");
        egui::ComboBox::from_id_salt("wire_class")
            .selected_text(self.wire_class.label())
            .show_ui(ui, |ui| {
                for class in NetClass::ALL {
                    ui.selectable_value(&mut self.wire_class, class, class.label());
                }
            });
        self.route_buttons(ui);
        if ui.button("Finish").clicked() {
            self.finish_wire();
        }
        if ui.button("Cancel").clicked() {
            self.wire_points.clear();
        }
    }

    fn route_buttons(&mut self, ui: &mut egui::Ui) {
        if ui.button("L-H").clicked() {
            self.apply_l_route(true);
        }
        if ui.button("L-V").clicked() {
            self.apply_l_route(false);
        }
    }

    fn apply_l_route(&mut self, horizontal_first: bool) {
        let Some(&last) = self.wire_points.last() else {
            return;
        };
        let Some(world) = self.pointer_world else {
            return;
        };
        let target = world_to_grid(world);
        let path = l_route(last, target, horizontal_first);
        let mut current = last;
        for &point in &path {
            if !self.document.can_add_segment(self.library, current, point) {
                return;
            }
            current = point;
        }
        self.wire_points.extend(path);
    }

    fn placement_controls(&mut self, ui: &mut egui::Ui) {
        if let Some(definition) = self.placing {
            ui.label(format!("Placing {}", definition.name));
            if ui.button("Rotate").clicked() {
                self.rotate();
            }
            if ui.button("Mirror").clicked() {
                self.mirror();
            }
            if ui.button("Cancel").clicked() {
                self.placing = None;
            }
            return;
        }
        ui.label("Run: idle");
    }

    fn canvas(&mut self, ui: &mut egui::Ui) -> canvas::CanvasState {
        let config = SceneConfig {
            selected: self.selected,
            selected_wire: self.selected_wire,
            placing: self.placing,
            rotation: self.placement_rotation,
            mirror: self.placement_mirror,
            wire_preview: &self.wire_points,
            wire_class: self.wire_class,
        };
        let scene = build_scene(&self.document, self.library, config);
        let camera = &mut self.camera;
        let grid_visible = self.grid_visible;
        let initialized = &mut self.initialized;
        show_canvas(ui, camera, grid_visible, initialized, scene)
    }

    fn handle_canvas(&mut self, state: canvas::CanvasState) {
        if self.placing.is_some() {
            self.handle_placement(state);
            return;
        }
        if self.tool == Tool::Wire {
            self.handle_wire(state);
            return;
        }
        self.begin_drag(&state);
        self.drag_move(&state);
        self.end_drag(&state);
        if state.secondary_clicked {
            self.selected = None;
            self.selected_wire = None;
            return;
        }
        if !state.clicked {
            return;
        }
        let Some(world) = state.click_world else {
            return;
        };
        self.select_at(world);
    }

    fn select_at(&mut self, world: egui::Pos2) {
        let point = world_to_grid(world);
        if let Some(id) = self.document.hit_test(self.library, point) {
            self.selected = Some(id);
            self.selected_wire = None;
            return;
        }
        self.selected = None;
        self.selected_wire = self.document.wire_at([world.x, world.y], 0.35);
    }

    fn begin_drag(&mut self, state: &canvas::CanvasState) {
        if !state.drag_started || state.space_down {
            return;
        }
        if self.begin_segment_drag(state) {
            return;
        }
        let Some(world) = state.pointer_world else {
            return;
        };
        let point = world_to_grid(world);
        let Some(id) = self.document.hit_test(self.library, point) else {
            return;
        };
        let Some(component) = self.document.component(id) else {
            return;
        };
        self.selected = Some(id);
        self.selected_wire = None;
        self.dragging = Some(DragState {
            id,
            offset: component.position.subtract(point),
        });
    }

    fn begin_segment_drag(&mut self, state: &canvas::CanvasState) -> bool {
        let Some(wire_id) = self.selected_wire else {
            return false;
        };
        let Some(world) = state.pointer_world else {
            return false;
        };
        let Some(index) = self.segment_at(wire_id, world) else {
            return false;
        };
        self.dragging_segment = Some(SegmentDrag {
            wire: wire_id,
            index,
        });
        true
    }

    fn segment_at(&self, wire_id: WireId, world: egui::Pos2) -> Option<usize> {
        self.document
            .wire_segment_at(wire_id, [world.x, world.y], 0.35)
    }

    fn drag_move(&mut self, state: &canvas::CanvasState) {
        if !state.dragged {
            return;
        }
        if self.drag_segment(state) {
            return;
        }
        let Some(drag) = self.dragging else {
            return;
        };
        let Some(world) = state.pointer_world else {
            return;
        };
        let target = world_to_grid(world).offset(drag.offset);
        self.document.move_component(self.library, drag.id, target);
    }

    fn drag_segment(&mut self, state: &canvas::CanvasState) -> bool {
        let Some(drag) = self.dragging_segment else {
            return false;
        };
        let Some(world) = state.pointer_world else {
            return true;
        };
        let Some(mut points) = self
            .document
            .wires()
            .iter()
            .find(|wire| wire.id == drag.wire)
            .map(|wire| wire.points.clone())
        else {
            return true;
        };
        if !move_segment(&mut points, drag.index, world_to_grid(world)) {
            return true;
        }
        self.document
            .update_wire_points(self.library, drag.wire, points);
        true
    }

    fn end_drag(&mut self, state: &canvas::CanvasState) {
        if state.drag_stopped {
            self.dragging = None;
            self.dragging_segment = None;
        }
    }

    fn handle_placement(&mut self, state: canvas::CanvasState) {
        if state.secondary_clicked {
            self.placing = None;
            return;
        }
        if !state.clicked {
            return;
        }
        let Some(world) = state.click_world else {
            return;
        };
        let Some(definition) = self.placing else {
            return;
        };
        let position =
            scene::placement_position(definition, world_to_grid(world), self.placement_rotation);
        if !self.document.can_place(
            self.library,
            definition,
            position,
            self.placement_rotation,
            self.placement_mirror,
        ) {
            return;
        }
        let id = self.document.add(
            definition,
            position,
            self.placement_rotation,
            self.placement_mirror,
        );
        self.selected = Some(id);
        self.selected_wire = None;
        self.placing = None;
    }

    fn handle_wire(&mut self, state: canvas::CanvasState) {
        if state.secondary_clicked {
            self.wire_points.clear();
            return;
        }
        if state.double_clicked {
            self.finish_wire();
            return;
        }
        if !state.clicked {
            return;
        }
        let Some(world) = state.click_world else {
            return;
        };
        self.wire_click(world_to_grid(world));
    }

    fn wire_click(&mut self, point: GridPos) {
        if self.wire_points.is_empty() {
            if self.document.is_connection_point(self.library, point) {
                self.wire_points.push(point);
            }
            return;
        }
        let Some(&last) = self.wire_points.last() else {
            return;
        };
        if point == last || !self.document.can_add_segment(self.library, last, point) {
            return;
        }
        self.wire_points.push(point);
    }

    fn finish_wire(&mut self) {
        if self.wire_points.len() < 2 {
            self.wire_points.clear();
            return;
        }
        let points = self.wire_points.clone();
        if self
            .document
            .add_wire(self.library, points, self.wire_class)
            .is_some()
        {
            self.wire_points.clear();
        }
    }

    fn handle_keys(&mut self, ui: &egui::Ui) {
        if ui.ctx().egui_wants_keyboard_input() {
            return;
        }
        self.handle_file_keys(ui);
        self.handle_edit_keys(ui);
    }

    fn handle_file_keys(&mut self, ui: &egui::Ui) {
        let save = ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::S));
        let open = ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::O));
        if save {
            self.save_project();
        }
        if open {
            self.open_project();
        }
    }

    fn handle_edit_keys(&mut self, ui: &egui::Ui) {
        if ui.input(|input| input.key_pressed(egui::Key::R)) {
            self.rotate();
        }
        if ui.input(|input| input.key_pressed(egui::Key::M)) {
            self.mirror();
        }
        if ui.input(|input| input.key_pressed(egui::Key::Enter)) && self.tool == Tool::Wire {
            self.finish_wire();
        }
        if ui.input(|input| input.key_pressed(egui::Key::Backspace)) && self.tool == Tool::Wire {
            self.wire_points.pop();
        }
        if ui.input(|input| input.key_pressed(egui::Key::Delete)) {
            self.delete_selection();
        }
        if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.placing = None;
            self.wire_points.clear();
        }
    }

    fn rotate(&mut self) {
        if self.placing.is_some() {
            self.placement_rotation = self.placement_rotation.next();
            return;
        }
        let Some(id) = self.selected else {
            return;
        };
        let Some(component) = self.document.component(id) else {
            return;
        };
        let rotation = component.rotation.next();
        self.document.set_rotation(self.library, id, rotation);
    }

    fn mirror(&mut self) {
        if self.placing.is_some() {
            self.placement_mirror = self.placement_mirror.next();
            return;
        }
        let Some(id) = self.selected else {
            return;
        };
        let Some(component) = self.document.component(id) else {
            return;
        };
        let mirror = component.mirror.next();
        self.document.set_mirror(self.library, id, mirror);
    }

    fn library_window(&mut self, ui: &mut egui::Ui) {
        let mut open = self.library_open;
        let mut search = self.search.clone();
        let mut category = self.category;
        let mut chosen = None;
        egui::Window::new("Library")
            .open(&mut open)
            .default_width(300.0)
            .show(ui.ctx(), |ui| {
                self.library_filters(ui, &mut search, &mut category);
                ui.separator();
                chosen = library_list(ui, self.library, &search, category);
            });
        self.library_open = open;
        self.search = search;
        self.category = category;
        if let Some(definition) = chosen {
            self.placing = Some(definition);
            self.library_open = false;
        }
    }

    fn library_filters(
        &self,
        ui: &mut egui::Ui,
        search: &mut String,
        category: &mut Option<Category>,
    ) {
        ui.text_edit_singleline(search);
        ui.horizontal_wrapped(|ui| {
            if ui.selectable_label(category.is_none(), "All").clicked() {
                *category = None;
            }
            for value in Category::ALL {
                if ui
                    .selectable_label(*category == Some(value), value.label())
                    .clicked()
                {
                    *category = Some(value);
                }
            }
        });
    }

    fn inspector_window(&mut self, ui: &mut egui::Ui) {
        let mut open = self.inspector_open;
        let mut action = None;
        egui::Window::new("Inspector")
            .open(&mut open)
            .default_width(260.0)
            .show(ui.ctx(), |ui| {
                self.inspector_contents(ui, &mut action);
            });
        self.inspector_open = open;
        if let Some(action) = action {
            self.apply_inspector(action);
        }
    }

    fn inspector_contents(&self, ui: &mut egui::Ui, action: &mut Option<InspectorAction>) {
        if let Some(id) = self.selected_wire {
            self.wire_inspector(ui, action, id);
            return;
        }
        let Some((component, definition)) = self.selected_pair() else {
            ui.label("No selection");
            return;
        };
        show_component_info(ui, component, definition);
        show_component_actions(ui, action);
    }

    fn wire_inspector(&self, ui: &mut egui::Ui, action: &mut Option<InspectorAction>, id: WireId) {
        let Some(wire) = self.document.wires().iter().find(|wire| wire.id == id) else {
            ui.label("Wire missing");
            return;
        };
        ui.label(format!("Wire {}", wire.id.0));
        ui.label(format!("Class: {}", wire.class.label()));
        ui.label(format!("Vertices: {}", wire.points.len()));
        if ui.button("Delete").clicked() {
            *action = Some(InspectorAction::Delete);
        }
    }

    fn selected_pair(&self) -> Option<(&ComponentInstance, &ComponentDefinition)> {
        let id = self.selected?;
        let component = self.document.component(id)?;
        let definition = library::find(component.definition)?;
        Some((component, definition))
    }

    fn delete_selection(&mut self) {
        if let Some(id) = self.selected_wire {
            self.document.remove_wire(id);
            self.selected_wire = None;
            return;
        }
        if let Some(id) = self.selected {
            self.document.remove(id);
            self.selected = None;
        }
    }

    fn apply_inspector(&mut self, action: InspectorAction) {
        match action {
            InspectorAction::Rotate => self.rotate(),
            InspectorAction::Mirror => self.mirror(),
            InspectorAction::Delete => self.delete_selection(),
        }
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("status").show(ui, |ui| {
            self.status_contents(ui);
        });
    }

    fn status_contents(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(self.pointer_text());
            ui.separator();
            ui.label(format!("Zoom {:.0}%", self.camera.zoom * 100.0));
            ui.separator();
            ui.label(format!("Grid {:.0}px", self.camera.cell_px));
            ui.separator();
            ui.label(format!("Tool: {}", self.tool.label()));
            ui.separator();
            let selection = usize::from(self.selected.is_some() || self.selected_wire.is_some());
            ui.label(format!("Sel {selection}"));
            ui.separator();
            ui.label(format!("Parts {}", self.document.components().len()));
            ui.separator();
            ui.label("Sim 0 cycles");
            ui.separator();
            ui.label(self.fps_text(ui));
        });
    }

    fn pointer_text(&self) -> String {
        match self.pointer_world {
            Some(point) => format!("x={:.1} y={:.1}", point.x, point.y),
            None => "x=- y=-".to_string(),
        }
    }

    fn fps_text(&self, ui: &egui::Ui) -> String {
        let dt = ui.ctx().input(|input| input.stable_dt).max(0.0001);
        format!("FPS {:.0}", 1.0 / dt)
    }
}

fn show_canvas(
    ui: &mut egui::Ui,
    camera: &mut GridCamera,
    grid_visible: bool,
    initialized: &mut bool,
    scene: Scene<'_>,
) -> canvas::CanvasState {
    let draw = |painter: &egui::Painter,
                rect: egui::Rect,
                camera: &GridCamera,
                pointer: Option<egui::Pos2>| {
        draw_scene(painter, rect, camera, pointer, scene);
    };
    let show = |ui: &mut egui::Ui| canvas::show(ui, camera, grid_visible, initialized, draw);
    egui::CentralPanel::default().show(ui, show).inner
}

fn build_scene<'a>(
    document: &'a Document,
    library: &'a [ComponentDefinition],
    config: SceneConfig<'a>,
) -> Scene<'a> {
    Scene {
        document,
        library,
        selected: config.selected,
        placing: config.placing,
        hover: None,
        rotation: config.rotation,
        mirror: config.mirror,
        selected_wire: config.selected_wire,
        wire_preview: config.wire_preview,
        wire_hover: None,
        wire_class: config.wire_class,
    }
}

fn draw_scene(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    pointer: Option<egui::Pos2>,
    scene: Scene<'_>,
) {
    let hover = pointer.map(world_to_grid);
    scene::draw(
        painter,
        rect,
        camera,
        Scene {
            hover,
            wire_hover: hover,
            ..scene
        },
    );
}

fn show_component_info(
    ui: &mut egui::Ui,
    component: &ComponentInstance,
    definition: &ComponentDefinition,
) {
    ui.label(format!("{} {}", component.label, component.value));
    ui.label(format!("Type: {}", definition.name));
    ui.label(format!(
        "Pos: {}, {}",
        component.position.x, component.position.y
    ));
    ui.label(format!(
        "Rot: {}  Mirror: {}",
        component.rotation.label(),
        component.mirror.label()
    ));
}

fn show_component_actions(ui: &mut egui::Ui, action: &mut Option<InspectorAction>) {
    ui.horizontal(|ui| {
        if ui.button("Rotate").clicked() {
            *action = Some(InspectorAction::Rotate);
        }
        if ui.button("Mirror").clicked() {
            *action = Some(InspectorAction::Mirror);
        }
        if ui.button("Delete").clicked() {
            *action = Some(InspectorAction::Delete);
        }
    });
}

fn library_list<'a>(
    ui: &mut egui::Ui,
    library: &'a [ComponentDefinition],
    search: &str,
    category: Option<Category>,
) -> Option<&'a ComponentDefinition> {
    let needle = search.to_lowercase();
    for definition in library {
        if category.is_some_and(|value| value != definition.category) {
            continue;
        }
        if !needle.is_empty() && !definition.name.to_lowercase().contains(&needle) {
            continue;
        }
        let text = format!("{}  {}", definition.name, definition.default_value);
        if ui.selectable_label(false, text).clicked() {
            return Some(definition);
        }
    }
    None
}

fn world_to_grid(position: egui::Pos2) -> GridPos {
    GridPos::new(position.x.round() as i32, position.y.round() as i32)
}

fn move_segment(points: &mut [GridPos], index: usize, target: GridPos) -> bool {
    let Some(&end) = points.get(index + 1) else {
        return false;
    };
    let start = points[index];
    if start.y == end.y {
        points[index].y = target.y;
        points[index + 1].y = target.y;
    } else {
        points[index].x = target.x;
        points[index + 1].x = target.x;
    }
    true
}

fn l_route(last: GridPos, target: GridPos, horizontal_first: bool) -> Vec<GridPos> {
    if last == target {
        return Vec::new();
    }
    if last.x == target.x || last.y == target.y {
        return vec![target];
    }
    let corner = if horizontal_first {
        GridPos::new(target.x, last.y)
    } else {
        GridPos::new(last.x, target.y)
    };
    vec![corner, target]
}

#[cfg(test)]
mod tests {
    use super::{l_route, move_segment};
    use crate::model::GridPos;

    #[test]
    fn l_route_adds_one_corner() {
        let route = l_route(GridPos::new(0, 0), GridPos::new(4, 2), true);
        assert_eq!(route, vec![GridPos::new(4, 0), GridPos::new(4, 2)]);
        let route = l_route(GridPos::new(0, 0), GridPos::new(4, 2), false);
        assert_eq!(route, vec![GridPos::new(0, 2), GridPos::new(4, 2)]);
    }

    #[test]
    fn l_route_handles_aligned_and_same_points() {
        assert_eq!(
            l_route(GridPos::new(0, 0), GridPos::new(4, 0), true),
            vec![GridPos::new(4, 0)]
        );
        assert!(l_route(GridPos::new(0, 0), GridPos::new(0, 0), true).is_empty());
    }

    #[test]
    fn move_segment_shifts_both_endpoints() {
        let mut points = vec![
            GridPos::new(0, 0),
            GridPos::new(0, 2),
            GridPos::new(4, 2),
            GridPos::new(4, 4),
        ];
        assert!(move_segment(&mut points, 1, GridPos::new(0, 3)));
        assert_eq!(points[1], GridPos::new(0, 3));
        assert_eq!(points[2], GridPos::new(4, 3));
    }
}
