use flate2::read::GzDecoder;
use serde::Deserialize;
use std::{collections::BTreeMap, fs, io::{Read, Write}, path::{Path, PathBuf}};
use tar::Archive;

#[derive(Debug, Clone)]
pub struct Icon { pub name: String, pub svg: String, pub tags: Vec<String> }

#[derive(Debug, Deserialize)]
struct NpmLatest { version: String, dist: NpmDist }
#[derive(Debug, Deserialize)]
struct NpmDist { tarball: String }

#[derive(Debug, thiserror::Error)]
pub enum CatalogueError {
    #[error("Could not retrieve the current Lucide package version: {0}")]
    Metadata(String),
    #[error("Could not download the Lucide package: {0}")]
    Download(String),
    #[error("Could not unpack the Lucide package: {0}")]
    Archive(String),
    #[error("The package contained no SVG icons at package/icons/*.svg.")]
    Empty,
    #[error("Cache error: {0}")]
    Cache(String),
}

pub fn load_cache(root: &Path) -> Vec<Icon> {
    let tags_path = root.join("tags.json");
    let tags: BTreeMap<String, Vec<String>> = fs::read_to_string(tags_path)
        .ok().and_then(|raw| serde_json::from_str(&raw).ok()).unwrap_or_default();
    let mut icons = Vec::new();
    let Ok(entries) = fs::read_dir(root) else { return icons; };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("svg") { continue; }
        let Ok(svg) = fs::read_to_string(&path) else { continue; };
        let Some(name) = path.file_stem().and_then(|s| s.to_str()) else { continue; };
        icons.push(Icon { name: name.to_owned(), svg, tags: tags.get(name).cloned().unwrap_or_default() });
    }
    icons.sort_by(|a, b| a.name.cmp(&b.name));
    icons
}

pub fn download_latest(root: &Path) -> Result<(Vec<Icon>, String), CatalogueError> {
    fs::create_dir_all(root).map_err(|e| CatalogueError::Cache(e.to_string()))?;
    let latest: NpmLatest = ureq::get("https://registry.npmjs.org/lucide-static/latest")
        .set("User-Agent", "Lucide-Icon-Companion/0.2.0").call()
        .map_err(|e| CatalogueError::Metadata(e.to_string()))?
        .into_json().map_err(|e| CatalogueError::Metadata(e.to_string()))?;
    let mut archive_bytes = Vec::new();
    ureq::get(&latest.dist.tarball).set("User-Agent", "Lucide-Icon-Companion/0.2.0")
        .call().map_err(|e| CatalogueError::Download(e.to_string()))?
        .into_reader().read_to_end(&mut archive_bytes)
        .map_err(|e| CatalogueError::Download(e.to_string()))?;
    let decoder = GzDecoder::new(archive_bytes.as_slice());
    let mut archive = Archive::new(decoder);
    let entries = archive.entries().map_err(|e| CatalogueError::Archive(e.to_string()))?;
    let mut count = 0usize;
    let mut tags_written = false;

    for entry in fs::read_dir(root).map_err(|e| CatalogueError::Cache(e.to_string()))? {
        let path = entry.map_err(|e| CatalogueError::Cache(e.to_string()))?.path();
        if path.extension().and_then(|s| s.to_str()) == Some("svg") {
            fs::remove_file(path).map_err(|e| CatalogueError::Cache(e.to_string()))?;
        }
    }

    for item in entries {
        let mut entry = item.map_err(|e| CatalogueError::Archive(e.to_string()))?;
        let archive_path = entry.path().map_err(|e| CatalogueError::Archive(e.to_string()))?.to_string_lossy().into_owned();
        if archive_path == "package/tags.json" {
            let mut contents = String::new();
            entry.read_to_string(&mut contents).map_err(|e| CatalogueError::Archive(e.to_string()))?;
            let parsed: BTreeMap<String, Vec<String>> = serde_json::from_str(&contents)
                .map_err(|e| CatalogueError::Archive(format!("invalid tags.json: {e}")))?;
            fs::write(root.join("tags.json"), serde_json::to_vec(&parsed).map_err(|e| CatalogueError::Archive(e.to_string()))?)
                .map_err(|e| CatalogueError::Cache(e.to_string()))?;
            tags_written = true;
            continue;
        }
        let Some(filename) = archive_path.strip_prefix("package/icons/") else { continue; };
        if filename.contains('/') || !filename.ends_with(".svg") || filename.is_empty() { continue; }
        let mut svg = String::new();
        entry.read_to_string(&mut svg).map_err(|e| CatalogueError::Archive(e.to_string()))?;
        if !svg.contains("<svg") { continue; }
        let destination: PathBuf = root.join(filename);
        let mut file = fs::File::create(&destination).map_err(|e| CatalogueError::Cache(e.to_string()))?;
        file.write_all(svg.as_bytes()).map_err(|e| CatalogueError::Cache(e.to_string()))?;
        count += 1;
    }
    if count == 0 { return Err(CatalogueError::Empty); }
    if !tags_written { let _ = fs::write(root.join("tags.json"), "{}"); }
    Ok((load_cache(root), latest.version))
}

pub fn search<'a>(icons: &'a [Icon], query: &str) -> Vec<&'a Icon> {
    let terms: Vec<String> = query.split_whitespace().map(|part| part.to_lowercase()).collect();
    if terms.is_empty() { return icons.iter().collect(); }
    icons.iter().filter(|icon| {
        let name = icon.name.to_lowercase();
        let tags = icon.tags.join(" ").to_lowercase();
        terms.iter().all(|term| name.contains(term) || tags.contains(term))
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> Vec<Icon> {
        vec![
            Icon { name: "arrow-down".into(), svg: "<svg/>".into(), tags: vec!["backwards".into(), "reverse".into()] },
            Icon { name: "alarm-clock".into(), svg: "<svg/>".into(), tags: vec!["time".into(), "schedule".into()] },
        ]
    }
    #[test] fn search_is_case_insensitive_and_matches_names() {
        let icons = sample();
        let matches = search(&icons, "ARROW");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "arrow-down");
    }
    #[test] fn search_matches_upstream_tags() {
        let icons = sample();
        let matches = search(&icons, "schedule");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "alarm-clock");
    }
    #[test] fn search_requires_all_tokens() {
        assert_eq!(search(&sample(), "arrow reverse").len(), 1);
        assert!(search(&sample(), "arrow schedule").is_empty());
    }
}
