//! GUI application shell, library sheet, inspector, and canvas integration.

use crate::canvas::{self, GridCamera};
use crate::clipboard::{self, Clipboard};
use crate::config;
use crate::error::{GuiError, GuiResult};
use crate::export;
use crate::history::History;
use crate::layout::{self, Align, Distribute};
use crate::library;
use crate::log::Log;
use crate::model::{
    Category, ComponentDefinition, ComponentId, ComponentInstance, Document, GridPos, GridRect,
    Mirror, NetClass, Rotation, WireId,
};
use crate::palette::{self, Command};
use crate::persist;
use crate::probe::{self, PlotView, ProbeSet, ProbeSource, ProbeSpec, SampleProvider, Sampling};
use crate::provider::{BuiltinComponents, ComponentProvider};
use crate::scene::{self, Scene};
use crate::selection::Selection;
use crate::sim::{self, Event, Sample, SimHandle, Status, Waveform};
use crate::uart::UartLog;
use eframe::egui;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

const AUTOSAVE_DELAY: Duration = Duration::from_secs(2);
const WATCH_INTERVAL: Duration = Duration::from_secs(1);
const TOAST_DURATION: Duration = Duration::from_secs(3);
const PB5_COLOR: egui::Color32 = egui::Color32::from_rgb(0, 190, 210);
const LED_COLOR: egui::Color32 = egui::Color32::from_rgb(85, 205, 125);
const CONTEXT_COLORS: [(&str, [u8; 3]); 6] = [
    ("Red", [225, 85, 85]),
    ("Green", [85, 205, 125]),
    ("Blue", [90, 155, 240]),
    ("Cyan", [80, 215, 225]),
    ("Yellow", [230, 210, 95]),
    ("White", [230, 235, 245]),
];

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

    const fn icon(self) -> &'static str {
        match self {
            Self::Select => "*",
            Self::Place => "+",
            Self::Wire => "~",
            Self::Probe => "?",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum InspectorAction {
    Rotate,
    Mirror,
    Delete,
    SetClass(NetClass),
    SetColor(Option<[u8; 3]>),
    SetLabel(Option<String>),
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

#[derive(Clone, Debug, PartialEq, Eq)]
enum ContextAction {
    Copy,
    Paste,
    Duplicate,
    Delete,
    Rotate,
    Mirror,
    Properties,
    Graph,
    ProbePb5,
    SetClass(NetClass),
    SetColor(Option<[u8; 3]>),
    SetLabel(Option<String>),
    SelectTool,
    WireTool,
}

struct ContextNetState {
    class: Option<NetClass>,
    has_override: bool,
    label: String,
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
struct SceneRefs<'a> {
    document: &'a Document,
    library: &'a [ComponentDefinition],
    selection: &'a Selection,
    wire_preview: &'a [GridPos],
}

#[derive(Clone, Copy)]
struct SceneState {
    hover_wire: Option<WireId>,
    placing: Option<&'static ComponentDefinition>,
    rotation: Rotation,
    mirror: Mirror,
    wire_class: NetClass,
    selection_box: Option<(GridPos, GridPos)>,
    minimap: bool,
    flow_phase: Option<f32>,
}

#[derive(Default)]
pub struct GuiApp {
    camera: GridCamera,
    canvas_rect: Option<egui::Rect>,
    grid_visible: bool,
    minimap_open: bool,
    flow_animation: bool,
    light_theme: bool,
    tool: Tool,
    pointer_world: Option<egui::Pos2>,
    initialized: bool,
    canvas_focused: bool,
    sample_every: u64,
    library: &'static [ComponentDefinition],
    recent_components: Vec<&'static ComponentDefinition>,
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
    drag_place: bool,
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
    log: Log,
    log_open: bool,
    history: History,
    drag_history_pending: bool,
    sim: Option<SimHandle>,
    sim_status: Status,
    waveform: Waveform,
    waveform_open: bool,
    analog_open: bool,
    probes: ProbeSet,
    plot_view: PlotView,
    uart: UartLog,
    uart_input: String,
    uart_open: bool,
    config: config::AppConfig,
    config_path: Option<PathBuf>,
    png_result: Arc<Mutex<Option<Result<(), String>>>>,
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
        let mut app = Self {
            library: BuiltinComponents.definitions(),
            document: Document::new(),
            grid_visible: true,
            inspector_open: true,
            sim: Some(SimHandle::spawn()),
            config_path,
            config,
            ..Self::default()
        };
        app.add_pb5_probe();
        app
    }
}

impl eframe::App for GuiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.palette_shortcut(ui);
        self.poll_sim();
        self.watch_file(ui.ctx());
        self.autosave(ui.ctx());
        self.poll_png();
        self.update_toast();
        self.toolbar(ui);
        self.status_bar(ui);
        let state = self.canvas(ui);
        self.pointer_world = state.pointer_world;
        self.canvas_rect = Some(state.rect);
        self.handle_canvas(state);
        self.finish_drag_place(ui.ctx());
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
        self.analog_window(ui);
        self.uart_window(ui);
        self.log_window(ui);
        self.show_context_menu(ui);
        self.command_palette(ui);
        self.delete_modal(ui);
        self.show_toast(ui);
        self.conflict_modal(ui);
    }
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
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
            .add_enabled(self.history.can_undo(), icon_button("<-", "Undo"))
            .clicked();
        let redo = ui
            .add_enabled(self.history.can_redo(), icon_button("->", "Redo"))
            .clicked();
        if undo {
            self.undo();
        }
        if redo {
            self.redo();
        }
    }

    fn file_buttons(&mut self, ui: &mut egui::Ui) {
        if ui.add(icon_button("[/]", "Open")).clicked() {
            self.open_project();
        }
        if ui.add(icon_button("[S]", "Save")).clicked() {
            self.save_project();
        }
        if ui.add(icon_button("[P]", "PNG")).clicked() {
            self.export_png(ui.ctx());
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
        if ui.add(icon_button("[H]", "Load HEX")).clicked() {
            self.open_hex();
        }
        self.run_buttons(ui);
        ui.checkbox(&mut self.waveform_open, "Wave");
        ui.checkbox(&mut self.analog_open, "Analog");
        ui.checkbox(&mut self.uart_open, "UART");
        self.sample_control(ui);
    }

    fn run_buttons(&mut self, ui: &mut egui::Ui) {
        let loaded = self.sim_status.loaded;
        let running = self.sim_status.running;
        let run = ui.add_enabled(loaded, run_button_label(running)).clicked();
        let step = ui
            .add_enabled(loaded && !running, icon_button(">|", "Step"))
            .clicked();
        let reset = ui.add_enabled(loaded, icon_button("[]", "Reset")).clicked();
        self.apply_run_actions(run, step, reset);
    }

    fn apply_run_actions(&mut self, run: bool, step: bool, reset: bool) {
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

    fn sample_control(&mut self, ui: &mut egui::Ui) {
        let current = self.sample_rate();
        let every = sample_combo(ui, current);
        if every == current {
            return;
        }
        let Ok(every) = crate::error::validate_sample_every(every) else {
            return;
        };
        self.sample_every = every;
        if let Some(handle) = &self.sim {
            let _ = handle.set_sample_every(every);
        }
    }

    fn sample_rate(&self) -> u64 {
        if self.sample_every == 0 {
            sim::DEFAULT_SAMPLE_EVERY
        } else {
            self.sample_every
        }
    }

    fn add_pb5_probe(&mut self) -> bool {
        let source = ProbeSource::Pin {
            component: ComponentId(0),
            pin: "PB5",
        };
        if self.probes.contains(source) {
            return false;
        }
        self.probes.add(ProbeSpec {
            label: "PB5 voltage".to_string(),
            unit: "V",
            color: PB5_COLOR,
            source,
            sampling: Sampling::default(),
        })
    }

    fn open_pb5_probe(&mut self) {
        self.add_pb5_probe();
        self.analog_open = true;
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

    fn export_png(&mut self, ctx: &egui::Context) {
        let Some(canvas) = self.canvas_rect else {
            self.message = "Canvas is not ready".to_string();
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG image", &["png"])
            .set_file_name("schematic.png")
            .save_file()
        else {
            self.message = "PNG export cancelled".to_string();
            return;
        };
        let screen = ctx.content_rect();
        let slot = self.png_result.clone();
        let callback = egui::ScreenshotCallback::new(move |image| {
            let result = export::save_crop(&path, &image, canvas, screen);
            if let Ok(mut slot) = slot.lock() {
                *slot = Some(result);
            }
        });
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(callback));
        self.message = "Exporting PNG".to_string();
    }

    fn poll_png(&mut self) {
        let Ok(mut slot) = self.png_result.lock() else {
            return;
        };
        let Some(result) = slot.take() else {
            return;
        };
        self.message = match result {
            Ok(()) => "PNG saved".to_string(),
            Err(error) => GuiError::Export(error).to_string(),
        };
    }

    fn update_toast(&mut self) {
        if self.message.is_empty() || self.message == self.last_toast {
            return;
        }
        self.last_toast = self.message.clone();
        self.log
            .push(format!("[{}] {}", self.sim_status.cycles, self.message));
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
        match sim::load_hex(&path).map_err(GuiError::Firmware) {
            Ok(words) => self.start_firmware(words, name),
            Err(error) => self.message = error.to_string(),
        }
    }

    fn start_firmware(&mut self, words: Vec<u16>, name: String) {
        match self.load_sim(words) {
            Ok(()) => {
                self.waveform.clear();
                self.probes.clear_samples();
                self.uart.clear();
                self.uart_input.clear();
                self.waveform_open = true;
                self.message = format!("Firmware {name}");
            }
            Err(error) => self.message = error.to_string(),
        }
    }

    fn load_sim(&self, words: Vec<u16>) -> GuiResult<()> {
        let Some(sim) = self.sim.as_ref() else {
            return Err(GuiError::Simulation("simulation unavailable".to_string()));
        };
        sim.load(words).map_err(GuiError::Simulation)
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
            self.probes.clear_samples();
            self.uart.clear();
            self.uart_input.clear();
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
            Event::Sample(sample) => self.push_sample(sample),
            Event::Uart(data) => self.uart.push(&data),
            Event::UartCleared => self.uart.clear(),
            Event::Error(error) => {
                self.sim_status.running = false;
                self.message = error;
            }
        }
    }

    fn push_sample(&mut self, sample: Sample) {
        self.waveform.push(sample);
        let provider = SampleProvider { sample };
        self.probes.sample_from(&provider, sample.cycle);
    }

    fn tool_buttons(&mut self, ui: &mut egui::Ui) {
        let previous = self.tool;
        for tool in Tool::ALL {
            let text = format!("{} {}", tool.icon(), tool.label());
            ui.selectable_value(&mut self.tool, tool, text);
        }
        if previous != self.tool {
            self.wire_points.clear();
        }
    }

    fn sheet_buttons(&mut self, ui: &mut egui::Ui) {
        if ui
            .selectable_label(self.library_open, "[L] Library")
            .clicked()
        {
            self.library_open = true;
        }
        if ui
            .selectable_label(self.inspector_open, "[I] Inspector")
            .clicked()
        {
            self.inspector_open = true;
        }
        if ui
            .selectable_label(self.legend_open, "[K] Legend")
            .clicked()
        {
            self.legend_open = true;
        }
    }

    fn toolbar_view(&mut self, ui: &mut egui::Ui) {
        self.view_options(ui);
        ui.separator();
        if self.tool == Tool::Wire {
            self.wire_controls(ui);
        } else {
            self.placement_controls(ui);
        }
    }

    fn view_options(&mut self, ui: &mut egui::Ui) {
        self.zoom_controls(ui);
        ui.checkbox(&mut self.grid_visible, "Grid");
        ui.checkbox(&mut self.minimap_open, "Mini");
        ui.checkbox(&mut self.flow_animation, "Flow");
        if self.flow_animation {
            ui.ctx().request_repaint();
        }
        if ui.checkbox(&mut self.light_theme, "Light").changed() {
            ui.ctx().set_theme(if self.light_theme {
                egui::ThemePreference::Light
            } else {
                egui::ThemePreference::Dark
            });
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
        ui.label("Class");
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
        let state = self.scene_state(ui.ctx().input(|input| input.time));
        let refs = SceneRefs {
            document: &self.document,
            library: self.library,
            selection: &self.selection,
            wire_preview: &self.wire_points,
        };
        let scene = build_scene(refs, state);
        let camera = &mut self.camera;
        let grid_visible = self.grid_visible;
        let initialized = &mut self.initialized;
        show_canvas(ui, camera, grid_visible, initialized, scene)
    }

    fn scene_state(&self, time: f64) -> SceneState {
        SceneState {
            hover_wire: self
                .pointer_world
                .and_then(|point| self.document.wire_at([point.x, point.y], 0.35)),
            placing: self.placing,
            rotation: self.placement_rotation,
            mirror: self.placement_mirror,
            wire_class: self.wire_class,
            selection_box: self.selection_box(),
            minimap: self.minimap_open,
            flow_phase: flow_phase(self.flow_animation, time),
        }
    }

    fn handle_canvas(&mut self, state: canvas::CanvasState) {
        self.canvas_focused = state.focused;
        if self.placing.is_some() {
            self.handle_placement(state);
            return;
        }
        if self.tool == Tool::Wire {
            self.handle_wire(state);
            return;
        }
        self.handle_select_canvas(&state);
    }

    fn handle_select_canvas(&mut self, state: &canvas::CanvasState) {
        self.begin_drag(state);
        self.drag_move(state);
        self.end_drag(state);
        if state.secondary_clicked {
            self.open_context_menu(state);
            return;
        }
        let Some(world) = state.click_world.filter(|_| state.clicked) else {
            return;
        };
        self.select_at(world, state.multi_select);
    }

    fn select_at(&mut self, world: egui::Pos2, additive: bool) {
        let point = world_to_grid(world);
        if let Some(id) = self.document.pin_wire_at(self.library, point) {
            self.select_wire_net(id, additive);
            return;
        }
        if let Some(id) = self.document.hit_test(self.library, point) {
            if additive {
                self.selection.toggle_component(id);
            } else {
                self.selection.select_component(id);
            }
            return;
        }
        if let Some(id) = self.document.wire_at([world.x, world.y], 0.35) {
            self.select_wire_net(id, additive);
            return;
        }
        if !additive {
            self.selection.clear();
        }
    }

    fn select_wire_net(&mut self, id: WireId, additive: bool) {
        if additive {
            self.selection.toggle_wire(id);
            return;
        }
        let net = self.document.net_wires(id);
        self.selection.apply([], net, false);
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
        if let Some(id) = self.document.pin_wire_at(self.library, point) {
            if !self.selection.contains_wire(id) {
                self.select_wire_net(id, false);
            }
            return;
        }
        if let Some(id) = self.document.hit_test(self.library, point) {
            if !self.selection.contains_component(id) {
                self.selection.select_component(id);
            }
            return;
        }
        if let Some(id) = self.document.wire_at([world.x, world.y], 0.35) {
            if !self.selection.contains_wire(id) {
                self.select_wire_net(id, false);
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
            self.drag_place = false;
            return;
        }
        if !state.clicked {
            return;
        }
        let Some(world) = state.click_world else {
            return;
        };
        self.place_at(world);
    }

    fn place_at(&mut self, world: egui::Pos2) {
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
        self.drag_place = false;
        self.mark_dirty();
        self.remember_component(definition);
    }

    fn finish_drag_place(&mut self, ctx: &egui::Context) {
        if !self.drag_place || !ctx.input(|input| input.pointer.any_released()) {
            return;
        }
        self.drag_place = false;
        let Some(world) = self.pointer_world else {
            return;
        };
        self.place_at(world);
    }

    fn remember_component(&mut self, definition: &'static ComponentDefinition) {
        self.recent_components
            .retain(|item| item.id != definition.id);
        self.recent_components.insert(0, definition);
        self.recent_components.truncate(5);
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
        let point = self.document.snap_pin(self.library, point).unwrap_or(point);
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
        if self.document.snap_pin(self.library, point).is_some() {
            self.finish_wire();
        }
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
        self.handle_nudge_keys(ui);
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
            self.drag_place = false;
            self.wire_points.clear();
        }
    }

    fn handle_nudge_keys(&mut self, ui: &egui::Ui) {
        if !self.canvas_focused {
            return;
        }
        let (dx, dy) = nudge_delta(ui);
        if dx == 0 && dy == 0 {
            return;
        }
        self.nudge_selection(dx, dy);
    }

    fn nudge_selection(&mut self, dx: i32, dy: i32) {
        let ids: Vec<_> = self.selection.components().iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        let moves = self.nudge_moves(&ids, dx, dy);
        let mut candidate = self.document.clone();
        if !candidate.move_components(self.library, &moves) {
            return;
        }
        self.push_undo();
        self.document = candidate;
        self.mark_dirty();
        self.message = "Nudge".to_string();
    }

    fn nudge_moves(&self, ids: &[ComponentId], dx: i32, dy: i32) -> Vec<(ComponentId, GridPos)> {
        let delta = GridPos::new(dx, dy);
        ids.iter()
            .filter_map(|id| {
                self.document
                    .component(*id)
                    .map(|component| (*id, component.position.offset(delta)))
            })
            .collect()
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
            self.context_net_state(),
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
            ContextAction::Graph => self.waveform_open = true,
            ContextAction::ProbePb5 => self.open_pb5_probe(),
            ContextAction::SetClass(class) => self.set_wire_class(class),
            ContextAction::SetColor(color) => self.set_wire_color_override(color),
            ContextAction::SetLabel(label) => self.set_wire_label(label),
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
            Command::ToggleMinimap => self.minimap_open = !self.minimap_open,
            Command::ToggleLibrary => self.library_open = !self.library_open,
            Command::ToggleInspector => self.inspector_open = !self.inspector_open,
            Command::ToggleLegend => self.legend_open = !self.legend_open,
            Command::ToggleWaveform => self.waveform_open = !self.waveform_open,
            Command::ToggleUart => self.uart_open = !self.uart_open,
            Command::ToggleLog => self.log_open = !self.log_open,
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
        let mut hovered = None;
        let mut dragged = None;
        egui::Window::new("Library")
            .open(&mut open)
            .default_width(300.0)
            .show(ui.ctx(), |ui| {
                if let Some(definition) = self.recent_buttons(ui) {
                    chosen = Some(definition);
                }
                self.library_filters(ui, &mut search, &mut category);
                ui.separator();
                if chosen.is_none() {
                    chosen = library_list(
                        ui,
                        self.library,
                        &search,
                        category,
                        &mut hovered,
                        &mut dragged,
                    );
                }
                if let Some(definition) = hovered {
                    library_preview(ui, definition);
                }
            });
        self.library_open = open;
        self.search = search;
        self.category = category;
        if let Some(definition) = dragged {
            self.placing = Some(definition);
            self.drag_place = true;
            self.library_open = false;
            return;
        }
        if let Some(definition) = chosen {
            self.placing = Some(definition);
            self.library_open = false;
        }
    }

    fn recent_buttons(&self, ui: &mut egui::Ui) -> Option<&'static ComponentDefinition> {
        if self.recent_components.is_empty() {
            return None;
        }
        ui.label("Recent");
        let mut chosen = None;
        ui.horizontal_wrapped(|ui| {
            for definition in &self.recent_components {
                if ui.button(definition.name).clicked() {
                    chosen = Some(*definition);
                }
            }
        });
        ui.separator();
        chosen
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

    fn log_window(&mut self, ui: &mut egui::Ui) {
        if !self.log_open {
            return;
        }
        let mut open = self.log_open;
        let mut clear = false;
        egui::Window::new("Log")
            .open(&mut open)
            .default_size([460.0, 220.0])
            .show(ui.ctx(), |ui| {
                if ui.button("Clear").clicked() {
                    clear = true;
                }
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for entry in self.log.entries() {
                            ui.monospace(entry);
                        }
                    });
            });
        self.log_open = open;
        if clear {
            self.log.clear();
        }
    }

    fn uart_window(&mut self, ui: &mut egui::Ui) {
        if !self.uart_open {
            return;
        }
        let mut open = self.uart_open;
        let mut clear = false;
        let mut send = false;
        let mut input = self.uart_input.clone();
        egui::Window::new("UART monitor")
            .open(&mut open)
            .default_size([420.0, 260.0])
            .show(ui.ctx(), |ui| {
                show_uart_contents(ui, &self.uart, &mut input, &mut clear, &mut send);
            });
        self.uart_open = open;
        self.uart_input = input;
        if clear {
            self.uart.clear();
        }
        if send {
            self.send_uart_input();
        }
    }

    fn send_uart_input(&mut self) {
        if self.uart_input.is_empty() {
            return;
        }
        let data = self.uart_input.as_bytes().to_vec();
        let Some(sim) = self.sim.as_ref() else {
            self.message = "Simulation unavailable".to_string();
            return;
        };
        match sim.send_uart(data) {
            Ok(()) => {
                self.uart_input.clear();
                self.message = "UART RX sent".to_string();
            }
            Err(error) => self.message = error,
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

    fn analog_window(&mut self, ui: &mut egui::Ui) {
        if !self.analog_open {
            return;
        }
        let mut open = self.analog_open;
        let status = self.sim_status;
        let probes = &self.probes;
        let view = &mut self.plot_view;
        egui::Window::new("Analog graph")
            .open(&mut open)
            .default_size([560.0, 300.0])
            .show(ui.ctx(), |ui| {
                show_analog_graph(ui, probes, view, status);
            });
        self.analog_open = open;
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
        let components = self.selection.components().len();
        let wires = self.selection.wires().len();
        ui.label(format!("{components} components, {wires} wires"));
        if components > 0 {
            show_component_actions(ui, action);
        }
        if wires > 0 {
            self.multi_wire_controls(ui, action);
        }
    }

    fn multi_wire_controls(&self, ui: &mut egui::Ui, action: &mut Option<InspectorAction>) {
        show_wire_class_combo(ui, "multi_wire_class", self.selection_wire_class(), action);
        if let Some(rgb) = self.selection_wire_rgb() {
            show_wire_color_button(ui, rgb, self.selection_has_color_override(), action);
        }
        if self.selection.components().is_empty() && ui.button("Delete").clicked() {
            *action = Some(InspectorAction::Delete);
        }
    }

    fn selection_wire_class(&self) -> Option<NetClass> {
        let mut classes = self.selection.wires().iter().filter_map(|id| {
            self.document
                .wires()
                .iter()
                .find(|wire| wire.id == *id)
                .map(|wire| wire.class)
        });
        let first = classes.next()?;
        classes.all(|class| class == first).then_some(first)
    }

    fn selection_wire_rgb(&self) -> Option<[u8; 3]> {
        self.selection.wires().iter().find_map(|id| {
            self.document
                .wires()
                .iter()
                .find(|wire| wire.id == *id)
                .map(wire_rgb)
        })
    }

    fn selection_wire_label(&self) -> String {
        self.selection
            .wires()
            .iter()
            .find_map(|id| {
                self.document
                    .wires()
                    .iter()
                    .find(|wire| wire.id == *id)
                    .and_then(|wire| wire.label.clone())
            })
            .unwrap_or_default()
    }

    fn context_net_state(&self) -> ContextNetState {
        ContextNetState {
            class: self.selection_wire_class(),
            has_override: self.selection_has_color_override(),
            label: self.selection_wire_label(),
        }
    }

    fn selection_has_color_override(&self) -> bool {
        self.selection.wires().iter().any(|id| {
            self.document
                .wires()
                .iter()
                .find(|wire| wire.id == *id)
                .is_some_and(|wire| wire.color_override.is_some())
        })
    }

    fn wire_inspector(&self, ui: &mut egui::Ui, action: &mut Option<InspectorAction>, id: WireId) {
        let Some(wire) = self.document.wires().iter().find(|wire| wire.id == id) else {
            ui.label("Wire missing");
            return;
        };
        ui.label(format!("Wire {}", wire.id.0));
        show_wire_class_combo(ui, "inspector_wire_class", Some(wire.class), action);
        ui.label(format!("Vertices: {}", wire.points.len()));
        show_wire_color_button(ui, wire_rgb(wire), wire.color_override.is_some(), action);
        show_wire_label_editor(ui, wire.label.as_deref(), action);
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
        self.drag_place = false;
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
            InspectorAction::SetClass(class) => self.set_wire_class(class),
            InspectorAction::SetColor(color) => self.set_wire_color_override(color),
            InspectorAction::SetLabel(label) => self.set_wire_label(label),
        }
    }

    fn set_wire_label(&mut self, label: Option<String>) {
        let ids: Vec<_> = self.selection.wires().iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        let before = self.document.clone();
        let mut changed = false;
        for id in ids {
            changed |= self.document.set_wire_label(id, label.clone());
        }
        if !changed {
            return;
        }
        self.history.record(&before);
        self.mark_dirty();
        self.message = match label {
            Some(_) => "Wire label updated".to_string(),
            None => "Wire label cleared".to_string(),
        };
    }

    fn set_wire_color_override(&mut self, color: Option<[u8; 3]>) {
        let ids: Vec<_> = self.selection.wires().iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        let before = self.document.clone();
        let mut changed = false;
        for id in ids {
            changed |= self.document.set_wire_color(id, color);
        }
        if !changed {
            return;
        }
        self.history.record(&before);
        self.mark_dirty();
        self.message = if color.is_some() {
            "Wire color updated".to_string()
        } else {
            "Wire color reset".to_string()
        };
    }

    fn set_wire_class(&mut self, class: NetClass) {
        let ids: Vec<_> = self.selection.wires().iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        let before = self.document.clone();
        let mut changed = false;
        for id in ids {
            changed |= self.document.set_wire_class(id, class);
        }
        if !changed {
            return;
        }
        self.history.record(&before);
        self.mark_dirty();
        self.message = format!("Net class {}", class.label());
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
            "PB5 {} {} mV  LED {}",
            high_low(self.sim_status.pb5_high),
            self.sim_status.pb5_millivolts,
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

fn build_scene(refs: SceneRefs<'_>, state: SceneState) -> Scene<'_> {
    Scene {
        document: refs.document,
        library: refs.library,
        selection: refs.selection,
        hover_wire: state.hover_wire,
        placing: state.placing,
        rotation: state.rotation,
        mirror: state.mirror,
        wire_preview: refs.wire_preview,
        wire_class: state.wire_class,
        selection_box: state.selection_box,
        minimap: state.minimap,
        flow_phase: state.flow_phase,
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
    scene::draw(painter, rect, camera, scene, hover);
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
    ui.separator();
    show_component_pins(ui, definition);
    show_component_simulation(ui);
}

fn show_component_simulation(ui: &mut egui::Ui) {
    ui.separator();
    ui.label("Simulation");
    ui.label("Global PB5/LED graph");
}

fn show_component_pins(ui: &mut egui::Ui, definition: &ComponentDefinition) {
    ui.label("Pins");
    for pin in definition.pins {
        ui.label(format!("{} ({})", pin.name, pin.number));
    }
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

fn show_wire_class_combo(
    ui: &mut egui::Ui,
    salt: &str,
    current: Option<NetClass>,
    action: &mut Option<InspectorAction>,
) {
    let mut choice = current;
    egui::ComboBox::from_id_salt(salt)
        .selected_text(current.map_or("Mixed", NetClass::label))
        .show_ui(ui, |ui| {
            for class in NetClass::ALL {
                ui.selectable_value(&mut choice, Some(class), class.label());
            }
        });
    if choice != current {
        if let Some(class) = choice {
            *action = Some(InspectorAction::SetClass(class));
        }
    }
}

fn wire_rgb(wire: &crate::model::Wire) -> [u8; 3] {
    let [red, green, blue, _] = scene::wire_color(wire).to_srgba_unmultiplied();
    [red, green, blue]
}

fn show_wire_color_button(
    ui: &mut egui::Ui,
    current: [u8; 3],
    has_override: bool,
    action: &mut Option<InspectorAction>,
) {
    let mut rgb = current;
    ui.horizontal(|ui| {
        ui.label("Color");
        if ui.color_edit_button_srgb(&mut rgb).changed() {
            *action = Some(InspectorAction::SetColor(Some(rgb)));
        }
        if ui
            .add_enabled(has_override, egui::Button::new("Reset"))
            .clicked()
        {
            *action = Some(InspectorAction::SetColor(None));
        }
    });
}

fn show_wire_label_editor(
    ui: &mut egui::Ui,
    current: Option<&str>,
    action: &mut Option<InspectorAction>,
) {
    let mut label = current.unwrap_or("").to_string();
    ui.horizontal(|ui| {
        ui.label("Label");
        ui.text_edit_singleline(&mut label);
        if ui.button("Set").clicked() {
            *action = Some(InspectorAction::SetLabel(normalize_label(label)));
        }
    });
}

fn normalize_label(label: String) -> Option<String> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

fn library_list<'a>(
    ui: &mut egui::Ui,
    library: &'a [ComponentDefinition],
    search: &str,
    category: Option<Category>,
    hovered: &mut Option<&'a ComponentDefinition>,
    dragged: &mut Option<&'a ComponentDefinition>,
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
        let response = ui
            .selectable_label(false, text)
            .interact(egui::Sense::click_and_drag());
        if response.hovered() {
            *hovered = Some(definition);
        }
        if response.drag_started() {
            *dragged = Some(definition);
        }
        if response.clicked() {
            return Some(definition);
        }
    }
    None
}

fn library_preview(ui: &mut egui::Ui, definition: &ComponentDefinition) {
    ui.separator();
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(180.0), 96.0),
        egui::Sense::hover(),
    );
    ui.painter()
        .rect_filled(rect, 4.0, egui::Color32::from_rgb(22, 26, 32));
    scene::draw_library_preview(ui.painter(), rect.shrink(8.0), definition);
    ui.label(format!(
        "{}  {} pins",
        definition.default_value,
        definition.pins.len()
    ));
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
    net: ContextNetState,
) -> (Option<ContextAction>, egui::Response) {
    let mut action = None;
    let area = egui::Area::new(egui::Id::unique("canvas-context"))
        .fixed_pos(position)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(190.0);
                context_buttons(ui, selection, clipboard_empty, &net, &mut action);
            });
        });
    (action, area.response)
}

