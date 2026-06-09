use crate::cli::arguments::ExecutorArgument;
use acorn::io::api::gitlab::{self, RunnerCreationResponse};
use acorn::io::api::Configuration;
use acorn::io::command_exists;
use acorn::io::config::{ApplicationConfiguration, RunnerDetails, RunnerType};
use acorn::io::{Executor, InputOutput};
use acorn::prelude::{create_dir_all, remove_file, write, OsString, PathBuf};
use acorn::util::constants::app::DEFAULT_RUNNER_NAME;
use acorn::util::Label;
use acorn::{args, cmd};
use acorn::{Location, Repository};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::eyre;
use color_eyre::eyre::WrapErr;
use color_eyre::eyre::{Report, Result};
use futures::future::join_all;
use tracing::{error, info};

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
    _executor: &Option<ExecutorArgument>,
    verbose: &Verbosity,
) -> Result<(), Report> {
    let config = ApplicationConfiguration::resolve(config)
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
    let response = match config {
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
                                    register_runner(details, verbose)
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
                        register_runner(details, verbose).await
                    }
                    | Err(why) => Err(why),
                }
            }
            | Err(why) => Err(why),
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
                            info!("=> {} Created GitLab runner — {response:#?}", Label::using());
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
async fn register_runner(details: RunnerDetails, verbose: &Verbosity) -> Result<(), Report> {
    let executor = details.executor;
    let identifier = details.identifier.unwrap_or_default();
    let gitlab_url = details.host.as_deref().unwrap_or("gitlab.com");
    let url = format!("https://{gitlab_url}/");
    let name = &details.name.unwrap_or_else(|| "gitlab-runner".to_string());
    let binary = executor.command_name();
    let description = format!("GitLab Runner [{identifier}]");
    let log_level = if verbose.is_silent() { "panic" } else { "debug" };
    command_exists(binary)
        .then_some(())
        .ok_or_else(|| eyre!("{binary} is required to register a GitLab runner but was not found"))?;
    let token = details
        .token
        .clone()
        .ok_or_else(|| eyre!("GitLab runner creation did not return a registration token"))?;
    let config_host_dir = if cfg!(target_os = "macos") {
        "/Users/Shared/gitlab-runner/config"
    } else {
        "/srv/gitlab-runner/config"
    };
    let gpu_template_inside = if details.gpu_enabled {
        let template_path = PathBuf::from(config_host_dir).join("gpu.template.toml");
        let content = "[[runners]]\n  [runners.docker]\n    gpus = \"all\"\n";
        if let Some(parent) = template_path.parent() {
            let _ = create_dir_all(parent);
        }
        let _ = write(&template_path, content);
        Some("/etc/gitlab-runner/gpu.template.toml".to_string())
    } else {
        None
    };
    let result = match executor {
        | Executor::Docker => {
            let create = args![
                "run",
                "--detach",
                "--name",
                name,
                "--restart",
                "always",
                "-v",
                format!("{config_host_dir}:/etc/gitlab-runner"),
                "-v",
                "/var/run/docker.sock:/var/run/docker.sock",
                &details.docker_image,
            ];
            match cmd!("docker", create) {
                | Err(why) => Err(eyre!("Failed to spawn docker container — {why}")),
                | Ok(output) if !output.status.success() => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(eyre!("Failed to create GitLab runner container: {stderr}"))
                }
                | Ok(_) => {
                    let register = build_container_register_cmd(name, &url, &description, &token, log_level, &gpu_template_inside);
                    match cmd!("docker", register) {
                        | Ok(output) if output.status.success() => {
                            info!("=> {} GitLab runner registered", Label::pass());
                            Ok(())
                        }
                        | Ok(output) => {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            Err(eyre!("gitlab-runner register failed: {stderr}"))
                        }
                        | Err(why) => Err(eyre!("Failed to execute docker exec — {why}")),
                    }
                }
            }
        }
        | Executor::Podman => {
            let socket = detect_podman_socket()
                .ok_or_else(|| eyre!("Podman socket not found. Start Podman with 'podman system service --time=0' then retry"))?;
            let socket_str = socket.to_string_lossy().to_string();
            let create = args![
                "run",
                "--detach",
                "--name",
                name,
                "--restart",
                "always",
                "-v",
                format!("{config_host_dir}:/etc/gitlab-runner"),
                "-v",
                format!("{socket_str}:/var/run/docker.sock"),
                &details.docker_image,
            ];
            match cmd!("podman", create) {
                | Err(why) => Err(eyre!("Failed to spawn podman container — {why}")),
                | Ok(output) if !output.status.success() => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(eyre!("Failed to create GitLab runner container: {stderr}"))
                }
                | Ok(_) => {
                    let register = build_container_register_cmd(name, &url, &description, &token, log_level, &gpu_template_inside);
                    match cmd!("podman", register) {
                        | Ok(output) if output.status.success() => {
                            info!("=> {} GitLab runner registered", Label::pass());
                            Ok(())
                        }
                        | Ok(output) => {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            Err(eyre!("gitlab-runner register failed: {stderr}"))
                        }
                        | Err(why) => Err(eyre!("Failed to execute podman exec — {why}")),
                    }
                }
            }
        }
        | Executor::Apptainer => {
            let instance = args![
                "instance",
                "start",
                "--bind",
                format!("{config_host_dir}:/etc/gitlab-runner"),
                "--bind",
                "/var/run/docker.sock:/var/run/docker.sock",
                format!("docker://{}", details.docker_image),
                name,
            ];
            match cmd!("apptainer", instance) {
                | Err(why) => Err(eyre!("Failed to spawn apptainer instance — {why}")),
                | Ok(output) if !output.status.success() => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(eyre!("Failed to create Apptainer instance: {stderr}"))
                }
                | Ok(_) => {
                    let register_args: Vec<OsString> = {
                        let base: Vec<OsString> = vec![
                            "exec".into(),
                            format!("instance://{name}").into(),
                            "gitlab-runner".into(),
                            "--log-level".into(),
                            log_level.into(),
                            "register".into(),
                            "--non-interactive".into(),
                            "--url".into(),
                            url.clone().into(),
                            "--description".into(),
                            description.clone().into(),
                            "--token".into(),
                            token.into(),
                            "--executor".into(),
                            "apptainer".into(),
                        ];
                        match &gpu_template_inside {
                            | Some(template) => base.into_iter().chain(["--template-config".into(), template.into()]).collect(),
                            | None => base,
                        }
                    };
                    match cmd!("apptainer", register_args) {
                        | Ok(output) if output.status.success() => {
                            info!("=> {} GitLab runner registered", Label::pass());
                            Ok(())
                        }
                        | Ok(output) => {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            Err(eyre!("gitlab-runner register failed: {stderr}"))
                        }
                        | Err(why) => Err(eyre!("Failed to execute apptainer exec — {why}")),
                    }
                }
            }
        }
        | Executor::Shell | Executor::Ssh | Executor::Kubernetes | Executor::Sandbox | Executor::VirtualMachine | Executor::Other => {
            let register_args: Vec<OsString> = {
                let base: Vec<OsString> = vec![
                    "gitlab-runner".into(),
                    "--log-level".into(),
                    log_level.into(),
                    "register".into(),
                    "--non-interactive".into(),
                    "--url".into(),
                    url.into(),
                    "--description".into(),
                    description.into(),
                    "--token".into(),
                    token.into(),
                    "--executor".into(),
                    executor.gitlab_executor_name().into(),
                ];
                match &gpu_template_inside {
                    | Some(template) => base.into_iter().chain(["--template-config".into(), template.into()]).collect(),
                    | None => base,
                }
            };
            match cmd!("gitlab-runner", register_args) {
                | Err(why) => Err(eyre!("Failed to spawn gitlab-runner register — {why}")),
                | Ok(output) if output.status.success() => {
                    info!("=> {} GitLab runner registered", Label::pass());
                    Ok(())
                }
                | Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(eyre!("gitlab-runner register failed: {stderr}"))
                }
            }
        }
    };
    if details.gpu_enabled {
        let template_path = PathBuf::from(config_host_dir).join("gpu.template.toml");
        let _ = remove_file(template_path);
    }
    result
}
/// Detect the Podman socket path, preferring rootless.
fn detect_podman_socket() -> Option<PathBuf> {
    if let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") {
        let path = PathBuf::from(runtime_dir).join("podman").join("podman.sock");
        if path.exists() {
            return Some(path);
        }
    }
    let rootful = PathBuf::from("/run/podman/podman.sock");
    if rootful.exists() {
        return Some(rootful);
    }
    None
}
/// Build the `exec <name> gitlab-runner register ...` argument list for
/// Docker/Podman container-based executors.
fn build_container_register_cmd(
    name: &str,
    url: &str,
    description: &str,
    token: &str,
    log_level: &str,
    gpu_template: &Option<String>,
) -> Vec<OsString> {
    let base: Vec<OsString> = vec![
        "exec".into(),
        name.into(),
        "gitlab-runner".into(),
        "--log-level".into(),
        log_level.into(),
        "register".into(),
        "--non-interactive".into(),
        "--url".into(),
        url.into(),
        "--description".into(),
        description.into(),
        "--token".into(),
        token.into(),
        "--executor".into(),
        "docker".into(),
        "--docker-image".into(),
        "docker".into(),
        "--docker-volumes".into(),
        "/var/run/docker.sock:/var/run/docker.sock".into(),
    ];
    match gpu_template {
        | Some(template) => base.into_iter().chain(["--template-config".into(), template.into()]).collect(),
        | None => base,
    }
}
