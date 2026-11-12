//! Explicit CLI, environment, and default server configuration.
use crate::{
    util::constants::env::{
        SERVER_ALLOW_MUTATION, SERVER_GRPC_BIND, SERVER_GRPC_ENABLED, SERVER_JSON_RPC_BIND, SERVER_JSON_RPC_ENABLED, SERVER_MAX_REQUEST_BYTES,
        SERVER_REQUEST_TIMEOUT_SECONDS, SERVER_SHUTDOWN_GRACE_SECONDS, SERVER_TLS_CERT, SERVER_TLS_KEY, SERVER_TRACING_LEVEL, SERVICE_TOKEN,
    },
    ServerResult,
};
use clap::Parser;
use core::fmt;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::time::Duration;
use std::env;
use std::path::PathBuf;

const DEFAULT_GRPC_BIND: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 50051);
const DEFAULT_JSON_RPC_BIND: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8001);
const DEFAULT_MAX_REQUEST_BYTES: usize = 1024 * 1024;
const DEFAULT_SECONDS: u64 = 30;
/// Command-line overrides for the dual-protocol server.
#[derive(Clone, Debug, Default, Parser)]
#[command(name = "acorn-server", about = "Serve ACORN over authenticated JSON-RPC and typed gRPC")]
pub struct ServerArgs {
    /// Permit mutation-gated durable operations.
    #[arg(long)]
    pub allow_mutation: bool,
    /// Disable the typed gRPC listener.
    #[arg(long)]
    pub disable_grpc: bool,
    /// Disable the JSON-RPC listener.
    #[arg(long)]
    pub disable_json_rpc: bool,
    /// Typed gRPC listener address.
    #[arg(long)]
    pub grpc_bind: Option<SocketAddr>,
    /// JSON-RPC listener address.
    #[arg(long)]
    pub json_rpc_bind: Option<SocketAddr>,
    /// Maximum request and response bytes.
    #[arg(long)]
    pub max_request_bytes: Option<usize>,
    /// Application request timeout in seconds.
    #[arg(long)]
    pub request_timeout_seconds: Option<u64>,
    /// Graceful shutdown drain in seconds.
    #[arg(long)]
    pub shutdown_grace_seconds: Option<u64>,
    /// PEM certificate chain path.
    #[arg(long)]
    pub tls_cert: Option<PathBuf>,
    /// PEM private key path.
    #[arg(long)]
    pub tls_key: Option<PathBuf>,
    /// Structured tracing filter (for example `info` or `acorn_server=debug`).
    #[arg(long)]
    pub tracing_level: Option<String>,
}
/// Validated server configuration after applying CLI-over-environment precedence.
#[derive(Clone)]
pub struct ServerConfig {
    /// Whether mutation-gated operations may run.
    pub allow_mutation: bool,
    /// Enabled gRPC bind address.
    pub grpc_bind: Option<SocketAddr>,
    /// Enabled JSON-RPC bind address.
    pub json_rpc_bind: Option<SocketAddr>,
    /// Maximum request and response bytes.
    pub max_request_bytes: usize,
    /// Application request timeout.
    pub request_timeout: Duration,
    /// Shutdown drain deadline.
    pub shutdown_grace: Duration,
    /// PEM certificate chain path.
    pub tls_cert: Option<PathBuf>,
    /// PEM private key path.
    pub tls_key: Option<PathBuf>,
    /// Structured tracing filter.
    pub tracing_level: String,
    token: String,
}
impl fmt::Debug for ServerConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ServerConfig")
            .field("allow_mutation", &self.allow_mutation)
            .field("grpc_bind", &self.grpc_bind)
            .field("json_rpc_bind", &self.json_rpc_bind)
            .field("max_request_bytes", &self.max_request_bytes)
            .field("request_timeout", &self.request_timeout)
            .field("shutdown_grace", &self.shutdown_grace)
            .field("tls_cert", &self.tls_cert)
            .field("tls_key", &self.tls_key)
            .field("tracing_level", &self.tracing_level)
            .field("token", &"[redacted]")
            .finish()
    }
}
impl ServerConfig {
    /// Resolve and validate configuration from explicit arguments and the environment.
    pub fn resolve(args: ServerArgs) -> ServerResult<Self> {
        let token: ServerResult<String> = env::var(SERVICE_TOKEN)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| format!("{SERVICE_TOKEN} is required and cannot be empty").into());
        token.and_then(|token| Self::resolve_with_token(args, token))
    }
    /// Resolve configuration with an explicitly supplied bearer secret.
    pub fn resolve_with_token(args: ServerArgs, token: impl Into<String>) -> ServerResult<Self> {
        let token = token.into();
        let token: ServerResult<String> = (!token.trim().is_empty())
            .then_some(token)
            .ok_or_else(|| "Server bearer secret cannot be empty".into());
        let json_rpc_enabled = Self::environment_boolean(SERVER_JSON_RPC_ENABLED).map(|value| value.unwrap_or(!args.disable_json_rpc));
        let grpc_enabled = Self::environment_boolean(SERVER_GRPC_ENABLED).map(|value| value.unwrap_or(!args.disable_grpc));
        let listeners: ServerResult<(Option<SocketAddr>, Option<SocketAddr>)> = json_rpc_enabled.and_then(|json_rpc_enabled| {
            grpc_enabled.and_then(|grpc_enabled| match json_rpc_enabled || grpc_enabled {
                | true => json_rpc_enabled
                    .then(|| address(args.json_rpc_bind, SERVER_JSON_RPC_BIND, DEFAULT_JSON_RPC_BIND))
                    .transpose()
                    .and_then(|json_rpc_bind| {
                        grpc_enabled
                            .then(|| address(args.grpc_bind, SERVER_GRPC_BIND, DEFAULT_GRPC_BIND))
                            .transpose()
                            .map(|grpc_bind| (json_rpc_bind, grpc_bind))
                    }),
                | false => Err("At least one server listener must be enabled".into()),
            })
        });
        let tls_cert = args.tls_cert.or_else(|| env::var_os(SERVER_TLS_CERT).map(PathBuf::from));
        let tls_key = args.tls_key.or_else(|| env::var_os(SERVER_TLS_KEY).map(PathBuf::from));
        let tls: ServerResult<(Option<PathBuf>, Option<PathBuf>)> = match tls_cert.is_some() == tls_key.is_some() {
            | true => Ok((tls_cert, tls_key)),
            | false => Err("Both TLS certificate and private-key paths are required".into()),
        };
        let max_request_bytes = positive_usize(args.max_request_bytes, SERVER_MAX_REQUEST_BYTES, DEFAULT_MAX_REQUEST_BYTES);
        let request_timeout = positive_u64(args.request_timeout_seconds, SERVER_REQUEST_TIMEOUT_SECONDS, DEFAULT_SECONDS).map(Duration::from_secs);
        let shutdown_grace = positive_u64(args.shutdown_grace_seconds, SERVER_SHUTDOWN_GRACE_SECONDS, DEFAULT_SECONDS).map(Duration::from_secs);
        let allow_mutation = match args.allow_mutation {
            | true => Ok(true),
            | false => Self::environment_boolean(SERVER_ALLOW_MUTATION).map(|value| value.unwrap_or(false)),
        };
        let tracing_level = args
            .tracing_level
            .or_else(|| env::var(SERVER_TRACING_LEVEL).ok())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "info".to_string());
        token
            .and_then(|token| listeners.map(|listeners| (token, listeners)))
            .and_then(|(token, (json_rpc_bind, grpc_bind))| tls.map(|tls| (token, json_rpc_bind, grpc_bind, tls)))
            .and_then(|(token, json_rpc_bind, grpc_bind, (tls_cert, tls_key))| {
                validate_listener_security(json_rpc_bind, grpc_bind, tls_cert.is_some())
                    .map(|()| (token, json_rpc_bind, grpc_bind, tls_cert, tls_key))
            })
            .and_then(|values| max_request_bytes.map(|max_request_bytes| (values, max_request_bytes)))
            .and_then(|(values, max_request_bytes)| request_timeout.map(|request_timeout| (values, max_request_bytes, request_timeout)))
            .and_then(|(values, max_request_bytes, request_timeout)| {
                shutdown_grace.map(|shutdown_grace| (values, max_request_bytes, request_timeout, shutdown_grace))
            })
            .and_then(|(values, max_request_bytes, request_timeout, shutdown_grace)| {
                allow_mutation.map(|allow_mutation| (values, max_request_bytes, request_timeout, shutdown_grace, allow_mutation))
            })
            .map(
                |((token, json_rpc_bind, grpc_bind, tls_cert, tls_key), max_request_bytes, request_timeout, shutdown_grace, allow_mutation)| Self {
                    allow_mutation,
                    grpc_bind,
                    json_rpc_bind,
                    max_request_bytes,
                    request_timeout,
                    shutdown_grace,
                    tls_cert,
                    tls_key,
                    tracing_level,
                    token,
                },
            )
    }
    /// Borrow the bearer secret for adapter construction.
    pub(crate) fn token(&self) -> &str {
        &self.token
    }
    fn environment_boolean(variable: &str) -> ServerResult<Option<bool>> {
        env::var(variable)
            .ok()
            .map(|value| match value.to_ascii_lowercase().as_str() {
                | "1" | "true" | "yes" => Ok(true),
                | "0" | "false" | "no" => Ok(false),
                | _ => Err(format!("Invalid boolean in {variable}").into()),
            })
            .transpose()
    }
}
fn address(cli: Option<SocketAddr>, variable: &str, default: SocketAddr) -> ServerResult<SocketAddr> {
    cli.map_or_else(
        || {
            env::var(variable).map_or(Ok(default), |value| {
                value.parse().map_err(|error| format!("Invalid {variable}: {error}").into())
            })
        },
        Ok,
    )
}
fn positive_u64(cli: Option<u64>, variable: &str, default: u64) -> ServerResult<u64> {
    let value = match (cli, env::var(variable).ok()) {
        | (Some(value), _) => Ok(value),
        | (None, Some(value)) => value.parse::<u64>().map_err(|error| format!("Invalid {variable}: {error}").into()),
        | (None, None) => Ok(default),
    };
    value.and_then(|value| {
        (value > 0)
            .then_some(value)
            .ok_or_else(|| format!("{variable} must be greater than zero").into())
    })
}
fn positive_usize(cli: Option<usize>, variable: &str, default: usize) -> ServerResult<usize> {
    let value = match (cli, env::var(variable).ok()) {
        | (Some(value), _) => Ok(value),
        | (None, Some(value)) => value.parse::<usize>().map_err(|error| format!("Invalid {variable}: {error}").into()),
        | (None, None) => Ok(default),
    };
    value.and_then(|value| {
        (value > 0)
            .then_some(value)
            .ok_or_else(|| format!("{variable} must be greater than zero").into())
    })
}
fn validate_listener_security(json_rpc_bind: Option<SocketAddr>, grpc_bind: Option<SocketAddr>, has_tls: bool) -> ServerResult<()> {
    let remote = json_rpc_bind.is_some_and(|value| !value.ip().is_loopback()) || grpc_bind.is_some_and(|value| !value.ip().is_loopback());
    match remote && !has_tls {
        | true => Err("Non-loopback listeners require a TLS certificate and private key".into()),
        | false => Ok(()),
    }
}

#[cfg(test)]
mod tests;
