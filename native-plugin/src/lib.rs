use figcraft_plugin_sdk::{
    Input, Result, Value, call, entry, json, notify, save_file, show_ui,
};

const CATALOGUE: &str = include_str!("../catalogue.json");
const DEFAULT_STROKE: &str = "#202020";
const PAGE_SIZE: usize = 64;

#[derive(Clone, Debug)]
struct Icon {
    name: String,
    path: String,
}

fn catalogue() -> Vec<Icon> {
    let data: Value = serde_json::from_str(CATALOGUE).unwrap_or(Value::Null);
    data.get("icons")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            Some(Icon {
                name: item.get("name")?.as_str()?.to_string(),
                path: item.get("path")?.as_str()?.to_string(),
            })
        })
        .collect()
}

fn find_icon(name: &str) -> Option<Icon> {
    catalogue().into_iter().find(|icon| icon.name == name)
}

fn matching_names(icons: &[Icon], query: &str) -> Vec<String> {
    let query = query.trim().to_lowercase();
    let mut matches: Vec<String> = icons
        .iter()
        .filter(|icon| {
            query.is_empty()
                || icon.name.to_lowercase().contains(&query)
                || icon.name.replace('-', " ").contains(&query)
        })
        .map(|icon| icon.name.clone())
        .collect();
    matches.sort();
    matches
}

fn results(query: &str) -> Vec<String> {
    matching_names(&catalogue(), query)
}

fn page_count(total: usize) -> usize {
    total.div_ceil(PAGE_SIZE).max(1)
}

fn page_names(names: &[String], page: usize) -> Vec<String> {
    let start = page.saturating_sub(1) * PAGE_SIZE;
    names.iter().skip(start).take(PAGE_SIZE).cloned().collect()
}

fn panel(values: &Value) -> Result<Value> {
    let query = values.get("query").and_then(Value::as_str).unwrap_or("");
    let all_matches = results(query);
    let total_matches = all_matches.len();
    let pages = page_count(total_matches);
    let requested_page = values
        .get("page")
        .and_then(Value::as_str)
        .and_then(|page| page.parse::<usize>().ok())
        .unwrap_or(1);
    let current_page = requested_page.clamp(1, pages);
    let options = page_names(&all_matches, current_page);

    let current = values.get("icon").and_then(Value::as_str).unwrap_or("");
    let selected = if options.iter().any(|name| name == current) {
        current.to_string()
    } else {
        options.first().cloned().unwrap_or_default()
    };

    let mut controls = vec![
        json!({"type":"text","name":"query","label":"Search icons","value":query}),
        json!({
            "type":"note",
            "label":format!(
                "{} icons found · Page {} of {}.",
                total_matches, current_page, pages
            )
        }),
    ];

    if !options.is_empty() {
        controls.push(json!({
            "type":"select",
            "name":"icon",
            "label":"Icon",
            "options":options,
            "value":selected
        }));
    } else {
        controls.push(json!({
            "type":"note",
            "label":"No icons match this search. Try another name."
        }));
    }

    if pages > 1 {
        controls.push(json!({
            "type":"select",
            "name":"page",
            "label":"Page",
            "options":(1..=pages).map(|page| page.to_string()).collect::<Vec<_>>(),
            "value":current_page.to_string()
        }));
    }

    let selected_size = number(values, "size", 24.0, 8.0, 512.0);
    let selected_stroke_width = number(values, "strokeWidth", 2.0, 0.5, 8.0);
    let selection_summary = if selected.is_empty() {
        "No icon selected".to_string()
    } else {
        format!(
            "Selected: {} · {} × {} · Stroke {}",
            selected, selected_size, selected_size, selected_stroke_width
        )
    };
    controls.extend([
        json!({
            "type":"note",
            "label":selection_summary
        }),
        json!({"type":"button","name":"insert","label":"Insert vector"}),
        json!({"type":"button","name":"refresh","label":"Refresh list"}),
        json!({"type":"button","name":"export","label":"Export SVG"}),
        json!({
            "type":"number",
            "name":"size",
            "label":"Icon size (px)",
            "min":8,
            "max":512,
            "value":values.get("size").and_then(Value::as_f64).unwrap_or(24.0)
        }),
        json!({
            "type":"number",
            "name":"strokeWidth",
            "label":"Stroke width",
            "min":0.5,
            "max":8,
            "value":values.get("strokeWidth").and_then(Value::as_f64).unwrap_or(2.0)
        }),
        json!({
            "type":"color",
            "name":"stroke",
            "label":"Stroke colour",
            "value":values.get("stroke").and_then(Value::as_str).unwrap_or(DEFAULT_STROKE)
        })
    ]);

    show_ui(&json!({"title":"Lucide Companion","controls":controls}))?;
    Ok(Value::Null)
}

