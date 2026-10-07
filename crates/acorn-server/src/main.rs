//! Standalone ACORN dual-protocol server.
use acorn_server::{serve, ServerArgs, ServerConfig};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn core::error::Error + Send + Sync>> {
    match ServerConfig::resolve(ServerArgs::parse()) {
        | Ok(config) => {
            tracing_subscriber::fmt().with_env_filter(config.tracing_level.clone()).init();
            serve(config).await
        }
        | Err(error) => Err(error),
    }
}
