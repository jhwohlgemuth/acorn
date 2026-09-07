//! Model list file parsing.
use crate::io::{ApiResult, Source};
use acorn_core::prelude::String;
use acorn_schema::agent::{ModelList, ModelSelectors};

/// Host I/O operations for schema-owned model selector collections.
#[async_trait::async_trait]
pub trait ModelSelectorsExt: Sized {
    /// Merge selectors read from an optional local or remote model-list source.
    async fn resolve(self, source: &Option<String>, offline: bool) -> ApiResult<Self>;
}

#[async_trait::async_trait]
impl ModelSelectorsExt for ModelSelectors {
    async fn resolve(self, source: &Option<String>, offline: bool) -> ApiResult<Self> {
        match source {
            | Some(source) => Source::read(source, offline)
                .await
                .and_then(|content| ModelList::parse(&content).and_then(ModelList::selectors).map_err(Into::into))
                .map(|file| self.merge(file)),
            | None => Ok(self),
        }
    }
}

#[cfg(test)]
mod tests;
