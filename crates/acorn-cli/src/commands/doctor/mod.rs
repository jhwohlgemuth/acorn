//! Host diagnostic reporting and trusted remediation entry point.

use crate::cli::arguments::Diagnostic;
use crate::cli::CommandOptions;
use crate::commands::preflight;
use acorn::analyzer::{
    fix::{self, IntoFixes},
    host::discovery::{FindingStatus, Record, Records},
};
use acorn_host::terminal::Label;
use color_eyre::eyre::{eyre, Report, Result};
use core::time::Duration;
use std::{env::current_dir, path::Path};
trait DoctorRecords {
    fn apply_safe_fixes(&self, policy: fix::Policy) -> Result<(), Report>;
    fn render(&self);
}
struct HostVerifier;
impl fix::Verifier for HostVerifier {
    fn verify(&self, references: &[fix::DiscoveryReference]) -> bool {
        let current = Records::diagnose_host();
        references.iter().all(|reference| {
            current.iter().any(|record| match record {
                | Record::Finding {
                    category,
                    status: FindingStatus::Healthy,
                    subject,
                    ..
                } => category == reference.category.as_ref() && subject == &reference.subject,
                | _ => false,
            })
        })
    }
}
impl DoctorRecords for Records {
    fn apply_safe_fixes(&self, policy: fix::Policy) -> Result<(), Report> {
        let report = fix::Provider::provide(&fix::HostProvider, &self.fix_references());
        let safe = report
            .fixes()
            .into_iter()
            .filter(|fix| fix.automation.is_safe_automatic() && fix.effects.reversible)
            .cloned()
            .collect::<Vec<_>>();
        fix::Plan::from_fixes(&safe, policy).and_then(|plan| match plan.preview().is_empty() {
            | true => {
                eprintln!("=> {} No safe automatic host fixes are available", Label::skip());
                Ok(())
            }
            | false => {
                let root = current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
                plan.execute(&root, &fix::CommandRunner::new(Duration::from_secs(120)), HostVerifier)
                    .map(|outcomes| outcomes.iter().for_each(|outcome| eprintln!("=> {} {outcome:?}", Label::pass())))
            }
        })
    }
    fn render(&self) {
        self.iter().for_each(|record| {
            if let Record::Finding {
                category,
                expected,
                observed,
                status,
                subject,
                ..
            } = record
            {
                println!("=> {} [{category}] {subject}: {observed} (expected {expected})", status.label());
            }
        });
    }
}
fn categories(checks: &[Diagnostic]) -> Vec<String> {
    if checks.contains(&Diagnostic::All) {
        ["gpu", "memory", "network", "software", "system"].map(str::to_string).to_vec()
    } else {
        checks.iter().map(ToString::to_string).collect()
    }
}
fn interactive_run(fix: bool, offline: bool) -> Result<(), Report> {
    #[cfg(feature = "tui")]
    {
        acorn_tui::run_with_options(
            acorn_tui::Screen::Doctor,
            None,
            acorn_tui::TuiOptions {
                database_path: None,
                doctor_fix: fix,
                no_local_database: true,
                offline,
            },
        )
        .map(|_| ())
    }
    #[cfg(not(feature = "tui"))]
    {
        let _ = (fix, offline);
        Err(eyre!("TUI feature not enabled. Build with: cargo build --features tui"))
    }
}
fn non_interactive_run(fix: bool, report: bool, check: &[Diagnostic], offline: bool) -> Result<(), Report> {
    let selected = categories(check);
    let before = Records::diagnose_host().findings_in(selected.iter().map(String::as_str));
    let applied = match fix {
        | false => Ok(()),
        | true => before.apply_safe_fixes(fix::Policy::safe(offline)),
    };
    applied.and_then(|()| {
        let after = match fix {
            | true => Records::diagnose_host().findings_in(selected.iter().map(String::as_str)),
            | false => before,
        };
        if report {
            serde_json::to_string_pretty(&after).map_err(Report::new).map(|value| println!("{value}"))
        } else {
            after.render();
            Ok(())
        }
    })
}
/// Run host diagnostics, optionally planning or applying trusted fixes.
pub fn run(fix: &bool, interactive: &bool, report: &bool, check: &[Diagnostic], offline: bool) -> Result<(), Report> {
    let options = CommandOptions::init().offline(offline).quiet(*report).supported(true).build();
    preflight!(&options);
    match (*interactive, *report) {
        | (true, true) => Err(eyre!("--report cannot be combined with --interactive")),
        | (true, false) => interactive_run(*fix, offline),
        | (false, _) => non_interactive_run(*fix, *report, check, offline),
    }
}
#[cfg(test)]
pub(super) fn should_run(checks: &[Diagnostic], diagnostic: Diagnostic) -> bool {
    [diagnostic, Diagnostic::All].iter().any(|candidate| checks.contains(candidate))
}
#[cfg(test)]
mod tests;
