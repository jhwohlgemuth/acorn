use super::*;
use acorn_service::{MethodName, OperationDefinition, OperationEffects};
use serde_json::{json, Value};
use tokio::sync::{oneshot, Notify};

fn config(address: core::net::SocketAddr) -> ServerConfig {
    ServerConfig::resolve_with_token(
        crate::ServerArgs {
            disable_grpc: true,
            json_rpc_bind: Some(address),
            ..crate::ServerArgs::default()
        },
        "injected-secret",
    )
    .expect("server configuration")
}
fn free_address() -> core::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("ephemeral listener");
    listener.local_addr().expect("ephemeral address")
}
async fn wait_for_ready(client: &reqwest::Client, address: core::net::SocketAddr) {
    let url = format!("http://{address}/readyz");
    tokio::time::timeout(core::time::Duration::from_secs(2), async {
        loop {
            if client.get(&url).send().await.is_ok_and(|response| response.status().is_success()) {
                break;
            }
            tokio::time::sleep(core::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("server readiness deadline");
}
#[tokio::test]
async fn test_injected_operation_and_external_shutdown() {
    let address = free_address();
    let definition =
        OperationDefinition::new::<Value, Value>(MethodName::from(["test", "echo"]), OperationEffects::default()).expect("operation definition");
    let registry = OperationRegistry::default()
        .register_definition(definition, |input: Value, _| async move { Ok(input) })
        .expect("injected operation");
    let (stop, stopped) = oneshot::channel::<()>();
    let server = tokio::spawn(serve_with_registry(config(address), registry, async move {
        stopped.await.map_err(Into::into)
    }));
    let client = reqwest::Client::new();
    wait_for_ready(&client, address).await;
    let response = client
        .post(format!("http://{address}/rpc"))
        .bearer_auth("injected-secret")
        .header("content-type", "application/json")
        .body(json!({"jsonrpc":"2.0","id":1,"method":"acorn.test.echo","params":{"value":"injected"}}).to_string())
        .send()
        .await
        .expect("injected request");
    assert!(response.status().is_success());
    let result: Value = serde_json::from_str(&response.text().await.expect("response body")).expect("injected result");
    assert_eq!(result.get("result"), Some(&json!({"value":"injected"})));
    stop.send(()).expect("shutdown receiver");
    tokio::time::timeout(core::time::Duration::from_secs(2), server)
        .await
        .expect("server shutdown deadline")
        .expect("server task")
        .expect("graceful shutdown");
}
#[tokio::test]
async fn test_listener_bind_failure_is_reported() {
    let occupied = TcpListener::bind("127.0.0.1:0").expect("occupied listener");
    let address = occupied.local_addr().expect("occupied address");
    let result = serve_with_registry(config(address), OperationRegistry::default(), core::future::pending()).await;
    assert!(result.is_err(), "occupied listener must fail");
}
#[tokio::test]
async fn test_external_shutdown_drains_inflight_request() {
    let address = free_address();
    let entered = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let definition =
        OperationDefinition::new::<Value, Value>(MethodName::from(["test", "wait"]), OperationEffects::default()).expect("operation definition");
    let registry = OperationRegistry::default()
        .register_definition(definition, {
            let entered = Arc::clone(&entered);
            let release = Arc::clone(&release);
            move |input: Value, _| {
                let entered = Arc::clone(&entered);
                let release = Arc::clone(&release);
                async move {
                    entered.notify_one();
                    release.notified().await;
                    Ok(input)
                }
            }
        })
        .expect("wait operation");
    let (stop, stopped) = oneshot::channel::<()>();
    let mut server = tokio::spawn(serve_with_registry(config(address), registry, async move {
        stopped.await.map_err(Into::into)
    }));
    let client = reqwest::Client::new();
    wait_for_ready(&client, address).await;
    let request = tokio::spawn(async move {
        client
            .post(format!("http://{address}/rpc"))
            .bearer_auth("injected-secret")
            .header("content-type", "application/json")
            .body(json!({"jsonrpc":"2.0","id":1,"method":"acorn.test.wait","params":{}}).to_string())
            .send()
            .await
    });
    entered.notified().await;
    stop.send(()).expect("shutdown receiver");
    assert!(tokio::time::timeout(core::time::Duration::from_millis(50), &mut server).await.is_err());
    release.notify_one();
    assert!(request.await.expect("request task").expect("drained request").status().is_success());
    server.await.expect("server task").expect("graceful drain");
}
