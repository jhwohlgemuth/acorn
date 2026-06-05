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
use tracing::{error, info};

fn gitlab_identifier(repository: &Repository) -> Option<String> {
    match repository {
        | Repository::GitLab { id: Some(value), .. } => Some(value.to_string()),
        | _ => None,
    }
}

pub async fn run(
    config: &Option<PathBuf>,
    description: &Option<String>,
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
            .and_then(|value| gitlab_identifier(&value.code_repository))
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
    let response = match gitlab::runners(&gitlab::Options::from_env()).await {
        | Ok(existing) => match config {
            | Some(runners) => {
                let mut created = 0usize;
                let mut failures = Vec::<String>::new();
                for runner in runners {
                    let name = runner.name.as_deref().unwrap_or("<unnamed runner>");
                    let target = build_target(Some(&runner));
                    let result = match target {
                        | Ok((desired_description, runner_type, identifier, desired_run_untagged, desired_tags)) => {
                            let repository = match runner.code_repository.clone() {
                                | Repository::GitLab { location, .. } => Repository::GitLab {
                                    id: identifier.parse::<u64>().ok(),
                                    location,
                                },
                                | _ => Repository::GitLab {
                                    id: identifier.parse::<u64>().ok(),
                                    location: Location::Simple("https://code.ornl.gov".to_string()),
                                },
                            };
                            let details = RunnerDetails::at(repository)
                                .name(name)
                                .description(desired_description)
                                .runner_type(&runner_type.to_string())
                                .gpu_enabled(runner.gpu_enabled)
                                .maybe_tags(desired_tags)
                                .run_untagged(desired_run_untagged)
                                .build();
                            create_runner(details, &existing).await
                        }
                        | Err(why) => Err(why),
                    };
                    match result {
                        | Ok(_) => {
                            created = created.saturating_add(1);
                        }
                        | Err(why) => {
                            failures.push(format!("{name}: {why}"));
                        }
                    }
                }
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
                        location: Location::Simple("https://code.ornl.gov".to_string()),
                    };
                    let details = RunnerDetails::at(repository)
                        .name("GitLab Runner".to_string())
                        .description(desired_description)
                        .runner_type(&runner_type.to_string())
                        .gpu_enabled(false)
                        .maybe_tags(desired_tags)
                        .run_untagged(desired_run_untagged)
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
    let RunnerDetails {
        description,
        runner_type,
        code_repository,
        run_untagged,
        tags,
        ..
    } = details;
    let description_result = match description {
        | Some(value) => Ok(value),
        | None => Err(eyre!("GitLab runner description is required to detect existing runners")),
    };
    let identifier_result = match gitlab_identifier(&code_repository) {
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
                    .runner_type(&runner_type.to_string())
                    .description(description.clone())
                    .run_untagged(run_untagged);
                let metadata = match tags {
                    | Some(values) if !values.is_empty() => {
                        let tag_refs = values.iter().map(String::as_str).collect::<Vec<_>>();
                        runner_builder.tags(&tag_refs).build()
                    }
                    | _ => runner_builder.build(),
                };
                let options = gitlab::Options::from_env();
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
