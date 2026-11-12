use figcraft_plugin_sdk::{
    Input, Result, Value, call, close_ui, entry, json, notify, save_file, show_ui,
};

const CATALOGUE: &str = include_str!("../catalogue.json");
const MAX_RESULTS: usize = 48;

#[derive(Clone)]
struct Icon {
    name: String,
    path: String,
}

fn catalogue() -> Vec<Icon> {
    let data: Value = serde_json::from_str(CATALOGUE).unwrap_or(Value::Null);
    data["icons"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| {
            Some(Icon {
                name: item["name"].as_str()?.to_string(),
                path: item["path"].as_str()?.to_string(),
            })
        })
        .collect()
}

fn find_icon(name: &str) -> Option<Icon> {
    catalogue().into_iter().find(|icon| icon.name == name)
}

fn results(query: &str) -> Vec<String> {
    let q = query.trim().to_lowercase();
    let mut matches: Vec<String> = catalogue()
        .into_iter()
        .filter(|icon| q.is_empty() || icon.name.replace('-', " ").contains(&q) || icon.name.contains(&q))
        .map(|icon| icon.name)
        .collect();
    matches.sort();
    matches.truncate(MAX_RESULTS);
    matches
}

fn panel(values: &Value) -> Result<Value> {
    let query = values["query"].as_str().unwrap_or("");
    let options = results(query);
    let current = values["icon"].as_str().unwrap_or("");
    let selected = if options.iter().any(|name| name == current) {
        current.to_string()
    } else {
        options.first().cloned().unwrap_or_default()
    };

    let mut controls = vec![
        json!({"type":"text","name":"query","label":"Search icons","value":query}),
        json!({"type":"note","label":format!("{} matching icons shown (maximum {}).", options.len(), MAX_RESULTS)}),
    ];

    if !options.is_empty() {
        controls.push(json!({
            "type":"list",
            "name":"icon",
            "label":"Icon",
            "options":options,
            "value":selected
        }));
    } else {
        controls.push(json!({"type":"note","label":"No icons match this search. Try another name."}));
    }

    controls.extend([
        json!({"type":"number","name":"size","label":"Icon size","min":8,"max":512,"value":values["size"].as_f64().unwrap_or(24.0)}),
        json!({"type":"number","name":"strokeWidth","label":"Stroke width","min":0.5,"max":8,"value":values["strokeWidth"].as_f64().unwrap_or(2.0)}),
        json!({"type":"color","name":"stroke","label":"Stroke colour","value":values["stroke"].as_str().unwrap_or("#202020")}),
        json!({"type":"button","name":"insert","label":"Insert editable vector"}),
        json!({"type":"button","name":"export","label":"Export SVG"}),
        json!({"type":"button","name":"close","label":"Close"})
    ]);

    show_ui(&json!({"title":"Lucide Companion","controls":controls}))?;
    Ok(Value::Null)
}

fn number(values: &Value, key: &str, fallback: f64, min: f64, max: f64) -> f64 {
    values[key].as_f64().unwrap_or(fallback).clamp(min, max)
}

fn selected_icon(values: &Value) -> Result<Icon> {
    let name = values["icon"].as_str().unwrap_or("");
    find_icon(name).ok_or_else(|| "Choose an icon from the search results first".into())
}

fn insert_icon(icon: &Icon, values: &Value) -> Result<Value> {
    let size = number(values, "size", 24.0, 8.0, 512.0);
    let stroke_width = number(values, "strokeWidth", 2.0, 0.5, 8.0);
    let stroke = values["stroke"].as_str().unwrap_or("#202020");

    let view = call("view.get", &json!({}))?;
    let cx = view["cx"].as_f64().unwrap_or(0.0);
    let cy = view["cy"].as_f64().unwrap_or(0.0);

    let created = call(
        "vector.create",
        &json!({
            "path": icon.path,
            "x": cx - size / 2.0,
            "y": cy - size / 2.0,
            "width": size,
            "height": size,
            "props": {
                "name": icon.name,
                "fill": "none",
                "stroke": stroke,
                "strokeWeight": stroke_width,
                "strokeCap": "round",
                "strokeJoin": "round"
            }
        }),
    )?;

    if let Some(id) = created["id"].as_u64() {
        let _ = call("select.set", &json!({"ids":[id]}));
    }

    notify(&format!("Inserted {} as an editable vector", icon.name));
    Ok(json!({"inserted":icon.name,"size":size}))
}

fn export_icon(icon: &Icon, values: &Value) -> Result<Value> {
    let size = number(values, "size", 24.0, 8.0, 512.0);
    let stroke_width = number(values, "strokeWidth", 2.0, 0.5, 8.0);
    let stroke = values["stroke"].as_str().unwrap_or("#202020");
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{size}\" height=\"{size}\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{stroke_width}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"{}\"/></svg>",
        icon.path
    );
    save_file(&format!("{}.svg", icon.name), "image/svg+xml", svg.as_bytes())?;
    notify(&format!("Exporting {}.svg", icon.name));
    Ok(json!({"exported":icon.name}))
}

fn run(input: Input) -> Result<Value> {
    if let Some(event) = input.event {
        let values = event.values;
        return match event.control.as_str() {
            "query" | "icon" => panel(&values),
            "insert" => {
                let icon = selected_icon(&values)?;
                insert_icon(&icon, &values)
            }
            "export" => {
                let icon = selected_icon(&values)?;
                export_icon(&icon, &values)
            }
            "close" => {
                close_ui();
                Ok(Value::Null)
            }
            _ => panel(&values),
        };
    }

    if catalogue().is_empty() {
        return Err("The Lucide catalogue is empty. Run scripts/build-wasm.sh to download and bundle lucide-static before compiling.".into());
    }

    let defaults = json!({
        "query":"",
        "icon":catalogue().first().map(|i| i.name.clone()).unwrap_or_default(),
        "size":24,
        "strokeWidth":2,
        "stroke":"#202020"
    });
    panel(&defaults)
}

entry!(run);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn results_are_bounded_and_case_insensitive() {
        assert!(results("ACTIVITY").iter().any(|name| name == "activity"));
        assert!(results("not-a-real-icon").is_empty());
        assert!(results("").len() <= MAX_RESULTS);
    }

    #[test]
    fn every_catalogue_path_is_nonempty() {
        for icon in catalogue() {
            assert!(!icon.name.is_empty());
            assert!(!icon.path.trim().is_empty());
        }
    }
}
