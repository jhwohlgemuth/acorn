//! Trusted fix providers for host discovery findings.

use super::{
    DiscoveryReference, FindingReference, Fix, FixAction, FixAutomation, FixEffects, FixProvenance, FixReport, FixRisk, FixStatus,
    FIX_ENVELOPE_VERSION,
};
use acorn_core::{
    prelude::{ToString, Vec},
    Noul,
};
use acorn_schema::software::Software;
/// Converts normalized observations into reviewable, non-arbitrary fixes.
pub trait Provider {
    /// Produce deterministic fixes for the supplied references.
    fn provide(&self, references: &[DiscoveryReference]) -> FixReport;
}
/// Built-in provider for ACORN host prerequisite findings.
#[derive(Clone, Copy, Debug, Default)]
pub struct HostProvider;
impl DiscoveryReference {
    fn fix(&self, index: usize) -> Fix {
        let recipe = match Software::from(self.subject.as_str()) {
            | Software::Chromium => Some(("chromium.install", FixEffects::recipe(false))),
            | Software::Node | Software::Npm | Software::Npx if cfg!(target_os = "macos") => Some(("node.homebrew", FixEffects::recipe(true))),
            | Software::Node | Software::Npm | Software::Npx if cfg!(windows) => Some(("node.winget", FixEffects::recipe(true))),
            | Software::Vale => Some(("vale.install", FixEffects::recipe(false))),
            | _ => None,
        };
        let (actions, automation, effects, status, summary) = match recipe {
            | Some((recipe_id, effects)) => (
                vec![FixAction::RegisteredRecipe {
                    recipe_id: recipe_id.to_string(),
                }],
                FixAutomation::ConfirmationRequired,
                effects,
                FixStatus::Proposed,
                format!("Install {}", self.subject),
            ),
            | None => (
                Vec::new(),
                FixAutomation::Manual,
                FixEffects::default(),
                FixStatus::Unsupported,
                format!("Install {} manually", self.subject),
            ),
        };
        let provenance = FixProvenance {
            agent: None,
            backend: "host".to_string(),
            model: None,
            prompt_version: FIX_ENVELOPE_VERSION.to_string(),
            request_id: None,
        };
        Fix::init()
            .actions(actions)
            .automation(automation)
            .confidence(Noul::ONE)
            .effects(effects)
            .finding(FindingReference::Discovery(self.clone()))
            .id(format!("host-fix-{index:04}"))
            .provenance(provenance)
            .rationale(format!("{} is required by an ACORN host workflow", self.subject))
            .risk(FixRisk::Medium)
            .status(status)
            .summary(summary)
            .target(self.subject.clone())
            .build()
    }
}
impl Provider for HostProvider {
    fn provide(&self, references: &[DiscoveryReference]) -> FixReport {
        let fixes: Vec<_> = references
            .iter()
            .filter(|reference| reference.category.is_software() && !reference.status.is_healthy())
            .enumerate()
            .map(|(index, reference)| reference.fix(index.saturating_add(1)))
            .collect();
        FixReport::from(fixes)
    }
}
#[cfg(test)]
mod tests;
