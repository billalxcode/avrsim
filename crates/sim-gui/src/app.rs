//! GUI application shell, library sheet, inspector, and canvas integration.

use crate::canvas::{self, GridCamera};
use crate::clipboard::{self, Clipboard};
use crate::config;
use crate::history::History;
use crate::layout::{self, Align, Distribute};
use crate::library;
use crate::model::{
    Category, ComponentDefinition, ComponentId, ComponentInstance, Document, GridPos, GridRect,
    Mirror, NetClass, Rotation, WireId,
};
use crate::palette::{self, Command};
use crate::persist;
use crate::scene::{self, Scene};
use crate::selection::Selection;
use crate::sim::{self, Event, Sample, SimHandle, Status, Waveform};
use crate::uart::UartLog;
use eframe::egui;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

const AUTOSAVE_DELAY: Duration = Duration::from_secs(2);
const WATCH_INTERVAL: Duration = Duration::from_secs(1);
const TOAST_DURATION: Duration = Duration::from_secs(3);
const PB5_COLOR: egui::Color32 = egui::Color32::from_rgb(0, 190, 210);
const LED_COLOR: egui::Color32 = egui::Color32::from_rgb(85, 205, 125);

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SimAction {
    Run,
    Pause,
    Step,
    Reset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConflictAction {
    Reload,
    SaveMine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContextAction {
    Copy,
    Paste,
    Duplicate,
    Delete,
    Rotate,
    Mirror,
    Properties,
    SelectTool,
    WireTool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileStamp {
    modified: Option<SystemTime>,
    len: u64,
}

#[derive(Clone, Copy, Debug)]
struct SegmentDrag {
    wire: WireId,
    index: usize,
}

#[derive(Clone, Debug)]
struct DragState {
    offsets: Vec<(ComponentId, GridPos)>,
}

#[derive(Clone, Copy)]
struct SceneConfig<'a> {
    selection: &'a Selection,
    hover_wire: Option<WireId>,
    placing: Option<&'static ComponentDefinition>,
    rotation: Rotation,
    mirror: Mirror,
    wire_preview: &'a [GridPos],
    wire_class: NetClass,
    selection_box: Option<(GridPos, GridPos)>,
}

#[derive(Default)]
pub struct GuiApp {
    camera: GridCamera,
    canvas_rect: Option<egui::Rect>,
    grid_visible: bool,
    tool: Tool,
    pointer_world: Option<egui::Pos2>,
    initialized: bool,
    library: &'static [ComponentDefinition],
    document: Document,
    selection: Selection,
    clipboard: Clipboard,
    context_menu: Option<egui::Pos2>,
    pending_delete: bool,
    toast_until: Option<Instant>,
    last_toast: String,
    box_start: Option<GridPos>,
    box_current: Option<GridPos>,
    box_additive: bool,
    dragging: Option<DragState>,
    dragging_segment: Option<SegmentDrag>,
    placing: Option<&'static ComponentDefinition>,
    placement_rotation: Rotation,
    placement_mirror: Mirror,
    library_open: bool,
    inspector_open: bool,
    legend_open: bool,
    palette: palette::State,
    search: String,
    category: Option<Category>,
    wire_points: Vec<GridPos>,
    wire_class: NetClass,
    current_path: Option<PathBuf>,
    message: String,
    history: History,
    drag_history_pending: bool,
    sim: Option<SimHandle>,
    sim_status: Status,
    waveform: Waveform,
    waveform_open: bool,
    uart: UartLog,
    uart_open: bool,
    config: config::AppConfig,
    config_path: Option<PathBuf>,
    dirty: bool,
    last_edit: Option<Instant>,
    file_stamp: Option<FileStamp>,
    watch_at: Option<Instant>,
    conflict: bool,
}

impl GuiApp {
    pub fn new(context: &eframe::CreationContext<'_>) -> Self {
        configure_style(context);
        let (config_path, config) = load_config();
        Self {
            library: library::builtin(),
            document: Document::new(),
            grid_visible: true,
            inspector_open: true,
            sim: Some(SimHandle::spawn()),
            config_path,
            config,
            ..Self::default()
        }
    }
}

impl eframe::App for GuiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.palette_shortcut(ui);
        self.poll_sim();
        self.watch_file(ui.ctx());
        self.autosave(ui.ctx());
        self.update_toast();
        self.toolbar(ui);
        self.status_bar(ui);
        let state = self.canvas(ui);
        self.pointer_world = state.pointer_world;
        self.canvas_rect = Some(state.rect);
        self.handle_canvas(state);
        self.handle_keys(ui);
        self.windows(ui);
    }

    fn on_exit(&mut self) {
        let _ = self.flush_autosave();
    }
}

