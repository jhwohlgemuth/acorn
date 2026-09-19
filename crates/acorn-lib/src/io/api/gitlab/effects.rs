//! Durable reconciliation for typed GitLab effects
use super::{
    add_work_item_label, commit_statuses, publish_commit_status, upsert_merge_request_note, upsert_work_item_note, CommitStatus, CommitStatusState,
    Note, Options, WorkItem,
};
use crate::io::api::webhooks::store::OperationQueue;
use crate::io::api::Configuration;
use crate::io::ApiResult;
use crate::param;
use crate::util::constants::app::APPLICATION;
use core::fmt;
use core::iter::once;
use data_encoding::HEXLOWER;
use ring::digest::{digest, SHA256};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub(crate) struct Effect<'a> {
    parts: Vec<String>,
    kind: &'static str,
    queue: Option<&'a OperationQueue>,
}
impl<'a> Effect<'a> {
    pub(crate) fn new<I, S>(parts: I, kind: &'static str, queue: Option<&'a OperationQueue>) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            parts: parts.into_iter().map(Into::into).collect(),
            kind,
            queue,
        }
    }
    pub(crate) fn begin(&self) -> ApiResult<()> {
        self.queue.map_or(Ok(()), |queue| queue.begin_effect(&self.to_string(), self.kind))
    }
    pub(crate) async fn commit_status(
        &self,
        options: &Options,
        state: CommitStatusState,
        description: &str,
        target_url: Option<&str>,
    ) -> ApiResult<CommitStatus> {
        match self.result() {
            | Ok(Some(status)) => Ok(status),
            | Ok(None) => match self.begin() {
                | Ok(()) => {
                    let name = format!("{APPLICATION}/check");
                    let query_options = options.clone().with_params(vec![param!(KeyValuePair, "name", &name)]);
                    match commit_statuses(&query_options).await {
                        | Ok(statuses) => {
                            let expected = state.to_string();
                            match options.sha().map(|sha| {
                                statuses.into_iter().find(|status| {
                                    status.name == name
                                        && status.sha == sha
                                        && status.status == expected
                                        && status.description.as_deref() == Some(description)
                                        && status.target_url.as_deref() == target_url
                                })
                            }) {
                                | Ok(Some(status)) => self.succeed(&status).map(|()| status),
                                | Ok(None) => match options.sha() {
                                    | Ok(sha) => publish_commit_status(options, None, sha, state, description, target_url)
                                        .await
                                        .and_then(|status| self.succeed(&status).map(|()| status)),
                                    | Err(why) => Err(why),
                                },
                                | Err(why) => Err(why),
                            }
                        }
                        | Err(why) => Err(why),
                    }
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    pub(crate) async fn completion_comment(&self, options: &Options, marker: &str, body: &str) -> ApiResult<Note> {
        match self.result() {
            | Ok(Some(note)) => Ok(note),
            | Ok(None) => match self.begin() {
                | Ok(()) => upsert_work_item_note(options, marker, body)
                    .await
                    .and_then(|note| self.succeed(&note).map(|()| note)),
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    pub(crate) fn content_key<I, S>(parts: I, content: &str) -> Vec<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let fingerprint = HEXLOWER.encode(digest(&SHA256, content.as_bytes()).as_ref());
        parts.into_iter().map(|part| part.as_ref().to_string()).chain(once(fingerprint)).collect()
    }
    pub(crate) async fn merge_request_note(&self, options: &Options, marker: &str, body: &str) -> ApiResult<Note> {
        match self.result() {
            | Ok(Some(note)) => Ok(note),
            | Ok(None) => match self.begin() {
                | Ok(()) => upsert_merge_request_note(options, marker, body)
                    .await
                    .and_then(|note| self.succeed(&note).map(|()| note)),
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    pub(crate) fn result<T: DeserializeOwned>(&self) -> ApiResult<Option<T>> {
        self.queue.map_or(Ok(None), |queue| queue.effect_result(&self.to_string(), self.kind))
    }
    pub(crate) fn succeed<T: Serialize>(&self, result: &T) -> ApiResult<()> {
        self.queue
            .map_or(Ok(()), |queue| queue.succeed_effect(&self.to_string(), self.kind, result))
    }
    pub(crate) async fn work_item_label(&self, options: &Options, label: &str) -> ApiResult<WorkItem> {
        match self.result() {
            | Ok(Some(item)) => Ok(item),
            | Ok(None) => match self.begin() {
                | Ok(()) => add_work_item_label(&options.clone().with_body(label))
                    .await
                    .and_then(|item| self.succeed(&item).map(|()| item)),
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
    pub(crate) async fn work_item_note(&self, options: &Options, marker: &str, body: &str) -> ApiResult<Note> {
        match self.result() {
            | Ok(Some(note)) => Ok(note),
            | Ok(None) => match self.begin() {
                | Ok(()) => upsert_work_item_note(options, marker, body)
                    .await
                    .and_then(|note| self.succeed(&note).map(|()| note)),
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
}
impl fmt::Display for Effect<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{APPLICATION}:{}", self.parts.join(":"))
    }
}
