//! Runtime callback resolution for PowerAutomate events.
use crate::io::api::webhooks::{CallbackDestination, CallbackResolver, CallbackTarget, ErrorKind, WebhookError};
use crate::io::config::application::FolderRegistry;
use crate::util::constants::env::POWER_AUTOMATE_CALLBACK_URL;
use std::env;

/// Resolver that loads callback URLs and signing credentials only at attempt time.
#[derive(Clone, Debug)]
pub struct PowerAutomateCallbackResolver {
    callback_url: Option<String>,
    destination: String,
}
impl CallbackResolver for PowerAutomateCallbackResolver {
    fn resolve(&self, name: &str) -> Result<CallbackDestination, WebhookError> {
        match name == self.destination {
            | false => Err(WebhookError::new(ErrorKind::InvalidTarget, "Unknown PowerAutomate callback destination")),
            | true => self
                .callback_url
                .as_deref()
                .ok_or_else(|| {
                    WebhookError::new(
                        ErrorKind::InvalidTarget,
                        "PowerAutomate callback URL is not referenced by variables.custom",
                    )
                })
                .and_then(|variable| {
                    env::var(variable).map_err(|_| WebhookError::new(ErrorKind::InvalidTarget, "PowerAutomate callback URL is unavailable"))
                })
                .and_then(CallbackTarget::new)
                .map(|target| CallbackDestination::new(target, Vec::new())),
        }
    }
}
impl PowerAutomateCallbackResolver {
    /// Construct a resolver from a validated callback destination and routing snapshot.
    pub fn new(destination: String, routing: &FolderRegistry) -> Self {
        Self {
            callback_url: routing.variable(POWER_AUTOMATE_CALLBACK_URL).map(str::to_owned),
            destination,
        }
    }
}