impl GuiApp {
    fn windows(&mut self, ui: &mut egui::Ui) {
        self.library_window(ui);
        self.inspector_window(ui);
        self.legend_window(ui);
        self.waveform_window(ui);
        self.uart_window(ui);
        self.show_context_menu(ui);
        self.command_palette(ui);
        self.delete_modal(ui);
        self.show_toast(ui);
        self.conflict_modal(ui);
    }
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                self.toolbar_tools(ui);
                ui.separator();
                self.toolbar_view(ui);
                if !self.message.is_empty() {
                    ui.separator();
                    ui.label(&self.message);
                }
            });
        });
    }

    fn toolbar_tools(&mut self, ui: &mut egui::Ui) {
        ui.strong("embed-emu");
        ui.separator();
        self.file_buttons(ui);
        self.edit_buttons(ui);
        ui.separator();
        self.sim_buttons(ui);
        ui.separator();
        self.tool_buttons(ui);
        ui.separator();
        self.sheet_buttons(ui);
    }

    fn edit_buttons(&mut self, ui: &mut egui::Ui) {
        let undo = ui
            .add_enabled(self.history.can_undo(), egui::Button::new("Undo"))
            .clicked();
        let redo = ui
            .add_enabled(self.history.can_redo(), egui::Button::new("Redo"))
            .clicked();
        if undo {
            self.undo();
        }
        if redo {
            self.redo();
        }
    }

    fn file_buttons(&mut self, ui: &mut egui::Ui) {
        if ui.button("Open").clicked() {
            self.open_project();
        }
        if ui.button("Save").clicked() {
            self.save_project();
        }
        self.recent_menu(ui);
    }

    fn recent_menu(&mut self, ui: &mut egui::Ui) {
        let mut chosen = None;
        ui.menu_button("Recent", |ui| {
            if self.config.recent_files.is_empty() {
                ui.label("No recent files");
            }
            for path in &self.config.recent_files {
                let label = file_name(path);
                let hint = path.display().to_string();
                if ui.button(label).on_hover_text(hint).clicked() {
                    chosen = Some(path.clone());
                    ui.close();
                }
            }
        });
        if let Some(path) = chosen {
            self.open_path(path);
        }
    }

    fn sim_buttons(&mut self, ui: &mut egui::Ui) {
        if ui.button("Load HEX").clicked() {
            self.open_hex();
        }
        let loaded = self.sim_status.loaded;
        let running = self.sim_status.running;
        let run_label = if running { "Pause" } else { "Run" };
        let run = ui
            .add_enabled(loaded, egui::Button::new(run_label))
            .clicked();
        let step = ui
            .add_enabled(loaded && !running, egui::Button::new("Step"))
            .clicked();
        let reset = ui.add_enabled(loaded, egui::Button::new("Reset")).clicked();
        ui.checkbox(&mut self.waveform_open, "Wave");
        ui.checkbox(&mut self.uart_open, "UART");
        if run {
            self.toggle_run();
        }
        if step {
            self.send_sim(SimAction::Step);
        }
        if reset {
            self.send_sim(SimAction::Reset);
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
        self.open_path(path);
    }

    fn open_path(&mut self, path: PathBuf) {
        if !self.flush_autosave() {
            return;
        }
        match persist::load(&path, self.library) {
            Ok(project) => {
                self.apply_project(project, path.clone());
                self.remember_recent(&path);
            }
            Err(error) => self.message = error,
        }
    }

    fn save_project(&mut self) {
        if self.external_changed() {
            self.conflict = true;
            self.message = "File changed on disk".to_string();
            return;
        }
        self.save_project_now();
    }

    fn save_project_now(&mut self) {
        let path = self.current_path.clone().or_else(select_save_path);
        let Some(path) = path else {
            self.message = "Save cancelled".to_string();
            return;
        };
        match persist::save(&path, &self.document, &self.camera, self.grid_visible) {
            Ok(()) => self.after_save(path),
            Err(error) => self.message = error,
        }
    }

    fn after_save(&mut self, path: PathBuf) {
        self.current_path = Some(path.clone());
        self.record_file_stamp(&path);
        self.dirty = false;
        self.last_edit = None;
        self.message = "Saved".to_string();
        self.remember_recent(&path);
    }

    fn remember_recent(&mut self, path: &Path) {
        config::push_recent(&mut self.config, path);
        self.save_config();
    }

    fn save_config(&mut self) {
        let Some(path) = self.config_path.as_deref() else {
            return;
        };
        if let Err(error) = config::save(path, &self.config) {
            self.message = error;
        }
    }

    fn update_toast(&mut self) {
        if self.message.is_empty() || self.message == self.last_toast {
            return;
        }
        self.last_toast = self.message.clone();
        self.toast_until = Some(Instant::now() + TOAST_DURATION);
    }

    fn show_toast(&mut self, ui: &egui::Ui) {
        let Some(until) = self.toast_until else {
            return;
        };
        let now = Instant::now();
        if now >= until {
            self.toast_until = None;
            return;
        }
        egui::Area::new(egui::Id::unique("toast"))
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -44.0))
            .order(egui::Order::Tooltip)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.label(&self.message);
                });
            });
        ui.ctx().request_repaint_after(until - now);
    }

    fn autosave(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        let Some(remaining) = self.autosave_remaining(now) else {
            return;
        };
        if remaining.is_zero() {
            self.write_autosave(now);
            return;
        }
        ctx.request_repaint_after(remaining);
    }

    fn autosave_remaining(&self, now: Instant) -> Option<Duration> {
        if self.conflict || !self.dirty || self.current_path.is_none() {
            return None;
        }
        let last = self.last_edit?;
        Some(AUTOSAVE_DELAY.saturating_sub(now.saturating_duration_since(last)))
    }

    fn write_autosave(&mut self, now: Instant) {
        let Some(path) = self.current_path.clone() else {
            return;
        };
        if self.external_changed() {
            self.conflict = true;
            self.message = "File changed on disk".to_string();
            return;
        }
        match persist::save(&path, &self.document, &self.camera, self.grid_visible) {
            Ok(()) => self.finish_autosave(&path),
            Err(error) => self.fail_autosave(now, error),
        }
    }

    fn finish_autosave(&mut self, path: &Path) {
        self.record_file_stamp(path);
        self.dirty = false;
        self.last_edit = None;
        self.message = "Autosaved".to_string();
    }

    fn fail_autosave(&mut self, now: Instant, error: String) {
        self.last_edit = Some(now);
        self.message = error;
    }

    fn external_changed(&self) -> bool {
        let Some((path, stamp)) = self.watch_target() else {
            return false;
        };
        file_stamp(&path) != Some(stamp)
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.last_edit = Some(Instant::now());
    }

    fn flush_autosave(&mut self) -> bool {
        if self.conflict {
            self.message = "Resolve file conflict before continuing".to_string();
            return false;
        }
        if !self.dirty {
            return true;
        }
        if self.current_path.is_none() {
            self.message = "Save project before opening another file".to_string();
            return false;
        }
        self.write_autosave(Instant::now());
        !self.dirty
    }

    fn record_file_stamp(&mut self, path: &Path) {
        self.file_stamp = file_stamp(path);
        self.watch_at = Some(Instant::now());
        self.conflict = false;
    }

    fn watch_file(&mut self, ctx: &egui::Context) {
        if self.conflict {
            return;
        }
        let Some((path, stamp)) = self.watch_target() else {
            return;
        };
        let now = Instant::now();
        let remaining = watch_remaining(self.watch_at, now);
        if !remaining.is_zero() {
            ctx.request_repaint_after(remaining);
            return;
        }
        self.watch_at = Some(now);
        ctx.request_repaint_after(WATCH_INTERVAL);
        if file_stamp(&path) != Some(stamp) {
            self.conflict = true;
            self.message = "File changed on disk".to_string();
        }
    }

    fn watch_target(&self) -> Option<(PathBuf, FileStamp)> {
        Some((self.current_path.clone()?, self.file_stamp?))
    }

    fn apply_project(&mut self, project: persist::Project, path: PathBuf) {
        self.document = project.document;
        self.camera = project.camera;
        self.grid_visible = project.grid_visible;
        self.reset_transient();
        self.history.clear();
        self.initialized = true;
        self.current_path = Some(path.clone());
        self.record_file_stamp(&path);
        self.dirty = false;
        self.last_edit = None;
        self.message = "Loaded".to_string();
    }

    fn open_hex(&mut self) {
        let Some(path) = pick_hex() else {
            self.message = "HEX open cancelled".to_string();
            return;
        };
        let name = file_name(&path);
        match sim::load_hex(&path) {
            Ok(words) => self.start_firmware(words, name),
            Err(error) => self.message = error,
        }
    }

    fn start_firmware(&mut self, words: Vec<u16>, name: String) {
        let Some(sim) = self.sim.as_ref() else {
            self.message = "Simulation unavailable".to_string();
            return;
        };
        match sim.load(words) {
            Ok(()) => {
                self.waveform.clear();
                self.uart.clear();
                self.waveform_open = true;
                self.message = format!("Firmware {name}");
            }
            Err(error) => self.message = error,
        }
    }

    fn toggle_run(&mut self) {
        let action = if self.sim_status.running {
            SimAction::Pause
        } else {
            SimAction::Run
        };
        self.send_sim(action);
    }

    fn send_sim(&mut self, action: SimAction) {
        let Some(sim) = self.sim.as_ref() else {
            self.message = "Simulation unavailable".to_string();
            return;
        };
        let result = match action {
            SimAction::Run => sim.run(),
            SimAction::Pause => sim.pause(),
            SimAction::Step => sim.step(),
            SimAction::Reset => sim.reset(),
        };
        if let Err(error) = result {
            self.message = error;
        }
        if action == SimAction::Reset {
            self.waveform.clear();
            self.uart.clear();
        }
    }

    fn poll_sim(&mut self) {
        let events = self.sim.as_ref().map(SimHandle::poll).unwrap_or_default();
        for event in events {
            self.apply_sim_event(event);
        }
    }

    fn apply_sim_event(&mut self, event: Event) {
        match event {
            Event::Status(status) => self.sim_status = status,
            Event::Sample(sample) => self.waveform.push(sample),
            Event::Uart(data) => self.uart.push(&data),
            Event::UartCleared => self.uart.clear(),
            Event::Error(error) => {
                self.sim_status.running = false;
                self.message = error;
            }
        }
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
        if ui.selectable_label(self.legend_open, "Legend").clicked() {
            self.legend_open = true;
        }
    }

    fn toolbar_view(&mut self, ui: &mut egui::Ui) {
        self.zoom_controls(ui);
        ui.checkbox(&mut self.grid_visible, "Grid");
        ui.separator();
        if self.tool == Tool::Wire {
            self.wire_controls(ui);
        } else {
            self.placement_controls(ui);
        }
    }

    fn zoom_controls(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_id_salt("zoom_preset")
            .selected_text(format!("{:.0}%", self.camera.zoom * 100.0))
            .show_ui(ui, |ui| {
                for zoom in [0.5, 1.0, 2.0] {
                    ui.selectable_value(
                        &mut self.camera.zoom,
                        zoom,
                        format!("{:.0}%", zoom * 100.0),
                    );
                }
            });
        if ui.button("Fit").clicked() {
            self.fit_content(false);
        }
        if ui.button("Fit Sel").clicked() {
            self.fit_content(true);
        }
    }

    fn fit_content(&mut self, selection_only: bool) {
        let Some(rect) = self.canvas_rect else {
            return;
        };
        let selection = selection_only.then_some(&self.selection);
        let Some((min, max)) = content_bounds(&self.document, self.library, selection) else {
            return;
        };
        self.camera
            .fit_world(rect, grid_pos_f(min), grid_pos_f(max), 24.0);
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
        if ui.button("Z-H").clicked() {
            self.apply_z_route(true);
        }
        if ui.button("Z-V").clicked() {
            self.apply_z_route(false);
        }
    }

    fn apply_l_route(&mut self, horizontal_first: bool) {
        self.apply_route(l_route, horizontal_first);
    }

    fn apply_z_route(&mut self, horizontal_first: bool) {
        self.apply_route(z_route, horizontal_first);
    }

    fn apply_route(
        &mut self,
        build: fn(GridPos, GridPos, bool) -> Vec<GridPos>,
        horizontal_first: bool,
    ) {
        let Some(&last) = self.wire_points.last() else {
            return;
        };
        let Some(world) = self.pointer_world else {
            return;
        };
        let target = world_to_grid(world);
        let path = build(last, target, horizontal_first);
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
        }
    }

    fn canvas(&mut self, ui: &mut egui::Ui) -> canvas::CanvasState {
        let config = SceneConfig {
            selection: &self.selection,
            hover_wire: self
                .pointer_world
                .and_then(|point| self.document.wire_at([point.x, point.y], 0.35)),
            placing: self.placing,
            rotation: self.placement_rotation,
            mirror: self.placement_mirror,
            wire_preview: &self.wire_points,
            wire_class: self.wire_class,
            selection_box: self.selection_box(),
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
            self.open_context_menu(&state);
            return;
        }
        if !state.clicked {
            return;
        }
        let Some(world) = state.click_world else {
            return;
        };
        self.select_at(world, state.multi_select);
    }

    fn select_at(&mut self, world: egui::Pos2, additive: bool) {
        let point = world_to_grid(world);
        if let Some(id) = self.document.hit_test(self.library, point) {
            if additive {
                self.selection.toggle_component(id);
            } else {
                self.selection.select_component(id);
            }
            return;
        }
        if let Some(id) = self.document.wire_at([world.x, world.y], 0.35) {
            if additive {
                self.selection.toggle_wire(id);
            } else {
                self.selection.select_wire(id);
            }
            return;
        }
        if !additive {
            self.selection.clear();
        }
    }

    fn open_context_menu(&mut self, state: &canvas::CanvasState) {
        let Some(click) = state.context_click else {
            return;
        };
        self.select_context_target(click.world);
        self.context_menu = Some(click.screen);
    }

    fn select_context_target(&mut self, world: egui::Pos2) {
        let point = world_to_grid(world);
        if let Some(id) = self.document.hit_test(self.library, point) {
            if !self.selection.contains_component(id) {
                self.selection.select_component(id);
            }
            return;
        }
        if let Some(id) = self.document.wire_at([world.x, world.y], 0.35) {
            if !self.selection.contains_wire(id) {
                self.selection.select_wire(id);
            }
            return;
        }
        self.selection.clear();
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
            self.begin_box(point, state.multi_select);
            return;
        };
        self.ensure_selected(id, state.multi_select);
        let offsets = self.drag_offsets(point);
        self.dragging = Some(DragState { offsets });
        self.drag_history_pending = true;
    }

    fn begin_box(&mut self, point: GridPos, additive: bool) {
        self.box_start = Some(point);
        self.box_current = Some(point);
        self.box_additive = additive;
    }

    fn ensure_selected(&mut self, id: ComponentId, additive: bool) {
        if self.selection.contains_component(id) {
            return;
        }
        if additive {
            self.selection.add_component(id);
        } else {
            self.selection.select_component(id);
        }
    }

    fn drag_offsets(&self, point: GridPos) -> Vec<(ComponentId, GridPos)> {
        self.selection
            .components()
            .iter()
            .filter_map(|id| {
                let component = self.document.component(*id)?;
                Some((*id, component.position.subtract(point)))
            })
            .collect()
    }

    fn begin_segment_drag(&mut self, state: &canvas::CanvasState) -> bool {
        let Some(wire_id) = self.selection.single_wire() else {
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
        self.drag_history_pending = true;
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
        if self.update_box(state) {
            return;
        }
        let Some(moves) = self.drag_moves(state) else {
            return;
        };
        if !self.any_moved(&moves) {
            return;
        }
        let before = self.document.clone();
        if self.document.move_components(self.library, &moves) {
            self.record_drag(&before);
        }
    }

    fn drag_moves(&self, state: &canvas::CanvasState) -> Option<Vec<(ComponentId, GridPos)>> {
        let drag = self.dragging.as_ref()?;
        let world = state.pointer_world?;
        let point = world_to_grid(world);
        Some(
            drag.offsets
                .iter()
                .map(|(id, offset)| (*id, point.offset(*offset)))
                .collect(),
        )
    }

    fn any_moved(&self, moves: &[(ComponentId, GridPos)]) -> bool {
        moves.iter().any(|(id, position)| {
            self.document
                .component(*id)
                .is_some_and(|component| component.position != *position)
        })
    }

    fn update_box(&mut self, state: &canvas::CanvasState) -> bool {
        if self.box_start.is_none() {
            return false;
        }
        if let Some(world) = state.pointer_world {
            self.box_current = Some(world_to_grid(world));
        }
        true
    }

    fn record_drag(&mut self, before: &Document) {
        if !self.drag_history_pending {
            return;
        }
        self.history.record(before);
        self.drag_history_pending = false;
        self.mark_dirty();
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
        let before = self.document.clone();
        if self
            .document
            .update_wire_points(self.library, drag.wire, points)
        {
            self.record_drag(&before);
        }
        true
    }

    fn end_drag(&mut self, state: &canvas::CanvasState) {
        if !state.drag_stopped {
            return;
        }
        if let (Some(start), Some(end)) = (self.box_start, self.box_current) {
            self.apply_box(start, end, self.box_additive);
        }
        self.box_start = None;
        self.box_current = None;
        self.box_additive = false;
        self.dragging = None;
        self.dragging_segment = None;
        self.drag_history_pending = false;
    }

    fn apply_box(&mut self, start: GridPos, end: GridPos, additive: bool) {
        let rect = GridRect::from_corners(start, end);
        let components = self.document.components_in_rect(self.library, rect);
        let wires = self.document.wires_in_rect(rect);
        self.selection.apply(components, wires, additive);
    }

    fn selection_box(&self) -> Option<(GridPos, GridPos)> {
        Some((self.box_start?, self.box_current?))
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
        self.push_undo();
        let id = self.document.add(
            definition,
            position,
            self.placement_rotation,
            self.placement_mirror,
        );
        self.selection.select_component(id);
        self.placing = None;
        self.mark_dirty();
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
        let before = self.document.clone();
        if self
            .document
            .add_wire(self.library, points, self.wire_class)
            .is_some()
        {
            self.history.record(&before);
            self.wire_points.clear();
            self.mark_dirty();
        }
    }

    fn handle_keys(&mut self, ui: &egui::Ui) {
        if self.palette.open {
            return;
        }
        if ui.ctx().egui_wants_keyboard_input() {
            return;
        }
        self.handle_file_keys(ui);
        self.handle_history_keys(ui);
        self.handle_clipboard_keys(ui);
        self.handle_edit_keys(ui);
    }

    fn handle_history_keys(&mut self, ui: &egui::Ui) {
        let undo = ui.input(|input| {
            input.modifiers.command && !input.modifiers.shift && input.key_pressed(egui::Key::Z)
        });
        let redo = ui.input(|input| {
            input.modifiers.command
                && (input.key_pressed(egui::Key::Y)
                    || (input.modifiers.shift && input.key_pressed(egui::Key::Z)))
        });
        if undo {
            self.undo();
        }
        if redo {
            self.redo();
        }
    }

    fn handle_clipboard_keys(&mut self, ui: &egui::Ui) {
        let copy = ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::C));
        let paste = ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::V));
        let duplicate =
            ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::D));
        if copy {
            self.copy_selection();
        }
        if paste {
            self.paste_selection();
        }
        if duplicate {
            self.duplicate_selection();
        }
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
        let ids: Vec<_> = self.selection.components().iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        let before = self.document.clone();
        if self.document.rotate_components(self.library, &ids) {
            self.history.record(&before);
            self.mark_dirty();
        }
    }

    fn mirror(&mut self) {
        if self.placing.is_some() {
            self.placement_mirror = self.placement_mirror.next();
            return;
        }
        let ids: Vec<_> = self.selection.components().iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        let before = self.document.clone();
        if self.document.mirror_components(self.library, &ids) {
            self.history.record(&before);
            self.mark_dirty();
        }
    }

    fn align_selection(&mut self, align: Align) {
        let moves = layout::align_moves(&self.document, self.library, &self.selection, align);
        self.apply_layout_moves(moves);
    }

    fn distribute_selection(&mut self, axis: Distribute) {
        let moves = layout::distribute_moves(&self.document, self.library, &self.selection, axis);
        self.apply_layout_moves(moves);
    }

    fn apply_layout_moves(&mut self, mut moves: Vec<(ComponentId, GridPos)>) {
        moves.retain(|(id, position)| {
            self.document
                .component(*id)
                .is_some_and(|component| component.position != *position)
        });
        if moves.is_empty() {
            return;
        }
        let before = self.document.clone();
        if self.document.move_components(self.library, &moves) {
            self.history.record(&before);
            self.mark_dirty();
            self.message = "Layout applied".to_string();
            return;
        }
        self.message = "Layout blocked by overlap".to_string();
    }

    fn legend_window(&mut self, ui: &mut egui::Ui) {
        if !self.legend_open {
            return;
        }
        let mut open = self.legend_open;
        egui::Window::new("Net legend")
            .open(&mut open)
            .default_width(190.0)
            .show(ui.ctx(), |ui| {
                show_net_legend(ui);
            });
        self.legend_open = open;
    }

    fn show_context_menu(&mut self, ui: &mut egui::Ui) {
        let Some(position) = self.context_menu else {
            return;
        };
        if self.conflict || self.palette.open {
            self.context_menu = None;
            return;
        }
        let (action, response) = show_context_area(
            ui.ctx(),
            position,
            &self.selection,
            self.clipboard.is_empty(),
        );
        let close = action.is_none()
            && ui.input(|input| {
                input.key_pressed(egui::Key::Escape)
                    || (!input.pointer.secondary_clicked()
                        && input.pointer.any_click()
                        && !response.contains_pointer())
            });
        if close {
            self.context_menu = None;
        }
        if let Some(action) = action {
            self.context_menu = None;
            self.run_context_action(action);
        }
    }

    fn run_context_action(&mut self, action: ContextAction) {
        match action {
            ContextAction::Copy => self.copy_selection(),
            ContextAction::Paste => self.paste_selection(),
            ContextAction::Duplicate => self.duplicate_selection(),
            ContextAction::Delete => self.delete_selection(),
            ContextAction::Rotate => self.rotate(),
            ContextAction::Mirror => self.mirror(),
            ContextAction::Properties => self.inspector_open = true,
            ContextAction::SelectTool => self.set_tool(Tool::Select),
            ContextAction::WireTool => self.set_tool(Tool::Wire),
        }
    }

    fn palette_shortcut(&mut self, ui: &egui::Ui) {
        if self.conflict {
            self.palette.close();
            return;
        }
        let toggle = ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::K));
        if toggle {
            self.palette.toggle();
        }
    }

    fn command_palette(&mut self, ui: &mut egui::Ui) {
        if !self.palette.open {
            return;
        }
        if self.conflict {
            self.palette.close();
            return;
        }
        let running = self.sim_status.running;
        let matches = palette::matching(&self.palette.query, running);
        let mut chosen = None;
        let response = egui::Modal::new(egui::Id::unique("command-palette")).show(ui.ctx(), |ui| {
            self.palette_contents(ui, running, &matches, &mut chosen);
        });
        if response.backdrop_response.clicked() {
            self.palette.close();
        }
        if let Some(command) = chosen {
            self.run_command(command);
        }
    }

    fn palette_contents(
        &mut self,
        ui: &mut egui::Ui,
        running: bool,
        matches: &[Command],
        chosen: &mut Option<Command>,
    ) {
        ui.set_width(460.0);
        self.palette_input(ui, matches.len());
        ui.separator();
        palette_list(ui, running, matches, self.palette.selected, chosen);
        self.palette_submit(ui, matches, chosen);
    }

    fn palette_input(&mut self, ui: &mut egui::Ui, count: usize) {
        self.palette_keys(ui, count);
        let edit = ui.text_edit_singleline(&mut self.palette.query);
        if self.palette.focus {
            edit.request_focus();
            self.palette.focus = false;
        }
        if edit.changed() {
            self.palette.selected = 0;
        }
    }

    fn palette_keys(&mut self, ui: &egui::Ui, count: usize) {
        let down =
            ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
        let up = ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
        if down {
            self.palette.move_selection(count, 1);
        }
        if up {
            self.palette.move_selection(count, -1);
        }
    }

    fn palette_submit(&mut self, ui: &egui::Ui, matches: &[Command], chosen: &mut Option<Command>) {
        let enter =
            ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
        if enter {
            *chosen = matches.get(self.palette.selected).copied();
        }
        let escape =
            ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if escape {
            self.palette.close();
        }
    }

    fn run_command(&mut self, command: Command) {
        self.palette.close();
        if self.run_edit_command(command) {
            return;
        }
        if self.run_layout_command(command) {
            return;
        }
        self.run_view_command(command);
    }

    fn run_layout_command(&mut self, command: Command) -> bool {
        match command {
            Command::AlignLeft => self.align_selection(Align::Left),
            Command::AlignRight => self.align_selection(Align::Right),
            Command::AlignTop => self.align_selection(Align::Top),
            Command::AlignBottom => self.align_selection(Align::Bottom),
            Command::AlignCenterX => self.align_selection(Align::CenterX),
            Command::AlignCenterY => self.align_selection(Align::CenterY),
            Command::DistributeHorizontal => self.distribute_selection(Distribute::Horizontal),
            Command::DistributeVertical => self.distribute_selection(Distribute::Vertical),
            _ => return false,
        }
        true
    }

    fn run_edit_command(&mut self, command: Command) -> bool {
        match command {
            Command::Save => self.save_project(),
            Command::Open => self.open_project(),
            Command::LoadHex => self.open_hex(),
            Command::RunPause => self.toggle_run(),
            Command::Step => self.send_sim(SimAction::Step),
            Command::Reset => self.send_sim(SimAction::Reset),
            Command::Undo => self.undo(),
            Command::Redo => self.redo(),
            Command::Copy => self.copy_selection(),
            Command::Paste => self.paste_selection(),
            Command::Duplicate => self.duplicate_selection(),
            Command::Delete => self.delete_selection(),
            _ => return false,
        }
        true
    }

    fn run_view_command(&mut self, command: Command) {
        match command {
            Command::ToggleGrid => self.grid_visible = !self.grid_visible,
            Command::ToggleLibrary => self.library_open = !self.library_open,
            Command::ToggleInspector => self.inspector_open = !self.inspector_open,
            Command::ToggleLegend => self.legend_open = !self.legend_open,
            Command::ToggleWaveform => self.waveform_open = !self.waveform_open,
            Command::ToggleUart => self.uart_open = !self.uart_open,
            Command::ZoomReset => self.camera.zoom = 1.0,
            Command::SelectTool => self.set_tool(Tool::Select),
            Command::WireTool => self.set_tool(Tool::Wire),
            _ => {}
        }
    }

    fn set_tool(&mut self, tool: Tool) {
        if self.tool == tool {
            return;
        }
        self.tool = tool;
        self.wire_points.clear();
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

    fn uart_window(&mut self, ui: &mut egui::Ui) {
        if !self.uart_open {
            return;
        }
        let mut open = self.uart_open;
        let mut clear = false;
        egui::Window::new("UART monitor")
            .open(&mut open)
            .default_size([420.0, 220.0])
            .show(ui.ctx(), |ui| {
                show_uart_contents(ui, &self.uart, &mut clear);
            });
        self.uart_open = open;
        if clear {
            self.uart.clear();
        }
    }

    fn waveform_window(&mut self, ui: &mut egui::Ui) {
        if !self.waveform_open {
            return;
        }
        let mut open = self.waveform_open;
        let status = self.sim_status;
        egui::Window::new("Digital graph")
            .open(&mut open)
            .default_size([520.0, 220.0])
            .show(ui.ctx(), |ui| {
                show_digital_graph(ui, &self.waveform, status);
            });
        self.waveform_open = open;
    }

    fn delete_modal(&mut self, ui: &mut egui::Ui) {
        if !self.pending_delete || self.conflict {
            return;
        }
        let mut action = None;
        egui::Modal::new(egui::Id::unique("delete-confirm")).show(ui.ctx(), |ui| {
            ui.set_width(320.0);
            ui.heading("Delete selection?");
            ui.label(format!("{} items will be removed.", self.selection.len()));
            ui.horizontal(|ui| {
                if ui.button("Delete").clicked() {
                    action = Some(true);
                }
                if ui.button("Cancel").clicked() {
                    action = Some(false);
                }
            });
        });
        match action {
            Some(true) => {
                self.pending_delete = false;
                self.delete_selection_now();
            }
            Some(false) => self.pending_delete = false,
            None => {}
        }
    }

    fn conflict_modal(&mut self, ui: &mut egui::Ui) {
        if !self.conflict {
            return;
        }
        let path = self.current_path.clone().unwrap_or_default();
        let mut action = None;
        egui::Modal::new(egui::Id::unique("file-conflict")).show(ui.ctx(), |ui| {
            conflict_contents(ui, &path, &mut action);
        });
        if let Some(action) = action {
            self.apply_conflict(action);
        }
    }

    fn apply_conflict(&mut self, action: ConflictAction) {
        match action {
            ConflictAction::Reload => self.reload_from_disk(),
            ConflictAction::SaveMine => self.save_project_now(),
        }
    }

    fn reload_from_disk(&mut self) {
        let Some(path) = self.current_path.clone() else {
            self.conflict = false;
            return;
        };
        match persist::load(&path, self.library) {
            Ok(project) => {
                self.apply_project(project, path);
                self.message = "Reloaded".to_string();
            }
            Err(error) => self.message = error,
        }
    }

    fn inspector_contents(&self, ui: &mut egui::Ui, action: &mut Option<InspectorAction>) {
        if let Some(id) = self.selection.single_wire() {
            self.wire_inspector(ui, action, id);
            return;
        }
        if let Some((component, definition)) = self.selected_pair() {
            show_component_info(ui, component, definition);
            show_component_actions(ui, action);
            return;
        }
        if self.selection.is_empty() {
            ui.label("No selection");
            return;
        }
        self.multi_inspector(ui, action);
    }

    fn multi_inspector(&self, ui: &mut egui::Ui, action: &mut Option<InspectorAction>) {
        ui.label(format!(
            "{} components, {} wires",
            self.selection.components().len(),
            self.selection.wires().len()
        ));
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
        let id = self.selection.single_component()?;
        let component = self.document.component(id)?;
        let definition = library::find(component.definition)?;
        Some((component, definition))
    }

    fn copy_selection(&mut self) {
        if self.selection.components().is_empty() {
            self.message = "Nothing to copy".to_string();
            return;
        }
        self.clipboard = Clipboard::capture(
            &self.document,
            self.selection.components(),
            self.selection.wires(),
        );
        if self.clipboard.is_empty() {
            self.message = "Nothing to copy".to_string();
            return;
        }
        self.message = format!(
            "Copied {} components, {} wires",
            self.selection.components().len(),
            self.selection.wires().len()
        );
    }

    fn paste_selection(&mut self) {
        let offset = self.paste_offset();
        let before = self.document.clone();
        match self
            .clipboard
            .paste(&mut self.document, self.library, offset)
        {
            Ok(result) => self.after_paste(result, before),
            Err(error) => self.message = error,
        }
    }

    fn duplicate_selection(&mut self) {
        let clipboard = Clipboard::capture(
            &self.document,
            self.selection.components(),
            self.selection.wires(),
        );
        if clipboard.is_empty() {
            self.message = "Nothing to duplicate".to_string();
            return;
        }
        let before = self.document.clone();
        match clipboard.paste(&mut self.document, self.library, GridPos::new(2, 2)) {
            Ok(result) => self.after_paste(result, before),
            Err(error) => self.message = error,
        }
    }

    fn after_paste(&mut self, result: clipboard::PasteResult, before: Document) {
        let components = result.components.len();
        let wires = result.wires.len();
        self.history.record(&before);
        self.selection.apply(result.components, result.wires, false);
        self.mark_dirty();
        self.message = format!("Pasted {components} components, {wires} wires");
    }

    fn paste_offset(&self) -> GridPos {
        let Some(world) = self.pointer_world else {
            return GridPos::new(2, 2);
        };
        let offset = world_to_grid(world).subtract(self.clipboard.origin());
        if offset == GridPos::new(0, 0) {
            GridPos::new(2, 2)
        } else {
            offset
        }
    }

    fn push_undo(&mut self) {
        self.history.record(&self.document);
    }

    fn undo(&mut self) {
        let Some(previous) = self.history.undo(&self.document) else {
            return;
        };
        self.document = previous;
        self.reset_transient();
        self.message = "Undo".to_string();
        self.mark_dirty();
    }

    fn redo(&mut self) {
        let Some(next) = self.history.redo(&self.document) else {
            return;
        };
        self.document = next;
        self.reset_transient();
        self.message = "Redo".to_string();
        self.mark_dirty();
    }

    fn reset_transient(&mut self) {
        self.selection.clear();
        self.box_start = None;
        self.box_current = None;
        self.box_additive = false;
        self.pending_delete = false;
        self.dragging = None;
        self.dragging_segment = None;
        self.drag_history_pending = false;
        self.placing = None;
        self.wire_points.clear();
    }

    fn delete_selection(&mut self) {
        if self.selection.is_empty() {
            return;
        }
        if self.needs_delete_confirmation() {
            self.pending_delete = true;
            self.context_menu = None;
            self.palette.close();
            return;
        }
        self.delete_selection_now();
    }

    fn needs_delete_confirmation(&self) -> bool {
        if self.selection.len() > 5 {
            return true;
        }
        self.selection
            .components()
            .iter()
            .any(|id| self.document.component_has_wire(self.library, *id))
    }

    fn delete_selection_now(&mut self) {
        self.push_undo();
        let wires: Vec<_> = self.selection.wires().iter().copied().collect();
        let components: Vec<_> = self.selection.components().iter().copied().collect();
        for id in wires {
            self.document.remove_wire(id);
        }
        for id in components {
            self.document.remove(id);
        }
        self.selection.clear();
        self.mark_dirty();
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
            self.view_status(ui);
            ui.separator();
            self.edit_status(ui);
            ui.separator();
            self.sim_status_labels(ui);
            ui.separator();
            ui.label(self.fps_text(ui));
        });
    }

    fn view_status(&self, ui: &mut egui::Ui) {
        ui.label(self.pointer_text());
        ui.separator();
        ui.label(format!("Zoom {:.0}%", self.camera.zoom * 100.0));
        ui.separator();
        ui.label(format!("Grid {:.0}px", self.camera.cell_px));
        ui.separator();
        ui.label("Snap grid");
    }

    fn edit_status(&self, ui: &mut egui::Ui) {
        ui.label(format!("Tool: {}", self.tool.label()));
        ui.separator();
        let selection = self.selection.len();
        ui.label(format!("Sel {selection}"));
        ui.separator();
        ui.label(format!("Parts {}", self.document.components().len()));
    }

    fn sim_status_labels(&self, ui: &mut egui::Ui) {
        ui.label(self.sim_text());
        ui.separator();
        ui.label(self.led_text());
    }

    fn sim_text(&self) -> String {
        if !self.sim_status.loaded {
            return "Sim idle".to_string();
        }
        let state = if self.sim_status.running {
            "running"
        } else {
            "paused"
        };
        format!("Sim {} cycles ({state})", self.sim_status.cycles)
    }

    fn led_text(&self) -> String {
        if !self.sim_status.loaded {
            return "PB5 -".to_string();
        }
        format!(
            "PB5 {}  LED {}",
            high_low(self.sim_status.pb5_high),
            on_off(self.sim_status.led_on)
        )
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
        selection: config.selection,
        placing: config.placing,
        hover: None,
        rotation: config.rotation,
        mirror: config.mirror,
        hover_wire: config.hover_wire,
        wire_preview: config.wire_preview,
        wire_hover: None,
        wire_class: config.wire_class,
        selection_box: config.selection_box,
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

fn file_stamp(path: &Path) -> Option<FileStamp> {
    let metadata = std::fs::metadata(path).ok()?;
    Some(FileStamp {
        modified: metadata.modified().ok(),
        len: metadata.len(),
    })
}

fn watch_remaining(last: Option<Instant>, now: Instant) -> Duration {
    last.map_or(Duration::ZERO, |last| {
        WATCH_INTERVAL.saturating_sub(now.saturating_duration_since(last))
    })
}

fn conflict_contents(ui: &mut egui::Ui, path: &Path, action: &mut Option<ConflictAction>) {
    ui.set_width(380.0);
    ui.heading("File changed on disk");
    ui.label("The project was modified outside embed-emu.");
    ui.label(path.display().to_string());
    ui.label("Reload discards local edits. Save Mine overwrites the file.");
    ui.horizontal(|ui| {
        if ui.button("Reload").clicked() {
            *action = Some(ConflictAction::Reload);
        }
        if ui.button("Save Mine").clicked() {
            *action = Some(ConflictAction::SaveMine);
        }
    });
}

fn configure_style(context: &eframe::CreationContext<'_>) {
    context.egui_ctx.set_theme(egui::ThemePreference::Dark);
    context.egui_ctx.style_mut_of(egui::Theme::Dark, |style| {
        for font in style.text_styles.values_mut() {
            font.family = egui::FontFamily::Monospace;
        }
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(0, 150, 170);
    });
}

fn load_config() -> (Option<PathBuf>, config::AppConfig) {
    let Some(path) = config::path() else {
        return (None, config::AppConfig::default());
    };
    let config = config::load(&path);
    (Some(path), config)
}

fn select_save_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("embed-emu project", &["json"])
        .set_file_name("project.emu.json")
        .save_file()
}

