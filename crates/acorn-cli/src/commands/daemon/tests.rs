use super::run;
#[cfg(unix)]
use super::{local_client, read_token};
use crate::cli::{arguments::DaemonCommand, Arguments, Commands};
use clap::Parser;

#[test]
fn parses_local_lifecycle_and_job_commands() {
    for args in [
        vec!["acorn", "daemon", "install", "--config", "local.jsonc"],
        vec!["acorn", "daemon", "uninstall"],
        vec!["acorn", "daemon", "start"],
        vec!["acorn", "daemon", "stop"],
        vec!["acorn", "daemon", "restart"],
        vec!["acorn", "daemon", "service-status"],
        vec!["acorn", "daemon", "list"],
        vec!["acorn", "daemon", "status", "job"],
        vec!["acorn", "daemon", "history", "job", "--limit", "10"],
        vec!["acorn", "daemon", "run", "job"],
        vec!["acorn", "daemon", "pause", "job"],
        vec!["acorn", "daemon", "resume", "job"],
        vec!["acorn", "daemon", "reload"],
    ] {
        assert!(matches!(Arguments::try_parse_from(args).unwrap().command, Some(Commands::Daemon { .. })));
    }
    assert!(Arguments::try_parse_from(["acorn", "daemon", "install"]).is_err());
    assert!(Arguments::try_parse_from(["acorn", "daemon", "history", "job", "--limit", "101"]).is_err());
}
#[tokio::test]
async fn absent_config_reports_failure_without_token() {
    let command = DaemonCommand::List {
        config: Some("/nonexistent/acorn-daemon.jsonc".into()),
    };
    let error = run(&command).await.unwrap_err().to_string();
    assert!(!error.is_empty());
    assert!(!error.contains("Bearer"));
}
#[cfg(unix)]
#[test]
fn token_requires_private_regular_file() {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir().unwrap();
    let token = directory.path().join("token");
    std::fs::write(&token, "super-secret\n").unwrap();
    std::fs::set_permissions(&token, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(read_token(&token).is_err());
    std::fs::set_permissions(&token, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(read_token(&token).unwrap(), "super-secret");
    let link = directory.path().join("link");
    std::os::unix::fs::symlink(&token, &link).unwrap();
    assert!(read_token(&link).is_err());
}
#[cfg(unix)]
#[tokio::test]
async fn test_occupied_loopback_port_receives_no_token() {
    use std::net::TcpListener;
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir_in(std::env::current_dir().unwrap()).unwrap();
    let data = directory.path().join("daemon");
    acorn_server::local::prepare_private_dir(&data).unwrap();
    let token = data.join("token");
    std::fs::write(&token, "private-bearer-token\n").unwrap();
    std::fs::set_permissions(&token, std::fs::Permissions::from_mode(0o600)).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let client = local_client(listener.local_addr().unwrap(), &data).unwrap();
    let result = client.call::<_, serde_json::Value>("acorn.jobs.list", &serde_json::json!({})).await;
    assert!(result.is_err());
    assert_eq!(listener.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
    std::fs::set_permissions(&data, std::fs::Permissions::from_mode(0o777)).unwrap();
    let error = local_client(listener.local_addr().unwrap(), &data).err().unwrap().to_string();
    assert!(error.contains("control directory"));
}
