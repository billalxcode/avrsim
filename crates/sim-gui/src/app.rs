//! GUI application shell and panel layout.

use crate::canvas::{self, GridCamera};
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

pub struct GuiApp {
    camera: GridCamera,
    grid_visible: bool,
    tool: Tool,
    pointer_world: Option<egui::Pos2>,
    initialized: bool,
}

impl Default for GuiApp {
    fn default() -> Self {
        Self {
            camera: GridCamera::default(),
            grid_visible: true,
            tool: Tool::Select,
            pointer_world: None,
            initialized: false,
        }
    }
}

impl eframe::App for GuiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.toolbar(ui);
        self.status_bar(ui);
        self.pointer_world = self.canvas(ui);
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

impl GuiApp {
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong("embed-emu");
                ui.separator();
                for tool in Tool::ALL {
                    ui.selectable_value(&mut self.tool, tool, tool.label());
                }
                ui.separator();
                if ui.button("100%").clicked() {
                    self.camera.zoom = 1.0;
                }
                ui.checkbox(&mut self.grid_visible, "Grid");
                ui.separator();
                ui.label("Run: idle");
            });
        });
    }

    fn canvas(&mut self, ui: &mut egui::Ui) -> Option<egui::Pos2> {
        let camera = &mut self.camera;
        let grid_visible = self.grid_visible;
        let initialized = &mut self.initialized;
        egui::CentralPanel::default()
            .show(ui, |ui| {
                canvas::show(ui, camera, grid_visible, initialized).pointer_world
            })
            .inner
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
            ui.label("Sel 0");
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
