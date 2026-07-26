use crate::cli::arguments::bot::EventSource;
use acorn::io::api::gitlab::{self, bot, WebhookOptions};
use acorn::io::api::Configuration;
use acorn::io::Executor;
use color_eyre::eyre::{self, Report, Result};
use core::net::SocketAddr;
use core::time::Duration;
use std::net::ToSocketAddrs;
use validator::Validate;

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
        )
        .await
    } else {
        run_local(identifier, bind, after, poll_interval, event_source, public_url, register_webhook).await
    }
}
async fn run_local(
    identifier: &str,
    bind: &str,
    after: Option<&str>,
    poll_interval: Option<u64>,
    event_source: EventSource,
    public_url: Option<&str>,
    register_webhook: bool,
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
    match (resolve_bind_address(bind), validation, project_id, register_webhook) {
        | (Ok(address), Ok(()), Ok(project_id), true) => match gitlab::upsert_project_webhook(&options, &webhook_options).await {
            | Ok(_) => start(options, address, event_source, &webhook_options, after, poll_interval, project_id).await,
            | Err(why) => Err(why),
        },
        | (Ok(address), Ok(()), Ok(project_id), false) => {
            start(options, address, event_source, &webhook_options, after, poll_interval, project_id).await
        }
        | (Err(why), _, _, _) | (_, Err(why), _, _) | (_, _, Err(why), _) => Err(why),
    }
}
async fn start(
    options: gitlab::Options,
    address: SocketAddr,
    event_source: EventSource,
    webhook_options: &WebhookOptions,
    after: Option<&str>,
    poll_interval: Option<u64>,
    project_id: Option<u64>,
) -> Result<(), Report> {
    let polling_enabled = event_source.is_poll() || event_source.is_hybrid();
    let config = bot::Config::new(options, address)
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
