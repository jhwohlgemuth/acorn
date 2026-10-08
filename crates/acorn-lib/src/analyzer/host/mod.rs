//! Native analyzer execution and rendering services
use crate::io::document::{DocumentPath, DocumentQuery};
use acorn_analysis::error::{DocumentTarget, ValidatorIssue};
use serde_json::Value;

pub mod context;
pub mod discovery;
pub mod oci;
pub mod render;
pub mod runtime;
pub mod service;
pub mod tabular;
pub mod vale;

/// Resolve a portable finding into host document-query criteria.
pub(crate) trait DocumentTargetQuery: DocumentTarget {
    /// Build the host query used to locate this finding in a source document.
    fn query(&self) -> DocumentQuery;
}
impl DocumentTargetQuery for ValidatorIssue {
    fn query(&self) -> DocumentQuery {
        let locator = self.locator();
        let params = self.params();
        let source_path = params.get("source_path").and_then(Value::as_str).unwrap_or(&locator);
        let query = DocumentQuery::new().with_path(DocumentPath::parse(source_path));
        let value = match params.get("value") {
            | Some(Value::Array(values)) => params.get("index").and_then(Value::as_u64).and_then(|index| values.get(index as usize)),
            | value => value,
        };
        match value {
            | Some(Value::String(value)) => query.with_value(value),
            | Some(value) if !value.is_array() && !value.is_object() => query.with_value(value.to_string()),
            | None | Some(_) => query,
        }
    }
}

#[cfg(test)]
mod tests;
