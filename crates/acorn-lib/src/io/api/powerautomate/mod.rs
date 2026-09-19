//! Authenticated PowerAutomate form intake and artifact-notification contracts.
mod callback;
mod config;
mod form;
mod webhook;

pub use callback::PowerAutomateCallbackResolver;
pub use config::{PowerAutomateConfig, PowerAutomateVariables, ProjectIntakeConfig};
pub use form::{FormEntry, FormReceipt, FormSubmission, ReceiptDisposition, Submitter};
pub use webhook::PowerAutomateProvider;

#[cfg(test)]
mod tests;
