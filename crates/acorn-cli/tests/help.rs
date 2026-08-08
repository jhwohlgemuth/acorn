#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]
//! Integration tests for CLI help behavior.
use assert_cmd::Command;

#[test]
fn test_command_help() {
    Command::cargo_bin("acorn").unwrap().arg("help").assert().success();
}
#[test]
fn test_subcommand_help() {
    Command::cargo_bin("acorn").unwrap().arg("check").arg("--help").assert().success();
}
