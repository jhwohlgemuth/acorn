//! Research activity schema migrations.
use super::ResearchActivity;
use crate::analyzer::Standard;
use crate::schema::migrate::{FieldMigration, Migration};

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
