use super::{format, ApiResult, Error, Fix, FixAction, FixAutomation, FixEffects, FixStatus, SafePath, String, Vec};

/// Policy used to resolve reviewable proposals into trusted plans.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FixPolicy {
    approved: bool,
    interactive: bool,
    offline: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ExecutableFix {
    Document {
        expected: String,
        replacement: String,
        target: SafePath,
    },
    Recipe(Recipe),
}
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(super) enum PlanError {
    #[error("Fix requires interactive approval")]
    ApprovalRequired,
    #[error("Fixes contain conflicting document changes")]
    ConflictingDocumentChange,
    #[error("Fix requires network access while offline")]
    Offline,
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
pub struct FixPlan {
    pub(super) actions: Vec<ExecutableFix>,
    preview: Vec<String>,
}
impl ExecutableFix {
    fn conflicts_with(&self, other: &Self) -> bool {
        match (self, other) {
            | (Self::Document { target: left, .. }, Self::Document { target: right, .. }) => left == right && self != other,
            | _ => false,
        }
    }
    pub(super) fn preview(&self) -> String {
        match self {
            | Self::Document { target, .. } => format!("write {}", target.as_path().display()),
            | Self::Recipe(recipe) => format!("run {recipe}"),
        }
    }
}
impl Fix {
    fn resolve(&self, policy: FixPolicy) -> ApiResult<Vec<ExecutableFix>> {
        policy.allows(self).and_then(|()| {
            self.actions
                .iter()
                .map(|action| action.resolve(&self.target))
                .collect::<Result<Vec<_>, PlanError>>()
                .and_then(|actions| {
                    let requires_network = actions
                        .iter()
                        .any(|action| matches!(action, ExecutableFix::Recipe(recipe) if !recipe.is_unsupported()));
                    match requires_network && policy.offline {
                        | true => Err(PlanError::Offline),
                        | false => Ok(actions),
                    }
                })
                .map_err(color_eyre::eyre::Report::from)
        })
    }
}
impl FixAction {
    fn resolve(&self, target: &str) -> Result<ExecutableFix, PlanError> {
        match self {
            | Self::DocumentChange { expected, replacement, .. } => SafePath::new(target)
                .map(|target| ExecutableFix::Document {
                    expected: expected.clone(),
                    replacement: replacement.clone(),
                    target,
                })
                .map_err(|_| PlanError::Unsupported),
            | Self::RegisteredRecipe { recipe_id } => match Recipe::from(recipe_id.as_str()) {
                | Recipe::Unsupported => Err(PlanError::Unsupported),
                | recipe => Ok(ExecutableFix::Recipe(recipe)),
            },
            | Self::Manual { .. } => Err(PlanError::Unsupported),
        }
    }
}
impl FixPlan {
    /// Resolve proposal-only fixes under `policy` without changing the host.
    pub fn from_fixes(fixes: &[Fix], policy: FixPolicy) -> ApiResult<Self> {
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
}
impl FixPolicy {
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
impl FixPolicy {
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
impl Recipe {
    pub(super) fn command(self) -> (&'static str, &'static [&'static str]) {
        match self {
            | Self::Chromium => ("acorn", &["download", "chromium"]),
            | Self::HomebrewNode => ("brew", &["install", "node"]),
            | Self::Unsupported => ("", &[]),
            | Self::Vale => ("acorn", &["download", "vale"]),
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
fn coalesce(actions: Vec<ExecutableFix>) -> ApiResult<Vec<ExecutableFix>> {
    actions.into_iter().try_fold(Vec::new(), |resolved, action| {
        let duplicate = resolved.contains(&action);
        let conflict = resolved.iter().any(|existing| existing.conflicts_with(&action));
        match (duplicate, conflict) {
            | (true, _) => Ok(resolved),
            | (false, true) => Err(PlanError::ConflictingDocumentChange.into()),
            | (false, false) => Ok(resolved.into_iter().chain([action]).collect()),
        }
    })
}
