use acorn::io::api::gitlab::{self, bot};
use acorn::io::api::Configuration;
use acorn::io::Executor;
use color_eyre::eyre::{self, Report, Result};
use core::net::SocketAddr;
use core::time::Duration;
use std::net::ToSocketAddrs;

/// Start a GitLab bot server
///
/// When `detach` is true the bot is created as a detached Docker/Podman container instead of running in-process.
pub async fn run(
    identifier: &str,
    bind: &str,
    after: Option<&str>,
    poll_interval: Option<u64>,
    detach: bool,
    runtime: &Option<Executor>,
) -> Result<(), Report> {
    if detach {
        run_detached(identifier, bind, after, poll_interval, runtime).await
    } else {
        let options = gitlab::Options::from_env().with_identifier(identifier);
        let address = resolve_bind_address(bind)?;
        let mut config = bot::Config::new(options, address);
        if let Some(after) = after {
            config = config.with_after(after);
        }
        if let Some(interval) = poll_interval {
            config = config.with_poll_interval(Duration::from_secs(interval));
        }
        bot::Server::new(config).run().await
    }
}
/// Create a detached Docker/Podman container running `acorn serve bot <identifier>`
async fn run_detached(
    identifier: &str,
    bind: &str,
    after: Option<&str>,
    poll_interval: Option<u64>,
    runtime: &Option<Executor>,
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
    )
    .await
}
fn resolve_bind_address(bind: &str) -> Result<SocketAddr, Report> {
    bind.to_socket_addrs()
        .map_err(|e| eyre::eyre!("Invalid bind address '{bind}' — {e}"))?
        .next()
        .ok_or_else(|| eyre::eyre!("Invalid bind address '{bind}' — no address resolved"))
}