fn number(values: &Value, key: &str, fallback: f64, min: f64, max: f64) -> f64 {
    values
        .get(key)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
        .unwrap_or(fallback)
        .clamp(min, max)
}

fn stroke_color(values: &Value) -> Result<String> {
    let raw = values
        .get("stroke")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_STROKE)
        .trim();
    let hex = raw.strip_prefix('#').unwrap_or(raw);

    if !hex.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err("Stroke colour must be a hexadecimal colour.".into());
    }

    match hex.len() {
        3 => {
            let mut expanded = String::from("#");
            for character in hex.chars() {
                expanded.push(character);
                expanded.push(character);
            }
            Ok(expanded.to_ascii_uppercase())
        }
        6 => Ok(format!("#{}", hex.to_ascii_uppercase())),
        _ => Err("Stroke colour must use #RGB or #RRGGBB.".into()),
    }
}

fn selected_icon(values: &Value) -> Result<Icon> {
    let name = values.get("icon").and_then(Value::as_str).unwrap_or("");
    find_icon(name).ok_or_else(|| "Choose an icon from the search results first".into())
}

fn create_vector(
    icon: &Icon,
    size: f64,
    stroke_width: f64,
    stroke: &str,
    size_by_height: bool,
) -> Result<Value> {
    let size_property = if size_by_height {
        json!({"height":size})
    } else {
        json!({"width":size})
    };
    let mut params = json!({
        "path": icon.path,
        "name": icon.name,
        "props": {
            "fill": "none",
            "stroke": stroke,
            "strokeWeight": stroke_width,
            "strokeCap": "round",
            "strokeJoin": "round"
        }
    });
    if let (Some(target), Some(source)) = (params.as_object_mut(), size_property.as_object()) {
        for (key, value) in source {
            target.insert(key.clone(), value.clone());
        }
    }
    call("vector.create", &params)
}

fn insert_icon(icon: &Icon, values: &Value) -> Result<Value> {
    let size = number(values, "size", 24.0, 8.0, 512.0);
    let stroke_width = number(values, "strokeWidth", 2.0, 0.5, 8.0);
    let stroke = stroke_color(values)?;

    let view = call("view.get", &json!({}))?;
    let cx = view.get("cx").and_then(Value::as_f64).unwrap_or(0.0);
    let cy = view.get("cy").and_then(Value::as_f64).unwrap_or(0.0);

    // Supplying only width preserves each icon's original aspect ratio.
    // Width preserves the normal icon aspect ratio. A few Lucide symbols are only vertical
    // lines and have no horizontal extent; for those, retry with height instead.
    let created = match create_vector(icon, size, stroke_width, &stroke, false) {
        Ok(value) => value,
        Err(error) if error.contains("no horizontal extent") => {
            create_vector(icon, size, stroke_width, &stroke, true)?
        }
        Err(error) => return Err(error),
    };

    let id = created
        .get("id")
        .and_then(Value::as_u64)
        .ok_or("FigCraft did not return the new vector layer ID")?;

    // Read the actual vector bounds, then position the layer by its bounds so it is centred
    // correctly even when the icon path does not start at (0, 0) or is not square.
    let bounds = call("node.get", &json!({"id":id,"fields":["width","height"]}))?;
    let width = bounds.get("width").and_then(Value::as_f64).unwrap_or(size);
    let height = bounds.get("height").and_then(Value::as_f64).unwrap_or(size);
    call(
        "node.set",
        &json!({
            "ids":[id],
            "x":cx - width / 2.0,
            "y":cy - height / 2.0
        }),
    )?;

    notify(&format!("Inserted {} as an editable vector", icon.name));
    Ok(json!({"inserted":icon.name,"size":size,"id":id}))
}

fn export_icon(icon: &Icon, values: &Value) -> Result<Value> {
    let size = number(values, "size", 24.0, 8.0, 512.0);
    let stroke_width = number(values, "strokeWidth", 2.0, 0.5, 8.0);
    let stroke = stroke_color(values)?;
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{size}\" height=\"{size}\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{stroke_width}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"{}\"/></svg>",
        icon.path
    );

    save_file(
        &format!("{}.svg", icon.name),
        "image/svg+xml",
        svg.as_bytes(),
    )?;
    notify(&format!("Exported {}.svg", icon.name));
    Ok(json!({"exported":icon.name}))
}

