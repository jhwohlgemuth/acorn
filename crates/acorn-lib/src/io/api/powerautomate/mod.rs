//! Authenticated PowerAutomate form intake and artifact-notification contracts.
pub use callback::PowerAutomateCallbackResolver;
pub use config::PowerAutomateConfig;
pub use form::{FormEntry, FormReceipt, ReceiptDisposition, Submission, SubmissionRevision, Submitter};
pub use webhook::PowerAutomateProvider;

mod callback;
mod config;
mod form;
pub mod jsonl;
pub mod local;
mod revisions;
mod webhook;

#[cfg(test)]
mod tests;
