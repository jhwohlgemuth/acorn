use acorn::{
    io::{
        api::{self, json_rpc},
        ContainerTarget, Executor,
    },
    util::constants::env::RPC_TOKEN,
};
use acorn_cmd::{args, cmd, runtime::command_string};
use color_eyre::eyre::{eyre, Report, Result};
use core::net::SocketAddr;
use std::env::var;

pub async fn run(
    address: SocketAddr,
    allow_mutation: bool,
    offline: bool,
    detach: bool,
    target: Option<&ContainerTarget>,
    runtime: &Option<Executor>,
) -> Result<(), Report> {
    let selected = runtime.as_ref().unwrap_or(&Executor::Docker);
    let validation = target.map_or(Ok(()), |target| target.validate_runtime(selected));
    let arguments = args![
        "serve",
        "rpc",
        ("--bind", address.to_string()),
        ..if allow_mutation { args!["--allow-mutation"] } else { args![] },
        ..if offline { args!["--offline"] } else { args![] },
    ];
    match (validation, target, detach) {
        | (Err(why), _, _) => Err(why),
        | (Ok(()), Some(ContainerTarget::Context(location)), _) => {
            Err(eyre!("--context '{location}' not supported for serve rpc; use --host with SSH alias"))
        }
        | (Ok(()), Some(ContainerTarget::Ssh(remote)), true) => {
            let ssh_host = remote.as_str().trim_start_matches("ssh://");
            let command = command_string("acorn", &arguments);
            Err(eyre!(
                "Detached remote rpc via --host not yet implemented; use `ssh {ssh_host} {command}`"
            ))
        }
        | (Ok(()), Some(ContainerTarget::Ssh(remote)), false) => {
            let ssh_host = remote.as_str().trim_start_matches("ssh://");
            let ssh_arguments = args![ssh_host, "acorn", ..arguments];
            match cmd!("ssh", ssh_arguments) {
                | Ok(output) if output.status.success() => Ok(()),
                | Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(eyre!("Remote rpc via ssh failed — {stderr}"))
                }
                | Err(why) => Err(eyre!("Failed to spawn ssh for remote rpc — {why}")),
            }
        }
        | (Ok(()), Some(ContainerTarget::Local) | None, true) => Err(eyre!(
            "--detach for serve rpc without --host/--context has no effect; use --host for remote detached"
        )),
        | (Ok(()), Some(ContainerTarget::Local) | None, false) => {
            let token = var(RPC_TOKEN)
                .ok()
                .filter(|token| !token.trim().is_empty())
                .map(api::Secret::from)
                .ok_or_else(|| eyre!("{RPC_TOKEN} is required to serve JSON-RPC"));
            match token {
                | Ok(token) => json_rpc::http::serve(address, token, allow_mutation, offline).await,
                | Err(why) => Err(why),
            }
        }
    }
}