fn run(input: Input) -> Result<Value> {
    if let Some(event) = input.event {
        let values = event.values;
        return match event.control.as_str() {
            "query" => {
                let mut searched = values.clone();
                if let Some(fields) = searched.as_object_mut() {
                    fields.insert("page".into(), json!("1"));
                }
                panel(&searched)
            }
            "icon" | "page" | "size" | "strokeWidth" | "stroke" => panel(&values),
            "refresh" => {
                let mut refreshed = values.clone();
                if let Some(fields) = refreshed.as_object_mut() {
                    fields.insert("query".into(), json!(""));
                    fields.insert("page".into(), json!("1"));
                }
                panel(&refreshed)
            }
            "insert" => {
                let icon = selected_icon(&values)?;
                insert_icon(&icon, &values)
            }
            "export" => {
                let icon = selected_icon(&values)?;
                export_icon(&icon, &values)
            }
            _ => panel(&values),
        };
    }

    if catalogue().is_empty() {
        return Err(
            "The Lucide catalogue is empty. Run build-wasm.sh to download and bundle the icons."
                .into(),
        );
    }

    let first_icon = catalogue()
        .first()
        .map(|icon| icon.name.clone())
        .unwrap_or_default();
    let defaults = json!({
        "query":"",
        "icon":first_icon,
        "page":"1",
        "size":24,
        "strokeWidth":2,
        "stroke":DEFAULT_STROKE
    });
    panel(&defaults)
}

entry!(run);

#[cfg(test)]
mod tests {
    use super::*;

    fn test_icons() -> Vec<Icon> {
        vec![
            Icon { name: "activity".into(), path: "M0 0L1 1".into() },
            Icon { name: "arrow-left".into(), path: "M0 0L1 1".into() },
            Icon { name: "calendar-days".into(), path: "M0 0L1 1".into() },
        ]
    }

    #[test]
    fn search_is_case_insensitive_and_supports_spaces() {
        let icons = test_icons();
        assert_eq!(matching_names(&icons, "ACTIVITY"), vec!["activity"]);
        assert_eq!(matching_names(&icons, "arrow left"), vec!["arrow-left"]);
        assert_eq!(matching_names(&icons, "not-a-real-icon"), Vec::<String>::new());
    }

    #[test]
    fn search_is_sorted_and_not_truncated_in_the_matcher() {
        let icons = test_icons();
        assert_eq!(
            matching_names(&icons, ""),
            vec!["activity", "arrow-left", "calendar-days"]
        );
    }


    #[test]
    fn search_returns_the_entire_large_catalogue_without_a_ui_result_cap() {
        let icons: Vec<Icon> = (0..2_400)
            .map(|index| Icon {
                name: format!("icon-{index:04}"),
                path: "M0 0L1 1".into(),
            })
            .collect();
        let matches = matching_names(&icons, "");
        assert_eq!(matches.len(), 2_400);
        assert_eq!(matches.first().map(String::as_str), Some("icon-0000"));
        assert_eq!(matches.last().map(String::as_str), Some("icon-2399"));
    }


    #[test]
    fn large_catalogues_are_browsable_in_pages_of_64() {
        let names: Vec<String> = (0..2_400).map(|index| format!("icon-{index:04}")).collect();
        assert_eq!(page_count(names.len()), 38);
        assert_eq!(page_names(&names, 1).len(), 64);
        assert_eq!(page_names(&names, 2).len(), 64);
        assert_eq!(page_names(&names, 38).len(), 32);
        assert_eq!(page_names(&names, 38).first().map(String::as_str), Some("icon-2368"));
        assert!(page_names(&names, 39).is_empty());
    }

    #[test]
    fn stroke_color_is_normalized_and_validated() {
        assert_eq!(stroke_color(&json!({"stroke":"#abc"})).unwrap(), "#AABBCC");
        assert_eq!(stroke_color(&json!({"stroke":"12abEF"})).unwrap(), "#12ABEF");
        assert!(stroke_color(&json!({"stroke":"red"})).is_err());
        assert!(stroke_color(&json!({"stroke":"#12345"})).is_err());
    }

    #[test]
    fn numeric_values_are_clamped() {
        assert_eq!(number(&json!({"size":900}), "size", 24.0, 8.0, 512.0), 512.0);
        assert_eq!(number(&json!({}), "size", 24.0, 8.0, 512.0), 24.0);
    }
}
