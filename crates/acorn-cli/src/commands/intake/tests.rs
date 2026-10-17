use crate::cli::{Arguments, Commands, IntakeCommands};
use clap::Parser;

#[test]
fn test_stage_and_apply_parse_with_explicit_paths() {
    let stage = Arguments::try_parse_from(["acorn", "intake", "stage", "--config", "config.json", "--inbox", "inbox.jsonl"]).unwrap();
    assert!(matches!(
        stage.command,
        Some(Commands::Intake {
            command: IntakeCommands::Stage { .. }
        })
    ));
    let apply = Arguments::try_parse_from([
        "acorn",
        "intake",
        "apply",
        "--config",
        "config.json",
        "--submission-id",
        "10000000-0000-4000-8000-000000000001",
    ])
    .unwrap();
    assert!(matches!(
        apply.command,
        Some(Commands::Intake {
            command: IntakeCommands::Apply { .. }
        })
    ));
}
