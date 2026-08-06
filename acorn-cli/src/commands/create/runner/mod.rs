use acorn::io::api::gitlab::{self, RunnerCreationResponse};
use acorn::io::api::Configuration;
use acorn::io::command_exists;
use acorn::io::config::{ApplicationConfiguration, RunnerDetails, RunnerType};
use acorn::io::{to_absolute_string, Executor, InputOutput, Remote};
use acorn::prelude::{remove_file, OsString, PathBuf};
use acorn::util::constants::app::{DEFAULT_RUNNER_NAME, DOCKER_SOCKET};
use acorn::util::Label;
use acorn::{args, cmd, Location, Repository};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{eyre, Report, Result, WrapErr};
use futures::future::join_all;
use tracing::{debug, error, info};
type BooleanOutcome = (Vec<Result<(), String>>, Vec<Result<(), String>>);

#[allow(clippy::too_many_arguments)]
pub async fn run(
    config: &Option<PathBuf>,
    description: &Option<String>,
    name: &Option<String>,
    _server: &Option<String>,
    repo: &Option<String>,
    group: &Option<u64>,
    project: &Option<u64>,
    tags: &[String],
    untagged: &bool,
    runtime: &Option<Executor>,
    remote: &Option<Remote>,
    verbose: &Verbosity,
) -> Result<(), Report> {
    let from_config = ApplicationConfiguration::resolve(config)
        .and_then(|path| ApplicationConfiguration::read(path).ok())
        .and_then(|value| value.runners)
        .filter(|values| !values.is_empty());
    let build_target = |runner: Option<&RunnerDetails>| {
        let desired_description_result = runner
            .and_then(|value| value.description.clone())
            .or_else(|| description.clone())
            .ok_or_else(|| eyre!("GitLab runner description is required to detect existing runners"));
        let desired_tags = runner
            .and_then(|value| value.tags.clone())
            .or_else(|| (!tags.is_empty()).then(|| tags.to_owned()));
        let desired_run_untagged = *untagged || runner.map(|value| value.run_untagged).unwrap_or(false);
        let maybe_runner_type = match runner {
            | Some(value) => Ok(value.runner_type.clone()),
            | None => match (group, project) {
                | (Some(_), None) => Ok(RunnerType::Group),
                | (None, Some(_)) => Ok(RunnerType::Project),
                | _ => Err(eyre!("Provide exactly one of --group or --project (or use --config with GitLab runners)")),
            },
        };
        let maybe_identifier = runner
            .and_then(|value| value.code_repository.id())
            .map(Ok)
            .unwrap_or_else(|| match (group, project) {
                | (Some(value), None) | (None, Some(value)) => Ok(value.to_string()),
                | _ => Err(eyre!(
                    "Missing GitLab runner target identifier. Provide --group/--project or repository.id in config"
                )),
            });
        match (desired_description_result, maybe_runner_type, maybe_identifier) {
            | (Ok(value), Ok(runner_type), Ok(identifier)) => {
                let repository = runner.map(|entry| entry.code_repository.clone()).unwrap_or_else(|| Repository::GitLab {
                    id: identifier.parse::<u64>().ok(),
                    location: Location::Simple("https://gitlab.com".to_string()),
                });
                let domain = match repo {
                    | Some(value) => Ok(value.clone()),
                    | None => repository
                        .domain()
                        .ok_or_else(|| eyre!("Failed to resolve runner domain from repository")),
                };
                match domain {
                    | Ok(domain) => Ok((value, runner_type, identifier, desired_run_untagged, desired_tags, repository, domain)),
                    | Err(why) => Err(why),
                }
            }
            | (Err(why), _, _) | (_, Err(why), _) | (_, _, Err(why)) => Err(why),
        }
    };
    let selected = runtime.as_ref().unwrap_or(&Executor::Docker);
    let remote_validation = selected.validate(from_config.as_deref(), remote.as_ref());
    let response = match remote_validation {
        | Err(why) => Err(why),
        | Ok(()) => match from_config {
            | Some(runner_details) => {
                let runners = join_all(runner_details.into_iter().map(|runner| {
                    let target = build_target(Some(&runner));
                    async move {
                        let base_name = runner.name.clone().unwrap_or_else(|| DEFAULT_RUNNER_NAME.to_string());
                        match target {
                            | Ok((desired_description, runner_type, _identifier, desired_run_untagged, desired_tags, repository, domain)) => {
                                let details = RunnerDetails::at(repository)
                                    .description(desired_description)
                                    .runner_type(&runner_type.to_string())
                                    .gpu_enabled(runner.gpu_enabled)
                                    .maybe_tags(desired_tags)
                                    .run_untagged(desired_run_untagged)
                                    .maybe_host(Some(domain))
                                    .build();
                                match create_runner(&details).await {
                                    | Ok(response) => {
                                        let RunnerCreationResponse { identifier, token, .. } = response;
                                        let container_name = format!("{base_name}-{identifier}");
                                        let details = details.with_name(container_name).with_id(identifier).with_token(token);
                                        register_runner(details, runtime, remote, verbose)
                                            .await
                                            .map(|_| ())
                                            .map_err(|why| format!("{base_name} — {why}"))
                                    }
                                    | Err(why) => Err(format!("{base_name} — {why}")),
                                }
                            }
                            | Err(why) => Err(format!("{base_name} — {why}")),
                        }
                    }
                }))
                .await;
                let (successful, failed): BooleanOutcome = runners.into_iter().partition(|result| result.is_ok());
                let created = successful.len();
                let failures = failed.into_iter().filter_map(|result| result.err()).collect::<Vec<String>>();
                if failures.is_empty() {
                    Ok(())
                } else {
                    let attempted = created.saturating_add(failures.len());
                    let failed = failures.len();
                    let message = format!("Failed to create {failed} of {attempted} configured runners:\n{}", failures.join("\n"));
                    Err(eyre!(message))
                }
            }
            | None => match build_target(None) {
                | Ok((desired_description, runner_type, _identifier, desired_run_untagged, desired_tags, repository, domain)) => {
                    let base_name = name
                        .clone()
                        .or_else(|| Some(desired_description.clone()))
                        .unwrap_or_else(|| "gitlab-runner".to_string());
                    let details = RunnerDetails::at(repository)
                        .description(desired_description)
                        .runner_type(&runner_type.to_string())
                        .gpu_enabled(false)
                        .maybe_tags(desired_tags)
                        .run_untagged(desired_run_untagged)
                        .maybe_host(Some(domain))
                        .build();
                    match create_runner(&details).await {
                        | Ok(response) => {
                            let RunnerCreationResponse { identifier, token, .. } = response;
                            let container_name = format!("{base_name}-{identifier}");
                            let details = details.with_name(container_name).with_id(identifier).with_token(token);
                            register_runner(details, runtime, remote, verbose).await
                        }
                        | Err(why) => Err(why),
                    }
                }
                | Err(why) => Err(why),
            },
        },
    };
    if let Err(why) = response {
        error!("=> {} Create GitLab runner — {why}", Label::fail());
        Err(why)
    } else {
        Ok(())
    }
}
async fn create_runner(details: &RunnerDetails) -> Result<RunnerCreationResponse, Report> {
    let maybe_description = match &details.description {
        | Some(value) => Ok(value.clone()),
        | None => Err(eyre!("GitLab runner description is required to detect existing runners")),
    };
    let maybe_identifier = match details.code_repository.id() {
        | Some(value) => Ok(value),
        | None => Err(eyre!(
            "Missing GitLab runner target identifier. Provide --group/--project or repository.id in config"
        )),
    };
    let options = match &details.host {
        | Some(value) => gitlab::Options::from_env().with_domain(value),
        | None => gitlab::Options::from_env(),
    };
    let existing_runners = gitlab::runners(&options).await;
    match (maybe_description, maybe_identifier, existing_runners) {
        | (Ok(description), Ok(identifier), Ok(existing)) => {
            let already_exists = existing
                .iter()
                .any(|value| value.description.as_deref().is_some_and(|candidate| candidate == description));
            if already_exists {
                Err(eyre!("A GitLab runner with the description, '{description}', already exists"))
            } else {
                let runner_builder = gitlab::RunnerMetadata::init()
                    .runner_type(&details.runner_type.to_string())
                    .description(&description)
                    .run_untagged(details.run_untagged);
                let metadata = match &details.tags {
                    | Some(values) if !values.is_empty() => {
                        let tag_refs = values.iter().map(String::as_str).collect::<Vec<_>>();
                        runner_builder.tags(&tag_refs).build()
                    }
                    | _ => runner_builder.build(),
                };
                let create_options = options.clone().with_identifier(identifier).with_runner(metadata);
                match gitlab::create_runner(&create_options).await {
                    | Ok(response) => {
                        if response.identifier > 0 {
                            debug!("=> {} Created GitLab runner — {response:#?}", Label::using());
                            Ok(response)
                        } else {
                            Err(eyre!(
                                "GitLab create runner returned an invalid response (missing runner id): {response:#?}"
                            ))
                        }
                    }
                    | Err(why) => Err(why).wrap_err("GitLab create runner request failed"),
                }
            }
        }
        | (Err(why), _, _) | (_, Err(why), _) => Err(why),
        | (_, _, Err(why)) => Err(why).wrap_err("GitLab list runners request failed"),
    }
}
async fn register_runner(details: RunnerDetails, runtime: &Option<Executor>, remote: &Option<Remote>, verbose: &Verbosity) -> Result<(), Report> {
    let RunnerDetails {
        docker_image,
        executor,
        gpu_enabled,
        host,
        identifier,
        name,
        token,
        ..
    } = details;
    let identifier = identifier.unwrap_or_default();
    let gitlab_url = host.as_deref().unwrap_or("gitlab.com");
    let url = format!("https://{gitlab_url}/");
    let name = &name.unwrap_or_else(|| "gitlab-runner".to_string());
    let binary = runtime
        .as_ref()
        .and_then(|r| r.command())
        .or_else(|| executor.command())
        .unwrap_or("gitlab-runner");
    let description = format!("GitLab Runner [{identifier}]");
    let log_level = if verbose.is_silent() { "panic" } else { "debug" };
    let config_host_dir = Executor::default_gitlab_runner_config_directory();
    let maybe_template = if gpu_enabled {
        Remote::create_gpu_template(remote.as_ref(), config_host_dir)
    } else {
        Ok(None)
    };
    let result = match maybe_template.as_ref() {
        | Err(why) => Err(eyre!("Failed to create GitLab runner GPU template — {why}")),
        | Ok(maybe_template) if command_exists(binary) => match token {
            | Some(token) => match executor {
                | Executor::Docker => match remote.as_ref().map(|_| DOCKER_SOCKET.to_string()).or_else(|| executor.socket()) {
                    | Some(socket) => {
                        let create_container = build_docker_runner_args(name, &docker_image, gpu_enabled, config_host_dir, &socket, remote.as_ref());
                        match cmd!(binary, create_container) {
                            | Ok(output) => {
                                if output.status.success() {
                                    let copy = match (remote.as_ref(), maybe_template.as_deref()) {
                                        | (Some(remote), Some(template)) => {
                                            remote.copy_gpu_template(runtime.as_ref().unwrap_or(&Executor::Docker), name, template)
                                        }
                                        | _ => Ok(()),
                                    };
                                    match copy {
                                        | Ok(()) => {
                                            let template = maybe_template.as_ref().map(|template| {
                                                remote
                                                    .as_ref()
                                                    .map(|_| "/etc/gitlab-runner/gpu.template.toml".to_string())
                                                    .unwrap_or_else(|| to_absolute_string(template))
                                            });
                                            let command = build_register_args(name, &url, &description, &token, log_level, &template);
                                            let register = match remote.as_ref() {
                                                | Some(remote) => remote.docker_args(command),
                                                | None => command,
                                            };
                                            match cmd!(binary, register) {
                                                | Ok(output) => {
                                                    if output.status.success() {
                                                        info!("=> {} GitLab runner registration", Label::pass());
                                                        Ok(())
                                                    } else {
                                                        let stderr = String::from_utf8_lossy(&output.stderr);
                                                        Err(eyre!("gitlab-runner register failed — {stderr}"))
                                                    }
                                                }
                                                | Err(why) => Err(eyre!("Failed to execute docker exec — {why}")),
                                            }
                                        }
                                        | Err(why) => Err(why),
                                    }
                                } else {
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    Err(eyre!("Failed to create GitLab runner container — {stderr}"))
                                }
                            }
                            | Err(why) => Err(eyre!("Failed to spawn Docker container — {why}")),
                        }
                    }
                    | None => Err(eyre!("Docker socket not found — is Docker running?")),
                },
                | Executor::Podman => match executor.socket() {
                    | Some(socket) => {
                        let gpu_args = if gpu_enabled { args!["--gpus", "all"] } else { args![] };
                        let create_container = args![
                            "run",
                            "--detach",
                            ("--name", name),
                            ("--restart", "always"),
                            ..gpu_args,
                            ("-v", format!("{config_host_dir}:/etc/gitlab-runner")),
                            ("-v", format!("{socket}:{DOCKER_SOCKET}")),
                            &docker_image,
                        ];
                        match cmd!(binary, create_container) {
                            | Ok(output) => {
                                if output.status.success() {
                                    let template = maybe_template.as_ref().map(to_absolute_string);
                                    let register = build_register_args(name, &url, &description, &token, log_level, &template);
                                    match cmd!(binary, register) {
                                        | Ok(output) => {
                                            if output.status.success() {
                                                info!("=> {} GitLab runner registration", Label::pass());
                                                Ok(())
                                            } else {
                                                let stderr = String::from_utf8_lossy(&output.stderr);
                                                Err(eyre!("gitlab-runner register failed — {stderr}"))
                                            }
                                        }
                                        | Err(why) => Err(eyre!("Failed to execute podman exec — {why}")),
                                    }
                                } else {
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    Err(eyre!("Failed to create GitLab runner container — {stderr}"))
                                }
                            }
                            | Err(why) => Err(eyre!("Failed to spawn Podman container — {why}")),
                        }
                    }
                    | None => Err(eyre!(
                        "Podman socket not found. Start Podman with 'podman system service --time=0' then retry"
                    )),
                },
                | Executor::Apptainer => match executor.socket() {
                    | Some(socket) => {
                        let create = args![
                            "instance",
                            "start",
                            ("--bind", format!("{config_host_dir}:/etc/gitlab-runner")),
                            ("--bind", format!("{socket}:{DOCKER_SOCKET}")),
                            format!("docker://{docker_image}"),
                            name,
                        ];
                        match cmd!(binary, create) {
                            | Ok(output) => {
                                if output.status.success() {
                                    let gpu_args = match maybe_template.as_ref() {
                                        | Some(template) => args!["--template-config", to_absolute_string(template)],
                                        | None => args![],
                                    };
                                    let register = args![
                                        "exec",
                                        format!("instance://{name}"),
                                        "gitlab-runner",
                                        ("--log-level", log_level),
                                        "register",
                                        "--non-interactive",
                                        ("--url", url),
                                        ("--description", description),
                                        ("--token", token),
                                        ("--executor", executor.gitlab_runner_type()),
                                        ..gpu_args,
                                    ];
                                    match cmd!(binary, register) {
                                        | Ok(output) => {
                                            if output.status.success() {
                                                info!("=> {} GitLab runner registration", Label::pass());
                                                Ok(())
                                            } else {
                                                let stderr = String::from_utf8_lossy(&output.stderr);
                                                Err(eyre!("gitlab-runner register failed — {stderr}"))
                                            }
                                        }
                                        | Err(why) => Err(eyre!("Failed to execute apptainer exec — {why}")),
                                    }
                                } else {
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    Err(eyre!("Failed to create Apptainer instance — {stderr}"))
                                }
                            }
                            | Err(why) => Err(eyre!("Failed to spawn apptainer instance — {why}")),
                        }
                    }
                    | None => Err(eyre!("Apptainer socket not found — is Apptainer running?")),
                },
                | Executor::Shell | Executor::Ssh | Executor::Kubernetes | Executor::Sandbox | Executor::VirtualMachine => {
                    let gpu_args = match maybe_template.as_ref() {
                        | Some(template) => args!["--template-config", to_absolute_string(template)],
                        | None => args![],
                    };
                    let register = args![
                        ("--log-level", log_level),
                        "register",
                        "--non-interactive",
                        ("--url", url),
                        ("--description", description),
                        ("--token", token),
                        ("--executor", executor.gitlab_runner_type()),
                        ..gpu_args,
                    ];
                    match cmd!(binary, register) {
                        | Ok(output) => {
                            if output.status.success() {
                                info!("=> {} GitLab runner registration", Label::pass());
                                Ok(())
                            } else {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                Err(eyre!("gitlab-runner register failed — {stderr}"))
                            }
                        }
                        | Err(why) => Err(eyre!("Failed to spawn gitlab-runner register — {why}")),
                    }
                }
                | Executor::Other(value) => Err(eyre!("Unsupported executor type ({value}) for automatic registration")),
            },
            | None => Err(eyre!("GitLab runner creation did not return a registration token")),
        },
        | Ok(_) => Err(eyre!("{binary} is required to register a GitLab runner but was not found")),
    };
    if let Ok(Some(template)) = maybe_template {
        let _ = remove_file(template);
    }
    result
}
fn build_docker_runner_args(
    name: &str,
    image: &str,
    gpu_enabled: bool,
    config_host_dir: &str,
    socket: &str,
    remote: Option<&Remote>,
) -> Vec<OsString> {
    let gpu_args = if gpu_enabled { args!["--gpus", "all"] } else { args![] };
    let config_source = remote.map_or_else(|| config_host_dir.to_string(), |_| format!("{name}-config"));
    let command = args![
        "run",
        "--detach",
        ("--name", name),
        ("--restart", "always"),
        ..gpu_args,
        ("-v", format!("{config_source}:/etc/gitlab-runner")),
        ("-v", format!("{socket}:{DOCKER_SOCKET}")),
        image,
    ];
    match remote {
        | Some(remote) => remote.docker_args(command),
        | None => command,
    }
}
/// Build the `exec <name> gitlab-runner register ...` argument list for
/// Docker/Podman container-based executors.
fn build_register_args(name: &str, url: &str, description: &str, token: &str, log_level: &str, gpu_template: &Option<String>) -> Vec<OsString> {
    let gpu_args = match gpu_template {
        | Some(template) => args!["--template-config", template],
        | None => args![],
    };
    args![
        "exec",
        name,
        "gitlab-runner",
        ("--log-level", log_level),
        "register",
        "--non-interactive",
        ("--url", url),
        ("--description", description),
        ("--token", token),
        ("--executor", "docker"),
        ("--docker-image", "docker"),
        ("--docker-volumes", format!("{DOCKER_SOCKET}:{DOCKER_SOCKET}")),
        ..gpu_args
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remote() -> Remote {
        "ssh://builder".parse().unwrap()
    }
    fn text(values: Vec<OsString>) -> Vec<String> {
        values.into_iter().map(|value| value.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn test_local_docker_runner_arguments_are_unchanged() {
        let values = text(build_docker_runner_args(
            "runner-42",
            "gitlab/gitlab-runner:latest",
            false,
            "/srv/gitlab-runner/config",
            DOCKER_SOCKET,
            None,
        ));
        assert_eq!(values.first().map(String::as_str), Some("run"));
        assert!(values
            .windows(2)
            .any(|pair| pair == ["-v", "/srv/gitlab-runner/config:/etc/gitlab-runner"]));
        assert!(!values.iter().any(|value| value == "--host"));
    }
    #[test]
    fn test_remote_docker_runner_uses_remote_volume_and_socket() {
        let values = text(build_docker_runner_args(
            "runner-42",
            "gitlab/gitlab-runner:latest",
            false,
            "C:\\local\\runner",
            DOCKER_SOCKET,
            Some(&remote()),
        ));
        assert!(values
            .iter()
            .map(String::as_str)
            .take(4)
            .eq(["--host", "ssh://builder", "run", "--detach"]));
        let socket_mount = format!("{DOCKER_SOCKET}:{DOCKER_SOCKET}");
        assert!(values.windows(2).any(|pair| pair == ["-v", "runner-42-config:/etc/gitlab-runner"]));
        assert!(values.windows(2).any(|pair| pair == ["-v", socket_mount.as_str()]));
        assert!(!values.iter().any(|value| value.contains("C:\\local")));
    }
    #[test]
    fn test_remote_runner_registration_targets_the_requested_docker_host() {
        let template = Some("/etc/gitlab-runner/gpu.template.toml".to_string());
        let values = text(remote().docker_args(build_register_args(
            "runner-42",
            "https://gitlab.example.org/",
            "GitLab Runner [42]",
            "token",
            "debug",
            &template,
        )));
        assert!(values
            .iter()
            .map(String::as_str)
            .take(5)
            .eq(["--host", "ssh://builder", "exec", "runner-42", "gitlab-runner"]));
        assert!(values
            .windows(2)
            .any(|pair| pair == ["--template-config", "/etc/gitlab-runner/gpu.template.toml"]));
    }
    #[test]
    fn test_remote_runner_validation_rejects_non_docker_config_before_creation() {
        let runner = RunnerDetails::at(Repository::GitLab {
            id: Some(42),
            location: Location::Simple("https://gitlab.com".to_string()),
        })
        .executor(Executor::Podman)
        .build();
        let result = Executor::Docker.validate(Some(&[runner]), Some(&remote()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires docker runner executors"));
    }
    #[test]
    fn test_remote_runner_validation_rejects_non_docker_runtime() {
        let result = Executor::Podman.validate(None, Some(&remote()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires the docker runtime"));
    }
}