fn context_buttons(
    ui: &mut egui::Ui,
    selection: &Selection,
    clipboard_empty: bool,
    net: &ContextNetState,
    action: &mut Option<ContextAction>,
) {
    context_edit_buttons(ui, selection, clipboard_empty, action);
    ui.separator();
    context_transform_buttons(ui, selection, action);
    if !selection.wires().is_empty() {
        ui.separator();
        context_net_controls(ui, net, action);
    }
    ui.separator();
    context_tool_buttons(ui, action);
}

fn context_net_controls(
    ui: &mut egui::Ui,
    net: &ContextNetState,
    action: &mut Option<ContextAction>,
) {
    context_class_combo(ui, net.class, action);
    context_color_presets(ui, net, action);
    context_label_editor(ui, net, action);
}

fn context_color_presets(
    ui: &mut egui::Ui,
    net: &ContextNetState,
    action: &mut Option<ContextAction>,
) {
    ui.horizontal_wrapped(|ui| {
        ui.label("Color");
        for &(name, rgb) in &CONTEXT_COLORS {
            if context_color_button(ui, name, rgb).clicked() {
                *action = Some(ContextAction::SetColor(Some(rgb)));
            }
        }
        if ui
            .add_enabled(net.has_override, egui::Button::new("Reset"))
            .clicked()
        {
            *action = Some(ContextAction::SetColor(None));
        }
    });
}

