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
    fs::write(root.join("version.txt"), &latest.version)
        .map_err(|e| CatalogueError::Cache(e.to_string()))?;
    Ok((load_cache(root), latest.version))
}

fn edit_distance_at_most_one(a: &str, b: &str) -> bool {
    if a == b { return true; }
    let aa: Vec<char> = a.chars().collect();
    let bb: Vec<char> = b.chars().collect();
    if aa.len().abs_diff(bb.len()) > 1 { return false; }
    let (mut i, mut j, mut edits) = (0usize, 0usize, 0usize);
    while i < aa.len() && j < bb.len() {
        if aa[i] == bb[j] { i += 1; j += 1; }
        else {
            edits += 1;
            if edits > 1 { return false; }
            if aa.len() > bb.len() { i += 1; }
            else if bb.len() > aa.len() { j += 1; }
            else { i += 1; j += 1; }
        }
    }
    if i < aa.len() || j < bb.len() { edits += 1; }
    edits <= 1
}

fn token_score(term: &str, words: &[&str], full: &str) -> Option<i32> {
    if full == term { return Some(100); }
    if words.iter().any(|word| *word == term) { return Some(90); }
    if full.starts_with(term) || words.iter().any(|word| word.starts_with(term)) { return Some(75); }
    // Short queries must be precise: substring/subsequence matching makes "pen"
    // match unrelated tags such as "open", "happen", and "expense".
    if term.chars().count() >= 4 && words.iter().any(|word| word.contains(term)) { return Some(60); }
    if term.chars().count() >= 4 && words.iter().any(|word| edit_distance_at_most_one(term, word)) { return Some(35); }
    // Allow forgiving subsequence matching only for longer, unambiguous queries.
    if term.chars().count() >= 5 && words.iter().any(|word| {
        let mut chars = word.chars();
        term.chars().all(|wanted| chars.by_ref().any(|got| got == wanted))
    }) { return Some(20); }
    None
}

fn field_score(term: &str, words: &[&str], full: &str, name_field: bool) -> i32 {
    let exact = if name_field { 160 } else { 78 };
    let prefix = if name_field { 138 } else { 58 };
    let contains = if name_field { 112 } else { 38 };
    let fuzzy = if name_field { 76 } else { 24 };
    if full == term { return exact + 20; }
    if words.iter().any(|word| *word == term) { return exact; }
    if full.starts_with(term) || words.iter().any(|word| word.starts_with(term)) { return prefix; }
    if term.chars().count() >= 4 && (full.contains(term) || words.iter().any(|word| word.contains(term))) {
        return contains;
    }
    if term.chars().count() >= 4 && words.iter().any(|word| edit_distance_at_most_one(term, word)) {
        return fuzzy;
    }
    0
}

pub fn search<'a>(icons: &'a [Icon], query: &str) -> Vec<&'a Icon> {
    let terms: Vec<String> = query.split_whitespace().map(|part| part.to_lowercase()).collect();
    if terms.is_empty() { return icons.iter().collect(); }
    let mut ranked: Vec<(i32, usize, &'a Icon)> = icons.iter().enumerate().filter_map(|(index, icon)| {
        let name = icon.name.to_lowercase().replace('-', " ");
        let name_words: Vec<&str> = name.split_whitespace().collect();
        let tags = icon.tags.iter().filter(|tag| !tag.starts_with("$group:"))
            .map(|tag| tag.to_lowercase()).collect::<Vec<_>>();
        let tag_words: Vec<&str> = tags.iter().flat_map(|tag| tag.split_whitespace()).collect();
        let tag_text = tags.join(" ");
        let mut total = 0;
        for term in &terms {
            let name_score = field_score(term, &name_words, &name, true);
            let tag_score = field_score(term, &tag_words, &tag_text, false);
            let best = name_score.max(tag_score);
            if best == 0 { return None; }
            total += best;
        }
        Some((total, index, icon))
    }).collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    ranked.into_iter().map(|(_, _, icon)| icon).collect()
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
    #[test] fn short_search_terms_do_not_match_accidental_substrings() {
        let icons = vec![
            Icon { name: "pen".into(), svg: "<svg/>".into(), tags: vec![] },
            Icon { name: "pencil".into(), svg: "<svg/>".into(), tags: vec!["writing".into()] },
            Icon { name: "pen-tool".into(), svg: "<svg/>".into(), tags: vec![] },
            Icon { name: "alarm-clock".into(), svg: "<svg/>".into(), tags: vec!["open".into(), "happen".into()] },
            Icon { name: "wallet".into(), svg: "<svg/>".into(), tags: vec!["expense".into()] },
        ];
        let matches = search(&icons, "PEN");
        let names: Vec<&str> = matches.iter().map(|icon| icon.name.as_str()).collect();
        assert_eq!(names, vec!["pen", "pen-tool", "pencil"]);
    }

    #[test] fn longer_searches_keep_fuzzy_matching() {
        let icons = vec![
            Icon { name: "alarm-clock".into(), svg: "<svg/>".into(), tags: vec![] },
            Icon { name: "calendar".into(), svg: "<svg/>".into(), tags: vec![] },
        ];
        assert_eq!(search(&icons, "alrm")[0].name, "alarm-clock");
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
