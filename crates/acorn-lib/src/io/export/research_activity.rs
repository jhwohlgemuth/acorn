//! Renderer-independent research activity values for export.
use crate::io::ApiResult;
use acorn_core::util::interpolation::ReplacementPlan;
use acorn_schema::{research_activity::ResearchActivity, ContactPoint, Other};
use color_eyre::eyre::eyre;
use core::marker::PhantomData;

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
pub struct ExportContext<T = ResearchActivity> {
    /// Contact details needed by the HTML contact and branding adapters.
    pub contact: ContactPoint,
    /// Image selected for the artifact.
    pub image_source: ImageSource,
    /// Research activity identifier used by the QR adapter.
    pub identifier: String,
    /// Raw named scalar and list values.
    pub replacements: ReplacementPlan,
    marker: PhantomData<T>,
}
impl ExportContext<ResearchActivity> {
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
            (
                "achievement".to_string(),
                sections.and_then(|sections| sections.achievement.clone()).unwrap_or_default().into(),
            ),
            (
                "areas".to_string(),
                sections.map(|sections| sections.research.areas.clone()).unwrap_or_default().into(),
            ),
            ("caption".to_string(), activity.meta.clone().first_image_caption().into()),
            (
                "challenge".to_string(),
                sections.map(|sections| sections.challenge.clone()).unwrap_or_default().into(),
            ),
            (
                "citation".to_string(),
                activity
                    .meta
                    .doi
                    .as_ref()
                    .and_then(|values| values.first().cloned())
                    .unwrap_or_default()
                    .into(),
            ),
            ("email".to_string(), activity.contact.email.clone().into()),
            ("first".to_string(), activity.contact.given_name.clone().into()),
            (
                "focus".to_string(),
                sections.map(|sections| sections.research.focus.clone()).unwrap_or_default().into(),
            ),
            (
                "impact".to_string(),
                sections.map(|sections| sections.impact.clone()).unwrap_or_default().into(),
            ),
            ("last".to_string(), activity.contact.family_name.clone().into()),
            ("managers".to_string(), managers.into()),
            (
                "mission".to_string(),
                sections.map(|sections| sections.mission.clone()).unwrap_or_default().into(),
            ),
            ("notes".to_string(), presentation_notes.into()),
            (
                "partners".to_string(),
                activity.meta.partners.as_ref().map(|values| values.join(", ")).unwrap_or_default().into(),
            ),
            (
                "portability".to_string(),
                activity
                    .aspect
                    .as_ref()
                    .and_then(|aspect| aspect.portability.as_ref())
                    .cloned()
                    .unwrap_or_default()
                    .to_string()
                    .into(),
            ),
            ("programs".to_string(), programs.into()),
            ("subtitle".to_string(), activity.subtitle.clone().unwrap_or_default().into()),
            (
                "technical".to_string(),
                sections.map(|sections| sections.approach.clone()).unwrap_or_default().into(),
            ),
            ("title".to_string(), activity.title.clone().into()),
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
            marker: PhantomData,
        }
    }
    /// Project a research activity without reading media or rendering markup
    pub fn project(activity: &ResearchActivity) -> ApiResult<Self> {
        (activity.meta.draft || activity.sections.is_some())
            .then(|| Self::from_activity(activity))
            .ok_or_else(|| eyre!("Non-draft research activity requires sections for export"))
    }
}
