//! GUI application shell, library sheet, inspector, and canvas integration.

use crate::canvas::{self, GridCamera};
use crate::library;
use crate::model::{
    Category, ComponentDefinition, ComponentId, ComponentInstance, Document, GridPos, Mirror,
    Rotation,
};
use crate::scene::{self, Scene};
use eframe::egui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tool {
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
struct DragState {
    id: ComponentId,
    offset: GridPos,
}

#[derive(Clone, Copy)]
struct SceneConfig {
    selected: Option<ComponentId>,
    placing: Option<&'static ComponentDefinition>,
    rotation: Rotation,
    mirror: Mirror,
}

pub struct GuiApp {
    camera: GridCamera,
    grid_visible: bool,
    tool: Tool,
    pointer_world: Option<egui::Pos2>,
    initialized: bool,
    library: &'static [ComponentDefinition],
    document: Document,
    selected: Option<ComponentId>,
    dragging: Option<DragState>,
    placing: Option<&'static ComponentDefinition>,
    placement_rotation: Rotation,
    placement_mirror: Mirror,
    library_open: bool,
    inspector_open: bool,
    search: String,
    category: Option<Category>,
}

impl Default for GuiApp {
    fn default() -> Self {
        Self {
            camera: GridCamera::default(),
            grid_visible: true,
            tool: Tool::Select,
            pointer_world: None,
            initialized: false,
            library: library::builtin(),
            document: Document::new(),
            selected: None,
            dragging: None,
            placing: None,
            placement_rotation: Rotation::Deg0,
            placement_mirror: Mirror::None,
            library_open: false,
            inspector_open: true,
            search: String::new(),
            category: None,
        }
    }
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
        Self::default()
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
        for tool in Tool::ALL {
            ui.selectable_value(&mut self.tool, tool, tool.label());
        }
        ui.separator();
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
        self.placement_controls(ui);
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
        let scene = build_scene(&self.document, self.library, self.scene_config());
        let camera = &mut self.camera;
        let grid_visible = self.grid_visible;
        let initialized = &mut self.initialized;
        egui::CentralPanel::default()
            .show(ui, |ui| {
                canvas::show(
                    ui,
                    camera,
                    grid_visible,
                    initialized,
                    |painter, rect, camera, pointer| {
                        draw_scene(painter, rect, camera, pointer, scene);
                    },
                )
            })
            .inner
    }

    fn scene_config(&self) -> SceneConfig {
        SceneConfig {
            selected: self.selected,
            placing: self.placing,
            rotation: self.placement_rotation,
            mirror: self.placement_mirror,
        }
    }

    fn handle_canvas(&mut self, state: canvas::CanvasState) {
        if self.placing.is_some() {
            self.handle_placement(state);
            return;
        }
        self.begin_drag(&state);
        self.drag_move(&state);
        self.end_drag(&state);
        if state.secondary_clicked {
            self.selected = None;
            return;
        }
        if !state.clicked {
            return;
        }
        let Some(world) = state.click_world else {
            return;
        };
        self.selected = self.document.hit_test(self.library, world_to_grid(world));
    }

    fn begin_drag(&mut self, state: &canvas::CanvasState) {
        if !state.drag_started || state.space_down {
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
        self.dragging = Some(DragState {
            id,
            offset: component.position.subtract(point),
        });
    }

    fn drag_move(&mut self, state: &canvas::CanvasState) {
        if !state.dragged {
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

    fn end_drag(&mut self, state: &canvas::CanvasState) {
        if state.drag_stopped {
            self.dragging = None;
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
        self.placing = None;
    }

    fn handle_keys(&mut self, ui: &egui::Ui) {
        if ui.ctx().egui_wants_keyboard_input() {
            return;
        }
        if ui.input(|input| input.key_pressed(egui::Key::R)) {
            self.rotate();
        }
        if ui.input(|input| input.key_pressed(egui::Key::M)) {
            self.mirror();
        }
        if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.placing = None;
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
        let Some((component, definition)) = self.selected_pair() else {
            ui.label("No selection");
            return;
        };
        show_component_info(ui, component, definition);
        show_component_actions(ui, action);
    }

    fn selected_pair(&self) -> Option<(&ComponentInstance, &ComponentDefinition)> {
        let id = self.selected?;
        let component = self.document.component(id)?;
        let definition = library::find(component.definition)?;
        Some((component, definition))
    }

    fn apply_inspector(&mut self, action: InspectorAction) {
        match action {
            InspectorAction::Rotate => self.rotate(),
            InspectorAction::Mirror => self.mirror(),
            InspectorAction::Delete => {
                if let Some(id) = self.selected {
                    self.document.remove(id);
                }
                self.selected = None;
            }
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
            let selection = usize::from(self.selected.is_some());
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

fn build_scene<'a>(
    document: &'a Document,
    library: &'a [ComponentDefinition],
    config: SceneConfig,
) -> Scene<'a> {
    Scene {
        document,
        library,
        selected: config.selected,
        placing: config.placing,
        hover: None,
        rotation: config.rotation,
        mirror: config.mirror,
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
    scene::draw(painter, rect, camera, Scene { hover, ..scene });
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
