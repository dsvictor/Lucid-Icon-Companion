use flate2::read::GzDecoder;
use serde::Deserialize;
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tar::Archive;

#[derive(Debug, Clone)]
pub struct Icon {
    pub name: String,
    pub svg: String,
}

#[derive(Debug, Deserialize)]
struct NpmLatest {
    version: String,
    dist: NpmDist,
}

#[derive(Debug, Deserialize)]
struct NpmDist {
    tarball: String,
}

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

/// Read the local cache of untouched upstream SVG assets.
pub fn load_cache(root: &Path) -> Vec<Icon> {
    let mut icons = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return icons;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("svg") {
            continue;
        }
        let Ok(svg) = fs::read_to_string(&path) else {
            continue;
        };
        let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        icons.push(Icon {
            name: name.to_owned(),
            svg,
        });
    }

    icons.sort_by(|a, b| a.name.cmp(&b.name));
    icons
}

/// Resolve the latest published lucide-static package, then cache every SVG it ships.
/// The package version is resolved dynamically: no hand-maintained icon subset or version pin.
pub fn download_latest(root: &Path) -> Result<(Vec<Icon>, String), CatalogueError> {
    fs::create_dir_all(root).map_err(|e| CatalogueError::Cache(e.to_string()))?;

    let latest: NpmLatest = ureq::get("https://registry.npmjs.org/lucide-static/latest")
        .set("User-Agent", "Lucide-Icon-Companion/0.1.0")
        .call()
        .map_err(|e| CatalogueError::Metadata(e.to_string()))?
        .into_json()
        .map_err(|e| CatalogueError::Metadata(e.to_string()))?;

    let mut archive_bytes = Vec::new();
    ureq::get(&latest.dist.tarball)
        .set("User-Agent", "Lucide-Icon-Companion/0.1.0")
        .call()
        .map_err(|e| CatalogueError::Download(e.to_string()))?
        .into_reader()
        .read_to_end(&mut archive_bytes)
        .map_err(|e| CatalogueError::Download(e.to_string()))?;

    let decoder = GzDecoder::new(archive_bytes.as_slice());
    let mut archive = Archive::new(decoder);
    let entries = archive
        .entries()
        .map_err(|e| CatalogueError::Archive(e.to_string()))?;
    let mut count = 0usize;

    for item in entries {
        let mut entry = item.map_err(|e| CatalogueError::Archive(e.to_string()))?;
        let archive_path = entry
            .path()
            .map_err(|e| CatalogueError::Archive(e.to_string()))?
            .to_string_lossy()
            .into_owned();
        let Some(filename) = archive_path.strip_prefix("package/icons/") else {
            continue;
        };

        // Only accept flat SVG files directly under package/icons. This both matches
        // lucide-static's asset layout and prevents archive paths escaping the cache.
        if filename.contains('/') || !filename.ends_with(".svg") || filename.is_empty() {
            continue;
        }

        let mut svg = String::new();
        entry
            .read_to_string(&mut svg)
            .map_err(|e| CatalogueError::Archive(e.to_string()))?;
        if !svg.contains("<svg") {
            continue;
        }

        let destination: PathBuf = root.join(filename);
        let mut file = fs::File::create(&destination)
            .map_err(|e| CatalogueError::Cache(e.to_string()))?;
        file.write_all(svg.as_bytes())
            .map_err(|e| CatalogueError::Cache(e.to_string()))?;
        count += 1;
    }

    if count == 0 {
        return Err(CatalogueError::Empty);
    }

    Ok((load_cache(root), latest.version))
}

pub fn search<'a>(icons: &'a [Icon], query: &str) -> Vec<&'a Icon> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return icons.iter().collect();
    }

    icons
        .iter()
        .filter(|icon| icon.name.to_lowercase().contains(&needle))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_is_case_insensitive_and_matches_names() {
        let icons = vec![
            Icon {
                name: "arrow-down".into(),
                svg: "<svg/>".into(),
            },
            Icon {
                name: "alarm-clock".into(),
                svg: "<svg/>".into(),
            },
        ];

        let matches = search(&icons, "ARROW");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "arrow-down");
    }
}
