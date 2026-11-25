//! Bundled, editable RADS starter projects.
use crate::session::SuppliedFile;
use acorn_core::util::MimeType;
use rust_embed::Embed;
use serde::Deserialize;
use std::sync::LazyLock;

/// Available project starters, loaded from the bundled catalog asset.
pub static TEMPLATES: LazyLock<Vec<Template>> = LazyLock::new(|| {
    let entries: Vec<CatalogEntry> = match serde_json::from_str(include_str!("../assets/templates/catalog.json")) {
        | Ok(entries) => entries,
        | Err(error) => {
            tracing::error!(%error, "Bundled template catalog is invalid");
            Vec::new()
        }
    };
    entries
        .into_iter()
        .filter_map(|entry| {
            let files = load_files(entry.files);
            let yaml_files = load_files(entry.yaml_files);
            if files.is_none() || yaml_files.is_none() {
                tracing::error!(template = entry.id, "Bundled template file is unavailable or is not UTF-8");
            }
            files.zip(yaml_files).map(|(files, yaml_files)| Template {
                id: entry.id,
                title: entry.title,
                description: entry.description,
                includes: entry.includes,
                replace: entry.replace,
                files,
                yaml_files,
            })
        })
        .collect()
});
#[derive(Deserialize)]
struct CatalogEntry {
    id: String,
    title: String,
    description: String,
    includes: String,
    replace: String,
    files: Vec<CatalogFile>,
    yaml_files: Vec<CatalogFile>,
}
#[derive(Deserialize)]
struct CatalogFile {
    path: String,
    asset: String,
}
/// One locally bundled RADS project starter.
pub struct Template {
    /// Stable identifier used in template links.
    pub id: String,
    /// Catalog heading.
    pub title: String,
    /// Short catalog description.
    pub description: String,
    /// Fields included in the starter.
    pub includes: String,
    /// Fictional details to replace before publishing.
    pub replace: String,
    /// Relative paths and UTF-8 content, in display order.
    pub files: Vec<SuppliedFile>,
    /// YAML equivalents of the JSON starter files.
    pub yaml_files: Vec<SuppliedFile>,
}
#[derive(Embed)]
#[folder = "assets/templates/"]
struct TemplateAssets;
impl Template {
    /// Return the bundled files for the selected format.
    pub fn files_for(&self, format: &MimeType) -> &[SuppliedFile] {
        if *format == MimeType::Yaml {
            &self.yaml_files
        } else {
            &self.files
        }
    }
}
fn load_files(files: Vec<CatalogFile>) -> Option<Vec<SuppliedFile>> {
    files
        .into_iter()
        .map(|file| {
            TemplateAssets::get(&file.asset)
                .and_then(|asset| String::from_utf8(asset.data.into_owned()).ok())
                .map(|content| SuppliedFile { path: file.path, content })
        })
        .collect()
}
/// Find a project starter by its stable identifier.
pub fn find(id: &str) -> Option<&'static Template> {
    TEMPLATES.iter().find(|template| template.id == id)
}