fn pick_hex() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Intel HEX", &["hex", "ihx"])
        .pick_file()
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name().map_or_else(
        || "firmware".to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

fn show_context_area(
    ctx: &egui::Context,
    position: egui::Pos2,
    selection: &Selection,
    clipboard_empty: bool,
) -> (Option<ContextAction>, egui::Response) {
    let mut action = None;
    let area = egui::Area::new(egui::Id::unique("canvas-context"))
        .fixed_pos(position)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(170.0);
                context_buttons(ui, selection, clipboard_empty, &mut action);
            });
        });
    (action, area.response)
}

fn context_buttons(
    ui: &mut egui::Ui,
    selection: &Selection,
    clipboard_empty: bool,
    action: &mut Option<ContextAction>,
) {
    context_edit_buttons(ui, selection, clipboard_empty, action);
    ui.separator();
    context_transform_buttons(ui, selection, action);
    ui.separator();
    context_tool_buttons(ui, action);
}

fn context_edit_buttons(
    ui: &mut egui::Ui,
    selection: &Selection,
    clipboard_empty: bool,
    action: &mut Option<ContextAction>,
) {
    let any = !selection.is_empty();
    context_button(ui, any, "Copy", ContextAction::Copy, action);
    context_button(ui, !clipboard_empty, "Paste", ContextAction::Paste, action);
    context_button(ui, any, "Duplicate", ContextAction::Duplicate, action);
    context_button(ui, any, "Delete", ContextAction::Delete, action);
    context_button(ui, any, "Properties", ContextAction::Properties, action);
}

