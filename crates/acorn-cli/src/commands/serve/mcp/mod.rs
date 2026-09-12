use acorn::io::mcp;
use acorn::io::{ContainerTarget, Executor};
use acorn::util::cmd::command_string;
use acorn::{args, cmd};
use color_eyre::eyre::{eyre, Report, Result};
use std::path::PathBuf;

pub async fn run(
    offline: bool,
    allow_mutation: bool,
    detach: bool,
    database_path: Option<PathBuf>,
    target: Option<&ContainerTarget>,
    runtime: &Option<Executor>,
) -> Result<(), Report> {
    let selected = runtime.as_ref().unwrap_or(&Executor::Docker);
    let validation = target.map_or(Ok(()), |target| target.validate_runtime(selected));
    let arguments = args![
        "serve",
        "mcp",
        ..if allow_mutation { args!["--allow-mutation"] } else { args![] },
        ..if offline { args!["--offline"] } else { args![] },
    ];
    match (validation, target, detach) {
        | (Err(why), _, _) => Err(why),
        | (Ok(()), Some(ContainerTarget::Context(location)), _) => {
            Err(eyre!("--context '{location}' not supported for serve mcp; use --host with SSH alias"))
        }
        | (Ok(()), Some(ContainerTarget::Ssh(remote)), true) => {
            let ssh_host = remote.as_str().trim_start_matches("ssh://");
            let command = command_string("acorn", &arguments);
            Err(eyre!(
                "Detached remote mcp via --host not yet implemented; use `ssh {ssh_host} {command}`"
            ))
        }
        | (Ok(()), Some(ContainerTarget::Ssh(remote)), false) => {
            let ssh_host = remote.as_str().trim_start_matches("ssh://");
            let ssh_arguments = args![ssh_host, "acorn", ..arguments];
            match cmd!("ssh", ssh_arguments) {
                | Ok(output) if output.status.success() => Ok(()),
                | Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(eyre!("Remote mcp via ssh failed — {stderr}"))
                }
                | Err(why) => Err(eyre!("Failed to spawn ssh for remote mcp — {why}")),
            }
        }
        | (Ok(()), Some(ContainerTarget::Local) | None, true) => Err(eyre!(
            "--detach for serve mcp without --host/--context has no effect; use --host for remote detached"
        )),
        | (Ok(()), Some(ContainerTarget::Local) | None, false) => mcp::run(offline, allow_mutation, database_path).await,
    }
}
