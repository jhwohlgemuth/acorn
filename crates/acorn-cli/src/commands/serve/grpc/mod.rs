use acorn::io::api::webhooks::store::OperationQueue;
use acorn::io::{ContainerTarget, Executor};
use acorn_cmd::runtime::command_string;
use acorn_cmd::{args, cmd};
use acorn_server::{serve_with_durable, ServerArgs, ServerConfig};
use acorn_service::DurableOperations;
use alloc::sync::Arc;
use color_eyre::eyre::{eyre, Report, Result};
use core::net::SocketAddr;
use std::path::Path;

#[derive(Clone, Copy)]
pub(crate) struct TlsPaths<'a> {
    certificate: Option<&'a Path>,
    private_key: Option<&'a Path>,
}
impl<'a> TlsPaths<'a> {
    pub(crate) fn new(certificate: Option<&'a Path>, private_key: Option<&'a Path>) -> Self {
        Self { certificate, private_key }
    }
}
pub async fn run(
    address: SocketAddr,
    allow_mutation: bool,
    offline: bool,
    detach: bool,
    target: Option<&ContainerTarget>,
    runtime: &Option<Executor>,
    tls: TlsPaths<'_>,
) -> Result<(), Report> {
    let selected = runtime.as_ref().unwrap_or(&Executor::Docker);
    let validation = target.map_or(Ok(()), |target| target.validate_runtime(selected));
    let arguments = args![
        "serve",
        "grpc",
        ("--bind", address.to_string()),
        ..if allow_mutation { args!["--allow-mutation"] } else { args![] },
        ..if offline { args!["--offline"] } else { args![] },
        ..tls
            .certificate
            .map_or_else(|| args![], |path| args!["--tls-cert", path.display().to_string()]),
        ..tls
            .private_key
            .map_or_else(|| args![], |path| args!["--tls-key", path.display().to_string()]),
    ];
    match (validation, target, detach) {
        | (Err(why), _, _) => Err(why),
        | (Ok(()), Some(ContainerTarget::Context(location)), _) => {
            Err(eyre!("--context '{location}' not supported for serve grpc; use --host with SSH alias"))
        }
        | (Ok(()), Some(ContainerTarget::Ssh(remote)), true) => {
            let ssh_host = remote.as_str().trim_start_matches("ssh://");
            let command = command_string("acorn", &arguments);
            Err(eyre!(
                "Detached remote gRPC via --host not yet implemented; use `ssh {ssh_host} {command}`"
            ))
        }
        | (Ok(()), Some(ContainerTarget::Ssh(remote)), false) => {
            let ssh_host = remote.as_str().trim_start_matches("ssh://");
            let ssh_arguments = args![ssh_host, "acorn", ..arguments];
            match cmd!("ssh", ssh_arguments) {
                | Ok(output) if output.status.success() => Ok(()),
                | Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(eyre!("Remote gRPC via ssh failed — {stderr}"))
                }
                | Err(why) => Err(eyre!("Failed to spawn ssh for remote gRPC — {why}")),
            }
        }
        | (Ok(()), Some(ContainerTarget::Local) | None, true) => Err(eyre!(
            "--detach for serve grpc without --host/--context has no effect; use --host for remote detached"
        )),
        | (Ok(()), Some(ContainerTarget::Local) | None, false) => {
            let config = ServerConfig::resolve(ServerArgs {
                allow_mutation,
                disable_json_rpc: true,
                grpc_bind: Some(address),
                tls_cert: tls.certificate.map(Path::to_path_buf),
                tls_key: tls.private_key.map(Path::to_path_buf),
                ..ServerArgs::default()
            })
            .map_err(|why| eyre!(why.to_string()));
            match config {
                | Err(why) => Err(why),
                | Ok(config) => {
                    let durable = allow_mutation.then(|| Arc::new(OperationQueue::configured()) as Arc<dyn DurableOperations>);
                    serve_with_durable(config, durable).await.map_err(|why| eyre!(why.to_string()))
                }
            }
        }
    }
}
