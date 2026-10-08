//! Listener orchestration, readiness, and graceful shutdown.
use crate::grpc::GrpcService;
use crate::health::HealthService;
use crate::health_proto::health_server::HealthServer;
use crate::http::HttpState;
use crate::proto::acorn_service_server::AcornServiceServer;
use crate::{ServerConfig, ServerResult};
use acorn_service::registry::DurableOperations;
use acorn_service::{AcornService, InvocationContext, Memory, OperationRegistry};
use alloc::sync::Arc;
use axum_server::tls_rustls::RustlsConfig;
use core::sync::atomic::{AtomicBool, Ordering};
use rustls::crypto::ring;
use std::{fs::read, net::TcpListener};
use tokio::net;
use tokio::sync::watch;
use tokio::task::JoinSet;
use tonic::transport::{Identity, Server, ServerTlsConfig};

async fn drain(listeners: &mut JoinSet<Result<(), String>>) -> ServerResult<()> {
    loop {
        match listeners.join_next().await {
            | None => break Ok(()),
            | Some(result) => match listener_result(Some(result)) {
                | Ok(()) => {}
                | Err(error) => break Err(error),
            },
        }
    }
}
async fn finish(
    config: &ServerConfig,
    ready: Arc<AtomicBool>,
    shutdown: watch::Sender<bool>,
    listeners: &mut JoinSet<Result<(), String>>,
) -> ServerResult<()> {
    ready.store(true, Ordering::Release);
    let unexpected = tokio::select! {
        signal = shutdown_signal() => signal.map(|()| None),
        result = listeners.join_next() => Ok(Some(listener_result(result))),
    };
    ready.store(false, Ordering::Release);
    let _ = shutdown.send(true);
    let drained = tokio::time::timeout(config.shutdown_grace, drain(listeners)).await;
    match (unexpected, drained) {
        | (Err(error), _) => Err(error),
        | (Ok(Some(result)), _) => result.and(Err("An enabled listener terminated unexpectedly".into())),
        | (Ok(None), Ok(result)) => result,
        | (Ok(None), Err(_)) => {
            listeners.abort_all();
            Err("Graceful shutdown deadline expired".into())
        }
    }
}
fn listener_result(result: Option<Result<Result<(), String>, tokio::task::JoinError>>) -> ServerResult<()> {
    match result {
        | Some(Ok(Ok(()))) => Ok(()),
        | Some(Ok(Err(error))) => Err(error.into()),
        | Some(Err(error)) => Err(format!("Listener task failed: {error}").into()),
        | None => Err("No listener task remained active".into()),
    }
}
/// Run every enabled listener over one shared portable service.
pub async fn serve(config: ServerConfig) -> ServerResult<()> {
    serve_with_durable(config, None).await
}
async fn serve_registry(config: ServerConfig, service: AcornService, context: InvocationContext, registry: OperationRegistry) -> ServerResult<()> {
    let ready = Arc::new(AtomicBool::new(false));
    let (shutdown, receiver) = watch::channel(false);
    let mut listeners = JoinSet::new();
    match spawn_json_rpc(&config, registry, context, Arc::clone(&ready), &receiver, &mut listeners).await {
        | Err(error) => Err(error),
        | Ok(()) => match spawn_grpc(&config, service, Arc::clone(&ready), &receiver, &mut listeners) {
            | Err(error) => Err(error),
            | Ok(()) => finish(&config, ready, shutdown, &mut listeners).await,
        },
    }
}
/// Run every enabled listener with an optional durable-operation adapter.
pub async fn serve_with_durable(config: ServerConfig, durable: Option<Arc<dyn DurableOperations>>) -> ServerResult<()> {
    let _ = ring::default_provider().install_default();
    match config.allow_mutation && durable.is_none() {
        | true => Err("Mutation requires a configured durable-operation adapter".into()),
        | false => {
            let service = AcornService;
            let max_request = Memory::kb(config.max_request_bytes as f64 / 1024.0);
            let context = InvocationContext {
                allow_mutation: config.allow_mutation,
                max_input: max_request.clone(),
                max_output: max_request,
                ..InvocationContext::default()
            };
            match OperationRegistry::acorn(service, env!("CARGO_PKG_VERSION"), durable) {
                | Err(error) => Err(error.into()),
                | Ok(registry) => serve_registry(config, service, context, registry).await,
            }
        }
    }
}
#[cfg(unix)]
async fn shutdown_signal() -> ServerResult<()> {
    use tokio::signal::unix::{signal, SignalKind};
    match signal(SignalKind::terminate()) {
        | Err(error) => Err(error.into()),
        | Ok(mut terminate) => tokio::select! {
            result = tokio::signal::ctrl_c() => result.map_err(Into::into),
            _ = terminate.recv() => Ok(()),
        },
    }
}
#[cfg(not(unix))]
async fn shutdown_signal() -> ServerResult<()> {
    tokio::signal::ctrl_c().await.map_err(Into::into)
}
fn spawn_grpc(
    config: &ServerConfig,
    service: AcornService,
    ready: Arc<AtomicBool>,
    receiver: &watch::Receiver<bool>,
    listeners: &mut JoinSet<Result<(), String>>,
) -> ServerResult<()> {
    match config.grpc_bind {
        | None => Ok(()),
        | Some(address) => {
            let adapter = AcornServiceServer::new(GrpcService::new(service, Arc::<str>::from(config.token())))
                .max_decoding_message_size(config.max_request_bytes)
                .max_encoding_message_size(config.max_request_bytes);
            let builder = Server::builder().timeout(config.request_timeout);
            let builder = match (&config.tls_cert, &config.tls_key) {
                | (Some(certificate), Some(private_key)) => read(certificate)
                    .and_then(|certificate| read(private_key).map(|private_key| Identity::from_pem(certificate, private_key)))
                    .map_err(Into::into)
                    .and_then(|identity| builder.tls_config(ServerTlsConfig::new().identity(identity)).map_err(Into::into)),
                | (None, None) => Ok(builder),
                | _ => Err("Both TLS certificate and private-key paths are required".into()),
            };
            builder.map(|mut builder| {
                let health = HealthServer::new(HealthService::new(ready));
                let mut receiver = receiver.clone();
                listeners.spawn(async move {
                    builder
                        .add_service(health)
                        .add_service(adapter)
                        .serve_with_shutdown(address, wait_for_shutdown(&mut receiver))
                        .await
                        .map_err(|error| format!("gRPC server failed: {error}"))
                });
            })
        }
    }
}
async fn spawn_json_rpc(
    config: &ServerConfig,
    registry: OperationRegistry,
    context: InvocationContext,
    ready: Arc<AtomicBool>,
    receiver: &watch::Receiver<bool>,
    listeners: &mut JoinSet<Result<(), String>>,
) -> ServerResult<()> {
    match config.json_rpc_bind {
        | None => Ok(()),
        | Some(address) => {
            let router =
                HttpState::new(registry, Arc::<str>::from(config.token()), context, ready, config.request_timeout).router(config.max_request_bytes);
            match (&config.tls_cert, &config.tls_key) {
                | (Some(certificate), Some(private_key)) => match RustlsConfig::from_pem_file(certificate, private_key).await {
                    | Err(error) => Err(error.into()),
                    | Ok(tls) => match TcpListener::bind(address) {
                        | Err(error) => Err(error.into()),
                        | Ok(listener) => match listener.set_nonblocking(true) {
                            | Err(error) => Err(error.into()),
                            | Ok(()) => match axum_server::from_tcp_rustls(listener, tls) {
                                | Err(error) => Err(error.into()),
                                | Ok(server) => {
                                    let handle = axum_server::Handle::new();
                                    let shutdown_handle = handle.clone();
                                    let grace = config.shutdown_grace;
                                    let mut receiver = receiver.clone();
                                    listeners.spawn(async move {
                                        let server = server.handle(handle).serve(router.into_make_service());
                                        tokio::pin!(server);
                                        tokio::select! {
                                            result = &mut server => result.map_err(|error| format!("JSON-RPC TLS server failed: {error}")),
                                            () = wait_for_shutdown(&mut receiver) => {
                                                shutdown_handle.graceful_shutdown(Some(grace));
                                                server.await.map_err(|error| format!("JSON-RPC TLS server failed: {error}"))
                                            }
                                        }
                                    });
                                    Ok(())
                                }
                            },
                        },
                    },
                },
                | (None, None) => match net::TcpListener::bind(address).await {
                    | Err(error) => Err(error.into()),
                    | Ok(listener) => {
                        let mut receiver = receiver.clone();
                        listeners.spawn(async move {
                            axum::serve(listener, router)
                                .with_graceful_shutdown(async move { wait_for_shutdown(&mut receiver).await })
                                .await
                                .map_err(|error| format!("JSON-RPC server failed: {error}"))
                        });
                        Ok(())
                    }
                },
                | _ => Err("Both TLS certificate and private-key paths are required".into()),
            }
        }
    }
}
async fn wait_for_shutdown(receiver: &mut watch::Receiver<bool>) {
    while !*receiver.borrow() {
        if receiver.changed().await.is_err() {
            break;
        }
    }
}