fn context_color_button(ui: &mut egui::Ui, name: &str, rgb: [u8; 3]) -> egui::Response {
    let button = egui::Button::new(" ")
        .fill(egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]))
        .min_size(egui::vec2(16.0, 16.0));
    ui.add(button).on_hover_text(name)
}

fn context_label_editor(
    ui: &mut egui::Ui,
    net: &ContextNetState,
    action: &mut Option<ContextAction>,
) {
    let mut label = net.label.clone();
    ui.horizontal(|ui| {
        ui.label("Label");
        ui.text_edit_singleline(&mut label);
        if ui.button("Set").clicked() {
            *action = Some(ContextAction::SetLabel(normalize_label(label)));
        }
    });
}

fn context_class_combo(
    ui: &mut egui::Ui,
    current: Option<NetClass>,
    action: &mut Option<ContextAction>,
) {
    let mut choice = current;
    let text = current.map_or_else(
        || "Net class: mixed".to_string(),
        |class| format!("Net class: {}", class.label()),
    );
    egui::ComboBox::from_id_salt("context_net_class")
        .selected_text(text)
        .show_ui(ui, |ui| {
            for class in NetClass::ALL {
                ui.selectable_value(&mut choice, Some(class), class.label());
            }
        });
    if choice != current {
        if let Some(class) = choice {
            *action = Some(ContextAction::SetClass(class));
        }
    }
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
    if ui.button("Digital graph").clicked() {
        *action = Some(ContextAction::Graph);
    }
    if ui.button("Probe PB5").clicked() {
        *action = Some(ContextAction::ProbePb5);
    }
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

fn show_uart_contents(
    ui: &mut egui::Ui,
    log: &UartLog,
    input: &mut String,
    clear: &mut bool,
    send: &mut bool,
) {
    ui.horizontal(|ui| {
        if ui.button("Clear").clicked() {
            *clear = true;
        }
        ui.label(format!("{} bytes", log.len()));
    });
    ui.horizontal(|ui| {
        let response = ui.text_edit_singleline(input);
        let enter = response.lost_focus() && ui.input(|state| state.key_pressed(egui::Key::Enter));
        if ui.button("Send").clicked() || enter {
            *send = true;
        }
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

fn show_analog_graph(ui: &mut egui::Ui, probes: &ProbeSet, view: &mut PlotView, status: Status) {
    let window = probe::window(probes.span(), *view);
    let range = analog_range(probes, view, window);
    ui.label(analog_header(probes, status, window));
    plot_controls(ui, view);
    let width = ui.available_width().max(320.0);
    let (response, painter) =
        ui.allocate_painter(egui::vec2(width, 190.0), egui::Sense::click_and_drag());
    let plot = response.rect.shrink2(egui::vec2(42.0, 14.0));
    handle_plot_input(&response, view, plot, window);
    paint_analog_plot(&painter, response.rect, plot, probes, window, range, *view);
}

fn analog_range(probes: &ProbeSet, view: &PlotView, window: (u64, u64)) -> (f64, f64) {
    if !view.autoscale {
        return (0.0, 5.0);
    }
    probe::value_range(probes.probes(), window)
}

fn analog_header(probes: &ProbeSet, status: Status, window: (u64, u64)) -> String {
    let samples = probe_sample_count(probes);
    let last = probe_last_text(probes);
    format!(
        "{} series  {} samples  last {}  cycle {}  window {}..{}",
        probes.probes().len(),
        samples,
        last,
        status.cycles,
        window.0,
        window.1
    )
}

fn probe_sample_count(probes: &ProbeSet) -> usize {
    probes
        .probes()
        .iter()
        .map(|probe| probe.samples().len())
        .sum()
}

fn probe_last_text(probes: &ProbeSet) -> String {
    let Some(probe) = probes.probes().first() else {
        return "n/a".to_string();
    };
    probe.latest().map_or_else(
        || "n/a".to_string(),
        |point| format!("{:.2}{}", point.value, probe.spec().unit),
    )
}

fn plot_controls(ui: &mut egui::Ui, view: &mut PlotView) {
    ui.horizontal(|ui| {
        ui.checkbox(&mut view.autoscale, "Auto");
        ui.checkbox(&mut view.legend, "Legend");
        if ui.button("-").clicked() {
            view.zoom = (view.zoom * 0.8).clamp(0.25, 8.0);
        }
        if ui.button("+").clicked() {
            view.zoom = (view.zoom * 1.25).clamp(0.25, 8.0);
        }
        if ui.button("Reset").clicked() {
            *view = PlotView::default();
        }
        ui.label(format!("zoom {:.1}x", view.zoom));
    });
}

fn handle_plot_input(
    response: &egui::Response,
    view: &mut PlotView,
    plot: egui::Rect,
    window: (u64, u64),
) {
    if response.dragged() {
        let fraction = f64::from(response.drag_delta().x / plot.width().max(1.0));
        view.pan = (view.pan - fraction).clamp(-0.95, 0.95);
    }
    let scroll = response.ctx.input(|input| input.smooth_scroll_delta.y);
    if response.hovered() && scroll != 0.0 {
        view.zoom = (view.zoom * (1.0 + scroll * 0.001)).clamp(0.25, 8.0);
    }
    view.cursor = response
        .hover_pos()
        .filter(|pos| plot.contains(*pos))
        .map(|pos| cycle_at(plot, window, pos.x));
}

fn paint_analog_plot(
    painter: &egui::Painter,
    rect: egui::Rect,
    plot: egui::Rect,
    probes: &ProbeSet,
    window: (u64, u64),
    range: (f64, f64),
    view: PlotView,
) {
    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(18, 22, 26));
    paint_value_grid(painter, plot, range);
    for probe in probes.probes() {
        paint_probe_series(painter, plot, probe, window, range);
    }
    if view.legend {
        paint_analog_legend(painter, plot, probes);
    }
    paint_analog_cursor(painter, plot, probes, window, view.cursor);
}

fn paint_value_grid(painter: &egui::Painter, plot: egui::Rect, range: (f64, f64)) {
    let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 46, 56));
    for step in 0..=4 {
        let value = range.0 + (range.1 - range.0) * f64::from(step) / 4.0;
        let y = value_to_y(plot, range, value);
        painter.line_segment(
            [egui::pos2(plot.left(), y), egui::pos2(plot.right(), y)],
            stroke,
        );
        painter.text(
            egui::pos2(plot.left() - 6.0, y),
            egui::Align2::RIGHT_CENTER,
            format!("{value:.1}"),
            egui::FontId::monospace(9.0),
            egui::Color32::from_rgb(150, 160, 175),
        );
    }
}

fn paint_probe_series(
    painter: &egui::Painter,
    plot: egui::Rect,
    probe: &probe::ProbeBuffer,
    window: (u64, u64),
    range: (f64, f64),
) {
    let screen = probe_screen_points(plot, probe, window, range);
    if screen.len() < 2 {
        return;
    }
    painter.add(egui::Shape::line(
        screen,
        egui::Stroke::new(1.5, probe.spec().color),
    ));
}

fn probe_screen_points(
    plot: egui::Rect,
    probe: &probe::ProbeBuffer,
    window: (u64, u64),
    range: (f64, f64),
) -> Vec<egui::Pos2> {
    let limit = (plot.width() as usize).max(32);
    let points = probe::decimate(probe.samples(), limit);
    points
        .iter()
        .filter(|point| point.cycle >= window.0 && point.cycle <= window.1)
        .map(|point| {
            egui::pos2(
                cycle_to_x(plot, window, point.cycle),
                value_to_y(plot, range, point.value),
            )
        })
        .collect()
}

fn paint_analog_legend(painter: &egui::Painter, plot: egui::Rect, probes: &ProbeSet) {
    let mut y = plot.top() + 6.0;
    for probe in probes.probes().iter().take(probe::MAX_SERIES) {
        let color = probe.spec().color;
        let swatch =
            egui::Rect::from_min_size(egui::pos2(plot.right() - 150.0, y), egui::vec2(8.0, 8.0));
        painter.rect_filled(swatch, 1.0, color);
        painter.text(
            egui::pos2(plot.right() - 138.0, y),
            egui::Align2::LEFT_TOP,
            format!("{} ({})", probe.spec().label, probe.spec().unit),
            egui::FontId::monospace(9.0),
            egui::Color32::from_rgb(190, 198, 210),
        );
        y += 12.0;
    }
}

fn paint_analog_cursor(
    painter: &egui::Painter,
    plot: egui::Rect,
    probes: &ProbeSet,
    window: (u64, u64),
    cursor: Option<u64>,
) {
    let Some(cycle) = cursor else {
        return;
    };
    let x = cycle_to_x(plot, window, cycle);
    paint_cursor_line(painter, plot, x);
    paint_cursor_label(painter, plot, x, probes, cycle);
}

fn paint_cursor_line(painter: &egui::Painter, plot: egui::Rect, x: f32) {
    painter.line_segment(
        [egui::pos2(x, plot.top()), egui::pos2(x, plot.bottom())],
        egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 215, 225)),
    );
}

