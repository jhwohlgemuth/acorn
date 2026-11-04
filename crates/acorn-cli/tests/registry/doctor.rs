//! Direct CLI equivalence for the native doctor report.
use acorn::io::mcp::DoctorInput;
use serde_json::Value;
use std::process::Command;

#[test]
fn test_registry_report_matches_direct_cli_json() {
    let cli = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .args(["doctor", "--report", "--check", "system"])
        .output()
        .expect("run doctor command");
    assert!(cli.status.success(), "{}", String::from_utf8_lossy(&cli.stderr));
    let direct: Value = serde_json::from_slice(&cli.stdout).expect("direct JSON report");
    let routed = serde_json::to_value(
        DoctorInput {
            categories: vec!["system".to_string()],
        }
        .execute()
        .expect("routed report"),
    )
    .expect("serialize routed report");
    assert_eq!(routed, direct);
}
