//! Explicit Hugging Face metadata resolution shared by direct and routed import.
use crate::io::api::huggingface::{self, HuggingFaceRepositoryFiles, RepositoryResolution};
use crate::io::ApiResult;
use acorn_schema::agent::Weights;
use color_eyre::eyre::eyre;

/// Resolve GGUF weight metadata for one Hugging Face repository without downloading weights.
pub async fn resolve_import_metadata(options: &huggingface::Options, offline: bool) -> ApiResult<RepositoryResolution<Weights>> {
    let identifier = options
        .identifier
        .as_deref()
        .ok_or_else(|| eyre!("Missing Hugging Face repository identifier"));
    match identifier {
        | Ok(identifier) => match huggingface::parse_identifier(identifier) {
            | Ok(_) if offline => Err(eyre!("Offline mode cannot import remote model metadata: {identifier}")),
            | Ok(_) => HuggingFaceRepositoryFiles::resolve(options)
                .await
                .map(|resolution| resolution.map(Weights::from))
                .and_then(|resolution| match resolution.value().0.is_empty() {
                    | true => Err(eyre!("No GGUF model files found for '{identifier}'")),
                    | false => Ok(resolution),
                }),
            | Err(why) => Err(why),
        },
        | Err(why) => Err(why),
    }
}
