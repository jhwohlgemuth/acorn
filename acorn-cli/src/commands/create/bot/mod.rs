use acorn::cmd;
use acorn::io::command_exists;
use acorn::io::Executor;
use acorn::util::Label;
use color_eyre::eyre::{eyre, Report, Result};
use tracing::{error, info};

/// Create a bot container.
///
/// Creates a detached Docker/Podman container using the acorn image, passes the
/// `GITLAB_TOKEN` environment variable for GitLab API authentication, and runs
/// `acorn serve bot <identifier>` with any provided flags forwarded to the
/// serve command.
#[allow(clippy::too_many_arguments)]
pub async fn run(
    identifier: &str,
    name: &Option<String>,
    image: &Option<String>,
    runtime: &Option<Executor>,
    bind: &Option<String>,
    poll_interval: &Option<u64>,
    after: &Option<String>,
    domain: &Option<String>,
) -> Result<(), Report> {
    let runtime_binary = runtime.as_ref().and_then(|r| r.command()).unwrap_or("docker");
    let container_name = name.clone().unwrap_or(format!("acorn-bot-{identifier}"));
    let image_tag = image.as_deref().unwrap_or("acorn:latest");

    if !command_exists(runtime_binary) {
        let err = eyre!("{runtime_binary} is required to create a bot container but was not found");
        error!("=> {} {err}", Label::fail());
        return Err(err);
    }

    let mut container_args = vec![
        std::ffi::OsString::from("run"),
        std::ffi::OsString::from("--detach"),
        std::ffi::OsString::from("--name"),
        std::ffi::OsString::from(&container_name),
        std::ffi::OsString::from("--restart"),
        std::ffi::OsString::from("always"),
    ];

    if let Some(token) = std::env::var("GITLAB_TOKEN").ok().filter(|t| !t.is_empty()) {
        container_args.push(std::ffi::OsString::from("--env"));
        container_args.push(std::ffi::OsString::from(format!("GITLAB_TOKEN={token}")));
    }

    if let Some(domain) = domain {
        container_args.push(std::ffi::OsString::from("--env"));
        container_args.push(std::ffi::OsString::from(format!("GITLAB_HOST={domain}")));
    }

    container_args.push(std::ffi::OsString::from(&image_tag));
    container_args.push(std::ffi::OsString::from("serve"));
    container_args.push(std::ffi::OsString::from("bot"));
    container_args.push(std::ffi::OsString::from(identifier));

    if let Some(bind) = bind {
        container_args.push(std::ffi::OsString::from("--bind"));
        container_args.push(std::ffi::OsString::from(bind));
    }

    if let Some(interval) = poll_interval {
        container_args.push(std::ffi::OsString::from("--poll-interval"));
        container_args.push(std::ffi::OsString::from(interval.to_string()));
    }

    if let Some(after) = after {
        container_args.push(std::ffi::OsString::from("--after"));
        container_args.push(std::ffi::OsString::from(after));
    }

    match cmd!(runtime_binary, container_args) {
        | Ok(output) if output.status.success() => {
            info!("{} Bot container '{container_name}' created", Label::pass());
            Ok(())
        }
        | Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let err = eyre!("Failed to create bot container — {stderr}");
            error!("=> {} {err}", Label::fail());
            Err(err)
        }
        | Err(why) => {
            let err = eyre!("Failed to spawn {runtime_binary} — {why}");
            error!("=> {} {err}", Label::fail());
            Err(err)
        }
    }
}
