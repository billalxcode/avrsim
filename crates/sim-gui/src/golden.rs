//! Headless RGBA golden renderer for GUI regression tests.

use crate::model::{Document, GridPos, Mirror, NetClass, Rotation};
use crate::scene::{self, Scene};
use crate::selection::Selection;
use eframe::egui::{self, Color32, Pos2, Rect, Vec2};
use std::collections::HashMap;
use std::path::PathBuf;

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

struct Texture {
    width: usize,
    height: usize,
    pixels: Vec<Color32>,
}

/// Renders the fixed golden scene into straight RGBA bytes.
pub fn render_golden() -> Vec<u8> {
    let context = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(WIDTH as f32, HEIGHT as f32),
        )),
        ..Default::default()
    };
    let mut output = context.run_ui(input, draw_golden);
    let textures = collect_textures(&output.textures_delta);
    output.textures_delta.clear();
    let primitives = context.tessellate(output.shapes, 1.0);
    rasterize(primitives, &textures)
}

fn draw_golden(ui: &mut egui::Ui) {
    let rect = ui.max_rect();
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, Color32::from_rgb(18, 20, 24));
    draw_golden_scene(painter, rect);
    painter.text(
        egui::pos2(10.0, 216.0),
        egui::Align2::LEFT_TOP,
        "EmuCAD golden",
        egui::FontId::monospace(12.0),
        Color32::from_rgb(220, 228, 238),
    );
}

fn draw_golden_scene(painter: &egui::Painter, rect: Rect) {
    let document = golden_document();
    let selection = Selection::default();
    let camera = golden_camera();
    let scene = golden_scene(&document, &selection);
    scene::draw(painter, rect, &camera, scene, None);
}

fn golden_camera() -> crate::canvas::GridCamera {
    crate::canvas::GridCamera {
        pan: Vec2::new(24.0, 20.0),
        zoom: 1.0,
        cell_px: 20.0,
    }
}

fn golden_scene<'a>(document: &'a Document, selection: &'a Selection) -> Scene<'a> {
    Scene {
        document,
        library: crate::library::builtin(),
        selection,
        hover_wire: None,
        placing: None,
        rotation: Rotation::Deg0,
        mirror: Mirror::None,
        wire_preview: &[],
        wire_class: NetClass::Digital,
        selection_box: None,
        minimap: false,
        flow_phase: None,
    }
}

fn golden_document() -> Document {
    let resistor = crate::library::find("resistor").expect("resistor");
    let led = crate::library::find("led").expect("led");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(1, 2), Rotation::Deg0, Mirror::None);
    document.add(led, GridPos::new(8, 2), Rotation::Deg0, Mirror::None);
    document
}

fn collect_textures(delta: &egui::TexturesDelta) -> HashMap<egui::TextureId, Texture> {
    let mut textures = HashMap::new();
    for (id, images) in &delta.set {
        for image in images {
            apply_delta(&mut textures, *id, image);
        }
    }
    textures
}

fn apply_delta(
    textures: &mut HashMap<egui::TextureId, Texture>,
    id: egui::TextureId,
    delta: &egui::epaint::ImageDelta,
) {
    let egui::epaint::ImageData::Color(image) = &delta.image;
    let Some([x, y]) = delta.pos else {
        textures.insert(id, texture_from(image.as_ref()));
        return;
    };
    if let Some(texture) = textures.get_mut(&id) {
        blit(texture, x, y, image.as_ref());
    }
}

fn texture_from(image: &egui::ColorImage) -> Texture {
    Texture {
        width: image.size[0],
        height: image.size[1],
        pixels: image.pixels.clone(),
    }
}

fn blit(texture: &mut Texture, x: usize, y: usize, image: &egui::ColorImage) {
    for row in 0..image.size[1] {
        for column in 0..image.size[0] {
            let index = (y + row) * texture.width + x + column;
            texture.pixels[index] = image.pixels[row * image.size[0] + column];
        }
    }
}

