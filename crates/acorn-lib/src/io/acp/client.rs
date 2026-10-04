//! ACP SDK-backed one-shot inference client.

use super::{AcpSessionEvent, AgentInstallation, AgentProcessConfig, AuthenticationMethod, Error, ProtocolReadiness, SessionObserver};
use crate::{
    agent::{
        InferenceBackend, InferenceCapabilities, InferenceFinishReason, InferenceFuture, InferencePermissionPolicy, InferenceProvenance,
        InferenceRequest, InferenceResult,
    },
    util::constants::app::APPLICATION,
};
use acorn_core::time::UnitOfTime;
use acorn_macros::With;
use acorn_schema::agent::ModelSelector;
use agent_client_protocol::{
    schema::{
        v1::{
            AuthMethod, ContentBlock, ContentChunk, Implementation, InitializeRequest, NewSessionRequest, PermissionOption, PermissionOptionKind,
            PromptRequest, RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse, SelectedPermissionOutcome,
            SessionConfigKind, SessionConfigOption, SessionConfigOptionCategory, SessionConfigSelectOptions, SessionId, SessionNotification,
            SessionUpdate, SetSessionConfigOptionRequest, StopReason, TextContent,
        },
        ProtocolVersion,
    },
    AcpAgent, AcpAgentConfig, Agent, ConnectionTo,
};
use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};
use bon::Builder;
use core::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, MutexGuard};
const DIAGNOSTIC_LIMIT: usize = 4_096;
trait SessionConfigOptionCategoryExt {
    fn is_model(&self) -> bool;
}
#[derive(Clone, Copy, Debug)]
enum ConnectionStage {
    Negotiation = 1,
    Process = 0,
    Protocol = 3,
    Session = 2,
}
/// One-shot ACP inference backend backed by an immutable process descriptor.
#[derive(Builder, Clone, With)]
#[builder(start_fn = init)]
pub struct Client {
    #[builder(default = AgentProcessConfig::opencode(None, ".".into()))]
    config: AgentProcessConfig,
    #[with(some)]
    observer: Option<Arc<dyn SessionObserver>>,
    #[builder(default)]
    offline: bool,
}
#[derive(Debug)]
struct PromptCompletion {
    permission_denied: bool,
    session_id: String,
    stop_reason: StopReason,
    text: String,
}
#[derive(Debug, Default)]
struct StreamState {
    permission_denied: bool,
    text: String,
}
impl AgentProcessConfig {
    fn sdk_agent(&self, installation: &AgentInstallation) -> Result<AcpAgent, Error> {
        self.acp_arguments
            .iter()
            .map(|argument| {
                argument.to_str().map(str::to_string).ok_or_else(|| Error::InvalidConfiguration {
                    agent_id: self.agent_id.clone(),
                    message: "ACP argument must be valid UTF-8 for the ACP SDK".to_string(),
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .and_then(|arguments| {
                self.environment
                    .iter()
                    .map(|(key, value)| {
                        key.to_str()
                            .map(str::to_string)
                            .ok_or_else(|| Error::InvalidConfiguration {
                                agent_id: self.agent_id.clone(),
                                message: "environment key must be valid UTF-8 for the ACP SDK".to_string(),
                            })
                            .and_then(|key| {
                                value
                                    .to_str()
                                    .map(str::to_string)
                                    .ok_or_else(|| Error::InvalidConfiguration {
                                        agent_id: self.agent_id.clone(),
                                        message: "environment value must be valid UTF-8 for the ACP SDK".to_string(),
                                    })
                                    .map(|value| (key, value))
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(|environment| AcpAgent::new(AcpAgentConfig::new(&installation.executable).args(arguments).envs(environment)))
            })
    }
}
impl Client {
    /// Create an ACP client for trusted process launch data.
    pub fn new(config: AgentProcessConfig) -> Self {
        Self::init().config(config).build()
    }
    /// Run the read-only executable and version preflight.
    pub fn check_installation(&self) -> Result<AgentInstallation, Error> {
        self.config.check_installation()
    }
    /// Borrow the immutable process descriptor.
    pub fn config(&self) -> &AgentProcessConfig {
        &self.config
    }
    /// Run one checked ACP inference operation.
    pub async fn infer_checked(&self, request: &InferenceRequest) -> Result<InferenceResult, Error> {
        match self.validate_request(request) {
            | Err(why) => Err(why),
            | Ok(()) => match self.check_installation() {
                | Err(why) => Err(why),
                | Ok(installation) => self.run_prompt(request, installation).await,
            },
        }
    }
    /// Run an explicit ACP initialize handshake without creating a session or sending a prompt.
    pub async fn protocol_readiness(&self) -> Result<ProtocolReadiness, Error> {
        match self.check_installation() {
            | Err(why) => Err(why),
            | Ok(installation) => self.run_readiness(installation).await,
        }
    }
    async fn run_prompt(&self, request: &InferenceRequest, installation: AgentInstallation) -> Result<InferenceResult, Error> {
        match self.config.sdk_agent(&installation) {
            | Err(why) => Err(why),
            | Ok(agent) => {
                let stage = Arc::new(AtomicU8::new(ConnectionStage::Process as u8));
                let stored_error = Arc::new(Mutex::new(None));
                let stream = Arc::new(Mutex::new(StreamState::default()));
                let notification_stream = Arc::clone(&stream);
                let permission_stream = Arc::clone(&stream);
                let notification_observer = self.observer.clone();
                let permission_observer = self.observer.clone();
                let connection_stage = Arc::clone(&stage);
                let connection_error = Arc::clone(&stored_error);
                let prompt = request.prompt.body.clone();
                let model = request.model.clone();
                let policy = self.config.permission_policy;
                let working_directory = self.config.working_directory.clone();
                let operation = agent_client_protocol::Client
                    .builder()
                    .name(APPLICATION)
                    .on_receive_notification(
                        async move |notification: SessionNotification, _connection| {
                            collect_notification(notification, &notification_stream, notification_observer.as_deref());
                            Ok(())
                        },
                        agent_client_protocol::on_receive_notification!(),
                    )
                    .on_receive_request(
                        async move |request: RequestPermissionRequest, responder, _connection| {
                            lock(&permission_stream).permission_denied = true;
                            if let Some(observer) = permission_observer.as_deref() {
                                observer.observe(&AcpSessionEvent::PermissionRequested {
                                    option_count: request.options.len(),
                                });
                            }
                            responder.respond(policy.acp_response(&request.options))
                        },
                        agent_client_protocol::on_receive_request!(),
                    )
                    .connect_with(agent, move |connection: ConnectionTo<Agent>| async move {
                        connection_stage.store(ConnectionStage::Negotiation as u8, Ordering::Release);
                        let initialized = connection.send_request(initialize_request()).block_task().await;
                        match initialized {
                            | Err(why) => Err(why),
                            | Ok(response) if response.protocol_version != ProtocolVersion::V1 => Err(agent_client_protocol::Error::internal_error()
                                .data(format!("unsupported negotiated protocol version {}", response.protocol_version))),
                            | Ok(_) => {
                                connection_stage.store(ConnectionStage::Session as u8, Ordering::Release);
                                let session = connection.send_request(NewSessionRequest::new(&working_directory)).block_task().await;
                                match session {
                                    | Err(why) => Err(why),
                                    | Ok(session) => {
                                        let selection = model
                                            .as_ref()
                                            .map(|model| {
                                                ModelSelector::new(model.clone())
                                                    .ok_or_else(|| Error::ModelUnavailable {
                                                        agent_id: self.config.agent_id.clone(),
                                                        model: model.clone(),
                                                    })
                                                    .and_then(|model| {
                                                        model_selection(
                                                            &self.config.agent_id,
                                                            &session.session_id,
                                                            &model,
                                                            session.config_options.as_deref(),
                                                        )
                                                    })
                                            })
                                            .transpose();
                                        match selection {
                                            | Err(why) => {
                                                *lock(&connection_error) = Some(why);
                                                Err(agent_client_protocol::Error::internal_error().data("requested model is unavailable"))
                                            }
                                            | Ok(selection) => {
                                                let selected = match selection {
                                                    | Some(selection) => connection.send_request(selection).block_task().await.map(|_| ()),
                                                    | None => Ok(()),
                                                };
                                                match selected {
                                                    | Err(why) => Err(why),
                                                    | Ok(()) => {
                                                        connection_stage.store(ConnectionStage::Protocol as u8, Ordering::Release);
                                                        connection
                                                            .send_request(PromptRequest::new(
                                                                session.session_id.clone(),
                                                                vec![ContentBlock::Text(TextContent::new(prompt))],
                                                            ))
                                                            .block_task()
                                                            .await
                                                            .map(|response| {
                                                                let state = lock(&stream);
                                                                PromptCompletion {
                                                                    permission_denied: state.permission_denied,
                                                                    session_id: session.session_id.to_string(),
                                                                    stop_reason: response.stop_reason,
                                                                    text: state.text.clone(),
                                                                }
                                                            })
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });
                match tokio::time::timeout(request.timeout.into(), operation).await {
                    | Err(_) => Err(Error::Timeout {
                        agent_id: self.config.agent_id.clone(),
                        timeout: request.timeout.into(),
                    }),
                    | Ok(Err(why)) => match lock(&stored_error).take() {
                        | Some(stored) => Err(stored),
                        | None => Err(map_connection_error(&self.config, &stage, why)),
                    },
                    | Ok(Ok(completion)) if completion.permission_denied && matches!(completion.stop_reason, StopReason::Cancelled) => {
                        Err(Error::PermissionDenied {
                            agent_id: self.config.agent_id.clone(),
                        })
                    }
                    | Ok(Ok(completion)) => Ok(InferenceResult::from_acp(&self.config, request, completion)),
                }
            }
        }
    }
    async fn run_readiness(&self, installation: AgentInstallation) -> Result<ProtocolReadiness, Error> {
        match self.config.sdk_agent(&installation) {
            | Err(why) => Err(why),
            | Ok(agent) => {
                let stage = Arc::new(AtomicU8::new(ConnectionStage::Process as u8));
                let connection_stage = Arc::clone(&stage);
                let operation = agent_client_protocol::Client
                    .builder()
                    .name(format!("{APPLICATION}-readiness"))
                    .connect_with(agent, move |connection: ConnectionTo<Agent>| async move {
                        connection_stage.store(ConnectionStage::Negotiation as u8, Ordering::Release);
                        connection.send_request(initialize_request()).block_task().await
                    });
                match tokio::time::timeout(self.config.timeout, operation).await {
                    | Err(_) => Err(Error::Timeout {
                        agent_id: self.config.agent_id.clone(),
                        timeout: self.config.timeout,
                    }),
                    | Ok(Err(why)) => Err(map_connection_error(&self.config, &stage, why)),
                    | Ok(Ok(response)) if response.protocol_version != ProtocolVersion::V1 => Err(Error::Negotiation {
                        agent_id: self.config.agent_id.clone(),
                        message: format!("unsupported negotiated protocol version {}", response.protocol_version),
                    }),
                    | Ok(Ok(response)) => {
                        let (agent_name, agent_version) = response
                            .agent_info
                            .map(|agent| (Some(agent.name), Some(agent.version)))
                            .unwrap_or_default();
                        let prompt = response.agent_capabilities.prompt_capabilities;
                        let mcp = response.agent_capabilities.mcp_capabilities;
                        Ok(ProtocolReadiness {
                            agent_name,
                            agent_version,
                            authentication_methods: response
                                .auth_methods
                                .iter()
                                .map(|method| AuthenticationMethod {
                                    id: method.id().to_string(),
                                    name: method.name().to_string(),
                                    terminal: matches!(method, AuthMethod::Terminal(_)),
                                })
                                .collect(),
                            capabilities: InferenceCapabilities {
                                attachments: prompt.audio || prompt.embedded_context || prompt.image,
                                mutating_tools: false,
                                streaming: true,
                                structured_output: false,
                                tools: mcp.http || mcp.sse,
                            },
                            installation,
                            protocol_version: response.protocol_version.as_u16(),
                        })
                    }
                }
            }
        }
    }
    fn validate_request(&self, request: &InferenceRequest) -> Result<(), Error> {
        match (
            &request.agent,
            self.offline,
            request.prompt.body.trim().is_empty(),
            request.timeout.is_zero(),
        ) {
            | (Some(requested), _, _, _) if requested != &self.config.agent_id => Err(Error::AgentMismatch {
                configured: self.config.agent_id.clone(),
                requested: requested.clone(),
            }),
            | (_, true, _, _) => Err(Error::Offline {
                agent_id: self.config.agent_id.clone(),
            }),
            | (_, _, true, _) => Err(Error::Protocol {
                agent_id: self.config.agent_id.clone(),
                message: "inference prompt body cannot be empty".to_string(),
            }),
            | (_, _, _, true) => Err(Error::InvalidConfiguration {
                agent_id: self.config.agent_id.clone(),
                message: "inference timeout must be greater than zero".to_string(),
            }),
            | (_, false, false, false) => Ok(()),
        }
    }
}
impl core::fmt::Debug for Client {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Client")
            .field("config", &self.config)
            .field("observer", &self.observer.as_ref().map(|_| "attached"))
            .field("offline", &self.offline)
            .finish()
    }
}
impl Default for Client {
    fn default() -> Self {
        Self::init().build()
    }
}
impl InferenceBackend for Client {
    fn capabilities(&self) -> InferenceCapabilities {
        InferenceCapabilities {
            attachments: false,
            mutating_tools: false,
            streaming: true,
            structured_output: false,
            tools: false,
        }
    }
    fn infer<'a>(&'a self, request: &'a InferenceRequest) -> InferenceFuture<'a> {
        Box::pin(async move { self.infer_checked(request).await.map_err(color_eyre::Report::new) })
    }
}
impl From<StopReason> for InferenceFinishReason {
    fn from(value: StopReason) -> Self {
        match value {
            | StopReason::EndTurn => InferenceFinishReason::Stop,
            | StopReason::MaxTokens => InferenceFinishReason::Length,
            | StopReason::Refusal => InferenceFinishReason::Refusal,
            | StopReason::Cancelled => InferenceFinishReason::Other("cancelled".to_string()),
            | StopReason::MaxTurnRequests => InferenceFinishReason::Other("max-turn-requests".to_string()),
            | _ => InferenceFinishReason::Other("unknown".to_string()),
        }
    }
}
impl InferencePermissionPolicy {
    fn acp_response(self, options: &[PermissionOption]) -> RequestPermissionResponse {
        let rejected = match self {
            | Self::DenyMutation => options
                .iter()
                .find(|option| matches!(option.kind, PermissionOptionKind::RejectAlways | PermissionOptionKind::RejectOnce)),
        };
        RequestPermissionResponse::new(match rejected {
            | Some(option) => RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(option.option_id.clone())),
            | None => RequestPermissionOutcome::Cancelled,
        })
    }
}
impl InferenceResult {
    fn from_acp(config: &AgentProcessConfig, request: &InferenceRequest, completion: PromptCompletion) -> Self {
        let structured = request.response_schema.as_ref().and_then(|_| serde_json::from_str(&completion.text).ok());
        Self {
            finish_reason: completion.stop_reason.into(),
            provenance: InferenceProvenance {
                agent: Some(config.agent_id.clone()),
                backend: "acp".to_string(),
                model: request.model.clone(),
                request_id: Some(completion.session_id),
            },
            structured,
            text: completion.text,
            usage: None,
        }
    }
}
impl SessionConfigOptionCategoryExt for SessionConfigOptionCategory {
    fn is_model(&self) -> bool {
        self == &Self::Model
    }
}
fn collect_notification(notification: SessionNotification, state: &Arc<Mutex<StreamState>>, observer: Option<&dyn SessionObserver>) {
    if let SessionUpdate::AgentMessageChunk(ContentChunk {
        content: ContentBlock::Text(content),
        ..
    }) = notification.update
    {
        if let Some(observer) = observer {
            observer.observe(&AcpSessionEvent::TextDelta { text: content.text.clone() });
        }
        lock(state).text.push_str(&content.text);
    }
}
fn initialize_request() -> InitializeRequest {
    InitializeRequest::new(ProtocolVersion::V1).client_info(Implementation::new(APPLICATION, env!("CARGO_PKG_VERSION")))
}
fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}
fn map_connection_error(config: &AgentProcessConfig, stage: &AtomicU8, why: agent_client_protocol::Error) -> Error {
    let message = why.to_string().chars().take(DIAGNOSTIC_LIMIT).collect::<String>();
    match message.contains("Process exited") || message.contains("Incoming transport closed") {
        | true => Error::ChildExit {
            agent_id: config.agent_id.clone(),
            message,
        },
        | false => match stage.load(Ordering::Acquire) {
            | value if value == ConnectionStage::Negotiation as u8 => Error::Negotiation {
                agent_id: config.agent_id.clone(),
                message,
            },
            | value if value == ConnectionStage::Session as u8 => Error::Session {
                agent_id: config.agent_id.clone(),
                message,
            },
            | value if value == ConnectionStage::Protocol as u8 => Error::Protocol {
                agent_id: config.agent_id.clone(),
                message,
            },
            | _ => Error::Process {
                agent_id: config.agent_id.clone(),
                message,
            },
        },
    }
}
fn model_selection(
    agent_id: &str,
    session_id: &SessionId,
    model: &ModelSelector,
    options: Option<&[SessionConfigOption]>,
) -> Result<SetSessionConfigOptionRequest, Error> {
    let selection = options.unwrap_or_default().iter().find_map(|option| {
        let is_model = option.category.as_ref().is_some_and(SessionConfigOptionCategoryExt::is_model);
        match (is_model, &option.kind) {
            | (true, SessionConfigKind::Select(select)) => match &select.options {
                | SessionConfigSelectOptions::Grouped(groups) => model
                    .contains(groups.iter().flat_map(|group| group.options.iter()).map(|option| &option.value))
                    .then(|| SetSessionConfigOptionRequest::new(session_id.clone(), option.id.clone(), model.as_str())),
                | SessionConfigSelectOptions::Ungrouped(options) => model
                    .contains(options.iter().map(|option| &option.value))
                    .then(|| SetSessionConfigOptionRequest::new(session_id.clone(), option.id.clone(), model.as_str())),
                | _ => None,
            },
            | _ => None,
        }
    });
    selection.ok_or_else(|| Error::ModelUnavailable {
        agent_id: agent_id.to_string(),
        model: model.to_string(),
    })
}
