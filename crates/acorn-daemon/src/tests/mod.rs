#![allow(clippy::indexing_slicing)]

use super::*;
use acorn_service::{InvocationContext, JobId, JobTrigger, MethodName, POLICY_DENIED};
use core::net::SocketAddr;
use serde_json::{json, Value};
use std::net::TcpListener;
use tempfile::TempDir;

fn address() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("ephemeral listener");
    listener.local_addr().expect("listener address")
}
fn config(temp: &TempDir, address: SocketAddr, kind: &str) -> DaemonConfig {
    let root = temp.path().join("project");
    fs::create_dir_all(&root).expect("project root");
    fs::write(root.join("input.json"), "{}").expect("project input");
    let source = json!({
        "schemaVersion": "2.0.0",
        "daemon": {
            "bind": { "grpc": null, "rpc": address.to_string() },
            "jobs": [{
                "id": "daily-check",
                "root": root,
                "input": "input.json",
                "kind": kind,
                "triggers": { "onChange": true }
            }]
        }
    });
    let path = temp.path().join("acorn.jsonc");
    fs::write(&path, source.to_string()).expect("daemon configuration");
    DaemonConfig::load(path).expect("validated daemon configuration")
}
#[cfg(unix)]
fn fake_binary(temp: &TempDir, script: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let path = temp.path().join("acorn");
    fs::write(&path, script).expect("fake binary");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("executable permission");
    path
}
async fn ready(client: &reqwest::Client, address: SocketAddr) {
    tokio::time::timeout(core::time::Duration::from_secs(3), async {
        loop {
            let response = client.get(format!("http://{address}/readyz")).send().await;
            if response.is_ok_and(|response| response.status().is_success()) {
                break;
            }
            tokio::time::sleep(core::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("daemon readiness");
}
async fn rpc(client: &reqwest::Client, address: SocketAddr, method: &str, params: Value) -> Value {
    let response = client
        .post(format!("http://{address}/rpc"))
        .bearer_auth("test-secret")
        .header("content-type", "application/json")
        .body(json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}).to_string())
        .send()
        .await
        .expect("RPC response");
    assert!(response.status().is_success(), "RPC HTTP status: {}", response.status());
    response.json().await.expect("RPC document")
}

#[cfg(unix)]
#[tokio::test]
async fn test_authenticated_job_control_and_history() {
    let temp = tempfile::tempdir().expect("daemon fixture");
    let address = address();
    let config = config(&temp, address, "check");
    let binary = fake_binary(&temp, "#!/bin/sh\nprintf '[]\\n'\n");
    let store = Arc::new(DaemonStore::open(temp.path().join("history.db")).expect("history store"));
    let (stop, stopped) = oneshot::channel::<()>();
    let daemon = tokio::spawn(run_composed(
        config,
        temp.path().join("acorn.jsonc"),
        Arc::clone(&store),
        binary,
        "test-secret".into(),
        None,
        async move { stopped.await.map_err(io::Error::other) },
    ));
    let client = reqwest::Client::new();
    ready(&client, address).await;
    let unauthenticated = client
        .post(format!("http://{address}/rpc"))
        .header("content-type", "application/json")
        .body(json!({"jsonrpc":"2.0","id":1,"method":"acorn.jobs.list","params":{}}).to_string())
        .send()
        .await
        .expect("unauthenticated request");
    assert_eq!(unauthenticated.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert_eq!(
        rpc(&client, address, "acorn.version", json!({})).await["result"]["version"],
        env!("CARGO_PKG_VERSION")
    );
    let listed = rpc(&client, address, "acorn.jobs.list", json!({})).await;
    assert_eq!(listed["result"]["jobs"][0]["id"], "daily-check");
    let id = JobId::new("daily-check").expect("job identifier");
    tokio::time::timeout(core::time::Duration::from_secs(3), async {
        loop {
            if store
                .history(&id, 1)
                .expect("history")
                .first()
                .is_some_and(|run| run.state != "running" && run.state != "queued")
            {
                break;
            }
            tokio::time::sleep(core::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("startup run");
    let status = rpc(&client, address, "acorn.jobs.status", json!({"jobId":"daily-check"})).await;
    assert_eq!(status["result"]["job"]["paused"], false);
    assert!(status["result"]["latest"]["id"].is_number());
    let history = rpc(&client, address, "acorn.jobs.history", json!({"jobId":"daily-check","limit":1})).await;
    assert_eq!(history["result"]["runs"].as_array().expect("run list").len(), 1);
    assert!(history["result"]["runs"][0]["reportPath"].is_string());
    let bounded = rpc(&client, address, "acorn.jobs.history", json!({"jobId":"daily-check","limit":101})).await;
    assert!(bounded["error"].is_object());
    let unknown = rpc(&client, address, "acorn.jobs.status", json!({"jobId":"missing"})).await;
    assert!(unknown["error"].is_object());

    let paused = rpc(&client, address, "acorn.jobs.pause", json!({"jobId":"daily-check"})).await;
    assert_eq!(paused["result"]["paused"], true);
    let declined = rpc(&client, address, "acorn.jobs.run", json!({"jobId":"daily-check"})).await;
    assert_eq!(declined["result"]["accepted"], false);
    let resumed = rpc(&client, address, "acorn.jobs.resume", json!({"jobId":"daily-check"})).await;
    assert_eq!(resumed["result"]["paused"], false);
    let manual = rpc(&client, address, "acorn.jobs.run", json!({"jobId":"daily-check"})).await;
    assert_eq!(manual["result"]["accepted"], true);
    tokio::time::timeout(core::time::Duration::from_secs(3), async {
        loop {
            if store.history(&id, 100).expect("history").iter().any(|run| run.trigger == "manual") {
                break;
            }
            tokio::time::sleep(core::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("manual run reached history");
    let reload = rpc(&client, address, "acorn.jobs.reload", json!({})).await;
    assert_eq!(reload["result"]["generation"], 1);
    stop.send(()).expect("shutdown signal");
    tokio::time::timeout(core::time::Duration::from_secs(3), daemon)
        .await
        .expect("daemon drain deadline")
        .expect("daemon task")
        .expect("clean daemon shutdown");
}
#[tokio::test]
async fn test_job_mutations_require_policy_approval() {
    let temp = tempfile::tempdir().expect("daemon fixture");
    let config = config(&temp, address(), "check");
    let store = Arc::new(DaemonStore::open(temp.path().join("history.db")).expect("history store"));
    let id = config.jobs()[0].id().clone();
    store.register_jobs(core::slice::from_ref(&id)).expect("registered job");
    let (scheduler, handle) = Scheduler::new(
        config.jobs().to_vec(),
        Arc::clone(&store),
        temp.path().join("acorn"),
        RunLimits::default(),
    );
    let (ready, announced) = oneshot::channel();
    let worker = tokio::task::spawn_blocking(move || {
        scheduler.run_with_ready(|| {
            let _ = ready.send(());
        })
    });
    announced.await.expect("scheduler readiness");
    let registry = JobControl::new(temp.path().join("acorn.jsonc"), config, handle.clone(), store)
        .registry()
        .expect("job registry");
    let list = registry
        .invoke(&MethodName::from(["jobs", "list"]), json!({}), InvocationContext::default())
        .await
        .expect("read operation");
    assert_eq!(list["jobs"][0]["id"], "daily-check");
    let denied = registry
        .invoke(
            &MethodName::from(["jobs", "pause"]),
            json!({"jobId":"daily-check"}),
            InvocationContext::default(),
        )
        .await
        .expect_err("mutation must be denied");
    assert_eq!(denied.code, POLICY_DENIED);
    handle.stop();
    worker.await.expect("worker task").expect("worker shutdown");
}
#[cfg(unix)]
#[tokio::test]
async fn test_shutdown_cancels_active_worker_and_drains_listener() {
    let temp = tempfile::tempdir().expect("daemon fixture");
    let address = address();
    let config = config(&temp, address, "check");
    let binary = fake_binary(&temp, "#!/bin/sh\nexec sleep 30\n");
    let store = Arc::new(DaemonStore::open(temp.path().join("history.db")).expect("history store"));
    let (stop, stopped) = oneshot::channel::<()>();
    let daemon = tokio::spawn(run_composed(
        config,
        temp.path().join("acorn.jsonc"),
        Arc::clone(&store),
        binary,
        "test-secret".into(),
        None,
        async move { stopped.await.map_err(io::Error::other) },
    ));
    let client = reqwest::Client::new();
    ready(&client, address).await;
    let id = JobId::new("daily-check").expect("job identifier");
    tokio::time::timeout(core::time::Duration::from_secs(3), async {
        loop {
            if store.history(&id, 1).expect("history").first().is_some_and(|run| run.state == "running") {
                break;
            }
            tokio::time::sleep(core::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("active worker");
    stop.send(()).expect("shutdown signal");
    tokio::time::timeout(core::time::Duration::from_secs(5), daemon)
        .await
        .expect("daemon drain deadline")
        .expect("daemon task")
        .expect("clean daemon shutdown");
    assert_ne!(store.history(&id, 1).expect("history")[0].state, "running");
    assert!(client.get(format!("http://{address}/readyz")).send().await.is_err());
}
#[cfg(windows)]
#[test]
fn test_token_is_rejected_without_acl_verification() {
    let temp = tempfile::tempdir().expect("token fixture");
    let path = temp.path().join("token");
    fs::write(&path, "sample-token\n").expect("token fixture");
    assert!(matches!(read_token(&path), Err(DaemonError::TokenProtection)));
}
#[cfg(unix)]
#[test]
fn test_token_requires_private_regular_file() {
    use std::os::unix::fs::{symlink, PermissionsExt};

    let temp = tempfile::tempdir().expect("token fixture");
    let path = temp.path().join("token");
    fs::write(&path, "sample-token\n").expect("token fixture");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("private token");
    assert_eq!(read_token(&path).expect("protected token"), "sample-token");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("public token");
    assert!(matches!(read_token(&path), Err(DaemonError::TokenProtection)));
    let link = temp.path().join("linked-token");
    symlink(&path, &link).expect("token symlink");
    assert!(matches!(read_token(&link), Err(DaemonError::TokenProtection)));
}
#[tokio::test]
async fn test_worker_failure_prevents_listener_readiness() {
    let temp = tempfile::tempdir().expect("daemon fixture");
    let address = address();
    let config = config(&temp, address, "format");
    let store = Arc::new(DaemonStore::open(temp.path().join("history.db")).expect("history store"));
    let result = run_composed(
        config,
        temp.path().join("acorn.jsonc"),
        store,
        temp.path().join("unused-acorn"),
        "test-secret".into(),
        None,
        core::future::pending(),
    )
    .await;
    assert!(matches!(result, Err(DaemonError::Scheduler(SchedulerError::UnsupportedKind(_)))));
    assert!(reqwest::Client::new().get(format!("http://{address}/readyz")).send().await.is_err());
}
#[tokio::test]
async fn test_worker_failure_stops_ready_listener() {
    let temp = tempfile::tempdir().expect("daemon fixture");
    let address = address();
    let source = json!({"schemaVersion":"2.0.0","daemon":{"bind":{"grpc":null,"rpc":address.to_string()},"jobs":[]}});
    let path = temp.path().join("acorn.jsonc");
    fs::write(&path, source.to_string()).expect("daemon configuration");
    let config = DaemonConfig::load(path).expect("empty daemon configuration");
    let store = Arc::new(DaemonStore::open(temp.path().join("history.db")).expect("history store"));
    let (stop, stopped) = oneshot::channel::<()>();
    let daemon = tokio::spawn(run_composed(
        config,
        temp.path().join("acorn.jsonc"),
        Arc::clone(&store),
        temp.path().join("unused-acorn"),
        "test-secret".into(),
        None,
        async move { stopped.await.map_err(io::Error::other) },
    ));
    let client = reqwest::Client::new();
    ready(&client, address).await;
    let rogue = JobId::new("rogue").expect("rogue identifier");
    store.register_jobs(core::slice::from_ref(&rogue)).expect("rogue registration");
    store.enqueue(&rogue, JobTrigger::Manual).expect("rogue run");
    let result = tokio::time::timeout(core::time::Duration::from_secs(3), daemon)
        .await
        .expect("worker failure deadline")
        .expect("daemon task");
    assert!(matches!(result, Err(DaemonError::Scheduler(SchedulerError::UnknownJob(_)))));
    assert!(client.get(format!("http://{address}/readyz")).send().await.is_err());
    drop(stop);
}
#[cfg(unix)]
mod reload_tests {
    #![allow(clippy::indexing_slicing)]

    use super::*;
    use acorn_service::JobId;
    use core::net::SocketAddr;
    use serde_json::{json, Value};
    use std::net::TcpListener;
    use std::os::unix::fs::PermissionsExt;

    fn address() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").expect("ephemeral listener");
        listener.local_addr().expect("listener address")
    }

    async fn rpc(client: &reqwest::Client, address: SocketAddr, method: &str) -> Value {
        client
            .post(format!("http://{address}/rpc"))
            .bearer_auth("test-secret")
            .header("content-type", "application/json")
            .body(json!({"jsonrpc":"2.0","id":1,"method":method,"params":{}}).to_string())
            .send()
            .await
            .expect("RPC response")
            .json()
            .await
            .expect("RPC document")
    }

    #[tokio::test]
    async fn test_http_reload_rejects_partial_file_and_preserves_inflight_snapshot() {
        let temp = tempfile::tempdir().expect("daemon fixture");
        let address = address();
        let root = temp.path().join("project");
        fs::create_dir(&root).expect("project root");
        fs::write(root.join("old.json"), "{}").expect("old input");
        fs::write(root.join("new.json"), "{}").expect("new input");
        let config_path = temp.path().join("acorn.jsonc");
        let source = |input: &str| {
            json!({
                "schemaVersion": "2.0.0",
                "daemon": {
                    "bind": {"grpc": null, "rpc": address.to_string()},
                    "jobs": [{"id":"daily-check","root":root,"input":input,"kind":"check","triggers":{"onChange":true}}]
                }
            })
        };
        fs::write(&config_path, source("old.json").to_string()).expect("initial configuration");
        let config = DaemonConfig::load(&config_path).expect("validated configuration");
        let binary = temp.path().join("acorn");
        fs::write(
            &binary,
            "#!/bin/sh\nprintf '%s\\n' \"$3\" >> trace\nwhile [ ! -f release ]; do sleep 0.05; done\nprintf '[]\\n'\n",
        )
        .expect("fake binary");
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).expect("executable permission");
        let store = Arc::new(DaemonStore::open(temp.path().join("history.db")).expect("history store"));
        let (stop, stopped) = oneshot::channel::<()>();
        let daemon = tokio::spawn(run_composed(
            config,
            config_path.clone(),
            Arc::clone(&store),
            binary,
            "test-secret".into(),
            None,
            async move { stopped.await.map_err(io::Error::other) },
        ));
        let client = reqwest::Client::new();
        tokio::time::timeout(core::time::Duration::from_secs(3), async {
            loop {
                if client
                    .get(format!("http://{address}/readyz"))
                    .send()
                    .await
                    .is_ok_and(|response| response.status().is_success())
                {
                    break;
                }
                tokio::time::sleep(core::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("daemon readiness");
        let pid = std::process::id();
        tokio::time::timeout(core::time::Duration::from_secs(3), async {
            loop {
                if fs::read_to_string(root.join("trace")).is_ok_and(|trace| trace.lines().count() == 1) {
                    break;
                }
                tokio::time::sleep(core::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("old run started");

        fs::write(&config_path, "{\"schemaVersion\":\"2.0.0\",\"daemon\":{").expect("partial configuration");
        assert!(rpc(&client, address, "acorn.jobs.reload").await["error"].is_object());
        assert_eq!(rpc(&client, address, "acorn.jobs.list").await["result"]["jobs"][0]["id"], "daily-check");
        fs::write(&config_path, source("new.json").to_string()).expect("complete replacement");
        let accepted = tokio::time::timeout(core::time::Duration::from_secs(2), rpc(&client, address, "acorn.jobs.reload"))
            .await
            .expect("reload during active run");
        assert_eq!(accepted["result"]["generation"], 2);
        assert_eq!(rpc(&client, address, "acorn.jobs.reload").await["result"]["generation"], 2);
        assert_eq!(std::process::id(), pid);
        let id = JobId::new("daily-check").expect("job identifier");
        assert_eq!(store.history(&id, 10).expect("in-flight history").len(), 1);

        fs::write(root.join("release"), "go").expect("release old run");
        tokio::time::timeout(core::time::Duration::from_secs(3), async {
            loop {
                let runs = store.history(&id, 10).expect("history");
                if runs.len() == 2 && runs.iter().all(|run| run.state != "running" && run.state != "queued") {
                    break;
                }
                tokio::time::sleep(core::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("old and new runs finished");
        let trace = fs::read_to_string(root.join("trace")).expect("input trace");
        let paths = trace.lines().collect::<Vec<_>>();
        assert_eq!(paths.len(), 2);
        assert!(paths[0].ends_with("old.json"));
        assert!(paths[1].ends_with("new.json"));
        stop.send(()).expect("shutdown signal");
        daemon.await.expect("daemon task").expect("clean daemon shutdown");
    }
}

#[cfg(unix)]
#[expect(clippy::indexing_slicing, reason = "fixture JSON assertions use fixed fields and array slots")]
mod runner_tests {
    use crate::config::DaemonConfig;
    use crate::runner::{run_check_at, run_check_at_until, run_gather_at, RunLimits, RunnerError};
    use acorn_service::{JobTrigger, RunState};
    use acorn_store::daemon::DaemonStore;
    use core::sync::atomic::{AtomicBool, Ordering};
    use core::time::Duration;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::thread;
    use std::time::Instant;

    fn fixture(script: &str) -> (tempfile::TempDir, DaemonConfig, DaemonStore, i64, std::path::PathBuf) {
        fixture_kind(script, "check")
    }
    fn fixture_kind(script: &str, kind: &str) -> (tempfile::TempDir, DaemonConfig, DaemonStore, i64, std::path::PathBuf) {
        let folder = tempfile::tempdir().expect("fixture");
        let root = folder.path().join("project with spaces");
        fs::create_dir(&root).expect("fixture");
        fs::write(root.join("input;echo injected.json"), "{}").expect("fixture");
        let binary = folder.path().join("fake acorn");
        fs::write(&binary, format!("#!/bin/sh\n{script}\n")).expect("fixture");
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).expect("fixture");
        let config_path = folder.path().join(".acorn.jsonc");
        fs::write(
            &config_path,
            format!(
                r#"{{"schemaVersion":"2.0.0","daemon":{{"jobs":[{{"id":"check-one","root":"{}","input":"input;echo injected.json","kind":"{kind}","triggers":{{"onChange":true}}}}]}}}}"#,
                root.display(),
            ),
        )
        .expect("fixture");
        let config = DaemonConfig::load(&config_path).expect("fixture");
        let store = DaemonStore::open(folder.path().join("runs.db")).expect("fixture");
        store
            .register_jobs(&[config.jobs().first().expect("configured job").id().clone()])
            .expect("fixture");
        let id = store
            .enqueue(config.jobs().first().expect("configured job").id(), JobTrigger::Startup)
            .expect("fixture")
            .expect("fixture");
        assert_eq!(store.claim_next().expect("fixture").expect("fixture").id, id);
        (folder, config, store, id, binary)
    }

    #[test]
    fn test_check_retains_nonzero_report_and_exit() {
        let (folder, config, store, id, binary) = fixture("printf '{\"findings\":[1]}'; echo 'validation failed' >&2; exit 2");
        let report = folder.path().join("report.json");
        let result = run_check_at(
            config.jobs().first().expect("configured job"),
            id,
            &store,
            &binary,
            &report,
            RunLimits::default(),
        )
        .expect("fixture");
        assert_eq!(result.exit_code, Some(2));
        assert_eq!(fs::read_to_string(&report).expect("fixture"), "{\"findings\":[1]}");
        let recorded = store.run(id).expect("fixture").expect("fixture");
        assert_eq!(recorded.state, "failed");
        assert_eq!(recorded.exit_code, Some(2));
        assert_eq!(recorded.report_path.as_deref(), Some(report.as_path()));
    }
    #[test]
    fn test_check_times_out_and_bounds_both_streams() {
        let (folder, config, store, id, binary) = fixture("sleep 1");
        let report = folder.path().join("report.json");
        let outcome = run_check_at(
            config.jobs().first().expect("configured job"),
            id,
            &store,
            &binary,
            &report,
            RunLimits {
                timeout: Duration::from_millis(30),
                output_bytes: 20,
            },
        )
        .expect("fixture");
        assert!(outcome.timed_out);
        assert_eq!(store.run(id).expect("fixture").expect("fixture").state, "failed");
        let (folder, config, store, id, binary) = fixture("printf '1234567890123456789012345'; printf 'stderr-over-limit-12345' >&2");
        let report = folder.path().join("report.json");
        let outcome = run_check_at(
            config.jobs().first().expect("configured job"),
            id,
            &store,
            &binary,
            &report,
            RunLimits {
                timeout: Duration::from_secs(5),
                output_bytes: 20,
            },
        )
        .expect("fixture");
        assert!(outcome.truncated);
        assert!(!report.exists());
        assert_eq!(store.run(id).expect("fixture").expect("fixture").state, "failed");
        assert!(RunState::Failed.is_terminal());
    }
    #[test]
    fn test_check_uses_fixed_arguments_and_project_directory() {
        let (folder, config, store, id, binary) =
            fixture(r#"printf '{"args":["%s","%s","%s","%s","%s"],"cwd":"%s"}' "$1" "$2" "$3" "$4" "$5" "$PWD""#);
        let report = folder.path().join("report.json");
        let result = run_check_at(
            config.jobs().first().expect("configured job"),
            id,
            &store,
            &binary,
            &report,
            RunLimits::default(),
        )
        .expect("fixture");
        assert_eq!(result.exit_code, Some(0));
        let value: serde_json::Value = serde_json::from_slice(&fs::read(&report).expect("fixture")).expect("fixture");
        assert_eq!(value["args"][0], "--quiet");
        assert_eq!(value["args"][1], "check");
        assert_eq!(
            value["args"][2],
            config
                .jobs()
                .first()
                .expect("configured job")
                .root()
                .join(config.jobs().first().expect("configured job").input())
                .to_string_lossy()
                .as_ref()
        );
        assert_eq!(value["args"][3], "--output");
        assert_eq!(value["args"][4], "json");
        assert_eq!(
            value["cwd"],
            config.jobs().first().expect("configured job").root().to_string_lossy().as_ref()
        );
        assert!(!config.jobs().first().expect("configured job").root().join("injected.json").exists());
        assert_eq!(store.run(id).expect("fixture").expect("fixture").state, "succeeded");
    }
    #[test]
    fn test_gather_completed_oversized_report_is_removed() {
        let (folder, config, store, id, binary) = fixture_kind("printf '1234567890123456789012345' > \"$4\"", "gather");
        let report = folder.path().join("gather.json");
        let outcome = run_gather_at(
            config.jobs().first().expect("configured job"),
            id,
            &store,
            &binary,
            &report,
            RunLimits {
                timeout: Duration::from_secs(3),
                output_bytes: 20,
            },
        )
        .expect("oversized gather report");
        assert!(outcome.truncated);
        assert!(!report.exists());
        assert_eq!(store.run(id).expect("fixture").expect("fixture").state, "failed");
    }
    #[test]
    fn test_gather_failure_and_report_inside_project_are_recorded() {
        let (folder, config, store, id, binary) = fixture_kind("echo 'gather failed' >&2; exit 3", "gather");
        let report = folder.path().join("gather.json");
        let outcome = run_gather_at(
            config.jobs().first().expect("configured job"),
            id,
            &store,
            &binary,
            &report,
            RunLimits::default(),
        )
        .expect("fixture");
        assert_eq!(outcome.exit_code, Some(3));
        assert!(outcome.report_path.is_none());
        assert_eq!(store.run(id).expect("fixture").expect("fixture").state, "failed");

        let id = store
            .enqueue(config.jobs().first().expect("configured job").id(), JobTrigger::Manual)
            .expect("fixture")
            .expect("fixture");
        assert_eq!(store.claim_next().expect("fixture").expect("fixture").id, id);
        let inside = config.jobs().first().expect("configured job").root().join("gather.json");
        assert!(matches!(
            run_gather_at(
                config.jobs().first().expect("configured job"),
                id,
                &store,
                &binary,
                &inside,
                RunLimits::default()
            ),
            Err(RunnerError::ReportInsideProject)
        ));
        assert!(!inside.exists());
        assert_eq!(store.run(id).expect("fixture").expect("fixture").state, "failed");
    }
    #[test]
    fn test_gather_oversized_report_is_stopped_and_removed() {
        let (folder, config, store, id, binary) = fixture_kind("printf '1234567890123456789012345' > \"$4\"; sleep 5", "gather");
        let report = folder.path().join("gather.json");
        let started = Instant::now();
        let outcome = run_gather_at(
            config.jobs().first().expect("configured job"),
            id,
            &store,
            &binary,
            &report,
            RunLimits {
                timeout: Duration::from_secs(3),
                output_bytes: 20,
            },
        )
        .expect("oversized gather report");
        assert!(outcome.truncated);
        assert!(!outcome.timed_out);
        assert!(started.elapsed() < Duration::from_secs(3));
        assert!(!report.exists());
        let recorded = store.run(id).expect("fixture").expect("fixture");
        assert_eq!(recorded.state, "failed");
        assert!(recorded.report_path.is_none());
    }
    #[test]
    fn test_gather_timeout_is_recorded() {
        let (folder, config, store, id, binary) = fixture_kind("sleep 5", "gather");
        let report = folder.path().join("gather.json");
        let outcome = run_gather_at(
            config.jobs().first().expect("configured job"),
            id,
            &store,
            &binary,
            &report,
            RunLimits {
                timeout: Duration::from_millis(30),
                output_bytes: 1024,
            },
        )
        .expect("timed-out gather");
        assert!(outcome.timed_out);
        assert!(!report.exists());
        assert_eq!(store.run(id).expect("fixture").expect("fixture").state, "failed");
    }
    #[test]
    fn test_gather_uses_fixed_arguments_and_explicit_report() {
        let script = r#"printf '{"args":["%s","%s","%s","%s","%s","%s"]}' "$1" "$2" "$3" "$4" "$5" "$6" > "$4""#;
        let (folder, config, store, id, binary) = fixture_kind(script, "gather");
        let report = folder.path().join("gather.json");
        let result = run_gather_at(
            config.jobs().first().expect("configured job"),
            id,
            &store,
            &binary,
            &report,
            RunLimits::default(),
        )
        .expect("fixture");
        assert_eq!(result.exit_code, Some(0));
        let value: serde_json::Value = serde_json::from_slice(&fs::read(&report).expect("fixture")).expect("fixture");
        assert_eq!(value["args"][0], "gather");
        assert_eq!(
            value["args"][1],
            config
                .jobs()
                .first()
                .expect("configured job")
                .root()
                .join(config.jobs().first().expect("configured job").input())
                .to_string_lossy()
                .as_ref()
        );
        assert_eq!(value["args"][2], "--output");
        assert_eq!(value["args"][3], report.to_string_lossy().as_ref());
        assert_eq!(value["args"][4], "--format");
        assert_eq!(value["args"][5], "json");
        assert_eq!(
            store.run(id).expect("fixture").expect("fixture").report_path.as_deref(),
            Some(report.as_path())
        );
    }
    #[test]
    fn test_shutdown_cancels_the_process_group_and_records_interruption() {
        let (folder, config, store, id, binary) = fixture("sleep 5");
        let report = folder.path().join("report.json");
        let stop = AtomicBool::new(false);
        let started = Instant::now();
        let outcome = thread::scope(|scope| {
            let worker = scope.spawn(|| {
                run_check_at_until(
                    config.jobs().first().expect("configured job"),
                    id,
                    &store,
                    &binary,
                    &report,
                    RunLimits::default(),
                    &stop,
                )
                .expect("cancelled run")
            });
            thread::sleep(Duration::from_millis(50));
            stop.store(true, Ordering::Relaxed);
            worker.join().expect("worker join")
        });
        assert!(outcome.cancelled);
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_eq!(store.run(id).expect("fixture").expect("fixture").state, "interrupted");
    }
}

mod scheduler_tests {
    use crate::config::JobConfig;
    use crate::runner::RunLimits;
    use crate::scheduler::{ActiveRun, Event, EventKindExt, Runtime, Scheduler, SchedulerError, SchedulerSnapshot, Slot};
    use acorn::io::config::DaemonJobConfiguration;
    use acorn_core::time::{Seconds, UnitOfTime};
    use acorn_service::{CoalescingState, JobSchedule, JobTrigger};
    use acorn_store::daemon::DaemonStore;
    use alloc::{collections::BTreeMap, sync::Arc};
    use core::cell::Cell;
    use core::time::Duration;
    use notify::event::{AccessKind, CreateKind, ModifyKind, RemoveKind};
    use notify::EventKind;
    use std::fs;
    use std::thread;
    use std::time::Instant;

    fn fixture_job(root: &std::path::Path, id: &str, input: &str) -> JobConfig {
        let wire: DaemonJobConfiguration = serde_json::from_value(serde_json::json!({
            "id": id, "root": root, "input": input, "kind": "check", "triggers": {"onChange": true}
        }))
        .expect("job configuration");
        JobConfig::try_from(wire).expect("validated job")
    }

    fn fixture_runtime(scheduler: &Scheduler, jobs: Vec<JobConfig>) -> Runtime {
        scheduler
            .store
            .register_jobs(&jobs.iter().map(|job| job.id().clone()).collect::<Vec<_>>())
            .expect("registered jobs");
        let slots = jobs
            .iter()
            .map(|job| (job.id().clone(), Slot::new(job.schedule(), false, 0)))
            .collect::<BTreeMap<_, _>>();
        *scheduler.snapshot.write().expect("snapshot lock") = SchedulerSnapshot {
            generation: 1,
            jobs: jobs.clone(),
        };
        Runtime {
            active: None,
            generation: 1,
            jobs,
            slots,
            started: Instant::now(),
            watch_epoch: 1,
            watchers: Vec::new(),
        }
    }

    #[tokio::test]
    async fn test_concurrent_control_requests_publish_complete_generations() {
        let folder = tempfile::tempdir().expect("fixture directory");
        for input in ["old.json", "first.json", "second.json"] {
            fs::write(folder.path().join(input), "{}").expect("input fixture");
        }
        let old = fixture_job(folder.path(), "stable", "old.json");
        let first = fixture_job(folder.path(), "stable", "first.json");
        let second = fixture_job(folder.path(), "stable", "second.json");
        let id = old.id().clone();
        let store = Arc::new(DaemonStore::open(folder.path().join("runs.db")).expect("store"));
        let (scheduler, handle) = Scheduler::new(vec![old], Arc::clone(&store), folder.path().join("missing-acorn"), RunLimits::default());
        let (ready, announced) = std::sync::mpsc::channel();
        let worker = thread::spawn(move || {
            scheduler.run_with_ready(|| {
                let _ = ready.send(());
            })
        });
        announced.recv_timeout(Duration::from_secs(2)).expect("scheduler readiness");
        let (first_result, second_result, paused, manual) = tokio::join!(
            handle.reload(vec![first.clone()]),
            handle.reload(vec![second.clone()]),
            handle.set_paused(&id, true),
            handle.request_run(&id)
        );
        let mut generations = [first_result.expect("first reload"), second_result.expect("second reload")];
        generations.sort_unstable();
        assert_eq!(generations, [2, 3]);
        assert!(paused.expect("pause"));
        assert!(manual.is_ok(), "manual request must be serialized with reload and pause");
        let snapshot = handle.snapshot().expect("snapshot");
        assert_eq!(snapshot.generation, 3);
        assert!(snapshot.jobs == vec![first] || snapshot.jobs == vec![second]);
        assert_eq!(store.paused(&id).expect("pause state"), Some(true));
        handle.stop();
        assert!(worker.join().expect("worker join").is_ok());
    }
    #[test]
    fn test_failed_run_waits_for_next_trigger_and_missed_intervals_collapse() {
        let schedule = JobSchedule::for_interval(Seconds::new(1)).expect("interval");
        let mut slot = Slot::new(schedule, false, 0);
        assert!(slot.trigger(JobTrigger::Startup));
        slot.start();
        assert_eq!(slot.finish(), None);
        assert_eq!(slot.state, CoalescingState::Idle);
        assert_eq!(slot.due(5_000), vec![JobTrigger::Interval]);
        assert!(slot.due(5_000).is_empty());
        assert!(slot.trigger(JobTrigger::Interval));
        slot.start();
        slot.file_event(5_100);
        assert_eq!(slot.due(5_350), Vec::new());
        slot.paused = true;
        assert_eq!(slot.finish(), None);
        assert_eq!(slot.state, CoalescingState::Idle);
    }
    #[test]
    fn test_nested_file_input_is_watched() {
        let folder = tempfile::tempdir().expect("fixture directory");
        let subdir = folder.path().join("subdir");
        fs::create_dir(&subdir).expect("nested directory");
        let input = subdir.join("input.json");
        fs::write(&input, "{}").expect("input fixture");
        let wire: DaemonJobConfiguration = serde_json::from_value(serde_json::json!({
            "id": "nested", "root": folder.path(), "input": "subdir/input.json", "kind": "check", "triggers": {"onChange": true}
        }))
        .expect("job configuration");
        let job = JobConfig::try_from(wire).expect("validated job");
        let store = Arc::new(DaemonStore::open(folder.path().join("runs.db")).expect("store"));
        let (scheduler, _handle) = Scheduler::new(vec![job.clone()], store, folder.path().join("acorn"), RunLimits::default());
        let watchers = scheduler.watch_inputs(&[job], 1).expect("watch nested input");
        fs::write(subdir.join("other.json"), "{}").expect("unwatched file");
        assert!(scheduler.receiver.recv_timeout(Duration::from_millis(200)).is_err());
        fs::write(&input, "{\"changed\":true}").expect("modified input");
        assert!(matches!(scheduler.receiver.recv_timeout(Duration::from_secs(3)), Ok(Event::File(_, 1))));
        drop(watchers);
    }
    #[test]
    fn test_file_trigger_accepts_mutations_but_not_reads() {
        assert!(!EventKind::Access(AccessKind::Read).is_content_change());
        assert!(!EventKind::Any.is_content_change());
        assert!(EventKind::Create(CreateKind::Any).is_content_change());
        assert!(EventKind::Modify(ModifyKind::Any).is_content_change());
        assert!(EventKind::Remove(RemoveKind::Any).is_content_change());
    }
    #[test]
    fn test_pause_and_resume_gate_triggers() {
        let mut slot = Slot::new(JobSchedule::for_changes(), true, 0);
        assert!(!slot.trigger(JobTrigger::Startup));
        slot.file_event(1);
        assert!(slot.due(500).is_empty());
        slot.paused = false;
        slot.file_event(600);
        assert_eq!(slot.due(850), vec![JobTrigger::FileChange]);
        assert!(slot.trigger(JobTrigger::FileChange));
    }
    #[tokio::test]
    async fn test_pause_handle_persists_state() {
        let folder = tempfile::tempdir().expect("fixture directory");
        fs::write(folder.path().join("input.json"), "{}").expect("input fixture");
        let wire: DaemonJobConfiguration = serde_json::from_value(serde_json::json!({
            "id": "paused-job", "root": folder.path(), "input": "input.json", "kind": "check", "triggers": {"onChange": true}
        }))
        .expect("job configuration");
        let job = JobConfig::try_from(wire).expect("validated job");
        let store = Arc::new(DaemonStore::open(folder.path().join("runs.db")).expect("store"));
        let id = acorn_service::JobId::new("paused-job").expect("job ID");
        let (scheduler, handle) = Scheduler::new(vec![job], Arc::clone(&store), folder.path().join("acorn"), RunLimits::default());
        let (ready, announced) = std::sync::mpsc::channel();
        let worker = thread::spawn(move || {
            scheduler.run_with_ready(|| {
                let _ = ready.send(());
            })
        });
        announced.recv_timeout(Duration::from_secs(2)).expect("scheduler readiness");
        assert!(handle.set_paused(&id, true).await.expect("pause"));
        assert_eq!(store.paused(&id).expect("pause state"), Some(true));
        assert!(handle.set_paused(&id, false).await.expect("resume"));
        assert_eq!(store.paused(&id).expect("pause state"), Some(false));
        handle.stop();
        assert!(worker.join().expect("worker join").is_ok());
    }
    #[test]
    fn test_reload_changed_active_job_uses_old_run_then_one_new_startup() {
        let folder = tempfile::tempdir().expect("fixture directory");
        fs::write(folder.path().join("old.json"), "{}").expect("old input");
        fs::write(folder.path().join("new.json"), "{}").expect("new input");
        let old = fixture_job(folder.path(), "changed", "old.json");
        let new = fixture_job(folder.path(), "changed", "new.json");
        let store = Arc::new(DaemonStore::open(folder.path().join("runs.db")).expect("store"));
        let (scheduler, _handle) = Scheduler::new(vec![old.clone()], Arc::clone(&store), folder.path().join("acorn"), RunLimits::default());
        let mut runtime = fixture_runtime(&scheduler, vec![old.clone()]);
        let id = old.id().clone();
        assert!(runtime.slots.get_mut(&id).expect("slot").trigger(JobTrigger::Startup));
        let run_id = store.enqueue(&id, JobTrigger::Startup).expect("enqueue").expect("run ID");
        assert_eq!(store.claim_next().expect("claim").map(|run| run.id), Some(run_id));
        runtime.slots.get_mut(&id).expect("slot").start();
        runtime.active = Some(ActiveRun {
            id: run_id,
            job: old,
            thread: thread::spawn(|| {}),
        });
        assert_eq!(
            scheduler
                .apply_reload_with(&mut runtime, vec![new.clone()], 500, |_, _| Ok(Vec::new()))
                .expect("reload"),
            2
        );
        assert_eq!(store.history(&id, 10).expect("history").len(), 1);
        assert_eq!(runtime.active.as_ref().expect("active run").job.input(), std::path::Path::new("old.json"));
        store
            .finish(run_id, acorn_service::RunState::Succeeded, Some(0), None, None)
            .expect("old run finish");
        scheduler.finish_one(&mut runtime, run_id, &id, Ok(())).expect("scheduler completion");
        let runs = store.history(&id, 10).expect("history");
        assert_eq!(runs.len(), 2);
        assert_eq!(runs.first().map(|run| run.trigger.as_str()), Some("startup"));
        assert_eq!(runs.get(1).map(|run| run.state.as_str()), Some("succeeded"));
        assert_eq!(runtime.jobs.first().map(JobConfig::input), Some(std::path::Path::new("new.json")));
    }
    #[test]
    fn test_reload_failed_watcher_and_stale_callback_keep_old_generation() {
        let folder = tempfile::tempdir().expect("fixture directory");
        fs::write(folder.path().join("old.json"), "{}").expect("old input");
        fs::write(folder.path().join("new.json"), "{}").expect("new input");
        let old = fixture_job(folder.path(), "changed", "old.json");
        let new = fixture_job(folder.path(), "changed", "new.json");
        let store = Arc::new(DaemonStore::open(folder.path().join("runs.db")).expect("store"));
        let (scheduler, handle) = Scheduler::new(vec![old.clone()], Arc::clone(&store), folder.path().join("acorn"), RunLimits::default());
        let mut runtime = fixture_runtime(&scheduler, vec![old]);
        let id = new.id().clone();
        let old_run = store.enqueue(&id, JobTrigger::Manual).expect("enqueue").expect("run ID");
        let failed_epoch = Cell::new(0);
        let failed = scheduler.apply_reload_with(&mut runtime, vec![new.clone()], 100, |_, epoch| {
            failed_epoch.set(epoch);
            Err(SchedulerError::Watch(notify::Error::generic("fixture watch failure")))
        });
        assert!(matches!(failed, Err(SchedulerError::Watch(_))));
        assert_eq!(runtime.generation, 1);
        assert_eq!(handle.snapshot().expect("snapshot").generation, 1);
        assert_eq!(store.run(old_run).expect("old run").map(|run| run.state), Some("queued".to_string()));
        assert_eq!(
            scheduler
                .apply_reload_with(&mut runtime, vec![new.clone()], 200, |_, _| Ok(Vec::new()))
                .expect("reload"),
            2
        );
        assert_ne!(runtime.watch_epoch, failed_epoch.get());
        scheduler
            .apply_event(&mut runtime, Event::File(id.clone(), failed_epoch.get()), 300)
            .expect("stale event");
        scheduler.apply_event(&mut runtime, Event::File(id.clone(), 1), 300).expect("old event");
        assert_eq!(runtime.slots.get(&id).expect("slot").file_due_ms, None);
        assert_eq!(store.run(old_run).expect("old run").map(|run| run.state), Some("interrupted".to_string()));
        assert_eq!(
            scheduler
                .apply_reload_with(&mut runtime, vec![new], 400, |_, _| Err(SchedulerError::Stopped))
                .expect("duplicate"),
            2
        );
        assert_eq!(store.history(&id, 10).expect("history").len(), 2);
    }
    #[test]
    fn test_reload_preserves_unchanged_and_paused_jobs_and_starts_new_job() {
        let folder = tempfile::tempdir().expect("fixture directory");
        for name in ["stable.json", "paused.json", "new.json"] {
            fs::write(folder.path().join(name), "{}").expect("input fixture");
        }
        let stable = fixture_job(folder.path(), "stable", "stable.json");
        let paused = fixture_job(folder.path(), "paused", "paused.json");
        let added = fixture_job(folder.path(), "added", "new.json");
        let store = Arc::new(DaemonStore::open(folder.path().join("runs.db")).expect("store"));
        let (scheduler, _handle) = Scheduler::new(
            vec![stable.clone(), paused.clone()],
            Arc::clone(&store),
            folder.path().join("acorn"),
            RunLimits::default(),
        );
        let mut runtime = fixture_runtime(&scheduler, vec![stable.clone(), paused.clone()]);
        assert!(store.set_paused(paused.id(), true).expect("pause"));
        runtime.slots.get_mut(paused.id()).expect("paused slot").paused = true;
        assert!(runtime.slots.get_mut(stable.id()).expect("stable slot").trigger(JobTrigger::Manual));
        let stable_run = store.enqueue(stable.id(), JobTrigger::Manual).expect("enqueue").expect("run ID");
        assert_eq!(
            scheduler
                .apply_reload_with(&mut runtime, vec![stable.clone(), paused.clone(), added.clone()], 100, |_, _| Ok(
                    Vec::new()
                ))
                .expect("reload"),
            2
        );
        assert_eq!(
            store.run(stable_run).expect("stable run").map(|run| run.state),
            Some("queued".to_string())
        );
        assert_eq!(runtime.slots.get(stable.id()).expect("stable slot").state, CoalescingState::Queued);
        assert!(runtime.slots.get(paused.id()).expect("paused slot").paused);
        assert!(store.history(paused.id(), 1).expect("paused history").is_empty());
        assert_eq!(
            store.history(added.id(), 1).expect("new history").first().map(|run| run.trigger.as_str()),
            Some("startup")
        );
    }
    #[test]
    fn test_reload_removed_active_job_finishes_without_requeue() {
        let folder = tempfile::tempdir().expect("fixture directory");
        fs::write(folder.path().join("old.json"), "{}").expect("old input");
        let old = fixture_job(folder.path(), "removed", "old.json");
        let id = old.id().clone();
        let store = Arc::new(DaemonStore::open(folder.path().join("runs.db")).expect("store"));
        let (scheduler, handle) = Scheduler::new(vec![old.clone()], Arc::clone(&store), folder.path().join("acorn"), RunLimits::default());
        let mut runtime = fixture_runtime(&scheduler, vec![old.clone()]);
        assert!(runtime.slots.get_mut(&id).expect("slot").trigger(JobTrigger::Startup));
        let run_id = store.enqueue(&id, JobTrigger::Startup).expect("enqueue").expect("run ID");
        assert_eq!(store.claim_next().expect("claim").map(|run| run.id), Some(run_id));
        runtime.slots.get_mut(&id).expect("slot").start();
        runtime.active = Some(ActiveRun {
            id: run_id,
            job: old,
            thread: thread::spawn(|| {}),
        });
        assert_eq!(
            scheduler
                .apply_reload_with(&mut runtime, Vec::new(), 500, |_, _| Ok(Vec::new()))
                .expect("reload"),
            2
        );
        assert!(handle.snapshot().expect("snapshot").jobs.is_empty());
        store
            .finish(run_id, acorn_service::RunState::Succeeded, Some(0), None, None)
            .expect("old run finish");
        scheduler.finish_one(&mut runtime, run_id, &id, Ok(())).expect("scheduler completion");
        assert_eq!(store.history(&id, 10).expect("history").len(), 1);
        assert_eq!(store.run(run_id).expect("old run").map(|run| run.state), Some("succeeded".to_string()));
        assert!(store.claim_next().expect("no further run").is_none());
    }
    #[test]
    fn test_startup_file_debounce_and_interval_collapse() {
        let schedule = JobSchedule::for_change_and_interval(Seconds::new(1)).unwrap();
        let mut slot = Slot::new(schedule, false, 0);
        assert!(slot.trigger(JobTrigger::Startup));
        slot.start();
        slot.file_event(10);
        slot.file_event(100);
        assert!(slot.due(349).is_empty());
        assert_eq!(slot.due(350), vec![JobTrigger::FileChange]);
        assert!(!slot.trigger(JobTrigger::FileChange));
        assert_eq!(slot.due(1000), vec![JobTrigger::Interval]);
        assert!(!slot.trigger(JobTrigger::Interval));
        assert_eq!(slot.finish(), Some(JobTrigger::FileChange));
        assert_eq!(slot.state, CoalescingState::Queued);
        slot.start();
        assert_eq!(slot.finish(), None);
        assert_eq!(slot.state, CoalescingState::Idle);
        assert_eq!(slot.due(4000), vec![JobTrigger::Interval]);
    }
    #[test]
    fn test_unsupported_action_is_rejected_before_startup() {
        let folder = tempfile::tempdir().expect("fixture directory");
        fs::write(folder.path().join("input.json"), "{}").expect("input fixture");
        let wire: DaemonJobConfiguration = serde_json::from_value(serde_json::json!({
            "id": "unsupported", "root": folder.path(), "input": "input.json", "kind": "format", "triggers": {"onChange": true}
        }))
        .expect("job configuration");
        let job = JobConfig::try_from(wire).expect("validated job");
        let store = Arc::new(DaemonStore::open(folder.path().join("runs.db")).expect("store"));
        let (scheduler, _handle) = Scheduler::new(vec![job], store, folder.path().join("acorn"), RunLimits::default());
        assert!(matches!(scheduler.run(), Err(SchedulerError::UnsupportedKind(_))));
    }
    #[test]
    fn test_watched_file_emits_an_event_and_other_files_do_not() {
        let folder = tempfile::tempdir().expect("fixture directory");
        let input = folder.path().join("input.json");
        fs::write(&input, "{}").expect("input fixture");
        let wire: DaemonJobConfiguration = serde_json::from_value(serde_json::json!({
            "id": "watched", "root": folder.path(), "input": "input.json", "kind": "check", "triggers": {"onChange": true}
        }))
        .expect("job configuration");
        let job = JobConfig::try_from(wire).expect("validated job");
        let store = Arc::new(DaemonStore::open(folder.path().join("runs.db")).expect("store"));
        let (scheduler, _handle) = Scheduler::new(vec![job.clone()], store, folder.path().join("acorn"), RunLimits::default());
        let watchers = scheduler.watch_inputs(&[job], 1).expect("watch input");
        fs::read_to_string(&input).expect("read watched input");
        assert!(scheduler.receiver.recv_timeout(Duration::from_millis(200)).is_err());
        fs::write(folder.path().join("other.json"), "{}").expect("unwatched file");
        assert!(scheduler.receiver.recv_timeout(Duration::from_millis(200)).is_err());
        fs::write(&input, "{\"changed\":true}").expect("modified input");
        assert!(matches!(scheduler.receiver.recv_timeout(Duration::from_secs(3)), Ok(Event::File(_, 1))));
        drop(watchers);
    }
    #[test]
    fn test_worker_stops_cleanly_without_jobs() {
        let folder = tempfile::tempdir().expect("fixture directory");
        let store = Arc::new(DaemonStore::open(folder.path().join("runs.db")).expect("store"));
        let (scheduler, handle) = Scheduler::new(Vec::new(), store, folder.path().join("acorn"), RunLimits::default());
        let worker = thread::spawn(move || scheduler.run());
        handle.stop();
        assert!(worker.join().expect("worker join").is_ok());
    }
}

mod store_tests {
    use acorn_service::registry::DurableOperations;
    use acorn_service::{JobId, JobTrigger, RunState};
    use acorn_store::daemon::{DaemonStore, StoreError};
    use alloc::sync::Arc;
    use tempfile::tempdir;

    fn job() -> JobId {
        JobId::new("example").unwrap()
    }

    #[test]
    fn test_activation_preserves_running_history_and_pause_while_retiring_old_queues() {
        let folder = tempdir().unwrap();
        let store = DaemonStore::open(folder.path().join("history.db")).unwrap();
        let active = JobId::new("active").unwrap();
        let removed = JobId::new("removed").unwrap();
        let changed = JobId::new("changed").unwrap();
        let paused = JobId::new("paused").unwrap();
        let added = JobId::new("added").unwrap();
        store
            .register_jobs(&[active.clone(), removed.clone(), changed.clone(), paused.clone()])
            .unwrap();
        let active_run = store.enqueue(&active, JobTrigger::Startup).unwrap().unwrap();
        assert_eq!(store.claim_next().unwrap().map(|run| run.id), Some(active_run));
        let removed_run = store.enqueue(&removed, JobTrigger::Manual).unwrap().unwrap();
        let changed_run = store.enqueue(&changed, JobTrigger::Manual).unwrap().unwrap();
        assert!(store.set_paused(&paused, true).unwrap());

        store
            .activate_jobs(
                &[active.clone(), changed.clone(), paused.clone(), added.clone()],
                core::slice::from_ref(&changed),
                &[changed.clone(), paused.clone(), added.clone()],
            )
            .unwrap();
        assert_eq!(store.run(active_run).unwrap().unwrap().state, "running");
        assert_eq!(store.run(removed_run).unwrap().unwrap().state, "interrupted");
        assert_eq!(store.run(changed_run).unwrap().unwrap().state, "interrupted");
        assert_eq!(store.paused(&paused).unwrap(), Some(true));
        assert!(!store.replay(&removed_run.to_string()).unwrap());
        assert!(store.claim_next().unwrap().is_none());
        store.finish(active_run, RunState::Succeeded, Some(0), None, None).unwrap();
        assert_eq!(store.run(active_run).unwrap().unwrap().state, "succeeded");
        assert_eq!(
            store.history(&changed, 2).unwrap().first().map(|run| run.trigger.as_str()),
            Some("startup")
        );
        assert_eq!(store.history(&added, 1).unwrap().first().map(|run| run.trigger.as_str()), Some("startup"));
        assert!(store.history(&paused, 1).unwrap().is_empty());
    }
    #[test]
    fn test_activation_rolls_back_when_replacement_is_invalid() {
        let folder = tempdir().unwrap();
        let store = DaemonStore::open(folder.path().join("history.db")).unwrap();
        let id = job();
        store.register_jobs(core::slice::from_ref(&id)).unwrap();
        let queued = store.enqueue(&id, JobTrigger::Manual).unwrap().unwrap();
        assert!(store.activate_jobs(&[id.clone(), id.clone()], core::slice::from_ref(&id), &[]).is_err());
        assert_eq!(store.run(queued).unwrap().unwrap().state, "queued");
        assert_eq!(store.claim_next().unwrap().map(|run| run.id), Some(queued));
    }
    #[test]
    fn test_bounds_terminal_history() {
        let folder = tempdir().unwrap();
        let store = DaemonStore::open(folder.path().join("history.db")).unwrap();
        store.register_jobs(&[job()]).unwrap();
        for _ in 0..105 {
            let id = store.enqueue(&job(), JobTrigger::Interval).unwrap().unwrap();
            assert_eq!(store.claim_next().unwrap().unwrap().id, id);
            store.finish(id, RunState::Succeeded, Some(0), None, None).unwrap();
        }
        assert_eq!(store.history(&job(), 300).unwrap().len(), 100);
        assert!(store.run(1).unwrap().is_none());
    }
    #[test]
    fn test_claims_one_run_and_replays_by_run_id() {
        let folder = tempdir().unwrap();
        let store = DaemonStore::open(folder.path().join("history.db")).unwrap();
        store.register_jobs(&[job()]).unwrap();
        let first = store.enqueue(&job(), JobTrigger::Startup).unwrap().unwrap();
        let second = store.enqueue(&job(), JobTrigger::Interval).unwrap().unwrap();
        assert_eq!(store.claim_next().unwrap().unwrap().id, first);
        assert!(store.claim_next().unwrap().is_none());
        store.finish(first, RunState::Succeeded, Some(0), None, None).unwrap();
        assert_eq!(store.claim_next().unwrap().unwrap().id, second);
        store.finish(second, RunState::Failed, Some(2), None, Some("check failed")).unwrap();
        assert_eq!(store.state(&first.to_string()).unwrap().as_deref(), Some("succeeded"));
        assert!(store.replay(&second.to_string()).unwrap());
        assert_eq!(store.history(&job(), 3).unwrap().first().map(|run| run.trigger.as_str()), Some("replay"));
    }
    #[test]
    fn test_concurrent_claims_allow_one_global_runner() {
        let folder = tempdir().unwrap();
        let store = Arc::new(DaemonStore::open(folder.path().join("history.db")).unwrap());
        store.register_jobs(&[job()]).unwrap();
        store.enqueue(&job(), JobTrigger::Startup).unwrap();
        store.enqueue(&job(), JobTrigger::Interval).unwrap();
        let threads = (0..8)
            .map(|_| {
                let store = Arc::clone(&store);
                std::thread::spawn(move || store.claim_next().unwrap())
            })
            .collect::<Vec<_>>();
        let claims = threads.into_iter().filter_map(|thread| thread.join().unwrap()).collect::<Vec<_>>();
        assert_eq!(claims.len(), 1);
    }
    #[test]
    fn test_pause_skips_queued_work_and_reopen_interrupts_it() {
        let folder = tempdir().unwrap();
        let path = folder.path().join("history.db");
        let store = DaemonStore::open(&path).unwrap();
        store.register_jobs(&[job()]).unwrap();
        let id = store.enqueue(&job(), JobTrigger::Startup).unwrap().unwrap();
        assert!(store.set_paused(&job(), true).unwrap());
        assert!(store.claim_next().unwrap().is_none());
        drop(store);
        let reopened = DaemonStore::open(path).unwrap();
        assert_eq!(reopened.run(id).unwrap().map(|run| run.state), Some("interrupted".to_string()));
        assert_eq!(reopened.paused(&job()).unwrap(), Some(true));
    }
    #[test]
    fn test_rejects_newer_schema() {
        let folder = tempdir().unwrap();
        let path = folder.path().join("history.db");
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection.pragma_update(None, "user_version", 2).unwrap();
        assert!(matches!(DaemonStore::open(path), Err(StoreError::Schema(2))));
    }
    #[test]
    fn test_removed_job_queued_runs_are_interrupted() {
        let folder = tempdir().unwrap();
        let store = DaemonStore::open(folder.path().join("history.db")).unwrap();
        store.register_jobs(&[job()]).unwrap();
        let run = store.enqueue(&job(), JobTrigger::Manual).unwrap().unwrap();
        store.register_jobs(&[]).unwrap();
        assert_eq!(store.state(&run.to_string()).unwrap().as_deref(), Some("interrupted"));
        assert!(!store.replay(&run.to_string()).unwrap());
        assert!(store.claim_next().unwrap().is_none());
    }
    #[test]
    fn test_reopens_and_recovers_active_run() {
        let folder = tempdir().unwrap();
        let path = folder.path().join("history.db");
        let store = DaemonStore::open(&path).unwrap();
        store.register_jobs(&[job()]).unwrap();
        let id = store.enqueue(&job(), JobTrigger::Startup).unwrap().unwrap();
        assert_eq!(store.claim_next().unwrap().unwrap().id, id);
        drop(store);
        let reopened = DaemonStore::open(&path).unwrap();
        assert_eq!(reopened.run(id).unwrap().unwrap().state, "interrupted");
        assert_eq!(reopened.paused(&job()).unwrap(), Some(false));
        assert!(reopened.set_paused(&job(), true).unwrap());
        drop(reopened);
        assert_eq!(DaemonStore::open(&path).unwrap().paused(&job()).unwrap(), Some(true));
    }
    #[test]
    fn test_replay_rejects_job_removed_after_restart() {
        let folder = tempdir().unwrap();
        let path = folder.path().join("history.db");
        let first = DaemonStore::open(&path).unwrap();
        first.register_jobs(&[job()]).unwrap();
        let run = first.enqueue(&job(), JobTrigger::Startup).unwrap().unwrap();
        assert_eq!(first.claim_next().unwrap().map(|record| record.id), Some(run));
        first.finish(run, RunState::Succeeded, Some(0), None, None).unwrap();
        drop(first);
        let restarted = DaemonStore::open(&path).unwrap();
        restarted.register_jobs(&[]).unwrap();
        assert_eq!(restarted.state(&run.to_string()).unwrap().as_deref(), Some("succeeded"));
        assert!(!restarted.replay(&run.to_string()).unwrap());
        assert!(restarted.claim_next().unwrap().is_none());
    }
    #[test]
    fn test_separate_connections_cannot_claim_the_same_run() {
        let folder = tempdir().unwrap();
        let path = folder.path().join("history.db");
        let first = DaemonStore::open(&path).unwrap();
        let second = DaemonStore::open(&path).unwrap();
        first.register_jobs(&[job()]).unwrap();
        let id = first.enqueue(&job(), JobTrigger::Startup).unwrap().unwrap();
        assert_eq!(first.claim_next().unwrap().map(|run| run.id), Some(id));
        assert!(second.claim_next().unwrap().is_none());
    }
}
