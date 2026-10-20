//! Explicit local and GitLab Power Automate JSONL intake commands.
use acorn::analyzer::diff::{DiffFormat, DiffInput, SemanticDiff};
use acorn::io::api::gitlab::automation::ReviewRevisions;
use acorn::io::api::gitlab::{self, bot, DeploymentConfig};
use acorn::io::api::powerautomate::jsonl::AdmittedSubmission;
use acorn::io::api::powerautomate::local::{ApplyStatus, BatchInspection, BatchSummary};
use acorn::io::api::webhooks::store::OperationQueue;
use acorn::io::api::Configuration;
use acorn::io::config::ApplicationConfiguration;
use acorn_core::util::MimeType;
use acorn_diff::unified_diff;
use acorn_host::fs::file_checksum;
use acorn_host::terminal::Label;
use acorn_schema::validation::Validate;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{eyre, Result};
use serde_json::json;
use std::env;
use std::fs::write;
use std::path::{Path, PathBuf};

const MAX_MANUAL_WORK: usize = 256;
/// Apply one proposal after a human reviews its stored source and proposed index.
pub fn apply(config: &Path, submission_id: &str, verbose: &Verbosity) -> Result<()> {
    ApplicationConfiguration::load_local_intake(config)
        .and_then(|(configuration, path)| configuration.apply(&path, submission_id))
        .map(|status| match (verbose.is_silent(), status) {
            | (true, _) => {}
            | (false, ApplyStatus::Applied) => println!("=> {} Applied {submission_id}", Label::pass()),
            | (false, ApplyStatus::AlreadyApplied) => println!("=> {} Already applied {submission_id}", Label::skip()),
        })
}
/// Apply every project in a reviewed local batch.
pub fn apply_batch(config: &Path, batch_id: &str, verbose: &Verbosity) -> Result<()> {
    ApplicationConfiguration::load_local_intake(config)
        .and_then(|(configuration, path)| configuration.apply_batch(&path, batch_id))
        .and_then(|summary| {
            serde_json::to_string_pretty(&summary)
                .map_err(Into::into)
                .map(|output| {
                    if !verbose.is_silent() {
                        println!("{output}");
                    }
                })
                .and_then(|()| match summary.error {
                    | Some(why) => Err(eyre!("Batch apply is partial: {why}")),
                    | None => Ok(()),
                })
        })
}
fn configured(config: &Path, require_intake: bool) -> Result<(ApplicationConfiguration, gitlab::Options, OperationQueue)> {
    match env::var("ACORN_DATABASE_PATH").ok().filter(|value| !value.trim().is_empty()) {
        | Some(_) => ApplicationConfiguration::load(&Some(config.to_path_buf())).and_then(|document| {
            document
                .validate()
                .map_err(color_eyre::Report::from)
                .and_then(|()| match document.power_automate() {
                    | Some(power_automate) if !require_intake || power_automate.intake.is_some() => {
                        power_automate.gitlab_project_id().map(|project| project.to_string()).map(|project| {
                            (
                                document,
                                gitlab::Options::from_env().with_identifier(project),
                                OperationQueue::configured(),
                            )
                        })
                    }
                    | Some(_) => Err(eyre!("options.powerAutomate.intake is required")),
                    | None => Err(eyre!("options.powerAutomate is required")),
                })
        }),
        | None => Err(eyre!("ACORN_DATABASE_PATH must name a persistent local operation database")),
    }
}
fn configured_server(config: &Path, require_intake: bool) -> Result<(bot::Server, OperationQueue)> {
    configured(config, require_intake).and_then(|(document, options, queue)| {
        let deployment: DeploymentConfig = document.workflow.clone().unwrap_or_default();
        document
            .power_automate()
            .cloned()
            .ok_or_else(|| eyre!("options.powerAutomate is required"))
            .and_then(|power_automate| {
                bot::Config::from(deployment)
                    .with_options(options)
                    .with_operation_queue(queue.clone())
                    .with_polling_enabled(false)
                    .with_powerautomate(power_automate, document.registry())
                    .map(|server_config| (bot::Server::new(server_config), queue))
            })
    })
}
/// Deliver ready callback intents from the durable queue without polling intake.
pub async fn deliver(config: &Path) -> Result<()> {
    match configured_server(config, false) {
        | Ok((server, queue)) => {
            let mut delivered = 0_usize;
            let mut failures = 0_usize;
            let mut consecutive_failures = 0_usize;
            let mut work_limit_reached = true;
            for _ in 0..MAX_MANUAL_WORK {
                match server.process_callback_next().await {
                    | Ok(true) => {
                        delivered = delivered.saturating_add(1);
                        consecutive_failures = 0;
                    }
                    | Ok(false) => {
                        work_limit_reached = false;
                        break;
                    }
                    | Err(_) => {
                        failures = failures.saturating_add(1);
                        consecutive_failures = consecutive_failures.saturating_add(1);
                        if consecutive_failures == 3 {
                            work_limit_reached = false;
                            break;
                        }
                    }
                }
            }
            queue.callback_counts().and_then(|callbacks| {
                let idle = delivered == 0 && failures == 0 && callbacks.queued == 0 && callbacks.retryable == 0 && callbacks.running == 0;
                println!(
                    "{}",
                    json!({
                        "delivered": delivered,
                        "attemptFailures": failures,
                        "callbacks": callbacks,
                        "idle": idle,
                        "workLimitReached": work_limit_reached,
                    })
                );
                match failures > 0 || callbacks.retryable > 0 || callbacks.failed > 0 {
                    | true => Err(eyre!("Callback delivery failed; inspect the durable callback queue")),
                    | false => Ok(()),
                }
            })
        }
        | Err(why) => Err(why),
    }
}
/// Export the exact current review-branch change for a durably admitted submission.
pub async fn export(config: &Path, submission_id: &str, output: &Path) -> Result<()> {
    let key = format!("powerautomate:form:v1:{}", submission_id.to_ascii_lowercase());
    match configured(config, true) {
        | Ok((_, options, queue)) => {
            let admission = queue
                .intake_admission(&key)
                .and_then(|value| value.ok_or_else(|| eyre!("Submission has no durable admission")));
            match admission.and_then(|admission| {
                serde_json::from_str::<AdmittedSubmission>(&admission.payload)
                    .map_err(Into::into)
                    .map(|admitted| (admitted, admission.digest))
            }) {
                | Ok((admitted, digest)) if admitted.submission.submission_id.eq_ignore_ascii_case(submission_id) => {
                    export_admission(&options, &admitted, &digest, output).await
                }
                | Ok(_) => Err(eyre!("Submission identity does not match durable admission")),
                | Err(why) => Err(why),
            }
        }
        | Err(why) => Err(why),
    }
}
async fn export_admission(options: &gitlab::Options, admitted: &AdmittedSubmission, source_digest: &str, output: &Path) -> Result<()> {
    let project_identifier = admitted.project.to_string();
    match gitlab::service_url(options) {
        | Ok(domain) if domain == admitted.domain && options.identifier() == Some(project_identifier.as_str()) => {
            match options.review_revisions(&admitted.branch).await {
                | Ok(review) if review.base != review.head => match revision_content(options, &admitted.path, &review.base).await {
                    | Ok(before) => match revision_content(options, &admitted.path, &review.head).await {
                        | Ok(after) => serde_json::to_value(&admitted.entry)
                            .map_err(Into::into)
                            .and_then(|entry| render_review_diff(&admitted.path, &entry, &before, &after, &review.base, &review.head))
                            .and_then(|diff| write_review_artifact(admitted, source_digest, output, &review, &diff)),
                        | Err(why) => Err(why),
                    },
                    | Err(why) => Err(why),
                },
                | Ok(_) => Err(eyre!("Review branch has no project change")),
                | Err(why) => Err(why),
            }
        }
        | Ok(_) => Err(eyre!("GitLab source does not match the durable admission")),
        | Err(why) => Err(why),
    }
}
/// Inspect the source lines and per-project diffs of one local batch.
pub fn inspect(config: &Path, batch_id: &str, verbose: &Verbosity) -> Result<()> {
    BatchInspection::inspect(config, batch_id).map(|inspection| {
        if !verbose.is_silent() {
            println!("=> {} Batch {}", Label::using(), inspection.batch_id);
            println!("Source {} ({})", inspection.source_path.display(), inspection.source_digest);
            println!("Can apply: {}", inspection.can_apply);
            println!("Status: {}", inspection.status);
            inspection.lines.iter().for_each(|line| {
                println!(
                    "Line {}: {} {} {}",
                    line.line,
                    line.status,
                    line.submission_id.as_deref().unwrap_or("unknown UUID"),
                    line.project_id.as_deref().unwrap_or("unknown project")
                );
                if let Some(reason) = &line.reason {
                    println!("  {reason}");
                }
            });
            inspection.projects.iter().for_each(|project| {
                println!("Project {} -> {} ({})", project.project_id, project.destination.display(), project.state);
                println!("Base: {}  Final: {}", project.base_digest, project.final_digest);
                print!(
                    "{}",
                    unified_diff(
                        &project.base_content,
                        &project.proposed_content,
                        &format!("a/{}", project.destination.display()),
                        &format!("b/{}", project.destination.display())
                    )
                );
            });
        }
    })
}
fn receipt_path(output: &Path) -> PathBuf {
    let mut name = output.as_os_str().to_os_string();
    name.push(".receipt.json");
    PathBuf::from(name)
}
fn render_review_diff(path: &str, entry: &serde_json::Value, before: &str, after: &str, base: &str, head: &str) -> Result<String> {
    let entry_identity = |value: &serde_json::Value| {
        value.as_object().map(|object| {
            let mut fields = object.clone();
            fields.remove("archived");
            fields
        })
    };
    let reviewed: Result<serde_json::Value> = serde_json::from_str(after).map_err(Into::into);
    reviewed.and_then(|reviewed| {
        entry_identity(entry)
            .ok_or_else(|| eyre!("Admitted entry is not a JSON object"))
            .and_then(|expected| {
                let present = reviewed
                    .pointer("/logbook/entries")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|entries| {
                        entries
                            .iter()
                            .any(|candidate| entry_identity(candidate).is_some_and(|fields| fields == expected))
                    });
                match present {
                    | true => {
                        let before_label = format!("{path}@{base}");
                        let after_label = format!("{path}@{head}");
                        let before_input = DiffInput {
                            content: before,
                            label: &before_label,
                            media_type: MimeType::Json,
                        };
                        let after_input = DiffInput {
                            content: after,
                            label: &after_label,
                            media_type: MimeType::Json,
                        };
                        match SemanticDiff::compare(&before_input, &after_input, DiffFormat::Unified, None) {
                            | Ok(SemanticDiff::Unified(diff)) if diff.is_empty() => Err(eyre!("Review branch has no semantic project change")),
                            | Ok(SemanticDiff::Unified(diff)) => Ok(diff),
                            | Ok(_) => Err(eyre!("Expected a unified diff")),
                            | Err(why) => Err(why.into()),
                        }
                    }
                    | false => Err(eyre!("Review branch does not contain the admitted submission entry")),
                }
            })
    })
}
async fn revision_content(options: &gitlab::Options, path: &str, revision: &str) -> Result<String> {
    gitlab::repository_file(&options.clone().with_path(path).with_sha(revision))
        .await
        .and_then(|file| file.decoded_content())
        .and_then(|content| String::from_utf8(content).map_err(Into::into))
}
/// Admit a single GitLab inbox snapshot and note page, then execute bounded review work.
pub async fn run(config: &Path) -> Result<()> {
    match configured_server(config, true) {
        | Ok((server, queue)) => {
            let intake_result = server.poll_jsonl_once().await;
            let notes_result = server.poll_once().await;
            let intake_failed = intake_result.is_err();
            let notes_failed = notes_result.is_err();
            let intake = intake_result.unwrap_or_default();
            let notes = notes_result.unwrap_or_default();
            let mut processed = 0_usize;
            let mut worker_failures = 0_usize;
            let mut consecutive_failures = 0_usize;
            let mut work_limit_reached = true;
            for _ in 0..MAX_MANUAL_WORK {
                match server.process_next().await {
                    | Ok(true) => {
                        processed = processed.saturating_add(1);
                        consecutive_failures = 0;
                    }
                    | Ok(false) => {
                        work_limit_reached = false;
                        break;
                    }
                    | Err(_) => {
                        worker_failures = worker_failures.saturating_add(1);
                        consecutive_failures = consecutive_failures.saturating_add(1);
                        if consecutive_failures == 3 {
                            work_limit_reached = false;
                            break;
                        }
                    }
                }
            }
            queue.counts().and_then(|queue_counts| {
                let failed = intake_failed
                    || notes_failed
                    || intake.failed_count > 0
                    || worker_failures > 0
                    || queue_counts.failed > 0
                    || queue_counts.queued > 0;
                println!(
                    "{}",
                    json!({
                        "intake": intake,
                        "intakeFailed": intake_failed,
                        "notes": notes,
                        "notesFailed": notes_failed,
                        "operationsProcessed": processed,
                        "operations": queue_counts,
                        "workerFailures": worker_failures,
                        "workLimitReached": work_limit_reached,
                    })
                );
                match failed {
                    | true => Err(eyre!("Manual intake has retryable or failed work; inspect durable operations")),
                    | false => Ok(()),
                }
            })
        }
        | Err(why) => Err(why),
    }
}
/// Stage valid local submissions as durable proposal files.
pub fn stage(config: &Path, inbox: &Path, batch: bool, verbose: &Verbosity) -> Result<()> {
    match batch {
        | true => BatchSummary::stage(config, inbox).and_then(|summary| {
            serde_json::to_string_pretty(&summary)
                .map_err(Into::into)
                .map(|output| {
                    if !verbose.is_silent() {
                        println!("{output}");
                    }
                })
                .and_then(|()| match summary.can_apply {
                    | true => Ok(()),
                    | false => Err(eyre!("Batch contains {} rejected line(s)", summary.rejected)),
                })
        }),
        | false => ApplicationConfiguration::load_local_intake(config)
            .and_then(|(configuration, path)| configuration.stage(&path, inbox))
            .and_then(|summary| {
                if !verbose.is_silent() {
                    summary.staged.iter().for_each(|id| println!("=> {} Staged {id}", Label::pass()));
                    summary
                        .existing
                        .iter()
                        .for_each(|id| println!("=> {} Already staged {id}", Label::skip()));
                    summary.rejected.iter().for_each(|rejected| {
                        eprintln!("=> {} Line {} rejected: {}", Label::fail(), rejected.line, rejected.reason);
                    });
                    println!("=> {} {summary}", Label::using());
                }
                match summary.rejected.len() {
                    | 0 => Ok(()),
                    | count => Err(eyre!("{count} local intake line(s) rejected")),
                }
            }),
    }
}
fn write_review_artifact(admitted: &AdmittedSubmission, source_digest: &str, output: &Path, review: &ReviewRevisions, diff: &str) -> Result<()> {
    write(output, diff.as_bytes())
        .map_err(color_eyre::Report::from)
        .and_then(|()| file_checksum(output, None).map_err(color_eyre::Report::from))
        .and_then(|checksum| {
            let receipt = json!({
                "submissionId": admitted.submission.submission_id,
                "sourceDigest": source_digest,
                "gitlabService": admitted.domain,
                "projectId": admitted.submission.project_id,
                "gitlabProject": admitted.project,
                "path": admitted.path,
                "branch": admitted.branch,
                "mergeRequestIid": review.merge_request_iid,
                "targetBranch": review.target_branch,
                "baseRevision": review.base,
                "headRevision": review.head,
                "diffSha256": checksum.checksum_value,
            });
            serde_json::to_vec_pretty(&receipt)
                .map_err(color_eyre::Report::from)
                .and_then(|content| write(receipt_path(output), content).map_err(color_eyre::Report::from))
                .map(|()| println!("{receipt}"))
        })
}
#[cfg(test)]
mod tests;
