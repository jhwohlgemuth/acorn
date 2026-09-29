use super::{
    plan::{ExecutableFix, Plan, Recipe},
    ApiResult, DiscoveryReference, String, Vec,
};
use crate::io::workflow::{Artifact, Artifacts};
#[cfg(feature = "analysis")]
use acorn_cmd::CommandSpec;
use acorn_core::util::constants::app::APPLICATION;
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
/// Verified result of applying one trusted remediation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The action completed and the verifier accepted the new state.
    Applied(String),
    /// Verification failed after a document write and the original content was restored.
    RolledBack(String),
}
/// Native runner for trusted fixed recipe vectors.
#[cfg(feature = "analysis")]
#[derive(Clone, Copy, Debug)]
pub struct CommandRunner {
    timeout: Duration,
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
impl<Verify> Verifier for Verify
where
    Verify: Fn() -> bool,
{
    fn verify(&self, _references: &[DiscoveryReference]) -> bool {
        self()
    }
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
    }
}