fn rasterize(
    primitives: Vec<egui::ClippedPrimitive>,
    textures: &HashMap<egui::TextureId, Texture>,
) -> Vec<u8> {
    let mut pixels = vec![[0.0_f32; 4]; WIDTH * HEIGHT];
    for clipped in primitives {
        let egui::epaint::Primitive::Mesh(mesh) = clipped.primitive else {
            continue;
        };
        let texture = textures.get(&mesh.texture_id);
        for triangle in mesh.indices.chunks_exact(3) {
            let Some(vertices) = mesh_vertices(&mesh, triangle) else {
                continue;
            };
            raster_triangle(&mut pixels, clipped.clip_rect, vertices, texture);
        }
    }
    pixels.into_iter().flat_map(pixel_rgba).collect()
}

fn mesh_vertices(mesh: &egui::epaint::Mesh, triangle: &[u32]) -> Option<[egui::epaint::Vertex; 3]> {
    let a = mesh.vertices.get(triangle[0] as usize)?;
    let b = mesh.vertices.get(triangle[1] as usize)?;
    let c = mesh.vertices.get(triangle[2] as usize)?;
    Some([*a, *b, *c])
}

fn raster_triangle(
    pixels: &mut [[f32; 4]],
    clip: Rect,
    vertices: [egui::epaint::Vertex; 3],
    texture: Option<&Texture>,
) {
    let [a, b, c] = vertices;
    let area = edge(a.pos, b.pos, c.pos);
    if area.abs() < f32::EPSILON {
        return;
    }
    let (min_x, min_y, max_x, max_y) = pixel_bounds(clip, a.pos, b.pos, c.pos);
    for y in min_y..max_y {
        for x in min_x..max_x {
            let point = egui::pos2(x as f32 + 0.5, y as f32 + 0.5);
            raster_pixel(pixels, area, [a, b, c], point, texture);
        }
    }
}

fn raster_pixel(
    pixels: &mut [[f32; 4]],
    area: f32,
    vertices: [egui::epaint::Vertex; 3],
    point: Pos2,
    texture: Option<&Texture>,
) {
    let Some(weights) = barycentric(vertices, point, area) else {
        return;
    };
    let mut color = interpolate(vertices, weights);
    apply_texture(&mut color, vertices, weights, texture);
    blend(
        &mut pixels[point.y as usize * WIDTH + point.x as usize],
        color,
    );
}

fn barycentric(vertices: [egui::epaint::Vertex; 3], point: Pos2, area: f32) -> Option<[f32; 3]> {
    let [a, b, c] = vertices;
    let weights = [
        edge(b.pos, c.pos, point) / area,
        edge(c.pos, a.pos, point) / area,
        edge(a.pos, b.pos, point) / area,
    ];
    if weights.iter().any(|weight| *weight < -1e-4) {
        return None;
    }
    Some(weights)
}

fn apply_texture(
    color: &mut [f32; 4],
    vertices: [egui::epaint::Vertex; 3],
    weights: [f32; 3],
    texture: Option<&Texture>,
) {
    let Some(texture) = texture else {
        return;
    };
    let uv = interpolate_uv(vertices, weights);
    let texel = sample_texture(texture, uv);
    for channel in 0..4 {
        color[channel] *= texel[channel];
    }
}

fn pixel_bounds(clip: Rect, a: Pos2, b: Pos2, c: Pos2) -> (usize, usize, usize, usize) {
    let min = a.min(b).min(c).max(clip.min).max(Pos2::ZERO);
    let max = a
        .max(b)
        .max(c)
        .min(clip.max)
        .min(egui::pos2(WIDTH as f32, HEIGHT as f32));
    let min_x = min.x.floor().max(0.0) as usize;
    let min_y = min.y.floor().max(0.0) as usize;
    let max_x = (max.x.ceil().max(0.0) as usize).min(WIDTH);
    let max_y = (max.y.ceil().max(0.0) as usize).min(HEIGHT);
    (min_x, min_y, max_x, max_y)
}

