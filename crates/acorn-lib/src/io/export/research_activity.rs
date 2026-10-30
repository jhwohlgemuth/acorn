//! Renderer-independent research activity values for export.
use crate::io::ApiResult;
use acorn_core::util::interpolation::{ReplacementPlan, ReplacementValue};
use acorn_schema::{research_activity::ResearchActivity, ContactPoint, Other};
use color_eyre::eyre::eyre;

/// The image selected by research activity metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImageSource {
    /// An authored image URL, resolved by the host renderer.
    Authored(String),
    /// The embedded draft placeholder image.
    DraftFallback,
}
/// Raw research activity values shared by export renderers.
#[derive(Clone, Debug)]
pub struct ExportContext {
    /// Contact details needed by the HTML contact and branding adapters.
    pub contact: ContactPoint,
    /// Image selected for the artifact.
    pub image_source: ImageSource,
    /// Research activity identifier used by the QR adapter.
    pub identifier: String,
    /// Raw named scalar and list values.
    pub replacements: ReplacementPlan,
}
impl ExportContext {
    fn from_activity(activity: &ResearchActivity) -> Self {
        let sections = activity.sections.as_ref();
        let formatted_notes = match activity.notes.as_ref() {
            | Some(Other::Formatted(notes)) => Some(notes),
            | _ => None,
        };
        let presentation_notes = match activity.notes.as_ref() {
            | Some(Other::Formatted(notes)) => notes.presentation.clone().unwrap_or_default(),
            | Some(Other::Unformatted(notes)) => notes.clone(),
            | None => String::new(),
        };
        let managers = formatted_notes
            .and_then(|notes| notes.managers.as_ref())
            .map(|values| values.join(" and "))
            .unwrap_or_default();
        let programs = formatted_notes
            .and_then(|notes| notes.programs.as_ref())
            .map(|values| values.join(" and "))
            .unwrap_or_default();
        let replacements = ReplacementPlan::new([
            list(
                "achievement",
                sections.and_then(|sections| sections.achievement.clone()).unwrap_or_default(),
            ),
            list("areas", sections.map(|sections| sections.research.areas.clone()).unwrap_or_default()),
            scalar("caption", activity.meta.clone().first_image_caption()),
            scalar("challenge", sections.map(|sections| sections.challenge.clone()).unwrap_or_default()),
            scalar(
                "citation",
                activity.meta.doi.as_ref().and_then(|values| values.first().cloned()).unwrap_or_default(),
            ),
            scalar("email", activity.contact.email.clone()),
            scalar("first", activity.contact.given_name.clone()),
            scalar("focus", sections.map(|sections| sections.research.focus.clone()).unwrap_or_default()),
            list("impact", sections.map(|sections| sections.impact.clone()).unwrap_or_default()),
            scalar("last", activity.contact.family_name.clone()),
            scalar("managers", managers),
            scalar("mission", sections.map(|sections| sections.mission.clone()).unwrap_or_default()),
            scalar("notes", presentation_notes),
            scalar(
                "partners",
                activity.meta.partners.as_ref().map(|values| values.join(", ")).unwrap_or_default(),
            ),
            scalar(
                "portability",
                activity
                    .aspect
                    .as_ref()
                    .and_then(|aspect| aspect.portability.as_ref())
                    .cloned()
                    .unwrap_or_default()
                    .to_string(),
            ),
            scalar("programs", programs),
            scalar("subtitle", activity.subtitle.clone().unwrap_or_default()),
            list("technical", sections.map(|sections| sections.approach.clone()).unwrap_or_default()),
            scalar("title", activity.title.clone()),
        ]);
        let image_source = activity
            .authored_image_url()
            .map(ImageSource::Authored)
            .unwrap_or(ImageSource::DraftFallback);
        Self {
            contact: activity.contact.clone(),
            image_source,
            identifier: activity.meta.identifier.clone(),
            replacements,
        }
    }
    /// Project a research activity without reading media or rendering markup
    pub fn project(activity: &ResearchActivity) -> ApiResult<Self> {
        (activity.meta.draft || activity.sections.is_some())
            .then(|| Self::from_activity(activity))
            .ok_or_else(|| eyre!("Non-draft research activity requires sections for export"))
    }
}
fn list(name: &str, values: Vec<String>) -> (String, ReplacementValue) {
    (name.to_string(), ReplacementValue::List(values))
}
fn scalar(name: &str, value: String) -> (String, ReplacementValue) {
    (name.to_string(), ReplacementValue::Scalar(value))
}