fn paint_cursor_label(
    painter: &egui::Painter,
    plot: egui::Rect,
    x: f32,
    probes: &ProbeSet,
    cycle: u64,
) {
    painter.text(
        egui::pos2(x + 4.0, plot.top() + 4.0),
        egui::Align2::LEFT_TOP,
        analog_cursor_text(probes, cycle),
        egui::FontId::monospace(9.0),
        egui::Color32::from_rgb(210, 218, 228),
    );
}

fn analog_cursor_text(probes: &ProbeSet, cycle: u64) -> String {
    let mut text = format!("cycle {cycle}");
    for probe in probes.probes().iter().take(4) {
        let Some(value) = probe_value_at(probe, cycle) else {
            continue;
        };
        text.push_str(&format!(
            "  {} {:.2}{}",
            probe.spec().label,
            value,
            probe.spec().unit
        ));
    }
    text
}

fn probe_value_at(probe: &probe::ProbeBuffer, cycle: u64) -> Option<f64> {
    probe
        .samples()
        .iter()
        .rev()
        .find(|point| point.cycle <= cycle)
        .map(|point| point.value)
}

fn cycle_to_x(plot: egui::Rect, window: (u64, u64), cycle: u64) -> f32 {
    let span = window.1.saturating_sub(window.0).max(1) as f32;
    let time = (cycle.saturating_sub(window.0)) as f32 / span;
    plot.left() + time.clamp(0.0, 1.0) * plot.width()
}

