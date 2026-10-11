use super::{
    format, ApiResult, CheckReference, DiscoveryReference, Error, FindingReference, Fix, FixAction, FixAutomation, FixEffects, FixStatus, SafePath,
    String, Vec,
};
use acorn_core::util::constants::app::APPLICATION;
#[derive(Clone, Debug)]
pub(super) enum ExecutableFix {
    Document {
        expected: String,
        references: Vec<DiscoveryReference>,
        replacement: String,
        target: SafePath,
    },
    Recipe {
        recipe: Recipe,
        references: Vec<DiscoveryReference>,
    },
    VocabularyDocument {
        references: Vec<CheckReference>,
        target: SafePath,
    },
}
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(super) enum PlanError {
    #[error("Fix requires interactive approval")]
    ApprovalRequired,
    #[error("Fixes contain conflicting document changes")]
    ConflictingDocumentChange,
    #[error("Fix requires network access while offline")]
    Offline,
    #[error("Fix effects do not match its registered remediation")]
    RecipeEffectsMismatch,
    #[error("Fix is not a registered deterministic remediation")]
    Unsupported,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, strum::EnumIs)]
pub(super) enum Recipe {
    Chromium,
    HomebrewNode,
    Unsupported,
    Vale,
    WinGetNode,
}
/// Immutable trusted actions resolved from deterministic proposals.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Plan {
    pub(super) actions: Vec<ExecutableFix>,
    preview: Vec<String>,
}
/// Policy used to resolve reviewable proposals into trusted plans.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Policy {
    approved: bool,
    interactive: bool,
    offline: bool,
}
impl Eq for ExecutableFix {}
impl ExecutableFix {
    fn conflicts_with(&self, other: &Self) -> bool {
        match (self, other) {
            | (Self::Document { target: left, .. }, Self::Document { target: right, .. }) => left == right && self != other,
            | (Self::Document { target: left, .. }, Self::VocabularyDocument { target: right, .. })
            | (Self::VocabularyDocument { target: left, .. }, Self::Document { target: right, .. }) => left == right,
            | _ => false,
        }
    }
    fn extend_references(&mut self, other: &Self) {
        match (self, other) {
            | (Self::Document { references, .. }, Self::Document { references: additions, .. })
            | (Self::Recipe { references, .. }, Self::Recipe { references: additions, .. }) => additions.iter().for_each(|reference| {
                if !references.iter().any(|existing| existing.fingerprint == reference.fingerprint) {
                    references.push(reference.clone());
                }
            }),
            | (Self::VocabularyDocument { references, .. }, Self::VocabularyDocument { references: additions, .. }) => {
                additions.iter().for_each(|reference| {
                    if !references.iter().any(|existing| existing.fingerprint == reference.fingerprint) {
                        references.push(reference.clone());
                    }
                });
            }
            | _ => {}
        }
    }
    pub(super) fn preview(&self) -> String {
        match self {
            | Self::Document { target, .. } | Self::VocabularyDocument { target, .. } => {
                format!("write {}", target.as_path().display())
            }
            | Self::Recipe { recipe, .. } => format!("run {recipe}"),
        }
    }
    pub(super) fn references(&self) -> &[DiscoveryReference] {
        match self {
            | Self::Document { references, .. } | Self::Recipe { references, .. } => references,
            | Self::VocabularyDocument { .. } => &[],
        }
    }
    fn effects(&self) -> FixEffects {
        match self {
            | Self::Document { .. } | Self::VocabularyDocument { .. } => FixEffects {
                filesystem_write: true,
                reversible: true,
                ..FixEffects::default()
            },
            | Self::Recipe { recipe, .. } => recipe.effects(),
        }
    }
}
impl PartialEq for ExecutableFix {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            | (
                Self::Document {
                    expected: left_expected,
                    replacement: left_replacement,
                    target: left_target,
                    ..
                },
                Self::Document {
                    expected: right_expected,
                    replacement: right_replacement,
                    target: right_target,
                    ..
                },
            ) => left_expected == right_expected && left_replacement == right_replacement && left_target == right_target,
            | (Self::Recipe { recipe: left, .. }, Self::Recipe { recipe: right, .. }) => left == right,
            | (Self::VocabularyDocument { target: left, .. }, Self::VocabularyDocument { target: right, .. }) => left == right,
            | _ => false,
        }
    }
}
impl Fix {
    fn resolve(&self, policy: Policy) -> ApiResult<Vec<ExecutableFix>> {
        let references = match &self.finding {
            | super::FindingReference::Discovery(reference) => vec![reference.clone()],
            | super::FindingReference::Check(_) => Vec::new(),
        };
        policy.allows(self).and_then(|()| match &self.finding {
            | FindingReference::Check(reference) if reference.vocabulary_advisory.is_some() => {
                self.resolve_vocabulary(reference).map(|action| vec![action]).map_err(Into::into)
            }
            | _ => self
                .actions
                .iter()
                .map(|action| action.resolve(&self.target, &references))
                .collect::<Result<Vec<_>, PlanError>>()
                .and_then(|actions| {
                    let effects = actions.iter().map(ExecutableFix::effects).reduce(FixEffects::combine).unwrap_or_default();
                    let requires_network = actions
                        .iter()
                        .any(|action| matches!(action, ExecutableFix::Recipe { recipe, .. } if !recipe.is_unsupported()));
                    match (effects == self.effects, requires_network && policy.offline) {
                        | (false, _) => Err(PlanError::RecipeEffectsMismatch),
                        | (_, true) => Err(PlanError::Offline),
                        | (_, false) => Ok(actions),
                    }
                })
                .map_err(color_eyre::eyre::Report::from),
        })
    }
    fn resolve_vocabulary(&self, reference: &CheckReference) -> Result<ExecutableFix, PlanError> {
        let action = match self.actions.as_slice() {
            | [FixAction::DocumentChange { expected, replacement, .. }] => reference
                .vocabulary_advisory
                .as_deref()
                .filter(|advisory| {
                    let same_change =
                        (advisory.original.as_str(), advisory.replacement.as_deref()) == (expected.as_str(), Some(replacement.as_str()));
                    advisory.complete && advisory.outcome.is_accepted() && same_change
                })
                .map(|_| ()),
            | _ => None,
        };
        action
            .ok_or(PlanError::Unsupported)
            .and_then(|()| SafePath::new(&self.target).map_err(|_| PlanError::Unsupported))
            .map(|target| ExecutableFix::VocabularyDocument {
                references: vec![reference.clone()],
                target,
            })
    }
}
impl FixAction {
    fn resolve(&self, target: &str, references: &[DiscoveryReference]) -> Result<ExecutableFix, PlanError> {
        match self {
            | Self::DocumentChange { expected, replacement, .. } => SafePath::new(target)
                .map(|target| ExecutableFix::Document {
                    expected: expected.clone(),
                    references: references.to_vec(),
                    replacement: replacement.clone(),
                    target,
                })
                .map_err(|_| PlanError::Unsupported),
            | Self::RegisteredRecipe { recipe_id } => match Recipe::from(recipe_id.as_str()) {
                | Recipe::Unsupported => Err(PlanError::Unsupported),
                | recipe => Ok(ExecutableFix::Recipe {
                    recipe,
                    references: references.to_vec(),
                }),
            },
            | Self::Manual { .. } => Err(PlanError::Unsupported),
        }
    }
}
impl Plan {
    /// Resolve proposal-only fixes under `policy` without changing the host.
    pub fn from_fixes(fixes: &[Fix], policy: Policy) -> ApiResult<Self> {
        fixes
            .iter()
            .filter(|fix| fix.status == FixStatus::Proposed)
            .try_fold(Vec::new(), |actions, fix| {
                fix.resolve(policy).map(|action| actions.into_iter().chain(action).collect())
            })
            .and_then(coalesce)
            .map(|actions| Self {
                preview: actions.iter().map(ExecutableFix::preview).collect(),
                actions,
            })
    }
    /// Return the exact operations that would run if this plan is applied.
    pub fn preview(&self) -> &[String] {
        &self.preview
    }
    /// Return all discovery references retained by planned actions.
    pub fn references(&self) -> Vec<&DiscoveryReference> {
        self.actions.iter().flat_map(ExecutableFix::references).collect()
    }
}
impl Policy {
    /// Permit only reversible, low-risk automatic document changes.
    pub fn safe(offline: bool) -> Self {
        Self { offline, ..Self::default() }
    }
    /// Permit an interactively confirmed remediation.
    pub fn confirmed(offline: bool) -> Self {
        Self {
            approved: true,
            interactive: true,
            offline,
        }
    }
}
impl Policy {
    fn allows(self, fix: &Fix) -> ApiResult<()> {
        let interactive = self.interactive && self.approved;
        let safe_effects = FixEffects {
            filesystem_write: true,
            reversible: true,
            ..FixEffects::default()
        };
        let safe_document = fix.automation == FixAutomation::SafeAutomatic && fix.risk.is_low() && fix.effects == safe_effects;
        let requires_network = fix.effects.network || fix.effects.package_manager;
        match (fix.automation, safe_document, interactive, requires_network && self.offline) {
            | (_, _, _, true) => Err(PlanError::Offline.into()),
            | (FixAutomation::SafeAutomatic, true, _, false) | (_, _, true, false) => Ok(()),
            | _ => Err(PlanError::ApprovalRequired.into()),
        }
    }
}
impl core::fmt::Display for Recipe {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            | Self::Chromium => formatter.write_str("chromium.install"),
            | Self::HomebrewNode => formatter.write_str("node.homebrew"),
            | Self::Unsupported => formatter.write_str("unsupported"),
            | Self::Vale => formatter.write_str("vale.install"),
            | Self::WinGetNode => formatter.write_str("node.winget"),
        }
    }
}
impl From<&str> for Recipe {
    fn from(value: &str) -> Self {
        match value {
            | "chromium.install" => Self::Chromium,
            | "node.homebrew" if cfg!(target_os = "macos") => Self::HomebrewNode,
            | "node.winget" if cfg!(windows) => Self::WinGetNode,
            | "vale.install" => Self::Vale,
            | _ => Self::Unsupported,
        }
    }
}
impl Recipe {
    pub(super) fn command(self) -> (&'static str, &'static [&'static str]) {
        match self {
            | Self::Chromium => (APPLICATION, &["download", "chromium"]),
            | Self::HomebrewNode => ("brew", &["install", "node"]),
            | Self::Unsupported => ("", &[]),
            | Self::Vale => (APPLICATION, &["download", "vale"]),
            | Self::WinGetNode => (
                "winget",
                &[
                    "install",
                    "--id",
                    "OpenJS.NodeJS.LTS",
                    "--exact",
                    "--accept-package-agreements",
                    "--accept-source-agreements",
                ],
            ),
        }
    }
    fn effects(self) -> FixEffects {
        match self {
            | Self::Chromium | Self::Vale => FixEffects {
                filesystem_write: true,
                network: true,
                process: true,
                ..FixEffects::default()
            },
            | Self::HomebrewNode | Self::WinGetNode => FixEffects {
                filesystem_write: true,
                network: true,
                package_manager: true,
                process: true,
                ..FixEffects::default()
            },
            | Self::Unsupported => FixEffects::default(),
        }
    }
}
fn coalesce(actions: Vec<ExecutableFix>) -> ApiResult<Vec<ExecutableFix>> {
    actions.into_iter().try_fold(Vec::<ExecutableFix>::new(), |mut resolved, action| {
        match resolved.iter_mut().find(|existing| **existing == action) {
            | Some(existing) => {
                existing.extend_references(&action);
                Ok(resolved)
            }
            | None => match resolved.iter().any(|existing| existing.conflicts_with(&action)) {
                | true => Err(PlanError::ConflictingDocumentChange.into()),
                | false => Ok(resolved.into_iter().chain([action]).collect()),
            },
        }
    })
}
