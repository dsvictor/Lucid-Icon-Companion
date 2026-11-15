//! Native macOS clipboard writer for vector-friendly SVG and raster fallback data.

#[cfg(target_os = "macos")]
pub fn copy_svg(svg: &str) -> Result<(), String> {
    use std::{fs, process::Command, time::{SystemTime, UNIX_EPOCH}};

    let tree = resvg::usvg::Tree::from_str(svg, &resvg::usvg::Options::default())
        .map_err(|error| format!("Could not prepare SVG for clipboard: {error}"))?;
    let source_size = tree.size().to_int_size();
    let scale = 4.0_f32;
    let width = source_size.width().max(1).saturating_mul(scale as u32);
    let height = source_size.height().max(1).saturating_mul(scale as u32);
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "Could not allocate the PNG clipboard image.".to_owned())?;
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    let rgba = image::RgbaImage::from_raw(width, height, pixmap.data().to_vec())
        .ok_or_else(|| "Could not prepare the PNG clipboard image.".to_owned())?;
    let mut png_cursor = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(rgba)
        .write_to(&mut png_cursor, image::ImageFormat::Png)
        .map_err(|error| format!("Could not encode PNG clipboard image: {error}"))?;
    let png = png_cursor.into_inner();

    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let base = std::env::temp_dir().join(format!("lucide-clipboard-{}-{nonce}", std::process::id()));
    let svg_path = base.with_extension("svg");
    let png_path = base.with_extension("png");
    fs::write(&svg_path, svg).map_err(|error| error.to_string())?;
    if let Err(error) = fs::write(&png_path, png) {
        let _ = fs::remove_file(&svg_path);
        return Err(error.to_string());
    }

    // Use AppKit directly via Swift. The JXA bridge used previously could
    // return success without updating the general pasteboard on some macOS builds.
    // A single NSPasteboard is populated with both SVG and PNG representations.
    let helper_path = std::env::temp_dir().join(format!("lucide-clipboard-helper-{}-{nonce}.swift", std::process::id()));
    fs::write(&helper_path, include_str!("clipboard_helper.swift"))
        .map_err(|error| format!("Could not prepare the native clipboard helper: {error}"))?;
    let result = Command::new("/usr/bin/swift")
        .arg(&helper_path)
        .arg(&svg_path)
        .arg(&png_path)
        .output()
        .map_err(|error| format!("Could not start the native macOS clipboard helper: {error}"));
    let _ = fs::remove_file(&helper_path);
    let _ = fs::remove_file(&svg_path);
    let _ = fs::remove_file(&png_path);
    let output = result?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

#[cfg(not(target_os = "macos"))]
pub fn copy_svg(_svg: &str) -> Result<(), String> {
    Err("Rich SVG clipboard copying is currently supported on macOS only.".into())
}