fn context_button(
    ui: &mut egui::Ui,
    enabled: bool,
    label: &str,
    value: ContextAction,
    action: &mut Option<ContextAction>,
) {
    if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
        *action = Some(value);
    }
}

fn context_transform_buttons(
    ui: &mut egui::Ui,
    selection: &Selection,
    action: &mut Option<ContextAction>,
) {
    let components = !selection.components().is_empty();
    if ui
        .add_enabled(components, egui::Button::new("Rotate"))
        .clicked()
    {
        *action = Some(ContextAction::Rotate);
    }
    if ui
        .add_enabled(components, egui::Button::new("Mirror"))
        .clicked()
    {
        *action = Some(ContextAction::Mirror);
    }
}

fn context_tool_buttons(ui: &mut egui::Ui, action: &mut Option<ContextAction>) {
    if ui.button("Select tool").clicked() {
        *action = Some(ContextAction::SelectTool);
    }
    if ui.button("Wire tool").clicked() {
        *action = Some(ContextAction::WireTool);
    }
}

fn palette_list(
    ui: &mut egui::Ui,
    running: bool,
    matches: &[Command],
    selected: usize,
    chosen: &mut Option<Command>,
) {
    egui::ScrollArea::vertical()
        .max_height(320.0)
        .show(ui, |ui| {
            for (index, command) in matches.iter().enumerate() {
                let active = index == selected;
                if ui
                    .selectable_label(active, command.label(running))
                    .clicked()
                {
                    *chosen = Some(*command);
                }
            }
        });
}

