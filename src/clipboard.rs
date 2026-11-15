//! Native macOS clipboard writer for SVG-only vector data.

#[cfg(target_os = "macos")]
pub fn copy_svg(svg: &str) -> Result<(), String> {
    use std::{fs, process::Command, time::{SystemTime, UNIX_EPOCH}};

    // Validate that the supplied content is a parseable SVG before touching the clipboard.
    resvg::usvg::Tree::from_str(svg, &resvg::usvg::Options::default())
        .map_err(|error| format!("Could not prepare SVG for clipboard: {error}"))?;

    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let base = std::env::temp_dir().join(format!("lucide-clipboard-{}-{nonce}", std::process::id()));
    let svg_path = base.with_extension("svg");
    let helper_path = std::env::temp_dir().join(format!("lucide-clipboard-helper-{}-{nonce}.swift", std::process::id()));

    fs::write(&svg_path, svg).map_err(|error| format!("Could not write temporary SVG: {error}"))?;
    if let Err(error) = fs::write(&helper_path, include_str!("clipboard_helper.swift")) {
        let _ = fs::remove_file(&svg_path);
        return Err(format!("Could not prepare the native clipboard helper: {error}"));
    }

    // AppKit publishes SVG as a vector image and as SVG text for apps that
    // consume the textual representation. No PNG or other raster data is added.
    let result = Command::new("/usr/bin/swift")
        .arg(&helper_path)
        .arg(&svg_path)
        .output()
        .map_err(|error| format!("Could not start the native macOS clipboard helper: {error}"));

    let _ = fs::remove_file(&helper_path);
    let _ = fs::remove_file(&svg_path);
    let output = result?;
    if output.status.success() {
        Ok(())
    } else {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(if message.is_empty() { "The macOS clipboard helper failed.".into() } else { message })
    }
}

#[cfg(not(target_os = "macos"))]
pub fn copy_svg(_svg: &str) -> Result<(), String> {
    Err("SVG clipboard copying is currently supported on macOS only.".into())
}
