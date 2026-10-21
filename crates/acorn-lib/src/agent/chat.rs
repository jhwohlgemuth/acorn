//! Terminal independent state for a metadata conversation.
use crate::util::constants::app::{MAX_CHAT_CONTEXT_BYTES, MAX_CHAT_CONTEXT_TURNS};
use acorn_core::{
    prelude::{String, Vec},
    Location,
};
use acorn_schema::standard::Standard;
use serde::{Deserialize, Serialize};

/// An input to the pure chat state transition.
#[derive(Clone, Debug, PartialEq)]
pub enum ChatAction {
    /// Attach a caller-confirmed metadata target.
    Attach(AttachedTarget),
    /// Record one assistant response.
    AssistantText(String),
    /// Cancel the pending operation without approving it.
    Cancel,
    /// Start a new conversation context.
    Clear,
    /// Record a backend or operation failure.
    Failure(String),
    /// Record a concise result from a metadata operation.
    ToolResult(String),
    /// Record one user message.
    UserText(String),
}
/// One event suitable for rendering or later persistence.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ChatEvent {
    /// A confirmed target became active.
    Attached(AttachedTarget),
    /// The current operation was cancelled.
    Cancelled,
    /// The prior context was closed.
    Cleared,
    /// A backend or operation failed.
    Failed(String),
    /// A user, assistant, or tool turn was recorded.
    Turn(ChatTurn),
}
/// Origin of a conversation turn.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ChatRole {
    /// Model response.
    Assistant,
    /// Concise metadata operation result.
    Tool,
    /// User input.
    User,
}
/// A metadata target whose path and standard were confirmed by the caller.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AttachedTarget {
    /// Exact source chosen by the user.
    pub source: Location,
    /// Detected or selected metadata standard.
    pub standard: Standard,
}
/// Selected backend context with an explicit omission indicator.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatContext {
    /// Confirmed target active for these turns.
    pub target: Option<AttachedTarget>,
    /// Complete, recent turns in their original order.
    pub turns: Vec<ChatTurn>,
    /// Whether older or oversized turns were left out.
    pub omitted: bool,
}
/// Complete in-memory state for one conversation context.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChatState {
    events: Vec<ChatEvent>,
    target: Option<AttachedTarget>,
}
/// Result of applying one chat action.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatTransition {
    /// New state; the previous state is unchanged.
    pub state: ChatState,
    /// Event emitted for the action.
    pub event: ChatEvent,
}
/// One complete text or result turn.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChatTurn {
    /// Origin of the turn.
    pub role: ChatRole,
    /// Complete turn text.
    pub text: String,
}
impl ChatState {
    /// Apply an action without mutating the preceding state.
    pub fn dispatch(&self, action: ChatAction) -> ChatTransition {
        let event = match action {
            | ChatAction::Attach(target) => ChatEvent::Attached(target),
            | ChatAction::AssistantText(text) => ChatEvent::Turn(ChatTurn {
                role: ChatRole::Assistant,
                text,
            }),
            | ChatAction::Cancel => ChatEvent::Cancelled,
            | ChatAction::Clear => ChatEvent::Cleared,
            | ChatAction::Failure(message) => ChatEvent::Failed(message),
            | ChatAction::ToolResult(text) => ChatEvent::Turn(ChatTurn { role: ChatRole::Tool, text }),
            | ChatAction::UserText(text) => ChatEvent::Turn(ChatTurn { role: ChatRole::User, text }),
        };
        let state = match &event {
            | ChatEvent::Cleared => Self::default(),
            | ChatEvent::Attached(target) => Self {
                events: self.events.iter().cloned().chain([event.clone()]).collect(),
                target: Some(target.clone()),
            },
            | _ => Self {
                events: self.events.iter().cloned().chain([event.clone()]).collect(),
                target: self.target.clone(),
            },
        };
        ChatTransition { state, event }
    }
    /// Select bounded recent context while retaining the full event history.
    pub fn context(&self) -> ChatContext {
        let turns = self.events.iter().filter_map(|event| match event {
            | ChatEvent::Turn(turn) => Some(turn),
            | _ => None,
        });
        let total = turns.clone().count();
        let selected = turns.rev().take(MAX_CHAT_CONTEXT_TURNS).scan(0usize, |bytes, turn| {
            let next = bytes.saturating_add(turn.text.len());
            (next <= MAX_CHAT_CONTEXT_BYTES).then(|| {
                *bytes = next;
                turn.clone()
            })
        });
        let recent: Vec<_> = selected.collect::<Vec<_>>().into_iter().rev().collect();
        let omitted = recent.len() < total;
        ChatContext {
            target: self.target.clone(),
            turns: recent,
            omitted,
        }
    }
    /// Return every event recorded in the current context.
    pub fn events(&self) -> &[ChatEvent] {
        &self.events
    }
    /// Return the active confirmed target.
    pub fn target(&self) -> Option<&AttachedTarget> {
        self.target.as_ref()
    }
}

#[cfg(test)]
mod tests;
