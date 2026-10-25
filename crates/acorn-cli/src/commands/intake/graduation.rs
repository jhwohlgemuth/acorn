//! Human-reviewed graduation of local research activity indexes.
use crate::cli::IntakeRunArguments;
use crate::io::interactive::{PromptOutcome, Viewport};
use acorn::io::{durable_replace_file, read_file_snapshot};
use acorn_core::util::MimeType;
use acorn_schema::codec::DocumentCodec;
use acorn_schema::research_activity::{GraduationCandidates, ResearchActivity};
use acorn_schema::validation::Validate;
use color_eyre::eyre::{eyre, Result};
use data_encoding::HEXLOWER;
use ring::digest::{digest, SHA256};
use serde_json::{json, Value};
use std::fs::OpenOptions;
use std::io::{stdin, stdout, ErrorKind, IsTerminal, Write};
use std::path::{Path, PathBuf};

const MAX_INDEX_BYTES: usize = 10 * 1024 * 1024;
fn apply(index: &Path, accepted: &[String], expected_digest: &str) -> Result<Value> {
    format(index).and_then(|format| {
        read_file_snapshot(index, MAX_INDEX_BYTES).and_then(|(path, before, _)| match digest_text(&before) == expected_digest {
            | false => Err(eyre!("Index changed since review; run intake run --index INDEX --dry-run again")),
            | true => decode(&before, &format).and_then(|activity| {
                let candidates = GraduationCandidates::from(&activity);
                validate_selection(&candidates, accepted)
                    .and_then(|()| encode_graduated(activity, accepted, &format))
                    .and_then(|after| {
                        persist(&path, &before, &after).map(|backup| {
                            json!({
                                "accepted": accepted,
                                "backup": backup,
                                "baseDigest": expected_digest,
                                "finalDigest": digest_text(&after),
                                "index": path,
                                "status": "graduated"
                            })
                        })
                    })
            }),
        })
    })
}
pub(in crate::commands) fn ask(prompt: &str) -> Result<PromptOutcome<bool>> {
    match stdin().is_terminal() && stdout().is_terminal() {
        | true => Viewport::open().and_then(|mut viewport| viewport.confirm(prompt, false)),
        | false => Err(eyre!(
            "--interactive requires terminal input and output; use --dry-run, reviewed --accept with --expected-digest, or --force"
        )),
    }
}
fn decode(content: &str, format: &MimeType) -> Result<ResearchActivity> {
    ResearchActivity::decode(content, format)
        .map_err(|errors| eyre!(errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")))
        .and_then(|activity| activity.validate().map(|()| activity).map_err(Into::into))
}
fn digest_text(content: &str) -> String {
    format!("sha256:{}", HEXLOWER.encode(digest(&SHA256, content.as_bytes()).as_ref()))
}
fn encode_graduated(activity: ResearchActivity, accepted: &[String], format: &MimeType) -> Result<String> {
    activity
        .graduate(accepted)
        .map_err(Into::into)
        .and_then(|graduated| graduated.validate().map(|()| graduated).map_err(Into::into))
        .and_then(|graduated| graduated.encode(format).map_err(|why| eyre!(why.to_string())))
        .and_then(|after| decode(&after, format).map(|_| after))
}
pub(in crate::commands) fn execute(
    index: &Path,
    options: &IntakeRunArguments,
    mut review: impl FnMut(&str) -> Result<PromptOutcome<bool>>,
) -> Result<Value> {
    match !options.accept.is_empty() && options.expected_digest.is_none() {
        | true => Err(eyre!("--accept requires --expected-digest from a previous --dry-run")),
        | false => format(index).and_then(|format| {
            read_file_snapshot(index, MAX_INDEX_BYTES).and_then(|(path, before, _)| {
                let base_digest = digest_text(&before);
                match options.expected_digest.as_ref().is_some_and(|expected| expected != &base_digest) {
                    | true => Err(eyre!("Index changed since review; run intake run --index INDEX --dry-run again")),
                    | false => decode(&before, &format).and_then(|activity| {
                        let candidates = GraduationCandidates::from(&activity);
                        select(&candidates, options, &mut review).and_then(|accepted| {
                            let proposed = match accepted.is_empty() {
                                | true => Ok(before.clone()),
                                | false => {
                                    validate_selection(&candidates, &accepted).and_then(|()| encode_graduated(activity.clone(), &accepted, &format))
                                }
                            };
                            proposed.and_then(|after| {
                                if options.dry_run || accepted.is_empty() {
                                    Ok(json!({
                                        "accepted": accepted,
                                        "baseDigest": base_digest,
                                        "candidates": candidates,
                                        "index": path,
                                        "logbook": activity.logbook,
                                        "proposedDigest": digest_text(&after),
                                        "status": if options.dry_run { "dry-run" } else { "unchanged" }
                                    }))
                                } else {
                                    let approval = match options.interactive {
                                        | true => review(&format!(
                                            "Apply {} selected promotion(s) to {} and archive their source entries?",
                                            accepted.len(),
                                            path.display()
                                        ))
                                        .and_then(|outcome| match outcome {
                                            | PromptOutcome::Selected(true) => Ok(()),
                                            | _ => Err(eyre!("Intake cancelled; no changes applied")),
                                        }),
                                        | false => Ok(()),
                                    };
                                    approval.and_then(|()| apply(&path, &accepted, &base_digest))
                                }
                            })
                        })
                    }),
                }
            })
        }),
    }
}
fn format(index: &Path) -> Result<MimeType> {
    match MimeType::from(index.to_string_lossy().as_ref()) {
        | format @ (MimeType::Json | MimeType::Jsonc | MimeType::Yaml | MimeType::Zon) => Ok(format),
        | _ => Err(eyre!("Graduation requires a JSON, JSONC, YAML, or ZONF index")),
    }
}
fn persist(path: &Path, before: &str, after: &str) -> Result<PathBuf> {
    let mut backup_name = path.as_os_str().to_os_string();
    backup_name.push(format!(".graduation-{}.before", digest_text(before).trim_start_matches("sha256:")));
    let backup = PathBuf::from(backup_name);
    let preserved = match OpenOptions::new().write(true).create_new(true).open(&backup) {
        | Ok(mut file) => file.write_all(before.as_bytes()).and_then(|()| file.sync_all()).map_err(Into::into),
        | Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            read_file_snapshot(&backup, MAX_INDEX_BYTES).and_then(|(_, content, _)| match content == before {
                | true => Ok(()),
                | false => Err(eyre!("Existing graduation backup does not match the reviewed index")),
            })
        }
        | Err(error) => Err(error.into()),
    };
    preserved
        .and_then(|()| read_file_snapshot(path, MAX_INDEX_BYTES))
        .and_then(|(_, current, _)| match current == before {
            | true => durable_replace_file(path, after.as_bytes()).map(|()| backup),
            | false => Err(eyre!("Index changed before replacement; preserved backup, no graduation applied")),
        })
}
fn select(
    candidates: &GraduationCandidates,
    options: &IntakeRunArguments,
    review: &mut impl FnMut(&str) -> Result<PromptOutcome<bool>>,
) -> Result<Vec<String>> {
    let accepted = match options.accept.is_empty() {
        | true => Ok(candidates
            .0
            .iter()
            .map(|candidate| candidate.entry_identifier.clone())
            .collect::<Vec<_>>()),
        | false => validate_selection(candidates, &options.accept).map(|()| options.accept.clone()),
    };
    accepted.and_then(|accepted| {
        if options.interactive {
            candidates.0.iter().filter(|candidate| accepted.contains(&candidate.entry_identifier))
                .try_fold(Vec::new(), |mut selected, candidate| {
                    review(&format!("Accept {} -> {:?}: {:?}?", candidate.entry_identifier, candidate.target, candidate.value))
                        .and_then(|outcome| match outcome {
                            | PromptOutcome::Cancelled => Err(eyre!("Intake cancelled; no changes applied")),
                            | PromptOutcome::Selected(yes) => {
                                if yes { selected.push(candidate.entry_identifier.clone()); }
                                Ok(selected)
                            }
                        })
                })
        } else if options.dry_run || options.force || !options.accept.is_empty() || accepted.is_empty() {
            Ok(accepted)
        } else {
            Err(eyre!("Graduation requires approval: use --interactive, --force to accept all eligible promotions, or --accept with --expected-digest; preview with --dry-run"))
        }
    })
}
fn validate_selection(candidates: &GraduationCandidates, accepted: &[String]) -> Result<()> {
    if accepted.is_empty() {
        Err(eyre!("Select at least one reviewed entry with --accept"))
    } else {
        accepted.iter().try_for_each(
            |id| match candidates.0.iter().filter(|candidate| &candidate.entry_identifier == id).count() {
                | 1 => Ok(()),
                | 0 => Err(eyre!("Unknown or already archived graduation entry: {id}")),
                | _ => Err(eyre!("Ambiguous graduation entry: {id}; target-specific approval is required")),
            },
        )
    }
}

#[cfg(test)]
mod tests;