fn cycle_at(plot: egui::Rect, window: (u64, u64), x: f32) -> u64 {
    let time = ((x - plot.left()) / plot.width().max(1.0)).clamp(0.0, 1.0);
    window.0 + (time * (window.1 - window.0) as f32) as u64
}

fn value_to_y(plot: egui::Rect, range: (f64, f64), value: f64) -> f32 {
    let span = (range.1 - range.0).abs().max(f64::EPSILON);
    let level = ((value - range.0) / span) as f32;
    plot.bottom() - level.clamp(0.0, 1.0) * plot.height()
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

fn icon_button(icon: &str, label: &str) -> egui::Button<'static> {
    egui::Button::new(format!("{icon} {label}"))
}

fn sample_combo(ui: &mut egui::Ui, current: u64) -> u64 {
    let mut every = current;
    egui::ComboBox::from_id_salt("sample_every")
        .selected_text(format!("N {every}"))
        .show_ui(ui, |ui| {
            for value in [10, 100, 1000] {
                ui.selectable_value(&mut every, value, format!("N {value}"));
            }
        });
    every
}

fn run_button_label(running: bool) -> egui::Button<'static> {
    if running {
        icon_button("||", "Pause")
    } else {
        icon_button(">", "Run")
    }
}

fn nudge_delta(ui: &egui::Ui) -> (i32, i32) {
    let step = if ui.input(|input| input.modifiers.shift) {
        5
    } else {
        1
    };
    let mut delta = (0, 0);
    ui.input(|input| {
        delta.0 = axis_delta(input, egui::Key::ArrowLeft, egui::Key::ArrowRight, step);
        delta.1 = axis_delta(input, egui::Key::ArrowUp, egui::Key::ArrowDown, step);
    });
    delta
}

