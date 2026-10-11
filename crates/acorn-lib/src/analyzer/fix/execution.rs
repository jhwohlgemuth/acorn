use super::{
    plan::{ExecutableFix, Plan, Recipe},
    ApiResult, CheckReference, DiscoveryReference, SafePath, String, Vec,
};
use crate::io::{
    read_file,
    vocabulary::write_vocabulary_advisory_audit,
    workflow::{Artifact, Artifacts},
    FromPath, InputOutput,
};
#[cfg(feature = "analysis")]
use acorn_cmd::CommandSpec;
use acorn_core::util::{constants::app::APPLICATION, stable_deduplicate, MimeType};
use acorn_schema::{
    research_activity::ResearchActivity,
    validation::Validate,
    vocabulary::{VocabularyKind, VocabularyOptions, VocabularyResolver},
    OneOrMany,
};
use color_eyre::eyre::eyre;
#[cfg(feature = "analysis")]
use core::time::Duration;
use std::{env::current_exe, path::Path};
/// Executes a command from ACORN's fixed recipe registry.
pub trait Runner {
    /// Run one fixed binary and argument vector, returning redacted output.
    fn run(&self, binary: &str, arguments: &[&str]) -> ApiResult<String>;
}
/// Verifies a trusted remediation against every finding that motivated it.
pub trait Verifier {
    /// Return whether the action corrected all supplied observations.
    fn verify(&self, references: &[DiscoveryReference]) -> bool;
}
pub(super) trait VocabularyDocument<Target>: Sized {
    fn apply_vocabulary_replacement(self, target: &Target, original: &str, replacement: &str) -> ApiResult<Self>;
    fn contains_vocabulary_value(&self, target: &Target, replacement: &str) -> bool;
    fn stable_deduplicate(self) -> Self;
}
/// Verified result of applying one trusted remediation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The action completed and the verifier accepted the new state.
    Applied(String),
    /// Verification failed after a document write and the original content was restored.
    RolledBack(String),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum VocabularyTarget {
    ContactAffiliation,
    ContactOrganization,
    Funder(usize),
    Keyword(usize),
    Partner(usize),
    Technology(usize),
}
/// Native runner for trusted fixed recipe vectors.
#[cfg(feature = "analysis")]
#[derive(Clone, Copy, Debug)]
pub struct CommandRunner {
    timeout: Duration,
}
impl Artifacts {
    fn restore(path: &Path, original: &str) -> ApiResult<()> {
        Self::from(vec![Artifact {
            content: original.to_string(),
            path: path.to_path_buf(),
        }])
        .apply()
        .map(|_| ())
    }
}
impl CheckReference {
    pub(super) fn verified_vocabulary_replacement(&self) -> ApiResult<(VocabularyTarget, String, String)> {
        self.fingerprint_matches()
            .then_some(())
            .ok_or_else(|| eyre!("Vocabulary fix source fingerprint is stale"))
            .and_then(|()| {
                self.vocabulary_advisory
                    .as_deref()
                    .ok_or_else(|| eyre!("Vocabulary fix is missing resolver evidence"))
            })
            .and_then(|advisory| {
                VocabularyKind::try_from(advisory.vocabulary.as_str())
                    .map_err(|_| eyre!("Vocabulary fix names an unknown vocabulary"))
                    .and_then(|kind| VocabularyResolver::embedded(kind).map_err(|why| eyre!(why)))
                    .and_then(|resolver| {
                        resolver
                            .resolve_located(
                                vec![(advisory.source_locator.clone(), advisory.original.clone())],
                                VocabularyOptions {
                                    fuzzy: advisory.mode,
                                    ..VocabularyOptions::default()
                                },
                            )
                            .map_err(|why| eyre!(why))
                    })
                    .and_then(|report| {
                        let resolution = report.resolutions.first();
                        let valid = advisory.complete
                            && report.policy_revision == advisory.policy_revision
                            && report.vocabulary_revision == advisory.vocabulary_revision
                            && resolution.is_some_and(|value| {
                                value.complete
                                    && value.outcome.is_accepted()
                                    && value.canonical.as_deref() == advisory.replacement.as_deref()
                                    && value.source_locator.as_deref() == Some(advisory.source_locator.as_str())
                            });
                        valid
                            .then_some(())
                            .ok_or_else(|| eyre!("Vocabulary fix resolver evidence is stale or ambiguous"))
                    })
                    .and_then(|()| {
                        VocabularyTarget::try_from(advisory.source_locator.as_str())
                            .map_err(|_| eyre!("Vocabulary fix locator is unsupported"))
                            .and_then(|target| {
                                advisory
                                    .replacement
                                    .clone()
                                    .map(|replacement| (target, advisory.original.clone(), replacement))
                                    .ok_or_else(|| eyre!("Vocabulary fix has no replacement"))
                            })
                    })
            })
    }
}
#[cfg(feature = "analysis")]
impl CommandRunner {
    /// Create a runner that bounds each native recipe process.
    pub const fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}
