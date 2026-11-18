//! Deterministic catalog projection of validated research activities in one workspace.
use crate::session::{many, Workspace};
use acorn_schema::pid::{Identifier, PID};
use alloc::collections::{BTreeMap, BTreeSet};
use serde_json::Value;
use strum::IntoEnumIterator;

/// User-selected catalog order.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CatalogSort {
    /// Sort titles alphabetically with stable identity tie breaks.
    #[default]
    Alphabetical,
    /// Sort query matches by title then other field relevance.
    Relevance,
}
/// One validated activity and its stable source identity within a workspace snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Activity {
    /// Source-qualified identity, including the loaded repository revision when present.
    pub id: String,
    /// Relative path of the validated source file.
    pub path: String,
    /// Project identity shared by records with the same metadata identifier.
    pub project_id: String,
    /// Display title.
    pub title: String,
    /// Short description when supplied.
    pub summary: String,
    /// Activity type.
    pub kind: String,
    /// Activity status.
    pub status: String,
    /// Organization names or identifiers associated with the activity.
    pub organizations: Vec<String>,
    /// Contributor names or identifiers associated with the activity.
    pub contributors: Vec<String>,
    /// Normalized persistent identifiers associated with the activity.
    pub identifiers: Vec<String>,
    /// Controlled vocabulary keywords assigned to the activity.
    pub keywords: Vec<String>,
    search_text: String,
}
/// A project groups validated activities by their RAD metadata identifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Project {
    /// Metadata identifier, or a source path for a record without one.
    pub id: String,
    /// Display title selected from the first activity in path order.
    pub title: String,
    /// Optional summary selected from the first activity that supplies one.
    pub summary: String,
    /// Source paths of the project's validated activities.
    pub paths: Vec<String>,
    /// Unique associated organizations.
    pub organizations: Vec<String>,
    /// Unique controlled vocabulary keywords across project activities.
    pub keywords: Vec<String>,
}
/// Derived catalog; no source files or validation results are copied into mutable app state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResearchIndex {
    /// Validated activities sorted by source path.
    pub activities: Vec<Activity>,
    /// Projects sorted by title and identity.
    pub projects: Vec<Project>,
}
impl Activity {
    fn from_value(workspace: &Workspace, path: &str, value: &Value) -> Self {
        let meta = &value["meta"];
        let title = string(&value["title"]).unwrap_or(path).to_string();
        let summary = string(&value["subtitle"])
            .or_else(|| value.get("sections").and_then(|sections| sections.get("mission")).and_then(string))
            .unwrap_or_default()
            .to_string();
        let organizations = entities(&meta["organizations"]);
        let contributors = entities(&meta["contributors"]);
        let keywords = many(&meta["keywords"])
            .into_iter()
            .filter_map(string)
            .map(str::to_owned)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let identifiers = PID::iter()
            .filter(PID::supports_discovery)
            .flat_map(|kind| many(&meta[kind.as_str()]))
            .filter_map(string)
            .chain([&meta["organizations"], &meta["contributors"]].into_iter().flat_map(|values| {
                many(values)
                    .into_iter()
                    .flat_map(|entity| many(&entity["identifiers"]))
                    .filter_map(|identifier| string(&identifier["value"]))
                    .collect::<Vec<_>>()
            }))
            .map(|value| {
                Identifier::new(value)
                    .normalized()
                    .map_or_else(|| value.trim().to_lowercase(), |identifier| identifier.identity_key())
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let project_id = string(&meta["identifier"]).map(str::to_lowercase).unwrap_or_else(|| path.to_lowercase());
        let source = workspace
            .source
            .as_ref()
            .map_or(String::new(), |source| format!("{}:{}:{}:", source.host, source.project, source.revision));
        let kind = string(&value["@type"]).unwrap_or("Research activity").to_string();
        let status = string(&meta["status"]).unwrap_or("Not provided").to_string();
        let search_text = format!(
            "{title} {summary} {path} {project_id} {kind} {status} {} {} {} {} {}",
            organizations.join(" "),
            contributors.join(" "),
            identifiers.join(" "),
            meta["keywords"],
            value["contact"]
        );
        let text = format!("{} {}", search_text.to_lowercase(), value["sections"].to_string().to_lowercase());
        Self {
            id: format!("{source}{path}"),
            path: path.to_string(),
            search_text: text,
            project_id,
            title,
            summary,
            kind,
            status,
            organizations,
            contributors,
            identifiers,
            keywords,
        }
    }
    fn relevance(&self, terms: &[String]) -> usize {
        let title = self.title.to_lowercase();
        let score = |term| {
            if title.starts_with(term) {
                3
            } else if title.contains(term) {
                2
            } else {
                1
            }
        };
        terms.iter().map(score).sum()
    }
}
impl From<&Workspace> for ResearchIndex {
    /// Derive the catalog from validated records in the current bounded workspace.
    fn from(workspace: &Workspace) -> Self {
        let activities = workspace
            .activities()
            .into_iter()
            .map(|(file, value)| Activity::from_value(workspace, &file.path, value))
            .collect::<Vec<_>>();
        let mut grouped = activities.iter().fold(BTreeMap::<String, Vec<&Activity>>::new(), |mut groups, activity| {
            groups.entry(activity.project_id.clone()).or_default().push(activity);
            groups
        });
        let mut projects = grouped
            .iter_mut()
            .filter_map(|(id, members)| {
                members.sort_by(|left, right| left.path.cmp(&right.path));
                members.first().map(|first| Project {
                    id: id.clone(),
                    title: first.title.clone(),
                    summary: members
                        .iter()
                        .find(|activity| !activity.summary.is_empty())
                        .map_or(String::new(), |activity| activity.summary.clone()),
                    paths: members.iter().map(|activity| activity.path.clone()).collect(),
                    organizations: members
                        .iter()
                        .flat_map(|activity| activity.organizations.iter().cloned())
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect(),
                    keywords: members
                        .iter()
                        .flat_map(|activity| activity.keywords.iter().cloned())
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect(),
                })
            })
            .collect::<Vec<_>>();
        projects.sort_by(|left, right| left.title.to_lowercase().cmp(&right.title.to_lowercase()).then(left.id.cmp(&right.id)));
        Self { activities, projects }
    }
}
impl From<Workspace> for ResearchIndex {
    fn from(workspace: Workspace) -> Self {
        Self::from(&workspace)
    }
}
impl ResearchIndex {
    /// Return other activities sharing an organization, contributor, or identifier.
    pub fn related(&self, path: &str) -> Vec<&Activity> {
        self.activities
            .iter()
            .find(|activity| activity.path == path)
            .map_or_else(Vec::new, |selected| {
                self.activities
                    .iter()
                    .filter(|activity| activity.path != path)
                    .filter(|activity| {
                        activity.project_id == selected.project_id
                            || activity.organizations.iter().any(|name| selected.organizations.contains(name))
                            || activity.contributors.iter().any(|name| selected.contributors.contains(name))
                            || activity.identifiers.iter().any(|id| selected.identifiers.contains(id))
                    })
                    .collect()
            })
    }
    /// Return matching activities with stable alphabetical or relevance ordering.
    pub fn search<'a>(&'a self, query: &str, sort: CatalogSort) -> Vec<&'a Activity> {
        let terms = query.split_whitespace().map(str::to_lowercase).collect::<Vec<_>>();
        let mut matches = self
            .activities
            .iter()
            .filter(|activity| terms.iter().all(|term| activity.search_text.contains(term)))
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| {
            let alphabetical = left.title.to_lowercase().cmp(&right.title.to_lowercase()).then(left.id.cmp(&right.id));
            match sort {
                | CatalogSort::Alphabetical => alphabetical,
                | CatalogSort::Relevance => right.relevance(&terms).cmp(&left.relevance(&terms)).then(alphabetical),
            }
        });
        matches
    }
}
fn entities(value: &Value) -> Vec<String> {
    many(value)
        .into_iter()
        .filter_map(|entity| {
            string(entity)
                .or_else(|| string(&entity["name"]))
                .or_else(|| string(&entity["identifier"]))
        })
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
fn string(value: &Value) -> Option<&str> {
    value.as_str().filter(|value| !value.trim().is_empty())
}
