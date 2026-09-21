#[cfg(feature = "grpc")]
use acorn::io::api::{grpc, Secret};
use acorn::io::{ContainerTarget, Executor};
#[cfg(feature = "grpc")]
use acorn::util::constants::env::GRPC_TOKEN;
#[cfg(feature = "grpc")]
use acorn_cmd::runtime::command_string;
#[cfg(feature = "grpc")]
use acorn_cmd::{args, cmd};
use color_eyre::eyre::{eyre, Report, Result};
use core::net::SocketAddr;
#[cfg(feature = "grpc")]
use std::env::var;
#[cfg(feature = "grpc")]
use std::fs::read;
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
    run_feature(address, allow_mutation, offline, detach, target, runtime, tls).await
}
#[cfg(not(feature = "grpc"))]
async fn run_feature(
    _address: SocketAddr,
    _allow_mutation: bool,
    _offline: bool,
    _detach: bool,
    _target: Option<&ContainerTarget>,
    _runtime: &Option<Executor>,
    tls: TlsPaths<'_>,
) -> Result<(), Report> {
    let TlsPaths { certificate, private_key } = tls;
    let _ = (certificate, private_key);

    Err(eyre!("gRPC support is disabled; rebuild acorn-cli with `--features grpc`"))
}
#[cfg(feature = "grpc")]
async fn run_feature(
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
            let token = var(GRPC_TOKEN)
                .ok()
                .filter(|token| !token.trim().is_empty())
                .map(Secret::from)
                .ok_or_else(|| eyre!("{GRPC_TOKEN} is required to serve gRPC"));
            let tls_identity = load_tls(tls);
            match (token, tls_identity) {
                | (Ok(token), Ok(tls)) => grpc::server::serve(address, token, allow_mutation, offline, tls).await,
                | (Err(why), _) | (_, Err(why)) => Err(why),
            }
        }
    }
}
#[cfg(feature = "grpc")]
fn load_tls(tls: TlsPaths<'_>) -> Result<Option<grpc::server::TlsIdentity>, Report> {
    match (tls.certificate, tls.private_key) {
        | (None, None) => Ok(None),
        | (Some(certificate), Some(private_key)) => read(certificate)
            .map_err(|why| eyre!("Failed to read gRPC TLS certificate {} — {why}", certificate.display()))
            .and_then(|certificate| {
                read(private_key)
                    .map_err(|why| eyre!("Failed to read gRPC TLS private key {} — {why}", private_key.display()))
                    .map(|private_key| Some(grpc::server::TlsIdentity::from_pem(certificate, private_key)))
            }),
        | _ => Err(eyre!("Both --tls-cert and --tls-key are required when configuring gRPC TLS")),
    }
}
