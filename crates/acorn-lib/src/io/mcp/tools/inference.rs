//! Routed bindings for explicitly configured OpenAI-compatible endpoints.
use super::{BindingAvailability, BindingKind, InferenceBackend, InferenceBindingOptions, ToolRegistry};
use crate::error::ApiResult;
use crate::io::api::openai::{Client, EndpointMode, Options};
use crate::io::api::{Configuration, Endpoint};
use crate::io::config::ApplicationConfiguration;
use crate::io::mcp::{EndpointDataPolicy, ExecutorConfiguration};
use acorn_schema::validation::Validate;
use alloc::sync::Arc;
use color_eyre::eyre::eyre;
use core::net::IpAddr;

impl ApplicationConfiguration {
    fn configured_endpoint(&self, name: &str) -> ApiResult<&Endpoint> {
        let matches = self
            .endpoints
            .as_ref()
            .into_iter()
            .flat_map(|endpoints| endpoints.iter())
            .filter(|endpoint| endpoint.name == name)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            | [endpoint] => Ok(*endpoint),
            | [] => Err(eyre!("Inference endpoint '{name}' is not configured")),
            | _ => Err(eyre!("Inference endpoint '{name}' is ambiguous")),
        }
    }
}
impl ToolRegistry {
    /// Register configured inference endpoints without contacting them.
    pub fn register_endpoint_bindings(self, configuration: &ApplicationConfiguration, offline: bool) -> ApiResult<Self> {
        self.register_endpoint_bindings_with(configuration, |endpoint, model, ollama| {
            let options = if ollama { Options::default() } else { Options::from_env() }.with_domain(endpoint.base());
            let mode = EndpointMode::ChatCompletions;
            let client = if ollama {
                Client::ollama(options, mode, model)
            } else {
                Client::new(options, mode, model)
            };
            Ok(Arc::new(client.with_offline(offline)))
        })
    }
    pub(super) fn register_endpoint_bindings_with<F>(self, configuration: &ApplicationConfiguration, backend_for: F) -> ApiResult<Self>
    where
        F: Fn(&Endpoint, &str, bool) -> ApiResult<Arc<dyn InferenceBackend>>,
    {
        configuration
            .executors
            .iter()
            .filter_map(|configured| match &configured.executor {
                | ExecutorConfiguration::Endpoint {
                    endpoint,
                    model,
                    data_policy,
                } => Some((configured, endpoint, model, data_policy)),
                | _ => None,
            })
            .try_fold(self, |registry, (configured, name, model, data_policy)| {
                configured
                    .validate()
                    .map_err(|why| eyre!("Invalid endpoint binding '{}' — {why}", configured.id))
                    .and_then(|()| configuration.configured_endpoint(name))
                    .and_then(|endpoint| {
                        endpoint_policy(endpoint, *data_policy, name).and_then(|(kind, availability, offline_safe)| {
                            backend_for(endpoint, model, name.eq_ignore_ascii_case("ollama")).and_then(|backend| {
                                registry.register_inference_binding(
                                    configured,
                                    backend,
                                    InferenceBindingOptions {
                                        agent: None,
                                        model: Some(model.clone()),
                                        kind,
                                        availability,
                                        offline_safe,
                                    },
                                )
                            })
                        })
                    })
            })
    }
}
fn endpoint_policy(endpoint: &Endpoint, data_policy: EndpointDataPolicy, name: &str) -> ApiResult<(BindingKind, BindingAvailability, bool)> {
    reqwest::Url::parse(&endpoint.base())
        .map_err(|why| eyre!("Inference endpoint '{name}' has an invalid URL — {why}"))
        .and_then(|url| {
            url.host_str()
                .ok_or_else(|| eyre!("Inference endpoint '{name}' has no host"))
                .map(|host| {
                    let loopback = host.eq_ignore_ascii_case("localhost") || host.parse::<IpAddr>().is_ok_and(|address| address.is_loopback());
                    let availability = if !loopback && data_policy == EndpointDataPolicy::LocalOnly {
                        BindingAvailability::Unavailable {
                            reason: "endpoint data policy allows only loopback".to_string(),
                        }
                    } else if !loopback && url.scheme() != "https" {
                        BindingAvailability::Unavailable {
                            reason: "remote inference endpoint requires HTTPS".to_string(),
                        }
                    } else {
                        BindingAvailability::Ready
                    };
                    (if loopback { BindingKind::Local } else { BindingKind::Remote }, availability, loopback)
                })
        })
}
