use acorn::io::api::{gitlab, Configuration};
use acorn::io::config::{ApplicationConfiguration, RunnerType};
use acorn::io::InputOutput;
use acorn::prelude::PathBuf;
use acorn::util::Label;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::eyre;
use color_eyre::eyre::WrapErr;
use color_eyre::eyre::{Report, Result};
use tracing::{error, info};

pub async fn run(
    config: &Option<PathBuf>,
    description: &Option<String>,
    group: &Option<u64>,
    project: &Option<u64>,
    tags: &[String],
    untagged: &bool,
    _verbose: &Verbosity,
) -> Result<(), Report> {
    let runner_config = ApplicationConfiguration::resolve(config)
        .and_then(|path| ApplicationConfiguration::read(path).ok())
        .and_then(|value| value.runners)
        .and_then(|mut values| values.drain(..).next());
    let desired_description_result = runner_config
        .as_ref()
        .and_then(|value| value.description.clone())
        .or_else(|| description.clone())
        .ok_or_else(|| eyre!("GitLab runner description is required to detect existing runners"));
    let desired_tags = runner_config
        .as_ref()
        .and_then(|value| value.tags.clone())
        .or_else(|| (!tags.is_empty()).then(|| tags.to_owned()));
    let desired_run_untagged = *untagged || runner_config.as_ref().map(|value| value.run_untagged).unwrap_or(false);
    let runner_type_result = runner_config
        .as_ref()
        .map(|value| Ok(value.runner_type.clone()))
        .unwrap_or_else(|| match (group, project) {
            | (Some(_), None) => Ok(RunnerType::Group),
            | (None, Some(_)) => Ok(RunnerType::Project),
            | _ => Err(eyre!("Provide exactly one of --group or --project (or use --config with GitLab runners)")),
        });
    let identifier_result = runner_config
        .as_ref()
        .and_then(|value| value.code_repository.id())
        .map(Ok)
        .unwrap_or_else(|| match (group, project) {
            | (Some(value), None) | (None, Some(value)) => Ok(value.to_string()),
            | _ => Err(eyre!(
                "Missing GitLab runner target identifier. Provide --group/--project or repository.id in config"
            )),
        });
    let target = match (desired_description_result, runner_type_result, identifier_result) {
        | (Ok(value), Ok(runner_type), Ok(identifier)) => Ok((value, runner_type, identifier)),
        | (Err(why), _, _) | (_, Err(why), _) | (_, _, Err(why)) => Err(why),
    };
    let result = match target {
        | Ok((desired_description, runner_type, identifier)) => {
            let runner_builder = gitlab::RunnerMetadata::init()
                .runner_type(&runner_type.to_string())
                .description(desired_description.clone())
                .run_untagged(desired_run_untagged);
            let runner_metadata = match desired_tags {
                | Some(values) if !values.is_empty() => {
                    let tag_refs = values.iter().map(String::as_str).collect::<Vec<_>>();
                    runner_builder.tags(&tag_refs).build()
                }
                | _ => runner_builder.build(),
            };
            let options = gitlab::Options::from_env();
            match gitlab::runners(&options).await {
                | Ok(existing) => {
                    let already_exists = existing
                        .iter()
                        .any(|value| value.description.as_deref().is_some_and(|candidate| candidate == desired_description));
                    if already_exists {
                        Err(eyre!("A GitLab runner with the description, '{desired_description}', already exists"))
                    } else {
                        let create_options = options.with_identifier(identifier).with_runner(runner_metadata);
                        match gitlab::create_runner(&create_options).await {
                            | Ok(response) => {
                                info!("=> {} Created GitLab runner — {response:#?}", Label::using());
                                Ok(())
                            }
                            | Err(why) => Err(why).wrap_err("GitLab create runner request failed"),
                        }
                        // TODO: Register runner
                    }
                }
                | Err(why) => Err(why),
            }
        }
        | Err(why) => Err(why),
    };
    if let Err(why) = result {
        error!("=> {} Create GitLab runner — {why}", Label::fail());
        Err(why)
    } else {
        Ok(())
    }
}