fn edge(a: Pos2, b: Pos2, point: Pos2) -> f32 {
    (point.x - a.x) * (b.y - a.y) - (point.y - a.y) * (b.x - a.x)
}

fn interpolate(vertices: [egui::epaint::Vertex; 3], weights: [f32; 3]) -> [f32; 4] {
    let mut color = [0.0; 4];
    for (vertex, weight) in vertices.iter().zip(weights) {
        let rgba = vertex.color.to_array();
        for channel in 0..4 {
            color[channel] += f32::from(rgba[channel]) * weight / 255.0;
        }
    }
    color
}

fn interpolate_uv(vertices: [egui::epaint::Vertex; 3], weights: [f32; 3]) -> Pos2 {
    let mut uv = Vec2::ZERO;
    for (vertex, weight) in vertices.iter().zip(weights) {
        uv += vertex.uv.to_vec2() * weight;
    }
    uv.to_pos2()
}

fn sample_texture(texture: &Texture, uv: Pos2) -> [f32; 4] {
    let x = (uv.x * texture.width as f32)
        .floor()
        .clamp(0.0, texture.width as f32 - 1.0);
    let y = (uv.y * texture.height as f32)
        .floor()
        .clamp(0.0, texture.height as f32 - 1.0);
    let color = texture.pixels[y as usize * texture.width + x as usize].to_array();
    [
        f32::from(color[0]) / 255.0,
        f32::from(color[1]) / 255.0,
        f32::from(color[2]) / 255.0,
        f32::from(color[3]) / 255.0,
    ]
}

fn blend(destination: &mut [f32; 4], source: [f32; 4]) {
    let alpha = source[3];
    for channel in 0..4 {
        destination[channel] = source[channel] + destination[channel] * (1.0 - alpha);
    }
}

fn pixel_rgba(pixel: [f32; 4]) -> [u8; 4] {
    let alpha = pixel[3].clamp(0.0, 1.0);
    let scale = if alpha > 0.0 { 1.0 / alpha } else { 0.0 };
    [
        (pixel[0] * scale * 255.0).round().clamp(0.0, 255.0) as u8,
        (pixel[1] * scale * 255.0).round().clamp(0.0, 255.0) as u8,
        (pixel[2] * scale * 255.0).round().clamp(0.0, 255.0) as u8,
        (alpha * 255.0).round() as u8,
    ]
}

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/scene-320x240.rgba")
}

#[cfg(test)]
mod tests {
    use super::{golden_path, render_golden};

    #[test]
    fn golden_render_is_deterministic() {
        assert_eq!(render_golden(), render_golden());
    }

    #[test]
    fn golden_scene_matches_snapshot() {
        let image = render_golden();
        let path = golden_path();
        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("golden dir");
            }
            std::fs::write(&path, &image).expect("write golden");
            return;
        }
        let expected = std::fs::read(&path).expect("read golden");
        assert_snapshot(&image, &expected);
    }

    fn assert_snapshot(actual: &[u8], expected: &[u8]) {
        assert_eq!(
            actual.len(),
            super::WIDTH * super::HEIGHT * 4,
            "render size"
        );
        assert_eq!(actual.len(), expected.len(), "golden size mismatch");
        let Some((index, left, right)) = first_difference(actual, expected) else {
            return;
        };
        panic_mismatch(index, left, right);
    }

    fn first_difference(actual: &[u8], expected: &[u8]) -> Option<(usize, u8, u8)> {
        actual
            .iter()
            .zip(expected.iter())
            .enumerate()
            .find(|(_, (left, right))| left != right)
            .map(|(index, (left, right))| (index, *left, *right))
    }

    fn panic_mismatch(index: usize, left: u8, right: u8) -> ! {
        let pixel = index / 4;
        let x = pixel % super::WIDTH;
        let y = pixel / super::WIDTH;
        panic!(
            "golden mismatch at pixel ({x}, {y}) channel {}: {left} != {right}",
            index % 4
        );
    }
}
