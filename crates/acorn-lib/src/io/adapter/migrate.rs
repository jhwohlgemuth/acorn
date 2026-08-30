//! Schema migration support.
use crate::analyzer::Standard;
use crate::error::ApiResult;
use crate::io::{jsonc_parse_value, read_file, FromPath};
use crate::prelude::std::PathBuf;
use crate::schema::research_activity::ResearchActivity;
use crate::util::{Label, MimeType};
use color_eyre::eyre::eyre;
use owo_colors::OwoColorize;
use tracing::warn;

/// A deprecated schema field and its canonical replacement.
pub type FieldMigration = (&'static str, &'static str);
/// Migration behavior for schemas that accept deprecated or versioned input.
pub trait Migration: super::InputOutput {
    /// Schema standard used in migration diagnostics.
    const STANDARD: Standard;
    /// Return migrations required by a serialized schema value.
    fn field_migrations(value: &serde_json::Value) -> Vec<FieldMigration>;
    /// Rewrite a schema file when deprecated fields require migration.
    fn migrate(path: impl Into<PathBuf>) -> ApiResult<bool> {
        let source = path.into();
        Self::migrations_from_path(&source).and_then(|migrations| match migrations.is_empty() {
            | true => Ok(false),
            | false => Self::read(source.clone()).and_then(|value| value.write(source).map(|()| true)),
        })
    }
    /// Return migrations required by a schema file.
    fn migrations_from_path(source: &PathBuf) -> ApiResult<Vec<FieldMigration>> {
        read_file(source.clone()).map(|content| {
            let value = match MimeType::from_path(source) {
                | MimeType::Json => serde_json::from_str(&content).ok(),
                | MimeType::Jsonc => jsonc_parse_value(&content).ok(),
                | MimeType::Yaml => serde_norway::from_str(&content).ok(),
                | _ => None,
            };
            value.as_ref().map(Self::field_migrations).unwrap_or_default()
        })
    }
    /// Warn about deprecated fields present in a serialized schema value.
    fn print_migration_warnings(value: &serde_json::Value) {
        Self::field_migrations(value).into_iter().for_each(|(deprecated, replacement)| {
            warn!(
                "=> {}Deprecated {} field {}; use {}",
                Label::CAUTION,
                Self::STANDARD,
                deprecated.red(),
                replacement.green()
            );
        });
    }
    /// Read a schema file while rejecting fields that require migration.
    fn read_strict(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let source = path.into();
        Self::migrations_from_path(&source).and_then(|migrations| match migrations.first() {
            | Some((deprecated, replacement)) => Err(eyre!(
                "Deprecated {} field `{deprecated}` is not allowed in strict mode; use `{replacement}`",
                Self::STANDARD
            )),
            | None => Self::read(source),
        })
    }
}
impl Migration for ResearchActivity {
    const STANDARD: Standard = Standard::ResearchActivityData;
    fn field_migrations(value: &serde_json::Value) -> Vec<FieldMigration> {
        let media_contains = |field| {
            value
                .pointer("/meta/media")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|media| media.iter().any(|item| item.get(field).is_some()))
        };
        [
            media_contains("href").then_some(("meta.media[].href", "meta.media[].contentUrl")),
            media_contains("url").then_some(("meta.media[].url", "meta.media[].contentUrl")),
            value.pointer("/contact/profile").is_some().then_some(("contact.profile", "contact.url")),
            value
                .pointer("/sections/outcomes")
                .is_some()
                .then_some(("sections.outcomes", "sections.impact")),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}
