use crate::agent::slash::{suggest, SlashCommand, SlashCommandId, SlashContext};

#[test]
fn test_chat_paths_and_context_restrictions() {
    assert_eq!(
        SlashCommand::parse("/open \"a folder/data.json\"", SlashContext::Chat),
        Ok(SlashCommand::Open("a folder/data.json".into()))
    );
    assert_eq!(SlashCommand::parse("/exit", SlashContext::Chat), Ok(SlashCommand::Exit));
    assert_eq!(SlashCommand::parse("/fix", SlashContext::Chat), Ok(SlashCommand::Fix));
    assert!(SlashCommand::parse("/fix now", SlashContext::Chat).is_err());
    assert!(SlashCommand::parse("/process repository-quality", SlashContext::Chat).is_err());
    assert!(SlashCommand::parse("/open", SlashContext::Chat).is_err());
}
#[test]
fn test_dashboard_suggestions_and_aliases() {
    assert_eq!(
        suggest(SlashContext::Dashboard, "/t").first().map(|item| item.id),
        Some(SlashCommandId::Theme)
    );
    assert_eq!(SlashCommand::parse("/harvest", SlashContext::Dashboard), Ok(SlashCommand::Gather));
    assert_eq!(SlashCommand::parse("/quit", SlashContext::Dashboard), Ok(SlashCommand::Exit));
}
#[test]
fn test_gitlab_uses_exact_prefixed_lines() {
    assert_eq!(SlashCommand::parse("/acorn check", SlashContext::GitLab), Ok(SlashCommand::Check));
    assert_eq!(
        SlashCommand::parse("/acorn process", SlashContext::GitLab),
        Ok(SlashCommand::Process("repository-quality".into()))
    );
    assert_eq!(
        SlashCommand::parse("/acorn graduate b a b", SlashContext::GitLab),
        Ok(SlashCommand::Graduate(vec!["a".into(), "b".into()]))
    );
    assert!(SlashCommand::parse("/check", SlashContext::GitLab).is_err());
    assert!(SlashCommand::parse("Please /acorn check", SlashContext::GitLab).is_err());
    assert!(SlashCommand::parse("/acorn check now", SlashContext::GitLab).is_err());
    assert!(SlashCommand::parse("/acorn Check", SlashContext::GitLab).is_err());
}