fn show_net_legend(ui: &mut egui::Ui) {
    for class in NetClass::ALL {
        ui.horizontal(|ui| {
            legend_swatch(ui, class);
            ui.label(class.label());
        });
    }
}

fn legend_swatch(ui: &mut egui::Ui, class: NetClass) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 8.0), egui::Sense::hover());
    let y = rect.center().y;
    let points = [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)];
    let stroke = egui::Stroke::new(2.0, scene::net_color(class));
    if let Some((dash, gap)) = scene::wire_dash(class) {
        ui.painter()
            .extend(egui::Shape::dashed_line(&points, stroke, dash, gap));
        return;
    }
    ui.painter().line_segment(points, stroke);
}

fn show_uart_contents(ui: &mut egui::Ui, log: &UartLog, clear: &mut bool) {
    ui.horizontal(|ui| {
        if ui.button("Clear").clicked() {
            *clear = true;
        }
        ui.label(format!("{} bytes", log.len()));
    });
    egui::ScrollArea::vertical()
        .stick_to_bottom(true)
        .show(ui, |ui| {
            show_uart_text(ui, log.text());
        });
}

fn show_uart_text(ui: &mut egui::Ui, text: &str) {
    if text.is_empty() {
        ui.label("No UART output");
        return;
    }
    let label = egui::RichText::new(text).monospace();
    ui.add(egui::Label::new(label).wrap());
}

