//! Shared slash-command names and parsing for interactive and GitLab surfaces.
use acorn_core::prelude::{String, Vec};
use core::fmt;

const COMMANDS: &[(SlashContext, SlashSuggestion)] = &[
    (
        SlashContext::Dashboard,
        SlashSuggestion {
            id: SlashCommandId::Check,
            name: "/check",
            description: "Browse research activity checks",
        },
    ),
    (
        SlashContext::GitLab,
        SlashSuggestion {
            id: SlashCommandId::Check,
            name: "/acorn check",
            description: "Check a work item",
        },
    ),
    (
        SlashContext::Chat,
        SlashSuggestion {
            id: SlashCommandId::Clear,
            name: "/clear",
            description: "Start a fresh conversation",
        },
    ),
    (
        SlashContext::Dashboard,
        SlashSuggestion {
            id: SlashCommandId::Doctor,
            name: "/doctor",
            description: "Inspect system diagnostics",
        },
    ),
    (
        SlashContext::Chat,
        SlashSuggestion {
            id: SlashCommandId::Exit,
            name: "/exit",
            description: "Leave the conversation",
        },
    ),
    (
        SlashContext::Dashboard,
        SlashSuggestion {
            id: SlashCommandId::Exit,
            name: "/exit",
            description: "Exit ACORN",
        },
    ),
    (
        SlashContext::Dashboard,
        SlashSuggestion {
            id: SlashCommandId::Gather,
            name: "/gather",
            description: "Discover persistent identifiers",
        },
    ),
    (
        SlashContext::GitLab,
        SlashSuggestion {
            id: SlashCommandId::Graduate,
            name: "/acorn graduate",
            description: "Graduate accepted logbook entries",
        },
    ),
    (
        SlashContext::Dashboard,
        SlashSuggestion {
            id: SlashCommandId::Gather,
            name: "/harvest",
            description: "Discover persistent identifiers",
        },
    ),
    (
        SlashContext::Chat,
        SlashSuggestion {
            id: SlashCommandId::Help,
            name: "/help",
            description: "Show chat commands",
        },
    ),
    (
        SlashContext::Chat,
        SlashSuggestion {
            id: SlashCommandId::History,
            name: "/history",
            description: "List saved transcripts",
        },
    ),
    (
        SlashContext::Chat,
        SlashSuggestion {
            id: SlashCommandId::Open,
            name: "/open",
            description: "Attach a file or folder",
        },
    ),
    (
        SlashContext::GitLab,
        SlashSuggestion {
            id: SlashCommandId::Process,
            name: "/acorn process",
            description: "Run a repository workflow",
        },
    ),
    (
        SlashContext::Dashboard,
        SlashSuggestion {
            id: SlashCommandId::Exit,
            name: "/quit",
            description: "Exit ACORN",
        },
    ),
    (
        SlashContext::Dashboard,
        SlashSuggestion {
            id: SlashCommandId::Theme,
            name: "/theme",
            description: "Choose an accessible terminal palette",
        },
    ),
];
/// Parsed command with validated argument shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlashCommand {
    /// Check metadata or open the TUI check screen.
    Check,
    /// Clear the current chat transcript.
    Clear,
    /// Open the TUI diagnostics screen.
    Doctor,
    /// Leave the current interactive interface.
    Exit,
    /// Open the TUI identifier gathering screen.
    Gather,
    /// Graduate the named accepted entries.
    Graduate(Vec<String>),
    /// Show commands for the current interface.
    Help,
    /// List saved chat transcripts.
    History,
    /// Attach the path supplied after `/open`.
    Open(String),
    /// Run the named repository workflow.
    Process(String),
    /// Open the TUI theme picker.
    Theme,
}
/// Stable command identity independent of its spelling or interface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlashCommandId {
    /// Check metadata or open the TUI check screen.
    Check,
    /// Clear the current chat transcript.
    Clear,
    /// Open the TUI diagnostics screen.
    Doctor,
    /// Leave the current interactive interface.
    Exit,
    /// Open the TUI identifier gathering screen.
    Gather,
    /// Graduate accepted logbook entries in GitLab.
    Graduate,
    /// Show commands for the current interface.
    Help,
    /// List saved chat transcripts.
    History,
    /// Attach a local metadata path in chat.
    Open,
    /// Run a GitLab repository workflow.
    Process,
    /// Open the TUI theme picker.
    Theme,
}
/// Interface that accepts a slash command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlashContext {
    /// Interactive metadata conversation.
    Chat,
    /// TUI dashboard command palette.
    Dashboard,
    /// Authenticated GitLab note command.
    GitLab,
}
/// A malformed, unknown, or unavailable slash command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlashParseError(pub String);
/// One available command spelling for help and completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SlashSuggestion {
    /// Command identity to dispatch after menu selection.
    pub id: SlashCommandId,
    /// Slash-prefixed spelling shown in the menu.
    pub name: &'static str,
    /// Brief interface-neutral description.
    pub description: &'static str,
}
impl SlashCommand {
    /// Parse one complete command line under its interface's syntax and availability.
    pub fn parse(input: &str, context: SlashContext) -> Result<Self, SlashParseError> {
        if input.contains(['\r', '\n']) {
            Err(SlashParseError("A slash command must occupy one line".into()))
        } else {
            let source = input.trim();
            let command = match context {
                | SlashContext::GitLab => source
                    .strip_prefix("/acorn")
                    .and_then(|rest| {
                        rest.chars()
                            .next()
                            .filter(|character| character.is_whitespace())
                            .map(|_| rest.trim_start())
                    })
                    .ok_or_else(|| SlashParseError("GitLab commands must start with '/acorn '".into())),
                | _ => source.strip_prefix('/').ok_or_else(|| SlashParseError("Expected a slash command".into())),
            };
            command.and_then(|command| parse_command(command, context))
        }
    }
}
impl core::error::Error for SlashParseError {}
impl fmt::Display for SlashParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
fn parse_command(command: &str, context: SlashContext) -> Result<SlashCommand, SlashParseError> {
    let boundary = command.find(char::is_whitespace).unwrap_or(command.len());
    let (name, rest) = command.split_at(boundary);
    let arguments = rest.trim();
    let name = if context == SlashContext::GitLab {
        name.to_string()
    } else {
        name.to_ascii_lowercase()
    };
    let available = |candidate: &str| {
        COMMANDS.iter().any(|(surface, item)| {
            *surface == context && item.name.strip_prefix(if context == SlashContext::GitLab { "/acorn " } else { "/" }) == Some(candidate)
        })
    };
    if !available(&name) {
        Err(SlashParseError(format!("Unknown or unavailable slash command '/{name}'")))
    } else {
        match name.as_str() {
            | "check" if arguments.is_empty() => Ok(SlashCommand::Check),
            | "clear" if arguments.is_empty() => Ok(SlashCommand::Clear),
            | "doctor" if arguments.is_empty() => Ok(SlashCommand::Doctor),
            | "exit" | "quit" if arguments.is_empty() => Ok(SlashCommand::Exit),
            | "gather" | "harvest" if arguments.is_empty() => Ok(SlashCommand::Gather),
            | "graduate" => {
                let mut identifiers = arguments.split_whitespace().map(str::to_string).collect::<Vec<_>>();
                identifiers.sort();
                identifiers.dedup();
                Ok(SlashCommand::Graduate(identifiers))
            }
            | "help" if arguments.is_empty() => Ok(SlashCommand::Help),
            | "history" if arguments.is_empty() => Ok(SlashCommand::History),
            | "open" if !arguments.is_empty() => {
                let path = arguments
                    .strip_prefix('"')
                    .and_then(|path| path.strip_suffix('"'))
                    .or_else(|| arguments.strip_prefix('\'').and_then(|path| path.strip_suffix('\'')))
                    .unwrap_or(arguments);
                (!path.is_empty())
                    .then(|| SlashCommand::Open(path.to_string()))
                    .ok_or_else(|| SlashParseError("/open requires a path".into()))
            }
            | "process" => match arguments.split_whitespace().collect::<Vec<_>>().as_slice() {
                | [] => Ok(SlashCommand::Process("repository-quality".into())),
                | [workflow] => Ok(SlashCommand::Process((*workflow).to_string())),
                | _ => Err(SlashParseError("/acorn process accepts at most one workflow".into())),
            },
            | _ => Err(SlashParseError(format!("Invalid arguments for '/{name}'"))),
        }
    }
}
/// Suggest available spellings in deterministic name order.
pub fn suggest(context: SlashContext, prefix: &str) -> Vec<SlashSuggestion> {
    let query = prefix.trim().to_ascii_lowercase();
    COMMANDS
        .iter()
        .filter(|(surface, item)| *surface == context && item.name.to_ascii_lowercase().starts_with(&query))
        .map(|(_, item)| *item)
        .collect()
}
