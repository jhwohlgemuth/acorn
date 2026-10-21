//! Durable, append-only history for interactive metadata conversations.
use crate::agent::{ChatEvent, InferenceProvenance};
use crate::io::{ApiResult, FromPath};
use crate::util::constants::app::{APPLICATION, ORGANIZATION, QUALIFIER};
use acorn_core::util::{redact_sensitive_text, MimeType, SemanticVersion};
use acorn_host::fs::{project_data_directory, SafePath};
use bon::Builder;
use color_eyre::eyre::eyre;
use jiff::Timestamp;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, read_dir, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const RECORD_VERSION: SemanticVersion = SemanticVersion {
    major: 1,
    minor: 0,
    patch: 0,
};

/// One complete, versioned JSONL history record.
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(on(String, into))]
#[serde(deny_unknown_fields)]
pub struct ChatHistoryRecord {
    /// Format version for future readers.
    pub version: SemanticVersion,
    /// Identifier of the file's conversation session.
    pub session_id: String,
    /// RFC 3339 record time.
    pub timestamp: String,
    /// Stable event category for indexing without inspecting the payload.
    pub event_type: String,
    /// Redacted conversation event.
    pub event: ChatEvent,
    /// Optional concise decision or outcome.
    pub note: Option<String>,
    /// Optional secret-free backend provenance.
    pub provenance: Option<InferenceProvenance>,
}
/// One active, append-only chat session.
pub struct ChatSession {
    directory: PathBuf,
    file: File,
    path: PathBuf,
    poisoned: bool,
    session_id: String,
}
impl ChatSession {
    /// Start a fresh session in ACORN's platform data directory.
    pub fn start() -> ApiResult<Self> {
        SafePath::new("ask/sessions")
            .and_then(|namespace| project_data_directory(QUALIFIER, ORGANIZATION, APPLICATION, &namespace))
            .map_err(Into::into)
            .and_then(Self::start_in)
    }
    fn start_in(directory: PathBuf) -> ApiResult<Self> {
        create_dir_all(&directory)
            .map_err(|why| eyre!("Create chat history directory {} — {why}", directory.display()))
            .and_then(|()| {
                let session_id = nanoid!();
                let path = directory.join(format!("{session_id}.jsonl"));
                let mut options = OpenOptions::new();
                options.create_new(true).append(true);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::OpenOptionsExt;
                    options.mode(0o600);
                }
                options
                    .open(&path)
                    .map_err(|why| eyre!("Create chat history {} — {why}", path.display()))
                    .map(|file| Self {
                        directory,
                        file,
                        path,
                        poisoned: false,
                        session_id,
                    })
            })
    }
    /// Append and durably flush a complete event before returning.
    pub fn append(&mut self, event: &ChatEvent, provenance: Option<&InferenceProvenance>, note: Option<&str>) -> ApiResult<()> {
        if self.poisoned {
            Err(eyre!("Chat history {} cannot accept records after a write failure", self.path.display()))
        } else {
            self.append_active(event, provenance, note)
        }
    }
    fn append_active(&mut self, event: &ChatEvent, provenance: Option<&InferenceProvenance>, note: Option<&str>) -> ApiResult<()> {
        let event = event.sanitized(redact_sensitive_text);
        let record = ChatHistoryRecord::builder()
            .version(RECORD_VERSION)
            .session_id(self.session_id.as_str())
            .timestamp(Timestamp::now().to_string())
            .event_type(event.to_string())
            .event(event)
            .maybe_note(note.map(redact_sensitive_text))
            .maybe_provenance(provenance.cloned())
            .build();
        let result = serde_json::to_vec(&record)
            .map_err(|why| eyre!("Encode chat history record — {why}"))
            .and_then(|mut encoded| {
                encoded.push(b'\n');
                self.file
                    .write_all(&encoded)
                    .and_then(|()| self.file.flush())
                    .and_then(|()| self.file.sync_data())
                    .map_err(|why| eyre!("Append chat history {} — {why}", self.path.display()))
            });
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    /// Close the current transcript and return its path and the fresh path.
    pub fn clear(&mut self) -> ApiResult<(PathBuf, PathBuf)> {
        self.append(&ChatEvent::Cleared, None, None).and_then(|()| {
            self.poisoned = true;
            Self::start_in(self.directory.clone()).map(|next| {
                let archived = self.path.clone();
                let current = next.path.clone();
                *self = next;
                (archived, current)
            })
        })
    }
    /// Return the active session's on-disk path.
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// List saved transcripts in deterministic filename order.
    pub fn session_paths(&self) -> ApiResult<Vec<PathBuf>> {
        read_dir(&self.directory)
            .and_then(|entries| entries.map(|entry| entry.map(|entry| entry.path())).collect::<std::io::Result<Vec<_>>>())
            .map_err(|why| eyre!("List chat history {} — {why}", self.directory.display()))
            .map(|mut paths| {
                paths.retain(|path| MimeType::from_path(path) == MimeType::JsonLines);
                paths.sort();
                paths
            })
    }
    /// Read complete records, ignoring an interrupted trailing record only.
    pub fn read_records(path: &Path) -> ApiResult<Vec<ChatHistoryRecord>> {
        let mut bytes = Vec::new();
        File::open(path)
            .and_then(|mut file| file.read_to_end(&mut bytes))
            .map_err(|why| eyre!("Read chat history {} — {why}", path.display()))
            .and_then(|_| {
                bytes
                    .split_inclusive(|byte| *byte == b'\n')
                    .filter(|line| line.ends_with(b"\n"))
                    .map(|line| {
                        serde_json::from_slice::<ChatHistoryRecord>(line)
                            .map_err(|why| eyre!("Invalid chat history {} — {why}", path.display()))
                            .and_then(|record| {
                                (record.version == RECORD_VERSION)
                                    .then_some(record)
                                    .ok_or_else(|| eyre!("Unsupported chat history version in {}", path.display()))
                            })
                    })
                    .collect()
            })
    }
}
#[cfg(test)]
#[path = "tests/chat_history.rs"]
mod tests;