fn show_digital_graph(ui: &mut egui::Ui, waveform: &Waveform, status: Status) {
    let span = waveform.span(status.cycles);
    ui.label(graph_header(status, span));
    let width = ui.available_width().max(220.0);
    let (response, painter) = ui.allocate_painter(egui::vec2(width, 130.0), egui::Sense::hover());
    let plot = response.rect.shrink2(egui::vec2(38.0, 12.0));
    let cursor = response.hover_pos().filter(|pos| plot.contains(*pos));
    paint_digital_graph(&painter, response.rect, plot, waveform, span, cursor);
    ui.label(graph_cursor_text(waveform, plot, span, cursor));
}

fn graph_header(status: Status, span: (u64, u64)) -> String {
    format!(
        "PB5 {}  LED {}  cycles {} to {}",
        high_low(status.pb5_high),
        on_off(status.led_on),
        span.0,
        span.1
    )
}

fn graph_cursor_text(
    waveform: &Waveform,
    plot: egui::Rect,
    span: (u64, u64),
    cursor: Option<egui::Pos2>,
) -> String {
    let Some(position) = cursor else {
        return "Hover the graph to inspect a cycle".to_string();
    };
    let cycle = cycle_at_x(plot, span, position.x);
    let (pb5, led) = waveform.state_at(cycle);
    format!("cycle {cycle}: PB5 {}  LED {}", high_low(pb5), on_off(led))
}

fn high_low(value: bool) -> &'static str {
    if value { "high" } else { "low" }
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

fn paint_digital_graph(
    painter: &egui::Painter,
    rect: egui::Rect,
    plot: egui::Rect,
    waveform: &Waveform,
    span: (u64, u64),
    cursor: Option<egui::Pos2>,
) {
    paint_graph_background(painter, rect, plot);
    paint_signals(painter, plot, waveform.samples(), span);
    paint_signal_labels(painter, rect, plot);
    paint_cursor(painter, plot, cursor);
}

fn paint_signals(
    painter: &egui::Painter,
    plot: egui::Rect,
    samples: &VecDeque<Sample>,
    span: (u64, u64),
) {
    let (top, bottom) = signal_rows(plot);
    paint_signal(painter, top, samples, span, pb5_value, PB5_COLOR);
    paint_signal(painter, bottom, samples, span, led_value, LED_COLOR);
}

fn signal_rows(plot: egui::Rect) -> (egui::Rect, egui::Rect) {
    let middle = plot.center().y;
    (
        egui::Rect::from_min_max(plot.min, egui::pos2(plot.max.x, middle)),
        egui::Rect::from_min_max(egui::pos2(plot.min.x, middle), plot.max),
    )
}

fn paint_graph_background(painter: &egui::Painter, rect: egui::Rect, plot: egui::Rect) {
    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(18, 22, 26));
    let middle = plot.center().y;
    let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 78, 90));
    painter.line_segment(
        [
            egui::pos2(plot.left(), middle),
            egui::pos2(plot.right(), middle),
        ],
        stroke,
    );
}

fn paint_cursor(painter: &egui::Painter, plot: egui::Rect, cursor: Option<egui::Pos2>) {
    let Some(position) = cursor else {
        return;
    };
    let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(240, 220, 120));
    painter.line_segment(
        [
            egui::pos2(position.x, plot.top()),
            egui::pos2(position.x, plot.bottom()),
        ],
        stroke,
    );
}

fn pb5_value(sample: &Sample) -> bool {
    sample.pb5_high
}

fn led_value(sample: &Sample) -> bool {
    sample.led_on
}

fn paint_signal(
    painter: &egui::Painter,
    row: egui::Rect,
    samples: &VecDeque<Sample>,
    span: (u64, u64),
    value: fn(&Sample) -> bool,
    color: egui::Color32,
) {
    let points = signal_points(row, samples, span, value);
    if points.is_empty() {
        return;
    }
    painter.add(egui::Shape::line(points, egui::Stroke::new(1.5, color)));
}

fn paint_signal_labels(painter: &egui::Painter, rect: egui::Rect, plot: egui::Rect) {
    let dim = egui::Color32::from_rgb(150, 160, 170);
    let font = egui::FontId::monospace(10.0);
    let high = egui::pos2(rect.left() + 4.0, plot.top() + 4.0);
    let low = egui::pos2(rect.left() + 4.0, plot.bottom() - 4.0);
    painter.text(high, egui::Align2::LEFT_TOP, "PB5", font.clone(), dim);
    painter.text(low, egui::Align2::LEFT_BOTTOM, "LED", font, dim);
}

