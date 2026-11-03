use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
pub struct IconMeta {
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Icon {
    pub name: String,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
    pub svg: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CatalogueError {
    #[error("Unable to download Lucide catalogue: {0}")]
    Download(String),
    #[error("Unable to parse Lucide metadata: {0}")]
    Metadata(String),
    #[error("Unable to read SVG {name}: {source}")]
    Svg { name: String, source: String },
}

/// Download the canonical Lucide static package and index every SVG in its icon directory.
/// Metadata and SVGs are kept intact; no curated subset is used.
pub fn download_catalogue() -> Result<Vec<Icon>, CatalogueError> {
    let metadata_url = "https://raw.githubusercontent.com/lucide-icons/lucide/main/packages/lucide-static/tags.json";
    let metadata_text = ureq::get(metadata_url).call()
        .map_err(|e| CatalogueError::Download(e.to_string()))?
        .into_string().map_err(|e| CatalogueError::Download(e.to_string()))?;
    let metadata: BTreeMap<String, Vec<String>> = serde_json::from_str(&metadata_text)
        .map_err(|e| CatalogueError::Metadata(e.to_string()))?;

    // Icons are fetched from the upstream repository one-by-one so the companion does
    // not need to bundle a stale or hand-maintained icon subset.
    let mut icons = Vec::with_capacity(metadata.len());
    for (name, tags) in metadata {
        let url = format!("https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/{name}.svg");
        let svg = ureq::get(&url).call()
            .map_err(|e| CatalogueError::Svg { name: name.clone(), source: e.to_string() })?
            .into_string().map_err(|e| CatalogueError::Svg { name: name.clone(), source: e.to_string() })?;
        icons.push(Icon { name, tags, categories: Vec::new(), svg });
    }
    Ok(icons)
}

pub fn search<'a>(icons: &'a [Icon], query: &str) -> Vec<&'a Icon> {
    let q = query.trim().to_lowercase();
    if q.is_empty() { return icons.iter().collect(); }
    icons.iter().filter(|i| {
        i.name.to_lowercase().contains(&q)
            || i.tags.iter().any(|t| t.to_lowercase().contains(&q))
            || i.categories.iter().any(|c| c.to_lowercase().contains(&q))
    }).collect()
}
