//! Immutable GitLab JSONL inbox snapshots for Power Automate intake.
//!
//! Reads `.acorn/intake/logbook.jsonl` at one immutable commit through the existing repository-file API and splits UTF-8 LF-delimited lines with per-line locations and SHA-256 digests.
//! Admission, deduplication, and removal belong to later phases; this module performs no mutation.
use super::service::{commit_sha, repository_file};
use super::Options;
use crate::io::{config::application::PowerAutomateIntake, http::HttpResponseError, ApiResult};
use acorn_core::prelude::{String, Vec};
use acorn_schema::{hardware::memory::Memory, validation::Validate};
use color_eyre::eyre::eyre;
use core::iter::once;
use data_encoding::HEXLOWER;
use ring::digest::{digest, SHA256};

/// One nonempty LF-delimited inbox record with its source location.
#[derive(Clone, Debug, Eq, PartialEq, Validate)]
pub struct JsonlLine {
    /// One-based physical line number in the decoded inbox file.
    #[validate(range(min = 1))]
    pub line: u64,
    /// Byte offset of the line start in the decoded inbox file.
    pub byte_offset: u64,
    /// Raw line content without the trailing line feed.
    #[validate(nonempty)]
    pub raw: String,
    /// `sha256:`-prefixed lowercase hex digest of the raw line bytes.
    #[validate(digest)]
    pub digest: String,
}
/// Immutable inbox snapshot read at one exact commit.
#[derive(Clone, Debug, Eq, PartialEq, Validate)]
pub struct JsonlSnapshot {
    /// Exact commit SHA the inbox was read at.
    #[validate(commit)]
    pub commit: String,
    /// Nonempty records in file order.
    #[validate(nested)]
    pub lines: Vec<JsonlLine>,
}
impl JsonlSnapshot {
    /// Whether the inbox contained no records.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
    /// Number of records in the snapshot.
    pub fn len(&self) -> usize {
        self.lines.len()
    }
}
/// Read the configured inbox at its current branch head without mutation.
///
/// Resolves the branch to one immutable commit, reads the inbox file at that commit, and splits it with [`split_snapshot`].
/// A missing inbox file yields an empty snapshot at the resolved commit; authentication, network, and oversize failures remain errors.
pub async fn read_snapshot(intake: &PowerAutomateIntake, options: &Options) -> ApiResult<JsonlSnapshot> {
    match commit_sha(options, &intake.branch).await {
        | Err(why) => Err(why),
        | Ok(commit) => {
            let file_options = options.clone().with_path(intake.path.clone()).with_sha(commit.clone());
            match repository_file(&file_options).await {
                | Err(why) if is_not_found(&why) => Ok(JsonlSnapshot { commit, lines: Vec::new() }),
                | Err(why) => Err(why),
                | Ok(file) => match file.decoded_content() {
                    | Err(why) => Err(why),
                    | Ok(bytes) => match file_size_within_limit(&bytes, &intake.max_file) {
                        | Err(why) => Err(why),
                        | Ok(()) => match String::from_utf8(bytes) {
                            | Err(why) => Err(eyre!("Intake inbox '{}' is not UTF-8 — {why}", intake.path)),
                            | Ok(content) => match split_snapshot(&content, &intake.max_record) {
                                | Err(why) => Err(why),
                                | Ok(lines) => Ok(JsonlSnapshot { commit, lines }),
                            },
                        },
                    },
                },
            }
        }
    }
}
/// Split decoded inbox content into located records without network access.
///
/// Skips empty lines, rejects a nonterminated trailing line instead of silently repairing it, enforces the per-record size limit, and digests each record with SHA-256.
/// An empty inbox yields no lines.
pub fn split_snapshot(content: &str, max_record: &Memory) -> ApiResult<Vec<JsonlLine>> {
    match max_record.checked_bytes() {
        | None => Err(eyre!("Intake record size limit must be nonzero")),
        | Some(limit) => match content.is_empty() {
            | true => Ok(Vec::new()),
            | false => match content.strip_suffix('\n') {
                | None => Err(eyre!("Intake inbox has a nonterminated trailing line")),
                | Some(body) => match body
                    .split('\n')
                    .enumerate()
                    .try_fold((Vec::new(), 0_u64), |(lines, offset), (index, raw)| {
                        accumulate_line(lines, offset, index, raw, limit)
                    }) {
                    | Err(why) => Err(why),
                    | Ok((lines, _)) => Ok(lines),
                },
            },
        },
    }
}
/// Fold one physical line into the located-record accumulator.
fn accumulate_line(lines: Vec<JsonlLine>, offset: u64, index: usize, raw: &str, limit: u64) -> ApiResult<(Vec<JsonlLine>, u64)> {
    match line_number(index) {
        | Err(why) => Err(why),
        | Ok(line) => match raw.is_empty() {
            | true => match advance_offset(offset, 0) {
                | Err(why) => Err(why),
                | Ok(next) => Ok((lines, next)),
            },
            | false => match record_size(raw, limit, line) {
                | Err(why) => Err(why),
                | Ok(size) => match advance_offset(offset, size) {
                    | Err(why) => Err(why),
                    | Ok(next) => Ok((
                        lines
                            .into_iter()
                            .chain(once(JsonlLine {
                                line,
                                byte_offset: offset,
                                raw: raw.to_string(),
                                digest: line_digest(raw),
                            }))
                            .collect(),
                        next,
                    )),
                },
            },
        },
    }
}
/// Convert a zero-based split position into a one-based line number.
fn line_number(index: usize) -> ApiResult<u64> {
    match u64::try_from(index) {
        | Err(why) => Err(eyre!("Intake inbox line number does not fit in u64 — {why}")),
        | Ok(number) => match number.checked_add(1) {
            | Some(line) => Ok(line),
            | None => Err(eyre!("Intake inbox line number overflow")),
        },
    }
}
/// Enforce the per-record size limit for one raw line.
fn record_size(raw: &str, limit: u64, line: u64) -> ApiResult<u64> {
    match u64::try_from(raw.len()) {
        | Err(why) => Err(eyre!("Intake record size does not fit in u64 — {why}")),
        | Ok(size) => match size > limit {
            | true => Err(eyre!("Intake record on line {line} exceeds the configured record size limit")),
            | false => Ok(size),
        },
    }
}
/// Advance a byte offset past one record plus its line feed.
fn advance_offset(offset: u64, size: u64) -> ApiResult<u64> {
    match size.checked_add(1) {
        | None => Err(eyre!("Intake inbox byte offset overflow")),
        | Some(advance) => match offset.checked_add(advance) {
            | Some(next) => Ok(next),
            | None => Err(eyre!("Intake inbox byte offset overflow")),
        },
    }
}
/// Enforce the whole-file size limit for decoded inbox bytes.
fn file_size_within_limit(bytes: &[u8], max_file: &Memory) -> ApiResult<()> {
    match max_file.checked_bytes() {
        | None => Err(eyre!("Intake file size limit must be nonzero")),
        | Some(limit) => match u64::try_from(bytes.len()) {
            | Err(why) => Err(eyre!("Intake inbox size does not fit in u64 — {why}")),
            | Ok(size) => match size > limit {
                | true => Err(eyre!("Intake inbox exceeds the configured file size limit")),
                | false => Ok(()),
            },
        },
    }
}
/// Digest one raw record line as a `sha256:`-prefixed lowercase hex digest.
///
/// Uses the same spelling as the shared byte digest helper so the `digest` builtin validator applies;
/// computed locally because that helper is gated behind a feature the intake path does not enable.
fn line_digest(raw: &str) -> String {
    format!("sha256:{}", HEXLOWER.encode(digest(&SHA256, raw.as_bytes()).as_ref()))
}
/// Whether a repository-file failure means the inbox file does not exist.
fn is_not_found(error: &color_eyre::Report) -> bool {
    error.downcast_ref::<HttpResponseError>().is_some_and(|error| error.status_code() == 404) || {
        let message = error.to_string().to_ascii_lowercase();
        message.contains("404") && message.contains("not found")
    }
}