fn axis_delta(
    input: &egui::InputState,
    negative: egui::Key,
    positive: egui::Key,
    step: i32,
) -> i32 {
    let mut delta = 0;
    if input.key_pressed(negative) {
        delta -= step;
    }
    if input.key_pressed(positive) {
        delta += step;
    }
    delta
}

fn flow_phase(enabled: bool, time: f64) -> Option<f32> {
    if !enabled {
        return None;
    }
    Some((time * 24.0).rem_euclid(1000.0) as f32)
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
        crate::model::extend_bounds(min, max, bounds.min);
        crate::model::extend_bounds(min, max, GridPos::new(bounds.max_x(), bounds.max_y()));
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
            crate::model::extend_bounds(min, max, *point);
        }
    }
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

    fn wire_app(points: Vec<GridPos>) -> GuiApp {
        let library = crate::library::builtin();
        let resistor = crate::library::find("resistor").expect("resistor");
        let mut document = Document::new();
        document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        GuiApp {
            library,
            document,
            tool: crate::app::Tool::Wire,
            wire_points: points,
            ..GuiApp::default()
        }
    }

    fn click_state(x: f32, y: f32) -> crate::canvas::CanvasState {
        crate::canvas::CanvasState {
            clicked: true,
            click_world: Some(eframe::egui::pos2(x, y)),
            ..Default::default()
        }
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
    fn recent_components_are_bounded_and_deduplicated() {
        let definitions = crate::library::builtin();
        let mut app = GuiApp::default();
        for definition in definitions.iter().take(6) {
            app.remember_component(definition);
        }
        assert_eq!(app.recent_components.len(), 5);
        assert_eq!(app.recent_components[0].id, definitions[5].id);
        app.remember_component(&definitions[0]);
        assert_eq!(app.recent_components[0].id, definitions[0].id);
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
    fn wire_tool_starts_at_pin() {
        let mut app = wire_app(Vec::new());
        app.handle_canvas(click_state(5.0, 1.0));
        assert_eq!(app.wire_points, vec![GridPos::new(4, 1)]);
    }

    #[test]
    fn wire_tool_extends_to_valid_point() {
        let mut app = wire_app(vec![GridPos::new(4, 1)]);
        app.handle_canvas(click_state(8.0, 1.0));
        assert_eq!(
            app.wire_points,
            vec![GridPos::new(4, 1), GridPos::new(8, 1)]
        );
    }

    #[test]
    fn wire_tool_finishes_at_second_pin() {
        let library = crate::library::builtin();
        let resistor = crate::library::find("resistor").expect("resistor");
        let mut document = Document::new();
        document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
        let mut app = GuiApp {
            library,
            document,
            tool: crate::app::Tool::Wire,
            wire_points: vec![GridPos::new(4, 1)],
            ..GuiApp::default()
        };
        app.handle_canvas(click_state(6.0, 1.0));
        assert_eq!(app.document.wires().len(), 1);
        assert!(app.wire_points.is_empty());
    }

    #[test]
    fn wire_class_is_applied_to_new_wire() {
        let library = crate::library::builtin();
        let resistor = crate::library::find("resistor").expect("resistor");
        let mut document = Document::new();
        document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
        let mut app = GuiApp {
            library,
            document,
            tool: crate::app::Tool::Wire,
            wire_class: NetClass::Power,
            wire_points: vec![GridPos::new(4, 1)],
            ..GuiApp::default()
        };
        app.handle_canvas(click_state(6.0, 1.0));
        assert_eq!(app.document.wires()[0].class, NetClass::Power);
    }

    #[test]
    fn selected_wire_class_can_be_changed() {
        let (mut app, _, _) = context_app();
        let wire = sample_wire(&mut app.document, app.library);
        app.selection.apply([], [wire], false);
        app.set_wire_class(NetClass::Analog);
        assert_eq!(app.document.wires()[0].class, NetClass::Analog);
        app.undo();
        assert_eq!(app.document.wires()[0].class, NetClass::Digital);
    }

    #[test]
    fn selected_wire_color_can_be_overridden() {
        let (mut app, _, _) = context_app();
        let wire = sample_wire(&mut app.document, app.library);
        app.selection.apply([], [wire], false);
        app.set_wire_color_override(Some([1, 2, 3]));
        assert_eq!(app.document.wires()[0].color_override, Some([1, 2, 3]));
        app.undo();
        assert_eq!(app.document.wires()[0].color_override, None);
    }

    #[test]
    fn selected_wire_label_can_be_set() {
        let (mut app, _, _) = context_app();
        let wire = sample_wire(&mut app.document, app.library);
        app.selection.apply([], [wire], false);
        app.set_wire_label(Some("LED_A".to_string()));
        assert_eq!(app.document.wires()[0].label.as_deref(), Some("LED_A"));
        app.undo();
        assert_eq!(app.document.wires()[0].label, None);
    }

    #[test]
    fn place_at_commits_definition() {
        let library = crate::library::builtin();
        let resistor = crate::library::find("resistor").expect("resistor");
        let mut app = GuiApp {
            library,
            placing: Some(resistor),
            drag_place: true,
            ..GuiApp::default()
        };
        app.place_at(eframe::egui::pos2(0.0, 0.0));
        assert_eq!(app.document.components().len(), 1);
        assert!(app.placing.is_none());
        assert!(!app.drag_place);
    }

    #[test]
    fn nudge_selection_moves_component() {
        let (mut app, id) = nudged_app();
        app.nudge_selection(1, 0);
        assert_component_position(&app, id, GridPos::new(1, 0));
        app.nudge_selection(0, 2);
        assert_component_position(&app, id, GridPos::new(1, 2));
    }

    #[test]
    fn analog_plot_maps_cycle_and_value() {
        let plot = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(10.0, 20.0),
            eframe::egui::vec2(100.0, 50.0),
        );
        assert!((super::cycle_to_x(plot, (0, 100), 50) - 60.0).abs() < 0.01);
        assert_eq!(super::cycle_at(plot, (0, 100), 60.0), 50);
        assert!((super::value_to_y(plot, (0.0, 10.0), 10.0) - 20.0).abs() < 0.01);
    }

    #[test]
    fn analog_graph_renders_in_egui_context() {
        let (probes, mut view) = analog_fixture();
        let mut output = render_analog(&probes, &mut view);
        output.textures_delta.clear();
        assert!(!output.shapes.is_empty());
    }

    fn analog_fixture() -> (crate::probe::ProbeSet, crate::probe::PlotView) {
        let mut probes = crate::probe::ProbeSet::default();
        assert!(probes.add(analog_spec()));
        probes.sample_from(&analog_provider(), 100);
        (probes, crate::probe::PlotView::default())
    }

    fn analog_spec() -> crate::probe::ProbeSpec {
        crate::probe::ProbeSpec {
            label: "PB5".to_string(),
            unit: "V",
            color: eframe::egui::Color32::WHITE,
            source: crate::probe::ProbeSource::Pin {
                component: ComponentId(0),
                pin: "PB5",
            },
            sampling: crate::probe::Sampling::default(),
        }
    }

    fn analog_provider() -> crate::probe::SampleProvider {
        crate::probe::SampleProvider {
            sample: crate::sim::Sample {
                cycle: 100,
                pb5_high: true,
                pb5_millivolts: 5000,
                led_on: true,
            },
        }
    }

    fn render_analog(
        probes: &crate::probe::ProbeSet,
        view: &mut crate::probe::PlotView,
    ) -> eframe::egui::FullOutput {
        eframe::egui::Context::default().run_ui(eframe::egui::RawInput::default(), |ui| {
            super::show_analog_graph(ui, probes, view, crate::sim::Status::default());
        })
    }

    fn nudged_app() -> (GuiApp, ComponentId) {
        let library = crate::library::builtin();
        let resistor = crate::library::find("resistor").expect("resistor");
        let mut app = GuiApp {
            library,
            ..GuiApp::default()
        };
        let id = app
            .document
            .add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        app.selection.select_component(id);
        (app, id)
    }

    fn assert_component_position(app: &GuiApp, id: ComponentId, expected: GridPos) {
        assert_eq!(
            app.document.component(id).expect("component").position,
            expected
        );
    }

    #[test]
    fn flow_phase_is_optional() {
        assert_eq!(super::flow_phase(false, 1.0), None);
        let phase = super::flow_phase(true, 1.0).expect("phase");
        assert!((0.0..1000.0).contains(&phase));
    }

    #[test]
    fn every_tool_has_an_icon() {
        for tool in super::Tool::ALL {
            assert!(!tool.icon().is_empty());
            assert!(!tool.label().is_empty());
        }
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
