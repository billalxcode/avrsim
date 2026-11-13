//! JSON project persistence for the GUI.

use crate::canvas::GridCamera;
use crate::model::{
    ComponentDefinition, ComponentId, ComponentInstance, Document, GridPos, Mirror, NetClass,
    Rotation, Wire,
};
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const FORMAT: &str = "embed-emu-gui";
const VERSION: u32 = 1;

/// Loaded project state.
pub struct Project {
    pub document: Document,
    pub camera: GridCamera,
    pub grid_visible: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectFile {
    format: String,
    version: u32,
    view: ViewFile,
    components: Vec<ComponentFile>,
    wires: Vec<WireFile>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ViewFile {
    pan: [f32; 2],
    zoom: f32,
    grid_visible: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ComponentFile {
    id: u32,
    kind: String,
    position: [i32; 2],
    rotation: String,
    mirror: String,
    label: String,
    value: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireFile {
    id: u32,
    class: String,
    points: Vec<[i32; 2]>,
}

/// Serializes the project to pretty JSON.
pub fn to_json(
    document: &Document,
    camera: &GridCamera,
    grid_visible: bool,
) -> Result<String, String> {
    let file = project_file(document, camera, grid_visible);
    serde_json::to_string_pretty(&file).map_err(|error| error.to_string())
}

fn project_file(document: &Document, camera: &GridCamera, grid_visible: bool) -> ProjectFile {
    ProjectFile {
        format: FORMAT.to_string(),
        version: VERSION,
        view: ViewFile {
            pan: [camera.pan.x, camera.pan.y],
            zoom: camera.zoom,
            grid_visible,
        },
        components: component_files(document),
        wires: wire_files(document),
    }
}

fn component_files(document: &Document) -> Vec<ComponentFile> {
    document
        .components()
        .iter()
        .map(ComponentFile::from_instance)
        .collect()
}

fn wire_files(document: &Document) -> Vec<WireFile> {
    document.wires().iter().map(WireFile::from_wire).collect()
}

/// Parses a project from JSON.
pub fn from_json(text: &str, library: &[ComponentDefinition]) -> Result<Project, String> {
    let file: ProjectFile =
        serde_json::from_str(text).map_err(|error| format!("invalid project JSON: {error}"))?;
    if file.format != FORMAT {
        return Err(format!("unsupported project format: {}", file.format));
    }
    if file.version != VERSION {
        return Err(format!("unsupported project version: {}", file.version));
    }
    let components = file.components(library)?;
    let mut document = Document::restore(components, Vec::new());
    file.wires(&mut document, library)?;
    Ok(Project {
        document,
        camera: file.view.camera(),
        grid_visible: file.view.grid_visible,
    })
}

/// Saves a project atomically to a JSON file.
pub fn save(
    path: &Path,
    document: &Document,
    camera: &GridCamera,
    grid_visible: bool,
) -> Result<(), String> {
    let text = to_json(document, camera, grid_visible)?;
    backup(path)?;
    write_atomic(path, &text)
}

/// Returns the backup path used before overwriting a project.
#[must_use]
pub fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".bak");
    PathBuf::from(name)
}

fn backup(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    std::fs::copy(path, backup_path(path)).map_err(|error| format!("backup failed: {error}"))?;
    Ok(())
}

/// Writes text through a temporary file and atomic rename.
pub(crate) fn write_atomic(path: &Path, text: &str) -> Result<(), String> {
    create_parent(path)?;
    let temp = path.with_extension("tmp");
    std::fs::write(&temp, text).map_err(|error| format!("write failed: {error}"))?;
    std::fs::rename(&temp, path).map_err(|error| format!("rename failed: {error}"))
}

fn create_parent(path: &Path) -> Result<(), String> {
    let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return Ok(());
    };
    std::fs::create_dir_all(parent).map_err(|error| format!("create dir failed: {error}"))
}

/// Loads a project from a JSON file.
pub fn load(path: &Path, library: &[ComponentDefinition]) -> Result<Project, String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("read failed: {error}"))?;
    from_json(&text, library)
}

impl ProjectFile {
    fn components(
        &self,
        library: &[ComponentDefinition],
    ) -> Result<Vec<ComponentInstance>, String> {
        let mut ids = BTreeSet::new();
        self.components
            .iter()
            .map(|component| {
                if !ids.insert(component.id) {
                    return Err(format!("duplicate component id: {}", component.id));
                }
                component.to_instance(library)
            })
            .collect()
    }

    fn wires(
        &self,
        document: &mut Document,
        library: &[ComponentDefinition],
    ) -> Result<(), String> {
        let mut ids = BTreeSet::new();
        for wire in &self.wires {
            if !ids.insert(wire.id) {
                return Err(format!("duplicate wire id: {}", wire.id));
            }
            wire.add_to(document, library)?;
        }
        Ok(())
    }
}

