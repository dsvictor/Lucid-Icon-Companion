//! Lucide Library — FigCraft API 2 plugin panel.
//! The panel is intentionally implemented with FigCraft's own controls. FigCraft currently
//! presents API 2 panels as draggable floating windows (not permanently dockable sidebars).

use figcraft_plugin_sdk::{Input, Result, Value, call, entry, json, notify, set_setting, setting, show_ui};

const ICONS: &[(&str, &str)] = &[
    ("activity", "M3 12h4l3-8 4 16 3-8h4"),
    ("arrow-right", "M5 12h14m-7-7 7 7-7 7"),
    ("check", "m5 12 4 4L19 6"),
    ("chevron-down", "m6 9 6 6 6-6"),
    ("circle", "M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0-18 0"),
    ("copy", "M8 8V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2h-4M5 6h9a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2z"),
    ("download", "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4m4-5 5 5 5-5m-5 5V3"),
    ("heart", "M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1.1-1.1a5.5 5.5 0 0 0-7.8 7.8l1.1 1.1L12 21l7.8-7.5 1.1-1.1a5.5 5.5 0 0 0-.1-7.8z"),
    ("home", "m3 10 9-7 9 7v10a1 1 0 0 1-1 1h-5v-7H9v7H4a1 1 0 0 1-1-1z"),
    ("image", "M4 4h16v16H4zM8.5 10a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3zM20 15l-5-5L5 20"),
    ("info", "M12 16v-4m0-4h.01M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0z"),
    ("layers", "m12 2 9 5-9 5-9-5 9-5zm-9 10 9 5 9-5m-18 5 9 5 9-5"),
    ("menu", "M4 6h16M4 12h16M4 18h16"),
    ("minus", "M5 12h14"),
    ("plus", "M12 5v14m-7-7h14"),
    ("search", "m21 21-4.3-4.3M19 10.5a8.5 8.5 0 1 1-17 0 8.5 8.5 0 0 1 17 0z"),
    ("settings", "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1-1.7 2.9-.2-.1a1.7 1.7 0 0 0-1.8.2l-.1.1h-3.4l-.1-.2a1.7 1.7 0 0 0-1.5-1l-.2.1-2.9-1.7.1-.2a1.7 1.7 0 0 0-.2-1.8l-.1-.1v-3.4l.2-.1a1.7 1.7 0 0 0 1-1.5l-.1-.2 1.7-2.9.2.1a1.7 1.7 0 0 0 1.8-.2l.1-.1h3.4l.1.2a1.7 1.7 0 0 0 1.5 1l.2-.1 2.9 1.7-.1.2a1.7 1.7 0 0 0 .2 1.8l.1.1z"),
    ("star", "m12 3 2.8 5.7 6.2.9-4.5 4.4 1.1 6.2-5.6-3-5.6 3 1.1-6.2L3 9.6l6.2-.9z"),
    ("trash", "M3 6h18m-2 0-.9 14H5.9L5 6m4 0V4h6v2m-5 4v6m4-6v6"),
    ("user", "M20 21a8 8 0 0 0-16 0m8-10a4 4 0 1 0 0-8 4 4 0 0 0 0 8z"),
    ("x", "M18 6 6 18M6 6l12 12"),
];

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let a = chunk[0] as u32;
        let b = chunk.get(1).copied().unwrap_or(0) as u32;
        let c = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (a << 16) | (b << 8) | c;
        for i in 0..4 {
            let index = ((n >> (18 - 6 * i)) & 63) as usize;
            out.push(TABLE[index] as char);
        }
        if chunk.len() < 3 { out.pop(); out.push('='); }
        if chunk.len() < 2 { out.pop(); out.push('='); }
    }
    out
}

fn safe_color(value: &str) -> &str {
    let v = value.trim();
    if v.len() == 7 && v.starts_with('#') && v[1..].bytes().all(|b| b.is_ascii_hexdigit()) { v } else { "#111111" }
}

fn filtered(query: &str) -> Vec<&'static str> {
    let q = query.trim().to_ascii_lowercase();
    ICONS.iter().filter(|(name, _)| q.is_empty() || name.contains(&q)).map(|(name, _)| *name).collect()
}

fn panel(values: &Value, search: &str) -> Result<Value> {
    let names = filtered(search);
    let chosen = values["icon"].as_str().filter(|v| names.contains(v)).or_else(|| names.first().copied()).unwrap_or("activity");
    let stroke = values["stroke"].as_str().unwrap_or("#111111");
    let width = values["width"].as_f64().unwrap_or(2.0).clamp(0.5, 4.0);
    show_ui(&json!({
        "title": "Lucide Library",
        "controls": [
            {"type":"note","label":"Search and insert editable SVG icons"},
            {"type":"text","name":"search","label":"Search icons","value":search},
            {"type":"list","name":"icon","label":"Icon","options":names,"value":chosen},
            {"type":"color","name":"stroke","label":"Stroke colour","value":safe_color(stroke)},
            {"type":"slider","name":"width","label":"Stroke width","min":0.5,"max":4.0,"value":width},
            {"type":"note","label":format!("{} icons in this starter library · selected: {}", names.len(), chosen)},
            {"type":"button","name":"insert","label":"Insert into document"},
            {"type":"button","name":"close","label":"Close"}
        ]
    }))?;
    Ok(Value::Null)
}

fn insert_icon(name: &str, stroke: &str, width: f64) -> Result<Value> {
    let (_, path) = ICONS.iter().find(|(n, _)| *n == name).ok_or("Choose an icon from the list")?;
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"{}\"/></svg>",
        safe_color(stroke), width.clamp(0.5, 4.0), path
    );
    // FigCraft plugins cannot call file.* commands. Create an editable vector directly
    // through the permitted node.create command instead of trying to import an SVG file.
    // Keep the original Lucide path data as a vector network so the result remains editable.
    call("node.create", &json!({
        "type": "vector",
        "path": path,
        "x": 0,
        "y": 0,
        "width": 24,
        "height": 24,
        "name": format!("{name}"),
        "props": {
            "fill": "none",
            "stroke": safe_color(stroke),
            "strokeWeight": width.clamp(0.5, 4.0),
            "strokeCap": "round",
            "strokeJoin": "round"
        }
    }))?;
    notify(&format!("Inserted {name}"));
    Ok(json!({"inserted": name}))
}

fn run(input: Input) -> Result<Value> {
    if let Some(event) = input.event {
        let search = event.values["search"].as_str().unwrap_or_default().to_string();
        let icon = event.values["icon"].as_str().unwrap_or("activity").to_string();
        let stroke = event.values["stroke"].as_str().unwrap_or("#111111").to_string();
        let width = event.values["width"].as_f64().unwrap_or(2.0);
        match event.control.as_str() {
            "search" => {
                set_setting("search", &json!(search))?;
                panel(&event.values, &search)?;
            }
            "icon" | "stroke" | "width" => panel(&event.values, &search)?,
            "insert" => return insert_icon(&icon, &stroke, width),
            "close" => figcraft_plugin_sdk::close_ui(),
            _ => {}
        }
        return Ok(Value::Null);
    }
    let search = setting("search").and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default();
    panel(&Value::Null, &search)?;
    Ok(json!({"panel":"Lucide Library","icons":ICONS.len()}))
}

entry!(run);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encodes_correctly() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn search_is_case_insensitive_and_name_based() {
        assert_eq!(filtered("ARRO"), vec!["arrow-right"]);
        assert!(filtered("not-an-icon").is_empty());
        assert_eq!(filtered("").len(), ICONS.len());
    }
}
