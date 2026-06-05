use acorn::io::api::{gitlab, Configuration};
use acorn::io::config::{ApplicationConfiguration, RunnerDetails, RunnerType};
use acorn::io::InputOutput;
use acorn::prelude::PathBuf;
use acorn::util::Label;
use acorn::{Location, Repository};
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::eyre;
use color_eyre::eyre::WrapErr;
use color_eyre::eyre::{Report, Result};
use futures::future::join_all;
use tracing::{error, info};

#[allow(clippy::too_many_arguments)]
pub async fn run(
    config: &Option<PathBuf>,
    description: &Option<String>,
    host: &Option<String>,
    group: &Option<u64>,
    project: &Option<u64>,
    tags: &[String],
    untagged: &bool,
    _verbose: &Verbosity,
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
        let runner_type_result = match runner {
            | Some(value) => Ok(value.runner_type.clone()),
            | None => match (group, project) {
                | (Some(_), None) => Ok(RunnerType::Group),
                | (None, Some(_)) => Ok(RunnerType::Project),
                | _ => Err(eyre!("Provide exactly one of --group or --project (or use --config with GitLab runners)")),
            },
        };
        let identifier_result = runner
            .and_then(|value| value.code_repository.id())
            .map(Ok)
            .unwrap_or_else(|| match (group, project) {
                | (Some(value), None) | (None, Some(value)) => Ok(value.to_string()),
                | _ => Err(eyre!(
                    "Missing GitLab runner target identifier. Provide --group/--project or repository.id in config"
                )),
            });
        match (desired_description_result, runner_type_result, identifier_result) {
            | (Ok(value), Ok(runner_type), Ok(identifier)) => Ok((value, runner_type, identifier, desired_run_untagged, desired_tags)),
            | (Err(why), _, _) | (_, Err(why), _) | (_, _, Err(why)) => Err(why),
        }
    };
    let options = match host.as_deref() {
        | Some(value) => gitlab::Options::from_env().with_domain(value),
        | None => gitlab::Options::from_env(),
    };
    let response = match gitlab::runners(&options).await {
        | Ok(existing) => match config {
            | Some(runners) => {
                let outcomes = join_all(runners.into_iter().map(|runner| {
                    let existing = &existing;
                    let target = build_target(Some(&runner));
                    async move {
                        let name = runner.name.clone().unwrap_or_else(|| "<unnamed runner>".to_string());
                        match target {
                            | Ok((desired_description, runner_type, identifier, desired_run_untagged, desired_tags)) => {
                                let target_host = host.clone().or_else(|| runner.host().map(|value| value.to_string()));
                                let repository = match runner.code_repository.clone() {
                                    | Repository::GitLab { location, .. } => Repository::GitLab {
                                        id: identifier.parse::<u64>().ok(),
                                        location,
                                    },
                                    | _ => unimplemented!("Only GitLab repositories are supported for runner configuration"),
                                };
                                let details = RunnerDetails::at(repository)
                                    .name(name.clone())
                                    .description(desired_description)
                                    .runner_type(&runner_type.to_string())
                                    .gpu_enabled(runner.gpu_enabled)
                                    .maybe_tags(desired_tags)
                                    .run_untagged(desired_run_untagged)
                                    .maybe_host(target_host)
                                    .build();
                                create_runner(details, existing).await.map_err(|why| format!("{name}: {why}"))
                            }
                            | Err(why) => Err(format!("{name}: {why}")),
                        }
                    }
                }))
                .await;
                let (successful, failed): (Vec<Result<(), String>>, Vec<Result<(), String>>) =
                    outcomes.into_iter().partition(|result| result.is_ok());
                let created = successful.len();
                let failures = failed.into_iter().filter_map(|result| result.err()).collect::<Vec<String>>();
                if failures.is_empty() {
                    Ok(())
                } else {
                    let attempted = created.saturating_add(failures.len());
                    Err(eyre!(
                        "Failed to create {} of {} configured runners:\n{}",
                        failures.len(),
                        attempted,
                        failures.join("\n")
                    ))
                }
            }
            | None => match build_target(None) {
                | Ok((desired_description, runner_type, identifier, desired_run_untagged, desired_tags)) => {
                    let repository = Repository::GitLab {
                        id: identifier.parse::<u64>().ok(),
                        location: Location::Simple("https://gitlab.com".to_string()),
                    };
                    let details = RunnerDetails::at(repository)
                        .name("GitLab Runner".to_string())
                        .description(desired_description)
                        .runner_type(&runner_type.to_string())
                        .gpu_enabled(false)
                        .maybe_tags(desired_tags)
                        .run_untagged(desired_run_untagged)
                        .maybe_host(host.clone())
                        .build();
                    create_runner(details, &existing).await
                }
                | Err(why) => Err(why),
            },
        },
        | Err(why) => Err(why),
    };
    if let Err(why) = response {
        error!("=> {} Create GitLab runner — {why}", Label::fail());
        Err(why)
    } else {
        Ok(())
    }
}
async fn create_runner(details: RunnerDetails, existing: &[gitlab::RunnerMetadata]) -> Result<(), Report> {
    let description_result = match details.description.clone() {
        | Some(value) => Ok(value),
        | None => Err(eyre!("GitLab runner description is required to detect existing runners")),
    };
    let identifier_result = match details.code_repository.id() {
        | Some(value) => Ok(value),
        | None => Err(eyre!(
            "Missing GitLab runner target identifier. Provide --group/--project or repository.id in config"
        )),
    };
    match (description_result, identifier_result) {
        | (Ok(description), Ok(identifier)) => {
            let already_exists = existing
                .iter()
                .any(|value| value.description.as_deref().is_some_and(|candidate| candidate == description));
            if already_exists {
                Err(eyre!("A GitLab runner with the description, '{description}', already exists"))
            } else {
                let runner_builder = gitlab::RunnerMetadata::init()
                    .runner_type(&details.runner_type.to_string())
                    .description(description.clone())
                    .run_untagged(details.run_untagged);
                let metadata = match details.tags.clone() {
                    | Some(values) if !values.is_empty() => {
                        let tag_refs = values.iter().map(String::as_str).collect::<Vec<_>>();
                        runner_builder.tags(&tag_refs).build()
                    }
                    | _ => runner_builder.build(),
                };
                let options = match details.host() {
                    | Some(value) => gitlab::Options::from_env().with_domain(value),
                    | None => gitlab::Options::from_env(),
                };
                let create_options = options.clone().with_identifier(identifier).with_runner(metadata);
                match gitlab::create_runner(&create_options).await {
                    | Ok(response) => {
                        if response.identifier > 0 {
                            info!("=> {} Created GitLab runner — {response:#?}", Label::using());
                            Ok(())
                        } else {
                            Err(eyre!(
                                "GitLab create runner returned an invalid response (missing runner id): {response:#?}"
                            ))
                        }
                    }
                    | Err(why) => Err(why).wrap_err("GitLab create runner request failed"),
                }
                // TODO: Register runner
            }
        }
        | (Err(why), _) | (_, Err(why)) => Err(why),
    }
}
