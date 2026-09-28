use super::{
    plan::{ExecutableFix, FixPlan, Recipe},
    ApiResult, String, Vec,
};
use crate::io::workflow::{Artifact, Artifacts};
use color_eyre::eyre::eyre;
use std::path::Path;

/// Executes a command from ACORN's fixed recipe registry.
pub trait FixRunner {
    /// Run one fixed binary and argument vector, returning redacted output.
    fn run(&self, binary: &str, arguments: &[&str]) -> ApiResult<String>;
}
/// Verified result of applying one trusted remediation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FixOutcome {
    /// The action completed and the verifier accepted the new state.
    Applied(String),
    /// Verification failed after a document write and the original content was restored.
    RolledBack(String),
}
impl FixPlan {
    /// Apply trusted actions beneath `root`, verifying each action before reporting success.
    pub fn execute<R, Verify>(&self, root: &Path, runner: &R, verify: Verify) -> ApiResult<Vec<FixOutcome>>
    where
        R: FixRunner,
        Verify: Fn() -> bool + Copy,
    {
        self.actions.iter().map(|action| execute(action, root, runner, verify)).collect()
    }
}
impl Recipe {
    fn execute<R, Verify>(self, runner: &R, verify: Verify) -> ApiResult<FixOutcome>
    where
        R: FixRunner,
        Verify: FnOnce() -> bool,
    {
        let (binary, arguments) = self.command();
        runner.run(binary, arguments).and_then(|_| match verify() {
            | true => Ok(FixOutcome::Applied(format!("run {self}"))),
            | false => Err(eyre!("Fix verification failed for {self}")),
        })
    }
}
fn execute<R, Verify>(action: &ExecutableFix, root: &Path, runner: &R, verify: Verify) -> ApiResult<FixOutcome>
where
    R: FixRunner,
    Verify: FnOnce() -> bool,
{
    match action {
        | ExecutableFix::Document {
            expected,
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
                    .and_then(|_| match verify() {
                        | true => Ok(FixOutcome::Applied(format!("write {}", target.as_path().display()))),
                        | false => Artifacts::from(vec![Artifact { path, content }])
                            .apply()
                            .map(|_| FixOutcome::RolledBack(format!("write {}", target.as_path().display()))),
                    })
                }
            }),
        | ExecutableFix::Recipe(recipe) => recipe.execute(runner, verify),
    }
}
