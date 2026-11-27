use super::run;
#[cfg(unix)]
use super::{local_client, read_token};
use crate::cli::{arguments::DaemonCommand, Arguments, Commands};
use clap::Parser;

#[tokio::test]
async fn test_absent_config_reports_failure_without_token() {
    let command = DaemonCommand::List {
        config: Some("/nonexistent/acorn-daemon.jsonc".into()),
    };
    let error = run(&command, None, false).await.unwrap_err().to_string();
    assert!(!error.is_empty());
    assert!(!error.contains("Bearer"));
}
#[cfg(unix)]
#[tokio::test]
async fn test_occupied_loopback_port_receives_no_token() {
    use std::net::TcpListener;
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir_in(std::env::var_os("HOME").expect("home directory")).unwrap();
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
#[test]
fn test_parses_local_lifecycle_and_job_commands() {
    for args in [
        vec!["acorn", "daemon", "verify-bundle", "signed.tar.gz"],
        vec!["acorn", "daemon", "install", "--config", "local.jsonc"],
        vec!["acorn", "daemon", "install", "--config", "local.jsonc", "--bundle", "signed.tar.gz"],
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
#[test]
fn test_parses_remote_daemon_commands() {
    for args in [
        vec!["acorn", "daemon", "--host", "ssh://owner@example.org:2222", "list"],
        vec!["acorn", "daemon", "--host", "server", "status", "job"],
        vec![
            "acorn",
            "daemon",
            "--host",
            "server",
            "install",
            "--config",
            "/host/config.jsonc",
            "--bundle",
            "signed.tar.gz",
        ],
        vec![
            "acorn",
            "daemon",
            "--host",
            "server",
            "install",
            "--config",
            "/host/config.jsonc",
            "--release-url",
            "https://example.org/packages",
        ],
    ] {
        assert!(matches!(
            Arguments::try_parse_from(args).unwrap().command,
            Some(Commands::Daemon { host: Some(_), .. })
        ));
    }
    assert!(Arguments::try_parse_from([
        "acorn",
        "daemon",
        "--host",
        "server",
        "install",
        "--config",
        "host.jsonc",
        "--bundle",
        "signed.tar.gz",
        "--release-url",
        "https://example.org",
    ])
    .is_err());
}
#[cfg(unix)]
#[test]
fn test_token_requires_private_regular_file() {
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
mod remote_tests {
    use super::super::remote::{quote, release_base, stage, stderr_excerpt, target, transfer, visible_excerpt, Endpoint, MAX_VISIBLE_BYTES};
    use acorn::io::Remote;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    fn fake_ssh(directory: &Path) -> Endpoint {
        let program = directory.join("ssh");
        let log = directory.join("ssh.log");
        let mode = directory.join("fail");
        let script = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\ncase \"$*\" in\n  *'uname -s; uname -m'*) printf 'Linux\\nx86_64\\n';;\n  *'visible-stderr'*) printf 'remote job list\\n' >&2;;\nesac\nif test -f '{}'; then\n  failure=$(cat '{}')\n  case \"$failure:$*\" in\n    transfer:*'cat >'*) printf 'Cannot write bundle\\n' >&2; exit 77;;\n    install:*'tar -xzf'*) printf 'Remote config is missing\\n' >&2; exit 77;;\n  esac\nfi\n",
            log.display(),
            mode.display(),
            mode.display()
        );
        assert!(fs::write(&program, script).is_ok(), "write fake SSH program");
        assert!(
            fs::set_permissions(&program, fs::Permissions::from_mode(0o700)).is_ok(),
            "make fake SSH program executable"
        );
        Endpoint {
            target: "owner@host".to_string(),
            port: Some(2222),
            program,
        }
    }

    #[test]
    fn test_fake_ssh_detects_target_and_cleans_failed_transfer() {
        let directory = tempfile::tempdir().unwrap();
        let endpoint = fake_ssh(directory.path());
        assert_eq!(target(&endpoint).unwrap(), "x86_64-unknown-linux-musl");
        let archive = directory.path().join("archive");
        let signature = directory.path().join("signature");
        fs::write(&archive, "bundle bytes").unwrap();
        fs::write(&signature, "signature bytes").unwrap();
        let name = "acornd-v0.7.1-x86_64-unknown-linux-musl.tar.gz";
        let config = Path::new("/remote/config's path.jsonc");
        transfer(&endpoint, name, &archive, &signature, config, false).unwrap();
        fs::write(directory.path().join("fail"), "transfer").unwrap();
        let error = transfer(&endpoint, name, &archive, &signature, config, false).unwrap_err().to_string();
        assert!(error.contains("Cannot write bundle"));
        fs::write(directory.path().join("fail"), "install").unwrap();
        let denied = transfer(&endpoint, name, &archive, &signature, config, false).unwrap_err().to_string();
        assert!(denied.contains("Remote config is missing"));
        fs::remove_file(directory.path().join("fail")).unwrap();
        transfer(&endpoint, name, &archive, &signature, config, false).unwrap();
        let log = fs::read_to_string(directory.path().join("ssh.log")).unwrap();
        assert!(log.contains("owner@host"));
        assert!(log.contains("-p 2222"));
        assert!(log.contains("daemon install --config"));
        assert!(log.contains("/remote/config"));
        assert!(log.contains("s path.jsonc"));
        assert!(log.contains("sha256sum"));
        assert_eq!(log.matches("rm -rf").count(), 4);
        assert!(!log.contains("token"));
    }
    #[test]
    fn test_host_and_shell_argument_validation() {
        let remote: Remote = "ssh://owner@example.org:2222/socket".parse().unwrap();
        assert!(Endpoint::parse(&remote).is_err());
        assert_eq!(quote("a'b"), "'a'\"'\"'b'");
        assert!(stage(None, None, "x86_64-unknown-linux-musl", true).is_err());
        let wrong = Path::new("acornd-v0.7.1-aarch64-unknown-linux-musl.tar.gz");
        assert!(stage(Some(wrong), None, "x86_64-unknown-linux-musl", true).is_err());
        assert!(release_base(Some("http://example.org/packages")).is_err());
        assert!(release_base(Some("https://user:secret@example.org/packages")).is_err());
    }
    #[test]
    fn test_remote_stderr_is_bounded_and_redacted() {
        let secret = stderr_excerpt(b"Authorization: Bearer private-value\nMissing config").expect("diagnostic excerpt");
        assert_eq!(secret, "[redacted]");
        assert!(!secret.contains("private-value"));
        let long = stderr_excerpt(&vec![b'x'; 4096]).expect("bounded excerpt");
        assert_eq!(long.len(), 256);
    }
    #[test]
    fn test_successful_remote_control_surfaces_stderr() {
        let directory = tempfile::tempdir().expect("private SSH fixture directory");
        let endpoint = fake_ssh(directory.path());
        assert_eq!(endpoint.execute("visible-stderr", None).expect("quiet SSH output"), "");
        assert_eq!(
            endpoint.execute_visible("visible-stderr", None).expect("visible SSH output"),
            "remote job list\n"
        );
        assert_eq!(visible_excerpt(b"Authorization: Bearer private-value"), "[redacted]");
        assert_eq!(visible_excerpt(&vec![b'x'; MAX_VISIBLE_BYTES + 1]).len(), MAX_VISIBLE_BYTES);
    }
}
