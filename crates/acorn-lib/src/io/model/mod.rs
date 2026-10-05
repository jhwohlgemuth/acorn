//! Model list parsing and local model metadata inspection.
use crate::io::{walk, ApiResult, Source};
use acorn_core::prelude::String;
use acorn_schema::agent::{ModelList, ModelSelectors};
use alloc::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(in crate::io) mod gguf;

/// Read a model's context length using the metadata or materialization context it requires.
pub trait ContextLength<Context = ()> {
    /// Value returned after reading the context length.
    type Output;

    /// Return the model's context length.
    fn context_length(&self, context: Context) -> Self::Output;
}
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
/// Read an architecture-specific context length from a GGUF file or directory.
pub fn gguf_context_length(path: &Path) -> ApiResult<Option<u64>> {
    match path.is_dir() {
        | true => walk(path)
            .map(|paths| paths.into_iter().collect::<BTreeSet<_>>())
            .map(|paths| paths.into_iter().map(|relative| path.join(relative)).collect::<Vec<_>>())
            .and_then(|paths| gguf::context_length(&paths)),
        | false => gguf::context_length(&[PathBuf::from(path)]),
    }
}

#[cfg(test)]
mod tests;
