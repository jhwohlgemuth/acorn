//! Inline metadata chat and its backend-neutral conversation driver.
use super::{
    metadata::{analyze_metadata_target, MetadataAnalysis},
    Backend, ConfiguredInference, Infer,
};
use crate::io::interactive::{Input, PromptOutcome, Viewport};
use acorn::agent::{
    slash::{self, SlashCommand, SlashContext},
    ChatAction, ChatEvent, ChatRole, ChatState, InferenceBackend, InferenceFinishReason, InferencePermissionPolicy, InferenceProvenance,
    InferenceRequest, PromptContext, PromptDataFormat, PromptHydrator,
};
use acorn::{
    analyzer::{
        fix::{self, Fix, FixOptions, FixPreviewTarget, FixReport, IntoFixes},
        host::{
            context::AnalysisContext,
            service::{analyze_paths, AnalysisReport},
        },
    },
    io::chat_history::ChatSession,
    util::StringConversion,
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
use std::{
    env::current_dir,
    fs,
    path::{Path, PathBuf},
};

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
    analysis: Option<MetadataAnalysis>,
    provenance: Option<InferenceProvenance>,
}
pub(super) struct NoRecipeRunner;
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
            analysis: None,
            provenance: None,
        }
    }
    pub(super) fn state(&self) -> &ChatState {
        &self.state
    }
    pub(super) async fn attach(&mut self, source: Location, standard: Option<Standard>) -> Result<()> {
        let analysis = analyze_metadata_target(&self.root, &source, standard, self.offline).await;
        self.attach_analysis(analysis)
    }
    fn attach_analysis(&mut self, analysis: Result<MetadataAnalysis>) -> Result<()> {
        match analysis {
            | Ok(analysis) if analysis.report.batches.is_empty() => {
                let message = "Metadata analysis returned no result batches";
                self.state = self.state.dispatch(ChatAction::Failure(message.to_string())).state;
                Err(eyre!(message))
            }
            | Ok(analysis) => match analysis.attachment.clone() {
                | Some(target) => {
                    self.state = self.state.dispatch(ChatAction::Attach(target)).state;
                    self.findings = Some(analysis.summary.clone());
                    self.state = self.state.dispatch(ChatAction::ToolResult(analysis.summary.clone())).state;
                    self.analysis = Some(analysis);
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
        self.analysis = None;
        self.provenance = None;
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
                | Err(why) => {
                    self.state = self.state.dispatch(ChatAction::Failure(redact_sensitive_text(&why.to_string()))).state;
                    Err(why)
                }
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
                            self.provenance = Some(result.provenance.clone());
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
impl<'a> ChatDriver<'a, Backend> {
    async fn interact(&mut self, prompt: Option<&str>, viewport: &mut Viewport, history: &mut ChatSession) -> Result<()> {
        let mut next = viewport
            .push("Metadata chat. /help lists commands; /exit closes the session.")
            .map(|()| false);
        if let Some(prompt) = prompt {
            next = match next {
                | Ok(false) => self.handle_line(prompt, viewport, history).await,
                | other => other,
            };
        }
        loop {
            match next {
                | Ok(true) => break Ok(()),
                | Err(why) => break Err(why),
                | Ok(false) => {
                    next = match viewport.read_line() {
                        | Err(why) => Err(why),
                        | Ok(Input::Cancelled) => {
                            let before = self.state().events().len();
                            self.cancel();
                            self.persist_since(history, before).map(|()| true)
                        }
                        | Ok(Input::Submitted(line)) => self.handle_line(&line, viewport, history).await,
                    };
                }
            }
        }
    }
    async fn handle_line(&mut self, line: &str, viewport: &mut Viewport, history: &mut ChatSession) -> Result<bool> {
        let line = line.trim();
        if line.is_empty() {
            Ok(false)
        } else if line.starts_with('/') {
            match SlashCommand::parse(line, SlashContext::Chat) {
                | Ok(SlashCommand::Exit) => Ok(true),
                | Ok(SlashCommand::Help) => {
                    let commands = slash::suggest(SlashContext::Chat, "")
                        .into_iter()
                        .map(|item| format!("{}: {}", item.name, item.description))
                        .collect::<Vec<_>>()
                        .join("\n");
                    viewport
                        .push(format!("{commands}\nEnter sends; Escape exits; Page Up/Down scrolls."))
                        .map(|()| false)
                }
                | Ok(SlashCommand::History) => history
                    .session_paths()
                    .and_then(|paths| {
                        viewport.push(format!(
                            "Current: {}\nSaved:\n{}",
                            history.path().display(),
                            paths.iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join("\n")
                        ))
                    })
                    .map(|()| false),
                | Ok(SlashCommand::Clear) => history
                    .clear()
                    .and_then(|(archived, current)| {
                        self.clear();
                        viewport.push(format!("Archived: {}\nCurrent: {}", archived.display(), current.display()))
                    })
                    .map(|()| false),
                | Ok(SlashCommand::Open(path)) => self.open_target(&path, viewport, history).await.map(|()| false),
                | Ok(SlashCommand::Fix) => self.review_fixes(viewport, history).await.map(|()| false),
                | Ok(SlashCommand::Check) => {
                    if let Some(target) = self.state().target().cloned() {
                        let before = self.state().events().len();
                        let result = viewport
                            .spin("Checking metadata", async {
                                self.attach(target.source, Some(target.standard))
                                    .await
                                    .map(|()| self.findings.clone().unwrap_or_default())
                            })
                            .await;
                        self.finish_operation(result, before, viewport, history).map(|()| false)
                    } else {
                        viewport.push("Open a metadata file or folder first.").map(|()| false)
                    }
                }
                | Ok(_) => viewport.push("This command is unavailable in metadata chat.").map(|()| false),
                | Err(why) => self.record_failure(viewport, history, &why.to_string()).map(|()| false),
            }
        } else {
            let opened = match (self.state().target(), path_hint(&self.root, line)) {
                | (None, Some(path)) => self.open_target(&path, viewport, history).await,
                | _ => Ok(()),
            };
            match opened {
                | Err(why) => Err(why),
                | Ok(()) => {
                    let before = self.state().events().len();
                    let result = viewport.spin("Thinking", self.reply(line)).await;
                    self.finish_operation(result, before, viewport, history).map(|()| false)
                }
            }
        }
    }
    async fn open_target(&mut self, path: &str, viewport: &mut Viewport, history: &mut ChatSession) -> Result<()> {
        match self.root.join(path).canonicalize() {
            | Err(why) => self.record_failure(viewport, history, &format!("Cannot open {path}: {why}")),
            | Ok(canonical) => match self.root.canonicalize() {
                | Err(why) => Err(why.into()),
                | Ok(root) if !canonical.starts_with(&root) => {
                    self.record_failure(viewport, history, "The selected path is outside the current working directory.")
                }
                | Ok(_) => self.open_canonical_target(canonical, viewport, history).await,
            },
        }
    }
    async fn open_canonical_target(&mut self, canonical: PathBuf, viewport: &mut Viewport, history: &mut ChatSession) -> Result<()> {
        let source = Location::from(canonical.to_string_lossy().as_ref());
        let inspected = viewport
            .spin("Checking target", analyze_metadata_target(&self.root, &source, None, self.offline))
            .await;
        match inspected {
            | Ok(PromptOutcome::Selected(analysis)) => self.select_target_standard(canonical, source, analysis, viewport, history).await,
            | Ok(PromptOutcome::Cancelled) => self.record_cancellation(viewport, history, "Open cancelled."),
            | Err(why) => self.record_failure(viewport, history, &format!("Open failed: {why}")),
        }
    }
    async fn select_target_standard(
        &mut self,
        canonical: PathBuf,
        source: Location,
        analysis: MetadataAnalysis,
        viewport: &mut Viewport,
        history: &mut ChatSession,
    ) -> Result<()> {
        let standards = analysis
            .report
            .batches
            .iter()
            .map(|batch| batch.standard)
            .fold(Vec::new(), |mut values, standard| {
                if !values.contains(&standard) {
                    values.push(standard);
                }
                values
            });
        let selected = if standards.len() > 1 {
            let labels = standards.iter().map(ToString::to_string).collect::<Vec<_>>();
            viewport.choose("Select a metadata standard", &labels, false).map(|choice| match choice {
                | PromptOutcome::Selected(index) => standards.get(index).copied(),
                | PromptOutcome::Cancelled => None,
            })
        } else {
            Ok(standards.first().copied())
        };
        match selected {
            | Err(why) => Err(why),
            | Ok(None) => self.record_cancellation(viewport, history, "Open cancelled."),
            | Ok(Some(standard)) => {
                let shown = viewport
                    .push(format!("Resolved target: {}\nStandard: {standard}", canonical.display()))
                    .and_then(|()| viewport.confirm(&format!("Attach this target as {standard}?"), false));
                match shown {
                    | Err(why) => Err(why),
                    | Ok(PromptOutcome::Cancelled | PromptOutcome::Selected(false)) => self.record_outcome(viewport, history, "Open declined."),
                    | Ok(PromptOutcome::Selected(true)) => {
                        let before = self.state().events().len();
                        let attached = if analysis.attachment.as_ref().is_some_and(|target| target.standard == standard) {
                            self.attach_analysis(Ok(analysis))
                        } else {
                            self.attach(source, Some(standard)).await
                        };
                        self.persist_since(history, before).and_then(|()| match attached {
                            | Ok(()) => viewport.push(self.findings.clone().unwrap_or_default()),
                            | Err(why) => viewport.push(format!("Open failed: {why}")),
                        })
                    }
                }
            }
        }
    }
    fn finish_operation<T: ToString>(
        &mut self,
        result: Result<PromptOutcome<T>>,
        before: usize,
        viewport: &mut Viewport,
        history: &mut ChatSession,
    ) -> Result<()> {
        if matches!(result, Ok(PromptOutcome::Cancelled)) {
            self.cancel();
        }
        self.persist_since(history, before).and_then(|()| match result {
            | Ok(PromptOutcome::Selected(value)) => viewport.push(value.to_string()),
            | Ok(PromptOutcome::Cancelled) => viewport.push("Operation cancelled."),
            | Err(why) => viewport.push(format!("Operation failed: {why}")),
        })
    }
    fn persist_since(&self, history: &mut ChatSession, before: usize) -> Result<()> {
        self.state().events().iter().skip(before).try_for_each(|event| {
            let provenance = matches!(event, ChatEvent::Turn(turn) if turn.role == ChatRole::Assistant)
                .then_some(self.provenance.as_ref())
                .flatten();
            history.append(event, provenance, None)
        })
    }
    async fn review_fixes(&mut self, viewport: &mut Viewport, history: &mut ChatSession) -> Result<()> {
        match self.analysis.as_ref() {
            | None => viewport.push("Open a metadata file or folder first."),
            | Some(analysis) => {
                let paths = analysis
                    .report
                    .batches
                    .iter()
                    .flat_map(|batch| batch.paths.iter().cloned())
                    .collect::<Vec<_>>();
                let mut options = AnalysisContext::default();
                options.analysis.standard = self.state().target().map_or(Standard::ResearchActivityData, |target| target.standard);
                options.execution.common.offline = self.offline;
                options.execution.extension.disable_website_checks = self.offline;
                options.execution.extension.skip_sync = true;
                let checked = viewport
                    .spin("Checking fix sources", async { Ok(analyze_paths(&paths, &options).await) })
                    .await;
                match checked {
                    | Err(why) => Err(why),
                    | Ok(PromptOutcome::Cancelled) => self.record_cancellation(viewport, history, "Fix review cancelled."),
                    | Ok(PromptOutcome::Selected(report)) => self.suggest_and_review_fixes(viewport, history, report).await,
                }
            }
        }
    }
    async fn suggest_and_review_fixes(&mut self, viewport: &mut Viewport, history: &mut ChatSession, report: AnalysisReport) -> Result<()> {
        let fix_options = FixOptions {
            agent: self.agent.clone(),
            model: self.model.clone(),
            timeout: self.timeout,
            ..FixOptions::default()
        };
        let suggested = viewport.spin("Suggesting fixes", report.suggest_fixes(self.backend, &fix_options)).await;
        match suggested {
            | Ok(PromptOutcome::Selected(proposals)) => {
                let eligible = proposals
                    .fixes()
                    .into_iter()
                    .filter(|fix| fix.is_reviewable_document_change())
                    .cloned()
                    .collect::<Vec<_>>();
                if eligible.is_empty() {
                    self.record_outcome(viewport, history, "No reviewable document fixes were proposed.")
                } else {
                    eligible
                        .into_iter()
                        .try_for_each(|fix| self.review_one_fix(viewport, history, &report, fix))
                }
            }
            | Ok(PromptOutcome::Cancelled) => self.record_cancellation(viewport, history, "Fix review cancelled."),
            | Err(why) => self.record_failure(viewport, history, &format!("Fix suggestion failed: {why}")),
        }
    }
    fn review_one_fix(&mut self, viewport: &mut Viewport, history: &mut ChatSession, report: &AnalysisReport, fix: Fix) -> Result<()> {
        let preview = FixReport::from(vec![fix.clone()]).build_preview(report, false);
        match preview.targets.as_slice() {
            | [target] => {
                let path = report
                    .batches
                    .iter()
                    .flat_map(|batch| batch.paths.iter())
                    .find(|path| path.file_name_with_parent() == target.target);
                match path {
                    | None => self.record_outcome(viewport, history, &format!("Fix {} has no authorized source", fix.id)),
                    | Some(path) if !path.starts_with(&self.root) => {
                        self.record_outcome(viewport, history, &format!("Fix {} target is outside the confirmed root", fix.id))
                    }
                    | Some(path) => match target.reviewed_diff() {
                        | Err(why) => self.record_outcome(viewport, history, &format!("Fix {} preview failed: {why}", fix.id)),
                        | Ok(diff) => self.show_fix_review(viewport, history, path, target, &fix, &diff),
                    },
                }
            }
            | _ => self.record_outcome(
                viewport,
                history,
                &format!("Fix {} cannot be previewed: {:?}", fix.id, preview.diagnostics),
            ),
        }
    }
    fn show_fix_review(
        &mut self,
        viewport: &mut Viewport,
        history: &mut ChatSession,
        path: &Path,
        target: &FixPreviewTarget,
        fix: &Fix,
        diff: &str,
    ) -> Result<()> {
        let confirmation = viewport
            .push(format!(
                "{}\nTarget: {}\nFinding: {:?}\nEffects: {:?}\n{diff}",
                fix.summary, target.target, fix.finding, fix.effects
            ))
            .and_then(|()| viewport.confirm(&format!("Apply fix {} to {}?", fix.id, target.target), false));
        match confirmation {
            | Err(why) => Err(why),
            | Ok(PromptOutcome::Cancelled | PromptOutcome::Selected(false)) => {
                self.record_outcome(viewport, history, &format!("Fix {} declined", fix.id))
            }
            | Ok(PromptOutcome::Selected(true)) => self.apply_reviewed_fix(viewport, history, path, target, fix),
        }
    }
    fn apply_reviewed_fix(
        &mut self,
        viewport: &mut Viewport,
        history: &mut ChatSession,
        path: &Path,
        target: &FixPreviewTarget,
        fix: &Fix,
    ) -> Result<()> {
        if !source_matches(path, &self.root, &target.original) {
            self.record_outcome(viewport, history, &format!("Fix {} stale source; no write", fix.id))
        } else {
            match execution_root(path, &target.target) {
                | Err(why) => self.record_outcome(viewport, history, &format!("Fix {} invalid target: {why}", fix.id)),
                | Ok(root) => match fix::Plan::from_fixes(core::slice::from_ref(fix), fix::Policy::confirmed(self.offline)) {
                    | Ok(plan) if plan.preview().len() == 1 => {
                        let result = plan.execute(&root, &NoRecipeRunner, || {
                            fs::read_to_string(path).is_ok_and(|content| content == target.proposed)
                        });
                        let message = match result {
                            | Ok(outcomes) => format!("Fix {}: {outcomes:?}", fix.id),
                            | Err(why) => format!("Fix {} failed: {why}", fix.id),
                        };
                        self.record_outcome(viewport, history, &message)
                    }
                    | Ok(_) => self.record_outcome(viewport, history, &format!("Fix {} has an unsupported plan", fix.id)),
                    | Err(why) => self.record_outcome(viewport, history, &format!("Fix {} invalid: {why}", fix.id)),
                },
            }
        }
    }
    fn record_outcome(&mut self, viewport: &mut Viewport, history: &mut ChatSession, message: &str) -> Result<()> {
        self.record_message(viewport, history, ChatAction::ToolResult(message.to_string()), message)
    }
    fn record_failure(&mut self, viewport: &mut Viewport, history: &mut ChatSession, message: &str) -> Result<()> {
        self.record_message(viewport, history, ChatAction::Failure(message.to_string()), message)
    }
    fn record_cancellation(&mut self, viewport: &mut Viewport, history: &mut ChatSession, message: &str) -> Result<()> {
        self.record_message(viewport, history, ChatAction::Cancel, message)
    }
    fn record_message(&mut self, viewport: &mut Viewport, history: &mut ChatSession, action: ChatAction, message: &str) -> Result<()> {
        let before = self.state().events().len();
        self.state = self.state.dispatch(action).state;
        self.persist_since(history, before).and_then(|()| viewport.push(message))
    }
}
impl fix::Runner for NoRecipeRunner {
    fn run(&self, _binary: &str, _arguments: &[&str]) -> Result<String> {
        Err(eyre!("Chat cannot execute fix recipes"))
    }
}
pub(super) fn execution_root(path: &Path, target: &str) -> Result<PathBuf> {
    Path::new(target)
        .components()
        .try_fold(path.to_path_buf(), |path, _| {
            path.parent().map(Path::to_path_buf).ok_or_else(|| eyre!("Fix target has no safe root"))
        })
        .and_then(|root| {
            (root.join(target) == path)
                .then_some(root)
                .ok_or_else(|| eyre!("Fix target does not match the checked path"))
        })
}
pub(super) fn path_hint(root: &Path, line: &str) -> Option<String> {
    let quoted = line.split('"').skip(1).step_by(2);
    quoted.chain(line.split_whitespace()).find_map(|candidate| {
        let candidate = candidate.trim_matches(|character: char| matches!(character, '"' | '\'' | ',' | ';' | '(' | ')' | ':'));
        let path = Path::new(candidate);
        (candidate != "." && candidate != ".." && !candidate.is_empty() && root.join(path).exists()).then(|| candidate.to_string())
    })
}
pub(super) async fn run(arguments: &Infer, offline: bool) -> Result<()> {
    let setup = validate_interactive(arguments, offline)
        .and_then(|()| ConfiguredInference::from_args(&arguments.inference.shared(), offline))
        .and_then(|configured| current_dir().map(|root| (configured, root)).map_err(Into::into))
        .and_then(|(configured, root)| Viewport::open().map(|viewport| (configured, root, viewport)))
        .and_then(|(configured, root, viewport)| ChatSession::start().map(|history| (configured, root, viewport, history)));
    match setup {
        | Err(why) => Err(why),
        | Ok((configured, root, mut viewport, mut history)) => {
            let mut driver = ChatDriver::new(
                &configured.backend,
                root,
                offline,
                configured.agent.clone(),
                configured.model.clone(),
                configured.timeout,
            );
            let result = driver.interact(arguments.prompt.as_deref(), &mut viewport, &mut history).await;
            let path = history.path().to_path_buf();
            drop(viewport);
            result.map(|()| println!("Chat history: {}", path.display()))
        }
    }
}
pub(super) fn source_matches(path: &Path, root: &Path, expected: &str) -> bool {
    path.canonicalize().is_ok_and(|resolved| resolved == path && resolved.starts_with(root))
        && fs::read_to_string(path).is_ok_and(|content| content == expected)
}
pub(super) fn validate_interactive(arguments: &Infer, offline: bool) -> Result<()> {
    let routed = arguments.route
        || arguments.route_preview
        || arguments.capability.is_some()
        || arguments.input_json.is_some()
        || arguments.binding.is_some()
        || arguments.apply
        || arguments.destination.is_some();
    if arguments.custom_location.is_some() || arguments.threshold.is_some() || arguments.max_rounds.is_some() {
        Err(eyre!(
            "--custom-location, --threshold, and --max-rounds are only valid for Needle inference"
        ))
    } else if routed {
        Err(eyre!("Routing flags cannot be combined with --interactive"))
    } else if offline && matches!(arguments.inference.backend, super::InferenceBackendKind::Acp) {
        Err(eyre!("ACP inference is unavailable in offline mode"))
    } else {
        Ok(())
    }
}
