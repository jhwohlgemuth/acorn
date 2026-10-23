//! Backend-neutral metadata conversation driver for a future CLI or TUI surface
use super::metadata::analyze_metadata_target;
use acorn::agent::{
    ChatAction, ChatState, InferenceBackend, InferenceFinishReason, InferencePermissionPolicy, InferenceRequest, PromptContext, PromptDataFormat,
    PromptHydrator,
};
use acorn_core::{time::Milliseconds, util::redact_sensitive_text, Location};
use acorn_schema::{
    agent::{PromptFileAsset, PromptTemplateConfiguration},
    standard::Standard,
};
use color_eyre::eyre::{eyre, Result};
use core::{
    future::{pending, Future},
    time::Duration,
};
use serde_json::json;
use std::path::PathBuf;

/// One metadata conversation using the shared pure chat state and native check binding.
pub(super) struct ChatDriver<'a, B: InferenceBackend> {
    backend: &'a B,
    state: ChatState,
    root: PathBuf,
    offline: bool,
    agent: Option<String>,
    model: Option<String>,
    timeout: Milliseconds,
    findings: Option<String>,
}
impl<'a, B: InferenceBackend> ChatDriver<'a, B> {
    pub(super) fn new(backend: &'a B, root: PathBuf, offline: bool, agent: Option<String>, model: Option<String>, timeout: Milliseconds) -> Self {
        Self {
            backend,
            state: ChatState::default(),
            root,
            offline,
            agent,
            model,
            timeout,
            findings: None,
        }
    }
    pub(super) fn state(&self) -> &ChatState {
        &self.state
    }
    pub(super) async fn attach(&mut self, source: Location, standard: Option<Standard>) -> Result<()> {
        match analyze_metadata_target(&self.root, &source, standard, self.offline).await {
            | Ok(analysis) if analysis.report.batches.is_empty() => {
                let message = "Metadata analysis returned no result batches";
                self.state = self.state.dispatch(ChatAction::Failure(message.to_string())).state;
                Err(eyre!(message))
            }
            | Ok(analysis) => match analysis.attachment {
                | Some(target) => {
                    self.state = self.state.dispatch(ChatAction::Attach(target)).state;
                    self.findings = Some(analysis.summary.clone());
                    self.state = self.state.dispatch(ChatAction::ToolResult(analysis.summary)).state;
                    Ok(())
                }
                | None => {
                    let message = "The target contains multiple metadata standards; select one explicitly";
                    self.state = self.state.dispatch(ChatAction::ToolResult(analysis.summary)).state;
                    self.state = self.state.dispatch(ChatAction::Failure(message.to_string())).state;
                    Err(eyre!(message))
                }
            },
            | Err(why) => {
                self.state = self.state.dispatch(ChatAction::Failure(redact_sensitive_text(&why.to_string()))).state;
                Err(why)
            }
        }
    }
    pub(super) fn clear(&mut self) {
        self.state = self.state.dispatch(ChatAction::Clear).state;
        self.findings = None;
    }
    pub(super) fn cancel(&mut self) {
        self.state = self.state.dispatch(ChatAction::Cancel).state;
    }
    pub(super) async fn reply(&mut self, message: &str) -> Result<String> {
        self.reply_with_cancel(message, pending()).await
    }
    pub(super) async fn reply_with_cancel<F>(&mut self, message: &str, cancellation: F) -> Result<String>
    where
        F: Future<Output = ()>,
    {
        if message.trim().is_empty() {
            Err(eyre!("A conversation message cannot be empty"))
        } else {
            self.state = self.state.dispatch(ChatAction::UserText(message.to_string())).state;
            let context = self.state.context();
            let data = json!({
                "target": context.target.as_ref().map(|target| json!({"source":target.source.to_string(),"standard":target.standard.to_string()})),
                "findings": self.findings,
                "turns": context.turns,
                "omitted": context.omitted,
                "message": message,
            });
            match PromptHydrator::hydrate_with_format(
                PromptFileAsset::MetadataChat,
                &PromptContext {
                    config: PromptTemplateConfiguration::default(),
                    data,
                },
                PromptDataFormat::Json,
            ) {
                | Err(why) => Err(why),
                | Ok(prompt) => {
                    let request = InferenceRequest {
                        agent: self.agent.clone(),
                        model: self.model.clone(),
                        permission_policy: InferencePermissionPolicy::DenyMutation,
                        prompt,
                        response_schema: None,
                        timeout: self.timeout,
                    };
                    tokio::pin!(cancellation);
                    let result = tokio::select! {
                        biased;
                        | _ = &mut cancellation => {
                            self.cancel();
                            None
                        },
                        | result = tokio::time::timeout(Duration::from(self.timeout), self.backend.infer(&request)) => match result {
                            | Ok(result) => Some(result),
                            | Err(_) => Some(Err(eyre!("Metadata conversation request timed out"))),
                        },
                    };
                    match result {
                        | None => Err(eyre!("Metadata conversation cancelled")),
                        | Some(Err(why)) => {
                            self.state = self.state.dispatch(ChatAction::Failure(redact_sensitive_text(&why.to_string()))).state;
                            Err(why)
                        }
                        | Some(Ok(result)) if result.finish_reason != InferenceFinishReason::Stop => {
                            let message = format!("Metadata conversation ended with {:?}", result.finish_reason);
                            self.state = self.state.dispatch(ChatAction::Failure(message.clone())).state;
                            Err(eyre!(message))
                        }
                        | Some(Ok(result)) => {
                            let answer = result.text.trim().to_string();
                            if answer.is_empty() {
                                let message = "Metadata conversation returned an empty answer";
                                self.state = self.state.dispatch(ChatAction::Failure(message.to_string())).state;
                                Err(eyre!(message))
                            } else {
                                self.state = self.state.dispatch(ChatAction::AssistantText(answer.clone())).state;
                                Ok(answer)
                            }
                        }
                    }
                }
            }
        }
    }
}
