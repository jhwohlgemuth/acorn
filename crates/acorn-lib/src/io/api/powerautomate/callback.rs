//! Runtime callback resolution for PowerAutomate events.
use super::PowerAutomateConfig;
use crate::io::api::webhooks::{CallbackDestination, CallbackResolver, CallbackTarget, ErrorKind, WebhookError};
use std::env;

/// Resolver that loads callback URLs and signing credentials only at attempt time.
#[derive(Clone, Debug)]
pub struct PowerAutomateCallbackResolver {
    config: PowerAutomateConfig,
}
impl CallbackResolver for PowerAutomateCallbackResolver {
    fn resolve(&self, name: &str) -> Result<CallbackDestination, WebhookError> {
        match name == self.config.callback_destination {
            | false => Err(WebhookError::new(ErrorKind::InvalidTarget, "Unknown PowerAutomate callback destination")),
            | true => env::var(&self.config.variables.callback_url)
                .map_err(|_| WebhookError::new(ErrorKind::InvalidTarget, "PowerAutomate callback URL is unavailable"))
                .and_then(CallbackTarget::new)
                .map(|target| CallbackDestination::new(target, Vec::new())),
        }
    }
}
impl PowerAutomateCallbackResolver {
    /// Construct a resolver from validated deployment configuration.
    pub fn new(config: PowerAutomateConfig) -> Self {
        Self { config }
    }
}