fn cycle_at_x(plot: egui::Rect, span: (u64, u64), x: f32) -> u64 {
    let ratio = ((x - plot.left()) / plot.width().max(1.0)).clamp(0.0, 1.0);
    let length = span.1.saturating_sub(span.0);
    span.0 + (f64::from(ratio) * length as f64).round() as u64
}

fn signal_points(
    row: egui::Rect,
    samples: &VecDeque<Sample>,
    span: (u64, u64),
    value: fn(&Sample) -> bool,
) -> Vec<egui::Pos2> {
    let Some(first) = samples.front() else {
        return Vec::new();
    };
    let width = span.1.saturating_sub(span.0).max(1) as f32;
    let x = |cycle: u64| row.left() + (cycle.saturating_sub(span.0) as f32 / width) * row.width();
    let y = |on: bool| if on { row.top() } else { row.bottom() };
    let mut points = vec![egui::pos2(row.left(), y(value(first)))];
    let mut previous = first;
    for sample in samples.iter().skip(1) {
        let px = x(sample.cycle);
        points.push(egui::pos2(px, y(value(previous))));
        points.push(egui::pos2(px, y(value(sample))));
        previous = sample;
    }
    points.push(egui::pos2(row.right(), y(value(previous))));
    points
}

fn world_to_grid(position: egui::Pos2) -> GridPos {
    GridPos::new(position.x.round() as i32, position.y.round() as i32)
}

fn grid_pos_f(position: GridPos) -> egui::Pos2 {
    egui::pos2(position.x as f32, position.y as f32)
}

fn content_bounds(
    document: &Document,
    definitions: &[ComponentDefinition],
    selection: Option<&Selection>,
) -> Option<(GridPos, GridPos)> {
    let mut min = None;
    let mut max = None;
    include_components(document, definitions, selection, &mut min, &mut max);
    include_wires(document, selection, &mut min, &mut max);
    Some((min?, max?))
}

fn include_components(
    document: &Document,
    definitions: &[ComponentDefinition],
    selection: Option<&Selection>,
    min: &mut Option<GridPos>,
    max: &mut Option<GridPos>,
) {
    for instance in document.components() {
        if selection.is_some_and(|selection| !selection.contains_component(instance.id)) {
            continue;
        }
        let Some(definition) = definitions
            .iter()
            .find(|definition| definition.id == instance.definition)
        else {
            continue;
        };
        let bounds = instance.bounds(definition);
        update_bounds(min, max, bounds.min);
        update_bounds(min, max, GridPos::new(bounds.max_x(), bounds.max_y()));
    }
}

fn include_wires(
    document: &Document,
    selection: Option<&Selection>,
    min: &mut Option<GridPos>,
    max: &mut Option<GridPos>,
) {
    for wire in document.wires() {
        if selection.is_some_and(|selection| !selection.contains_wire(wire.id)) {
            continue;
        }
        for point in &wire.points {
            update_bounds(min, max, *point);
        }
    }
}

