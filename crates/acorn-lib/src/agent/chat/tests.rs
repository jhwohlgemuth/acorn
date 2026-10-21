use super::*;

#[test]
fn test_attachment_and_clear_are_explicit_events() {
    let target = AttachedTarget {
        source: Location::from("activity.json"),
        standard: Standard::ResearchActivityData,
    };
    let first = ChatState::default().dispatch(ChatAction::Attach(target.clone()));
    assert_eq!(first.event, ChatEvent::Attached(target.clone()));
    assert_eq!(first.state.target(), Some(&target));
    let cleared = first.state.dispatch(ChatAction::Clear);
    assert_eq!(cleared.event, ChatEvent::Cleared);
    assert!(cleared.state.events().is_empty());
    assert!(cleared.state.target().is_none());
    assert_eq!(first.state.events().len(), 1);
}
#[test]
fn test_cancellation_does_not_add_a_turn() {
    let state = ChatState::default().dispatch(ChatAction::Cancel).state;
    assert_eq!(state.events(), &[ChatEvent::Cancelled]);
    assert!(state.context().turns.is_empty());
}
#[test]
fn test_context_keeps_complete_utf8_turns_under_byte_limit() {
    let first = ChatState::default().dispatch(ChatAction::UserText("é".repeat(16_384))).state;
    let state = first.dispatch(ChatAction::AssistantText("next".into())).state;
    let context = state.context();
    assert_eq!(state.events().len(), 2);
    assert_eq!(context.turns.len(), 1);
    assert_eq!(context.turns.first().map(|turn| turn.text.as_str()), Some("next"));
    assert!(context.omitted);
}
#[test]
fn test_context_trims_to_sixteen_turns_without_losing_history() {
    let state = (0..18).fold(ChatState::default(), |state, index| {
        state.dispatch(ChatAction::UserText(index.to_string())).state
    });
    let context = state.context();
    assert_eq!(state.events().len(), 18);
    assert_eq!(context.turns.len(), 16);
    assert_eq!(context.turns.first().map(|turn| turn.text.as_str()), Some("2"));
    assert_eq!(context.turns.last().map(|turn| turn.text.as_str()), Some("17"));
    assert!(context.omitted);
}
#[test]
fn test_event_order_and_failure_retention() {
    let state = [
        ChatAction::UserText("question".into()),
        ChatAction::Failure("timeout".into()),
        ChatAction::ToolResult("check: 1 issue".into()),
    ]
    .into_iter()
    .fold(ChatState::default(), |state, action| state.dispatch(action).state);
    assert_eq!(state.events().len(), 3);
    assert_eq!(
        state.context().turns.iter().map(|turn| turn.role).collect::<Vec<_>>(),
        vec![ChatRole::User, ChatRole::Tool]
    );
    assert!(matches!(state.events().get(1), Some(ChatEvent::Failed(message)) if message == "timeout"));
}
