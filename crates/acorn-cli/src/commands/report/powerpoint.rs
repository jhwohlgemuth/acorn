//! Category slide inputs for project logbook presentations.
use acorn::io::ApiResult;
use acorn_schema::research_activity::logbook::MAX_LOGBOOK_SUMMARY_CHARACTERS;
use acorn_schema::research_activity::{LogbookCategory, LogbookUpdates, ResearchActivity};
use color_eyre::eyre::eyre;
use strum::IntoEnumIterator;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CategorySlide {
    pub category: LogbookCategory,
    pub project_id: String,
    pub title: String,
    pub subtitle: String,
    pub achievement: String,
    pub notes: String,
    pub keywords: Vec<String>,
    pub principal_investigators: Vec<String>,
    pub sponsors: Vec<String>,
    pub image_url: Option<String>,
}
pub(super) fn compose(activity: &ResearchActivity, updates: &LogbookUpdates) -> ApiResult<Vec<CategorySlide>> {
    LogbookCategory::iter()
        .filter_map(|category| {
            let entries = updates
                .entries
                .iter()
                .filter(|entry| !entry.global && entry.project_id == updates.identifier && entry.category == category)
                .collect::<Vec<_>>();
            (!entries.is_empty()).then_some((category, entries))
        })
        .map(|(category, entries)| {
            let oversized = entries
                .iter()
                .find(|entry| entry.summary.chars().count() > MAX_LOGBOOK_SUMMARY_CHARACTERS);
            let achievement = entries
                .iter()
                .map(|entry| {
                    let items = entry.items.as_ref().map_or(String::new(), |items| {
                        items.iter().map(|item| format!("\n  • {item}")).collect::<String>()
                    });
                    format!("{} — {}{}", entry.timestamp, entry.summary, items)
                })
                .collect::<Vec<_>>()
                .join("\n\n");
            let window = updates.through.as_deref().unwrap_or("unspecified");
            match oversized {
                | Some(entry) => Err(eyre!(
                    "PowerPoint source '{}' for project '{}' category '{category:?}' exceeds the {MAX_LOGBOOK_SUMMARY_CHARACTERS}-character logbook summary limit",
                    entry.identifier,
                    updates.identifier
                )),
                | None => Ok(CategorySlide {
                    category,
                    project_id: updates.identifier.clone(),
                    title: updates.title.clone(),
                    subtitle: format!("{category:?} | {} to {window}", updates.since),
                    achievement,
                    notes: entries.iter().map(|entry| entry.identifier.as_str()).collect::<Vec<_>>().join(", "),
                    keywords: updates.keywords.clone(),
                    principal_investigators: updates.principal_investigators.clone(),
                    sponsors: updates.sponsors.clone(),
                    image_url: activity.authored_image_url(),
                }),
            }
        })
        .collect()
}
