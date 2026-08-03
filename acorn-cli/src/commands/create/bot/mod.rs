use crate::cli::arguments::bot::EventSource;
use acorn::io::api::gitlab::WebhookOptions;
use acorn::io::{Executor, Remote};
use acorn::prelude::OsString;
use acorn::util::constants::env::{CI_SERVER_HOST, GITLAB_CONTAINER_ENV};
use acorn::util::Label;
use acorn::{args, cmd};
use color_eyre::eyre::{eyre, Report, Result};
use tracing::{error, info};
use validator::Validate;

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
    event_source: EventSource,
    public_url: &Option<String>,
    register_webhook: bool,
    volume: &Option<String>,
    remote: &Option<Remote>,
) -> Result<(), Report> {
    let selected = runtime.as_ref().unwrap_or(&Executor::Docker);
    let container_name = name.clone().unwrap_or(format!("acorn-bot-{identifier}"));
    let image_tag = image.as_deref().unwrap_or("acorn:latest");
    let webhook_options = WebhookOptions::from_env(public_url.as_deref());
    let webhooks_enabled = event_source.is_webhook() || event_source.is_hybrid() || register_webhook;
    let validated = if webhooks_enabled {
        webhook_options.validate().map_err(Report::from)
    } else {
        Ok(())
    };
    let is_valid = match (remote.as_ref(), selected.is_docker()) {
        | (Some(endpoint), false) => Err(eyre!("Remote Docker target '{endpoint}' requires the docker runtime, not {selected}")),
        | _ => Ok(()),
    };
    match (is_valid, selected.is_available(), validated) {
        | (Err(why), _, _) | (_, true, Err(why)) => Err(why),
        | (_, false, _) => {
            let err = eyre!("{selected} is required to create a bot container but was not found");
            error!("=> {} {err}", Label::fail());
            Err(err)
        }
        | (_, true, Ok(())) => {
            let command = create_bot_args(
                identifier,
                &container_name,
                image_tag,
                bind.as_deref(),
                *poll_interval,
                after.as_deref(),
                domain.as_deref(),
                event_source,
                public_url.as_deref(),
                register_webhook,
                volume.as_deref(),
            );
            let container_args = match remote.as_ref() {
                | Some(remote) => remote.docker_args(command),
                | None => command,
            };
            match cmd!(selected, container_args) {
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
                    let err = eyre!("Failed to spawn {selected} — {why}");
                    error!("=> {} {err}", Label::fail());
                    Err(err)
                }
            }
        }
    }
}
#[allow(clippy::too_many_arguments)]
fn create_bot_args(
    identifier: &str,
    container_name: &str,
    image: &str,
    bind: Option<&str>,
    poll_interval: Option<u64>,
    after: Option<&str>,
    domain: Option<&str>,
    event_source: EventSource,
    public_url: Option<&str>,
    register_webhook: bool,
    volume: Option<&str>,
) -> Vec<OsString> {
    let port = bind.and_then(bind_port).unwrap_or(3000);
    let internal_bind = format!("0.0.0.0:{port}");
    let volume = volume.map(str::to_string).unwrap_or_else(|| format!("acorn-bot-{identifier}-state"));
    let environment_args = GITLAB_CONTAINER_ENV
        .into_iter()
        .filter(|name| std::env::var(name).is_ok_and(|value| !value.trim().is_empty()))
        .flat_map(|name| args!["--env", name])
        .collect::<Vec<_>>();
    let domain_args = domain.map_or_else(|| args![], |domain| args!["--env", format!("{CI_SERVER_HOST}={domain}")]);
    let poll_args = poll_interval.map_or_else(|| args![], |interval| args!["--poll-interval", interval.to_string()]);
    let after_args = after.map_or_else(|| args![], |after| args!["--after", after]);
    let public_url_args = public_url.map_or_else(|| args![], |public_url| args!["--public-url", public_url]);
    let register_args = if register_webhook { args!["--register-webhook"] } else { args![] };
    args![
        "run",
        "--detach",
        "--name",
        container_name,
        "--restart",
        "always",
        "--publish",
        format!("{port}:{port}"),
        "--mount",
        format!("type=volume,source={volume},target=/var/lib/acorn"),
        "--env",
        "ACORN_DATABASE_PATH=/var/lib/acorn/acorn.db",
        ..environment_args,
        ..domain_args,
        image,
        "serve",
        "bot",
        identifier,
        "--bind",
        internal_bind,
        "--event-source",
        event_source.to_string(),
        ..poll_args,
        ..after_args,
        ..public_url_args,
        ..register_args
    ]
}
fn bind_port(bind: &str) -> Option<u16> {
    bind.rsplit_once(':')
        .and_then(|(_, port)| port.parse().ok())
        .or_else(|| bind.parse().ok())
}
#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    #[test]
    fn test_remote_container_targets_the_requested_docker_host() {
        let remote = "ssh://builder".parse::<Remote>().unwrap();
        let command = remote
            .docker_args(create_bot_args(
                "42",
                "acorn-bot-42",
                "acorn:latest",
                None,
                Some(10),
                None,
                None,
                EventSource::Poll,
                None,
                false,
                None,
            ))
            .into_iter()
            .map(|value| value.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert!(command
            .iter()
            .map(String::as_str)
            .take(4)
            .eq(["--host", "ssh://builder", "run", "--detach"]));
    }
    #[tokio::test]
    async fn test_remote_container_rejects_non_docker_runtime_before_execution() {
        let remote = "ssh://builder".parse::<Remote>().unwrap();
        let result = run(
            "42",
            &None,
            &None,
            &Some(Executor::Podman),
            &None,
            &None,
            &None,
            &None,
            EventSource::Poll,
            &None,
            false,
            &None,
            &Some(remote),
        )
        .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires the docker runtime"));
    }
    #[test]
    fn test_webhook_container_is_reachable_durable_and_redacts_credentials() {
        temp_env::with_vars(
            [
                ("GITLAB_TOKEN", Some("outbound-secret")),
                ("GITLAB_WEBHOOK_TOKEN", Some("inbound-secret")),
                ("GITLAB_WEBHOOK_SIGNING_TOKEN", None),
            ],
            || {
                let args = create_bot_args(
                    "42",
                    "acorn-bot-42",
                    "acorn:latest",
                    Some("localhost:8080"),
                    Some(10),
                    None,
                    None,
                    EventSource::Webhook,
                    Some("https://bot.example.org"),
                    true,
                    None,
                )
                .into_iter()
                .map(|value| value.to_string_lossy().into_owned())
                .collect::<Vec<_>>();
                assert!(args.windows(2).any(|values| values == ["--publish", "8080:8080"]));
                assert!(args
                    .windows(2)
                    .any(|values| { values == ["--mount", "type=volume,source=acorn-bot-42-state,target=/var/lib/acorn",] }));
                assert!(args.windows(2).any(|values| values == ["--bind", "0.0.0.0:8080"]));
                assert!(args.windows(2).any(|values| values == ["--env", "GITLAB_TOKEN"]));
                assert!(args.windows(2).any(|values| values == ["--env", "GITLAB_WEBHOOK_TOKEN"]));
                assert!(!args.iter().any(|value| value.contains("secret")));
            },
        );
    }
}