#[cfg(feature = "analysis")]
impl Runner for CommandRunner {
    fn run(&self, binary: &str, arguments: &[&str]) -> ApiResult<String> {
        CommandSpec::new(binary)
            .args(arguments)
            .checked_output_with_timeout(self.timeout)
            .map(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                [stdout.as_ref(), stderr.as_ref()].concat()
            })
            .map_err(|why| eyre!("Run registered fix {binary} — {why}"))
    }
}
impl Plan {
    /// Apply trusted actions beneath `root`, verifying each action before reporting success.
    pub fn execute<R, Verify>(&self, root: &Path, runner: &R, verify: Verify) -> ApiResult<Vec<Outcome>>
    where
        R: Runner,
        Verify: Verifier,
    {
        self.actions.iter().map(|action| execute(action, root, runner, &verify)).collect()
    }
}
impl Recipe {
    fn execute<R, Verify>(self, references: &[DiscoveryReference], runner: &R, verify: &Verify) -> ApiResult<Outcome>
    where
        R: Runner,
        Verify: Verifier,
    {
        let (binary, arguments) = self.command();
        let binary = match binary {
            | APPLICATION => current_exe()
                .map(|path| path.display().to_string())
                .map_err(|why| eyre!("Resolve current ACORN executable — {why}")),
            | value => Ok(value.to_string()),
        };
        binary
            .and_then(|binary| runner.run(&binary, arguments))
            .and_then(|_| match verify.verify(references) {
                | true => Ok(Outcome::Applied(format!("run {self}"))),
                | false => Err(eyre!("Fix verification failed for {self}")),
            })
    }
}
impl VocabularyDocument<VocabularyTarget> for ResearchActivity {
    fn apply_vocabulary_replacement(mut self, target: &VocabularyTarget, original: &str, replacement: &str) -> ApiResult<Self> {
        let selected = match target {
            | VocabularyTarget::ContactAffiliation => self.contact.affiliation.as_mut(),
            | VocabularyTarget::ContactOrganization => Some(&mut self.contact.organization),
            | VocabularyTarget::Funder(index) => self.meta.funders.as_mut().and_then(|values| match values {
                | OneOrMany::One(value) => (*index == 0).then_some(&mut value.name),
                | OneOrMany::Many(values) => values.get_mut(*index).map(|value| &mut value.name),
            }),
            | VocabularyTarget::Keyword(index) => self.meta.keywords.get_mut(*index),
            | VocabularyTarget::Partner(index) => self.meta.partners.as_mut().and_then(|values| match values {
                | OneOrMany::One(value) => (*index == 0).then_some(value),
                | OneOrMany::Many(values) => values.get_mut(*index),
            }),
            | VocabularyTarget::Technology(index) => self.meta.technology.get_mut(*index),
        };
        selected
            .filter(|value| value.as_str() == original)
            .map(|value| *value = replacement.to_string())
            .ok_or_else(|| eyre!("Vocabulary fix precondition is stale at {target:?}"))
            .map(|()| self)
    }
    fn contains_vocabulary_value(&self, target: &VocabularyTarget, replacement: &str) -> bool {
        match target {
            | VocabularyTarget::ContactAffiliation => self.contact.affiliation.as_deref() == Some(replacement),
            | VocabularyTarget::ContactOrganization => self.contact.organization == replacement,
            | VocabularyTarget::Funder(_) => self
                .meta
                .funders
                .as_ref()
                .is_some_and(|values| values.iter().any(|value| value.name == replacement)),
            | VocabularyTarget::Keyword(_) => self.meta.keywords.iter().any(|value| value == replacement),
            | VocabularyTarget::Partner(_) => self
                .meta
                .partners
                .as_ref()
                .is_some_and(|values| values.iter().any(|value| value == replacement)),
            | VocabularyTarget::Technology(_) => self.meta.technology.iter().any(|value| value == replacement),
        }
    }
    fn stable_deduplicate(mut self) -> Self {
        self.meta.keywords = stable_deduplicate(self.meta.keywords);
        self.meta.partners = self
            .meta
            .partners
            .map(|values| stable_deduplicate(values).into_iter().collect::<OneOrMany<_>>());
        self.meta.technology = stable_deduplicate(self.meta.technology);
        self
    }
}
impl<Verify> Verifier for Verify
where
    Verify: Fn() -> bool,
{
    fn verify(&self, _references: &[DiscoveryReference]) -> bool {
        self()
    }
}
impl TryFrom<&str> for VocabularyTarget {
    type Error = &'static str;

