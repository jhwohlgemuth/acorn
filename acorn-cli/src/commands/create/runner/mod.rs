use acorn::io::api::gitlab::{self, RunnerCreationResponse};
use acorn::io::api::Configuration;
use acorn::io::command_exists;
use acorn::io::config::{ApplicationConfiguration, RunnerDetails, RunnerType};
use acorn::io::{to_absolute_string, Executor, InputOutput};
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
    _executor: &Option<Executor>,
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
async fn register_runner(details: RunnerDetails, verbose: &Verbosity) -> Result<(), Report> {
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
    let binary = executor.command().unwrap_or("gitlab-runner");
    let description = format!("GitLab Runner [{identifier}]");
    let log_level = if verbose.is_silent() { "panic" } else { "debug" };
    let config_host_dir = Executor::default_gitlab_runner_config_directory();
    let maybe_template = if gpu_enabled {
        let parent = PathBuf::from(config_host_dir);
        let template = parent.join("gpu.template.toml");
        let content = "[[runners]]\n  [runners.docker]\n    gpus = \"all\"\n";
        create_dir_all(parent)
            .and_then(|_| write(&template, content))
            .map(|_| template)
            .map(to_absolute_string)
            .ok()
    } else {
        None
    };
    let result = if command_exists(binary) {
        match token {
            | Some(token) => match executor {
                | Executor::Docker => match executor.socket() {
                    | Some(socket) => {
                        let gpu_args = if gpu_enabled { args!["--gpus", "all"] } else { args![] };
                        let create_container = args![
                            "run",
                            "--detach",
                            ("--name", name),
                            ("--restart", "always"),
                            ..gpu_args,
                            ("-v", format!("{config_host_dir}:/etc/gitlab-runner")),
                            ("-v", format!("{socket}:/var/run/docker.sock")),
                            &docker_image,
                        ];
                        match cmd!(binary, create_container) {
                            | Ok(output) => {
                                if output.status.success() {
                                    let register = build_register_args(name, &url, &description, &token, log_level, &maybe_template);
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
                            ("-v", format!("{socket}:/var/run/docker.sock")),
                            &docker_image,
                        ];
                        match cmd!(binary, create_container) {
                            | Ok(output) => {
                                if output.status.success() {
                                    let register = build_register_args(name, &url, &description, &token, log_level, &maybe_template);
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
                            ("--bind", format!("{socket}:/var/run/docker.sock")),
                            format!("docker://{docker_image}"),
                            name,
                        ];
                        match cmd!(binary, create) {
                            | Ok(output) => {
                                if output.status.success() {
                                    let gpu_args = match &maybe_template {
                                        | Some(template) => args!["--template-config", template],
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
                    let gpu_args = match &maybe_template {
                        | Some(template) => args!["--template-config", template],
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
        }
    } else {
        Err(eyre!("{binary} is required to register a GitLab runner but was not found"))
    };
    if gpu_enabled {
        let template = PathBuf::from(config_host_dir).join("gpu.template.toml");
        let _ = remove_file(template);
    }
    result
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
        ("--docker-volumes", "/var/run/docker.sock:/var/run/docker.sock"),
        ..gpu_args
    ]
}