fn update_bounds(min: &mut Option<GridPos>, max: &mut Option<GridPos>, point: GridPos) {
    let low = min.get_or_insert(point);
    low.x = low.x.min(point.x);
    low.y = low.y.min(point.y);
    let high = max.get_or_insert(point);
    high.x = high.x.max(point.x);
    high.y = high.y.max(point.y);
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

fn z_route(last: GridPos, target: GridPos, horizontal_first: bool) -> Vec<GridPos> {
    if last == target || last.x == target.x || last.y == target.y {
        return l_route(last, target, horizontal_first);
    }
    let path = if horizontal_first {
        let middle = GridPos::new((last.x + target.x) / 2, last.y);
        vec![middle, GridPos::new(middle.x, target.y), target]
    } else {
        let middle = GridPos::new(last.x, (last.y + target.y) / 2);
        vec![middle, GridPos::new(target.x, middle.y), target]
    };
    without_repeats(last, path)
}

fn without_repeats(start: GridPos, path: Vec<GridPos>) -> Vec<GridPos> {
    let mut result = Vec::new();
    let mut previous = start;
    for point in path {
        if point == previous {
            continue;
        }
        result.push(point);
        previous = point;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{
        AUTOSAVE_DELAY, ConflictAction, GuiApp, WATCH_INTERVAL, content_bounds, cycle_at_x,
        file_stamp, l_route, move_segment, watch_remaining, z_route,
    };
    use crate::canvas::GridCamera;
    use crate::model::{
        ComponentDefinition, ComponentId, Document, GridPos, Mirror, NetClass, Rotation, WireId,
    };
    use crate::persist;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("embed-emu-app-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn dirty_app(path: &Path) -> GuiApp {
        let resistor = crate::library::find("resistor").expect("resistor");
        let mut document = Document::new();
        document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        GuiApp {
            library: crate::library::builtin(),
            document,
            current_path: Some(path.to_path_buf()),
            dirty: true,
            last_edit: Some(Instant::now()),
            ..GuiApp::default()
        }
    }

    fn sample_wire(document: &mut Document, library: &[ComponentDefinition]) -> WireId {
        document
            .add_wire(
                library,
                vec![GridPos::new(4, 1), GridPos::new(6, 1)],
                NetClass::Digital,
            )
            .expect("wire")
    }

    #[test]
    fn open_flushes_dirty_document_and_updates_recent() {
        let dir = temp_dir("open-flush");
        let old = dir.join("old.emu.json");
        let new = dir.join("new.emu.json");
        let camera = GridCamera::default();
        persist::save(&old, &Document::new(), &camera, true).expect("old");
        persist::save(&new, &Document::new(), &camera, true).expect("new");
        let mut app = dirty_app(&old);
        app.open_path(new.clone());
        let text = std::fs::read_to_string(&old).expect("flushed");
        assert!(text.contains("resistor"));
        assert_eq!(app.current_path, Some(new.clone()));
        assert_eq!(app.config.recent_files.first(), Some(&new));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn additive_click_collects_components() {
        let library = crate::library::builtin();
        let resistor = crate::library::find("resistor").expect("resistor");
        let mut document = Document::new();
        let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
        let mut app = GuiApp {
            library,
            document,
            ..GuiApp::default()
        };
        app.select_at(eframe::egui::pos2(1.0, 1.0), false);
        assert_eq!(app.selection.single_component(), Some(first));
        app.select_at(eframe::egui::pos2(7.0, 1.0), true);
        assert_eq!(app.selection.components().len(), 2);
    }

    #[test]
    fn drag_offsets_keep_relative_positions() {
        let library = crate::library::builtin();
        let resistor = crate::library::find("resistor").expect("resistor");
        let mut document = Document::new();
        let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        let second = document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
        let mut app = GuiApp {
            library,
            document,
            ..GuiApp::default()
        };
        app.selection.apply([first, second], [], false);
        let offsets = app.drag_offsets(GridPos::new(1, 0));
        assert_eq!(
            offsets,
            vec![(first, GridPos::new(-1, 0)), (second, GridPos::new(5, 0))]
        );
    }

    #[test]
    fn selection_box_selects_components_and_wires() {
        let library = crate::library::builtin();
        let resistor = crate::library::find("resistor").expect("resistor");
        let mut document = Document::new();
        document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
        let wire = sample_wire(&mut document, library);
        let mut app = GuiApp {
            library,
            document,
            ..GuiApp::default()
        };
        app.apply_box(GridPos::new(-1, -1), GridPos::new(10, 3), false);
        assert_eq!(app.selection.components().len(), 2);
        assert!(app.selection.contains_wire(wire));
    }

    #[test]
    fn copy_and_paste_selects_new_components() {
        let library = crate::library::builtin();
        let resistor = crate::library::find("resistor").expect("resistor");
        let mut document = Document::new();
        let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        let mut app = GuiApp {
            library,
            document,
            ..GuiApp::default()
        };
        app.selection.select_component(first);
        app.copy_selection();
        app.paste_selection();
        assert_eq!(app.document.components().len(), 2);
        assert_eq!(app.selection.components().len(), 1);
        assert_ne!(app.selection.single_component(), Some(first));
    }

    fn context_click(
        world: eframe::egui::Pos2,
        screen: eframe::egui::Pos2,
    ) -> crate::canvas::CanvasState {
        crate::canvas::CanvasState {
            context_click: Some(crate::canvas::ContextClick { screen, world }),
            ..Default::default()
        }
    }

    fn context_app() -> (GuiApp, ComponentId, ComponentId) {
        let library = crate::library::builtin();
        let resistor = crate::library::find("resistor").expect("resistor");
        let mut document = Document::new();
        let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        let second = document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
        (
            GuiApp {
                library,
                document,
                ..GuiApp::default()
            },
            first,
            second,
        )
    }

    #[test]
    fn content_bounds_cover_components() {
        let library = crate::library::builtin();
        let resistor = crate::library::find("resistor").expect("resistor");
        let mut document = Document::new();
        document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
        let bounds = content_bounds(&document, library, None).expect("bounds");
        assert_eq!(bounds, (GridPos::new(0, 0), GridPos::new(10, 2)));
    }

    #[test]
    fn delete_confirmation_for_wired_or_large_selection() {
        let (mut app, first, _) = context_app();
        app.selection.select_component(first);
        assert!(!app.needs_delete_confirmation());
        sample_wire(&mut app.document, app.library);
        assert!(app.needs_delete_confirmation());
        let large: Vec<_> = (0..6u32).map(ComponentId).collect();
        app.selection.apply(large, [], false);
        assert!(app.needs_delete_confirmation());
    }

    #[test]
    fn context_click_selects_target() {
        let (mut app, first, second) = context_app();
        app.selection.select_component(second);
        app.open_context_menu(&context_click(
            eframe::egui::pos2(1.0, 1.0),
            eframe::egui::pos2(10.0, 10.0),
        ));
        assert_eq!(app.selection.single_component(), Some(first));
        assert_eq!(app.context_menu, Some(eframe::egui::pos2(10.0, 10.0)));
    }

    #[test]
    fn context_click_empty_clears() {
        let (mut app, first, _) = context_app();
        app.selection.select_component(first);
        app.open_context_menu(&context_click(
            eframe::egui::pos2(30.0, 30.0),
            eframe::egui::pos2(20.0, 20.0),
        ));
        assert!(app.selection.is_empty());
    }

    #[test]
    fn open_is_blocked_for_unsaved_project() {
        let mut app = GuiApp {
            dirty: true,
            ..GuiApp::default()
        };
        assert!(!app.flush_autosave());
        assert!(app.message.contains("Save project"));
    }

    #[test]
    fn autosave_waits_for_conflict_resolution() {
        let now = Instant::now();
        let app = GuiApp {
            conflict: true,
            dirty: true,
            current_path: Some(PathBuf::from("/tmp/project.emu.json")),
            last_edit: Some(now),
            ..GuiApp::default()
        };
        assert_eq!(app.autosave_remaining(now + AUTOSAVE_DELAY), None);
    }

    #[test]
    fn watch_remaining_respects_interval() {
        let now = Instant::now();
        assert_eq!(watch_remaining(None, now), Duration::ZERO);
        assert_eq!(watch_remaining(Some(now), now), WATCH_INTERVAL);
        assert_eq!(
            watch_remaining(Some(now), now + WATCH_INTERVAL),
            Duration::ZERO
        );
    }

    #[test]
    fn external_write_sets_conflict() {
        let dir = temp_dir("watch");
        let path = dir.join("project.emu.json");
        let camera = GridCamera::default();
        persist::save(&path, &Document::new(), &camera, true).expect("save");
        let stamp = file_stamp(&path).expect("stamp");
        let past = Instant::now().checked_sub(WATCH_INTERVAL).expect("clock");
        let mut app = GuiApp {
            current_path: Some(path.clone()),
            file_stamp: Some(stamp),
            watch_at: Some(past),
            ..GuiApp::default()
        };
        std::fs::write(&path, "external change").expect("write");
        app.watch_file(&eframe::egui::Context::default());
        assert!(app.conflict);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn autosave_refuses_external_change() {
        let dir = temp_dir("autosave-conflict");
        let path = dir.join("project.emu.json");
        persist::save(&path, &Document::new(), &GridCamera::default(), true).expect("save");
        let stamp = file_stamp(&path).expect("stamp");
        let mut app = dirty_app(&path);
        app.file_stamp = Some(stamp);
        std::fs::write(&path, "external change").expect("write");
        app.write_autosave(Instant::now());
        assert!(app.conflict);
        assert!(app.dirty);
        let text = std::fs::read_to_string(&path).expect("file");
        assert_eq!(text, "external change");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn manual_save_detects_external_change() {
        let dir = temp_dir("save-watch");
        let path = dir.join("project.emu.json");
        persist::save(&path, &Document::new(), &GridCamera::default(), true).expect("save");
        let stamp = file_stamp(&path).expect("stamp");
        let mut app = dirty_app(&path);
        app.file_stamp = Some(stamp);
        std::fs::write(&path, "external change").expect("write");
        app.save_project();
        assert!(app.conflict);
        let text = std::fs::read_to_string(&path).expect("file");
        assert_eq!(text, "external change");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn reload_replaces_document_and_clears_conflict() {
        let dir = temp_dir("reload");
        let path = dir.join("project.emu.json");
        persist::save(&path, &Document::new(), &GridCamera::default(), true).expect("save");
        let mut external = Document::new();
        let resistor = crate::library::find("resistor").expect("resistor");
        external.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        persist::save(&path, &external, &GridCamera::default(), true).expect("external");
        let mut app = GuiApp {
            library: crate::library::builtin(),
            current_path: Some(path.clone()),
            conflict: true,
            ..GuiApp::default()
        };
        app.reload_from_disk();
        assert!(!app.conflict);
        assert_eq!(app.document.components().len(), 1);
        assert_eq!(app.message, "Reloaded");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn save_mine_overwrites_external_file() {
        let dir = temp_dir("save-mine");
        let path = dir.join("project.emu.json");
        let camera = GridCamera::default();
        persist::save(&path, &Document::new(), &camera, true).expect("save");
        let mut app = dirty_app(&path);
        app.conflict = true;
        app.apply_conflict(ConflictAction::SaveMine);
        assert!(!app.conflict);
        let text = std::fs::read_to_string(&path).expect("saved");
        assert!(text.contains("resistor"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn autosave_requires_a_path_and_full_delay() {
        let now = Instant::now();
        let mut app = GuiApp {
            dirty: true,
            last_edit: Some(now),
            ..GuiApp::default()
        };
        assert_eq!(app.autosave_remaining(now), None);
        app.current_path = Some(PathBuf::from("/tmp/project.emu.json"));
        assert_eq!(app.autosave_remaining(now), Some(AUTOSAVE_DELAY));
        assert_eq!(
            app.autosave_remaining(now + AUTOSAVE_DELAY),
            Some(Duration::ZERO)
        );
        app.current_path = None;
    }

    #[test]
    fn autosave_writes_and_clears_dirty() {
        let dir = std::env::temp_dir().join(format!("embed-emu-autosave-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("project.emu.json");
        let mut app = GuiApp {
            current_path: Some(path.clone()),
            dirty: true,
            last_edit: Some(Instant::now()),
            ..GuiApp::default()
        };
        app.write_autosave(Instant::now());
        assert!(!app.dirty);
        assert!(path.exists());
        assert_eq!(app.message, "Autosaved");
        let _ = std::fs::remove_dir_all(dir);
    }

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
    fn z_route_adds_two_corners() {
        let route = z_route(GridPos::new(0, 0), GridPos::new(6, 4), true);
        assert_eq!(
            route,
            vec![GridPos::new(3, 0), GridPos::new(3, 4), GridPos::new(6, 4)]
        );
        let route = z_route(GridPos::new(0, 0), GridPos::new(6, 4), false);
        assert_eq!(
            route,
            vec![GridPos::new(0, 2), GridPos::new(6, 2), GridPos::new(6, 4)]
        );
    }

    #[test]
    fn z_route_simplifies_short_and_aligned() {
        let route = z_route(GridPos::new(0, 0), GridPos::new(1, 2), true);
        assert_eq!(route, vec![GridPos::new(0, 2), GridPos::new(1, 2)]);
        assert_eq!(
            z_route(GridPos::new(0, 0), GridPos::new(4, 0), true),
            vec![GridPos::new(4, 0)]
        );
        assert!(z_route(GridPos::new(0, 0), GridPos::new(0, 0), false).is_empty());
    }

    #[test]
    fn cycle_at_x_maps_plot_edges() {
        let plot = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(0.0, 0.0),
            eframe::egui::vec2(100.0, 40.0),
        );
        assert_eq!(cycle_at_x(plot, (10, 110), 0.0), 10);
        assert_eq!(cycle_at_x(plot, (10, 110), 100.0), 110);
        assert_eq!(cycle_at_x(plot, (10, 110), 50.0), 60);
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
