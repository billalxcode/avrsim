//! Canvas screenshot cropping and PNG export.

use eframe::egui::{ColorImage, Rect};
use std::path::Path;

/// Saves a screenshot crop as an RGBA PNG.
pub fn save_crop(path: &Path, image: &ColorImage, crop: Rect, screen: Rect) -> Result<(), String> {
    let (x0, y0, x1, y1) = pixel_bounds(image, crop, screen)?;
    let rgba = crop_rgba(image, (x0, y0, x1, y1));
    write_png(path, x1 - x0, y1 - y0, &rgba)
}

fn pixel_bounds(
    image: &ColorImage,
    crop: Rect,
    screen: Rect,
) -> Result<(usize, usize, usize, usize), String> {
    let scale_x = image.size[0] as f32 / screen.width().max(1.0);
    let scale_y = image.size[1] as f32 / screen.height().max(1.0);
    let x0 = ((crop.min.x - screen.min.x) * scale_x).floor().max(0.0) as usize;
    let y0 = ((crop.min.y - screen.min.y) * scale_y).floor().max(0.0) as usize;
    let x1 = (((crop.max.x - screen.min.x) * scale_x).ceil() as usize).min(image.size[0]);
    let y1 = (((crop.max.y - screen.min.y) * scale_y).ceil() as usize).min(image.size[1]);
    if x1 <= x0 || y1 <= y0 {
        return Err("PNG crop is empty".to_string());
    }
    Ok((x0, y0, x1, y1))
}

fn crop_rgba(image: &ColorImage, bounds: (usize, usize, usize, usize)) -> Vec<u8> {
    let (x0, y0, x1, y1) = bounds;
    let mut data = Vec::with_capacity((x1 - x0) * (y1 - y0) * 4);
    for y in y0..y1 {
        for x in x0..x1 {
            let pixel = image.pixels[y * image.size[0] + x];
            data.extend_from_slice(&pixel.to_srgba_unmultiplied());
        }
    }
    data
}

fn write_png(path: &Path, width: usize, height: usize, rgba: &[u8]) -> Result<(), String> {
    let file =
        std::fs::File::create(path).map_err(|error| format!("PNG create failed: {error}"))?;
    let writer = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|error| format!("PNG header failed: {error}"))?;
    writer
        .write_image_data(rgba)
        .map_err(|error| format!("PNG write failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{crop_rgba, pixel_bounds, save_crop};
    use eframe::egui::{Color32, ColorImage, Rect};
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("emucad-png-{}-{name}", std::process::id()))
    }

    #[test]
    fn pixel_bounds_scale_crop() {
        let image = ColorImage::filled([8, 8], Color32::RED);
        let screen = Rect::from_min_max(eframe::egui::pos2(0.0, 0.0), eframe::egui::pos2(4.0, 4.0));
        let crop = Rect::from_min_max(eframe::egui::pos2(1.0, 1.0), eframe::egui::pos2(3.0, 3.0));
        assert_eq!(pixel_bounds(&image, crop, screen), Ok((2, 2, 6, 6)));
    }

    #[test]
    fn crop_collects_rgba() {
        let image = ColorImage::filled([2, 2], Color32::from_rgb(1, 2, 3));
        assert_eq!(crop_rgba(&image, (0, 0, 2, 2)).len(), 16);
    }

    #[test]
    fn save_crop_writes_png_signature() {
        let path = temp_path("signature");
        let image = ColorImage::filled([2, 2], Color32::GREEN);
        let screen = Rect::from_min_max(eframe::egui::pos2(0.0, 0.0), eframe::egui::pos2(2.0, 2.0));
        save_crop(&path, &image, screen, screen).expect("png");
        let bytes = std::fs::read(&path).expect("read");
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        let _ = std::fs::remove_file(path);
    }
}