    fn try_from(locator: &str) -> Result<Self, Self::Error> {
        match locator {
            | "contact.affiliation" => Ok(Self::ContactAffiliation),
            | "contact.organization" => Ok(Self::ContactOrganization),
            | _ => indexed_locator(locator, "meta.funders[", "].name")
                .map(Self::Funder)
                .or_else(|| indexed_locator(locator, "meta.keywords[", "]").map(Self::Keyword))
                .or_else(|| indexed_locator(locator, "meta.partners[", "]").map(Self::Partner))
                .or_else(|| indexed_locator(locator, "meta.technology[", "]").map(Self::Technology))
                .ok_or("unsupported vocabulary locator"),
        }
    }
}
pub(super) fn apply_vocabulary_replacement<Data, Target>(data: Data, target: &Target, original: &str, replacement: &str) -> ApiResult<Data>
where
    Data: VocabularyDocument<Target>,
{
    data.apply_vocabulary_replacement(target, original, replacement)
}
fn execute<R, Verify>(action: &ExecutableFix, root: &Path, runner: &R, verify: &Verify) -> ApiResult<Outcome>
where
    R: Runner,
    Verify: Verifier,
{
    match action {
        | ExecutableFix::Document {
            expected,
            references,
            replacement,
            target,
        } => target
            .materialize_under(root)
            .map_err(|why| eyre!(why))
            .and_then(|path| {
                std::fs::read_to_string(&path)
                    .map_err(|why| eyre!("Read fix target {} — {why}", path.display()))
                    .map(|content| (content, path))
            })
            .and_then(|(content, path)| match content.contains(expected) {
                | false => Err(eyre!("Fix precondition is stale for {}", path.display())),
                | true => {
                    let changed = content.replacen(expected, replacement, 1);
                    Artifacts::from(vec![Artifact {
                        path: path.clone(),
                        content: changed,
                    }])
                    .apply()
                    .and_then(|_| match verify.verify(references) {
                        | true => Ok(Outcome::Applied(format!("write {}", target.as_path().display()))),
                        | false => Artifacts::from(vec![Artifact { path, content }])
                            .apply()
                            .map(|_| Outcome::RolledBack(format!("write {}", target.as_path().display()))),
                    })
                }
            }),
        | ExecutableFix::Recipe { recipe, references } => recipe.execute(references, runner, verify),
        | ExecutableFix::VocabularyDocument { references, target } => execute_vocabulary_document(references, target, root),
    }
}
fn execute_vocabulary_document(references: &[CheckReference], target: &SafePath, root: &Path) -> ApiResult<Outcome> {
    target
        .materialize_under(root)
        .map_err(|why| eyre!(why))
        .and_then(|path| read_file(&path).and_then(|original| ResearchActivity::read(path.clone()).map(|data| (data, original, path))))
        .and_then(|(data, original, path)| {
            references
                .iter()
                .map(CheckReference::verified_vocabulary_replacement)
                .collect::<ApiResult<Vec<_>>>()
                .map(|replacements| (data, original, path, replacements))
        })
        .and_then(|(data, original, path, replacements)| {
            replacements
                .iter()
                .try_fold(data, |data, (target, original, replacement)| {
                    apply_vocabulary_replacement(data, target, original, replacement)
                })
                .map(|data| (data, original, path, replacements))
        })
        .and_then(|(data, original, path, replacements)| {
            let data = data.stable_deduplicate();
            data.validate()
                .map_err(|why| eyre!("Vocabulary fix produced invalid research activity data — {why}"))
                .map(|()| (data, original, path, replacements))
        })
        .and_then(|(data, original, path, replacements)| {
            data.serialize_as(&MimeType::from_path(&path))
                .map_err(|why| eyre!(why))
                .map(|content| (data, original, path, replacements, content))
        })
        .and_then(|(data, original, path, replacements, content)| {
            Artifacts::from(vec![Artifact {
                content: content.clone(),
                path: path.clone(),
            }])
            .apply()
            .map(|_| (content, data, original, path, replacements))
        })
        .and_then(|(content, data, original, path, replacements)| {
            let verified = replacements
                .iter()
                .all(|(target, _, replacement)| data.contains_vocabulary_value(target, replacement));
            match verified {
                | true => {
                    let source = path.display().to_string();
                    let advisories = references.iter().filter_map(|reference| reference.vocabulary_advisory.as_deref());
                    write_vocabulary_advisory_audit(&source, &original, &content, advisories)
                        .map_err(|why| eyre!(why))
                        .map(|_| Outcome::Applied(format!("write {}", target.as_path().display())))
                        .or_else(|why| Artifacts::restore(&path, &original).and_then(|()| Err(why)))
                }
                | false => Artifacts::restore(&path, &original).map(|_| Outcome::RolledBack(format!("write {}", target.as_path().display()))),
            }
        })
}
fn indexed_locator(locator: &str, prefix: &str, suffix: &str) -> Option<usize> {
    locator
        .strip_prefix(prefix)
        .and_then(|value| value.strip_suffix(suffix))
        .and_then(|value| value.parse().ok())
}
