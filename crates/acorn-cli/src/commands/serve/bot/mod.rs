use crate::cli::arguments::bot::EventSource;
use acorn::io::api::gitlab::{self, bot, DeploymentConfig, WebhookOptions};
use acorn::io::api::Configuration;
use acorn::io::config::ApplicationConfiguration;
use acorn::io::Executor;
use acorn_schema::validation::Validate;
use color_eyre::eyre::{self, Report, Result};
use core::net::SocketAddr;
use core::time::Duration;
use std::net::ToSocketAddrs;
use std::path::Path;

/// Start a GitLab bot server
///
/// When `detach` is true the bot is created as a detached Docker/Podman container instead of running in-process.
#[allow(clippy::too_many_arguments)]
pub async fn run(
    identifier: &str,
    bind: &str,
    after: Option<&str>,
    poll_interval: Option<u64>,
    detach: bool,
    runtime: &Option<Executor>,
    event_source: EventSource,
    public_url: Option<&str>,
    register_webhook: bool,
    config: Option<&Path>,
) -> Result<(), Report> {
    if detach {
        run_detached(
            identifier,
            bind,
            event_source,
            register_webhook,
            after,
            poll_interval,
            runtime,
            public_url,
            config,
        )
        .await
    } else {
        run_local(identifier, bind, after, poll_interval, event_source, public_url, register_webhook, config).await
    }
}
// These values are the direct CLI deployment contract and are forwarded without mutation.
#[allow(clippy::too_many_arguments)]
async fn run_local(
    identifier: &str,
    bind: &str,
    after: Option<&str>,
    poll_interval: Option<u64>,
    event_source: EventSource,
    public_url: Option<&str>,
    register_webhook: bool,
    config: Option<&Path>,
) -> Result<(), Report> {
    let options = gitlab::Options::from_env().with_identifier(identifier);
    let webhook_options = WebhookOptions::from_env(public_url);
    let webhooks_enabled = event_source.is_webhook() || event_source.is_hybrid() || register_webhook;
    let validation = if webhooks_enabled {
        webhook_options.validate().map_err(Report::from)
    } else {
        Ok(())
    };
    let project_id = webhooks_enabled
        .then(|| {
            identifier
                .parse::<u64>()
                .map_err(|why| eyre::eyre!("Webhook mode requires a numeric GitLab project ID — {why}"))
        })
        .transpose();
    let config_path = config.map(Path::to_path_buf);
    let deployment = ApplicationConfiguration::load(&config_path).and_then(|configuration| {
        let deployment = configuration.workflow.unwrap_or_default();
        deployment.validate().map(|()| deployment)
    });
    match (resolve_bind_address(bind), validation, project_id, deployment, register_webhook) {
        | (Ok(address), Ok(()), Ok(project_id), Ok(deployment), true) => match gitlab::upsert_project_webhook(&options, &webhook_options).await {
            | Ok(_) => {
                start(
                    options,
                    address,
                    event_source,
                    &webhook_options,
                    after,
                    poll_interval,
                    project_id,
                    deployment,
                )
                .await
            }
            | Err(why) => Err(why),
        },
        | (Ok(address), Ok(()), Ok(project_id), Ok(deployment), false) => {
            start(
                options,
                address,
                event_source,
                &webhook_options,
                after,
                poll_interval,
                project_id,
                deployment,
            )
            .await
        }
        | (Err(why), _, _, _, _) | (_, Err(why), _, _, _) | (_, _, Err(why), _, _) | (_, _, _, Err(why), _) => Err(why),
    }
}
// Keep startup inputs explicit so validation completes before constructing the long-running server.
#[allow(clippy::too_many_arguments)]
async fn start(
    options: gitlab::Options,
    address: SocketAddr,
    event_source: EventSource,
    webhook_options: &WebhookOptions,
    after: Option<&str>,
    poll_interval: Option<u64>,
    project_id: Option<u64>,
    deployment: DeploymentConfig,
) -> Result<(), Report> {
    let polling_enabled = event_source.is_poll() || event_source.is_hybrid();
    let config = bot::Config::from(deployment)
        .with_address(address)
        .with_options(options)
        .with_polling_enabled(polling_enabled)
        .with_webhook_options(webhook_options, project_id);
    let config = match after {
        | Some(after) => config.with_after(after),
        | None => config,
    };
    let config = match poll_interval {
        | Some(interval) => config.with_poll_interval(Duration::from_secs(interval)),
        | None => config,
    };
    bot::Server::new(config).run().await
}
/// Create a detached Docker/Podman container running `acorn serve bot <identifier>`
#[allow(clippy::too_many_arguments)]
async fn run_detached(
    identifier: &str,
    bind: &str,
    event_source: EventSource,
    register_webhook: bool,
    after: Option<&str>,
    poll_interval: Option<u64>,
    runtime: &Option<Executor>,
    public_url: Option<&str>,
    config: Option<&Path>,
) -> Result<(), Report> {
    crate::commands::create::bot::run(
        identifier,
        &None,
        &None,
        runtime,
        &Some(bind.to_string()),
        &poll_interval,
        &after.map(String::from),
        &None,
        event_source,
        &public_url.map(String::from),
        register_webhook,
        &config.map(Path::to_path_buf),
        &None,
        &None,
    )
    .await
}
fn resolve_bind_address(bind: &str) -> Result<SocketAddr, Report> {
    match bind.to_socket_addrs() {
        | Ok(addresses) => addresses
            .into_iter()
            .next()
            .ok_or_else(|| eyre::eyre!("Invalid bind address '{bind}' — no address resolved")),
        | Err(why) => Err(eyre::eyre!("Invalid bind address '{bind}' — {why}")),
    }
}