impl ViewFile {
    fn camera(&self) -> GridCamera {
        GridCamera {
            pan: egui::vec2(self.pan[0], self.pan[1]),
            zoom: self.zoom.clamp(0.25, 4.0),
            cell_px: 20.0,
        }
    }
}

impl ComponentFile {
    fn from_instance(instance: &ComponentInstance) -> Self {
        Self {
            id: instance.id.0,
            kind: instance.definition.to_string(),
            position: [instance.position.x, instance.position.y],
            rotation: instance.rotation.label().to_string(),
            mirror: instance.mirror.label().to_string(),
            label: instance.label.clone(),
            value: instance.value.clone(),
        }
    }

    fn to_instance(&self, library: &[ComponentDefinition]) -> Result<ComponentInstance, String> {
        let definition = library
            .iter()
            .find(|definition| definition.id == self.kind)
            .ok_or_else(|| format!("unknown component: {}", self.kind))?;
        let rotation = parse_rotation(&self.rotation)?;
        let mirror = parse_mirror(&self.mirror)?;
        Ok(ComponentInstance {
            id: ComponentId(self.id),
            definition: definition.id,
            position: GridPos::new(self.position[0], self.position[1]),
            rotation,
            mirror,
            label: self.label.clone(),
            value: self.value.clone(),
        })
    }
}

impl WireFile {
    fn from_wire(wire: &Wire) -> Self {
        Self {
            id: wire.id.0,
            class: wire.class.label().to_string(),
            points: wire.points.iter().map(|point| [point.x, point.y]).collect(),
        }
    }

    fn add_to(
        &self,
        document: &mut Document,
        library: &[ComponentDefinition],
    ) -> Result<(), String> {
        let class = parse_class(&self.class)?;
        let points = self
            .points
            .iter()
            .map(|point| GridPos::new(point[0], point[1]))
            .collect();
        document
            .add_wire(library, points, class)
            .map(|_| ())
            .ok_or_else(|| format!("invalid wire: {}", self.id))
    }
}

fn parse_rotation(value: &str) -> Result<Rotation, String> {
    match value {
        "0" => Ok(Rotation::Deg0),
        "90" => Ok(Rotation::Deg90),
        "180" => Ok(Rotation::Deg180),
        "270" => Ok(Rotation::Deg270),
        _ => Err(format!("invalid rotation: {value}")),
    }
}

fn parse_mirror(value: &str) -> Result<Mirror, String> {
    match value {
        "none" => Ok(Mirror::None),
        "horizontal" => Ok(Mirror::Horizontal),
        "vertical" => Ok(Mirror::Vertical),
        _ => Err(format!("invalid mirror: {value}")),
    }
}

fn parse_class(value: &str) -> Result<NetClass, String> {
    NetClass::ALL
        .into_iter()
        .find(|class| class.label() == value)
        .ok_or_else(|| format!("invalid net class: {value}"))
}

#[cfg(test)]
mod tests {
    use super::{backup_path, from_json, save, to_json};
    use crate::canvas::GridCamera;
    use crate::library;
    use crate::model::{Document, GridPos, Mirror, NetClass, Rotation};
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("embed-emu-gui-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn sample_text() -> String {
        let library = library::builtin();
        let resistor = library::find("resistor").expect("resistor");
        let mut document = crate::model::Document::new();
        document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
        document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
        document
            .add_wire(
                library,
                vec![GridPos::new(4, 1), GridPos::new(6, 1)],
                NetClass::Digital,
            )
            .expect("wire");
        let camera = crate::canvas::GridCamera::default();
        to_json(&document, &camera, true).expect("json")
    }

    #[test]
    fn round_trip_preserves_components_and_wires() {
        let library = library::builtin();
        let project = from_json(&sample_text(), library).expect("load");
        assert_eq!(project.document.components().len(), 2);
        assert_eq!(project.document.wires().len(), 1);
    }

    #[test]
    fn save_creates_backup_of_previous_file() {
        let dir = temp_dir("backup");
        let path = dir.join("project.emu.json");
        let camera = GridCamera::default();
        let document = Document::new();
        save(&path, &document, &camera, true).expect("first save");
        let first = std::fs::read_to_string(&path).expect("first");
        save(&path, &document, &camera, false).expect("second save");
        let backup = std::fs::read_to_string(backup_path(&path)).expect("backup");
        assert_eq!(backup, first);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_wrong_version() {
        let text = sample_text().replace("\"version\": 1", "\"version\": 2");
        assert!(from_json(&text, library::builtin()).is_err());
    }

    #[test]
    fn rejects_unknown_component() {
        let text = sample_text().replace("\"kind\": \"resistor\"", "\"kind\": \"missing\"");
        assert!(from_json(&text, library::builtin()).is_err());
    }
}
