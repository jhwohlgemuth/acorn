//! Module for downloading model and provider data from models.dev
//!
//! See <https://models.dev> for more information.
use crate::io::api::{ApiResult, DatabasePersistence, RemoteResource, INCLUDED_ENDPOINTS};
use crate::io::database::schema::{ModelRow, ProviderRow, Table};
use crate::io::database::{Database, Operations};
use crate::io::{with_progress, ProgressType};
use crate::schema::agent::{ModelDetails, ProviderDetails};
use crate::util::{Label, Searchable};
use async_trait::async_trait;
use color_eyre::eyre::{eyre, Report};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Process-global cache of the models.dev catalog to avoid redundant downloads
static CATALOG_CACHE: OnceLock<CatalogResponse> = OnceLock::new();
/// Top-level catalog response from models.dev
#[derive(Clone, Debug, Deserialize)]
pub struct CatalogResponse {
    /// Flat map of all models keyed by model ID
    pub models: HashMap<String, ModelDetails>,
    /// Provider entries keyed by provider ID
    pub providers: HashMap<String, ProviderDetails>,
}
/// Response containing only model data from models.dev
#[derive(Clone, Debug, Deserialize)]
pub struct Models {
    /// Flat map of all models keyed by model ID
    pub models: HashMap<String, ModelDetails>,
}
/// Response containing only provider data from models.dev
#[derive(Clone, Debug, Deserialize)]
pub struct Providers {
    /// Provider entries keyed by provider ID
    pub providers: HashMap<String, ProviderDetails>,
}
impl From<ModelDetails> for ModelRow {
    fn from(details: ModelDetails) -> Self {
        let model_id = details.id;
        let name = details.name;
        let family = details.family;
        let variant = details.variant;
        let version = details.version.as_ref().map(|v| v.to_string());
        let attachment = details.attachment;
        let open_weights = details.open_weights;
        let reasoning = details.reasoning;
        let structured_output = details.structured_output;
        let temperature = details.temperature;
        let tool_call = details.tool_call;
        let parameters = details.parameters;
        let release_date = details.release_date;
        let knowledge = details.knowledge;
        let last_updated = details.last_updated;
        let limit = details.limit;
        let cost = details.cost;
        let modalities = details.modalities;
        let benchmarks = details.benchmarks;
        let weights = details.weights;
        let limit_context = limit.as_ref().map(|l| l.context as i64);
        let limit_output = limit.as_ref().and_then(|l| l.output.map(|v| v as i64));
        let limit_input = limit.as_ref().and_then(|l| l.input.map(|v| v as i64));
        let cost_input = cost.as_ref().and_then(|c| c.input);
        let cost_output = cost.as_ref().and_then(|c| c.output);
        let cost_cache_read = cost.as_ref().and_then(|c| c.cache_read);
        let cost_cache_write = cost.as_ref().and_then(|c| c.cache_write);
        let cost_reasoning = cost.as_ref().and_then(|c| c.reasoning);
        let cost_input_audio = cost.as_ref().and_then(|c| c.input_audio);
        let cost_output_audio = cost.as_ref().and_then(|c| c.output_audio);
        let cost_over_200k = cost
            .as_ref()
            .and_then(|c| c.context_over_200k.as_ref())
            .and_then(|c| serde_json::to_string(c).ok());
        let cost_tiers = cost.as_ref().and_then(|c| c.tiers.as_ref()).and_then(|t| serde_json::to_string(t).ok());
        let modality_input = modalities
            .as_ref()
            .map(|m| &m.input)
            .filter(|v| !v.is_empty())
            .map(|v| v.iter().map(ToString::to_string).collect::<Vec<_>>())
            .map(|v| v.join(","));
        let modality_output = modalities
            .as_ref()
            .map(|m| &m.output)
            .filter(|v| !v.is_empty())
            .map(|v| v.iter().map(ToString::to_string).collect::<Vec<_>>())
            .map(|v| v.join(","));
        let benchmarks_json = benchmarks.as_ref().and_then(|b| serde_json::to_string(b).ok());
        let mut row = ModelRow::init()
            .maybe_model_id(model_id.clone())
            .maybe_name(name)
            .maybe_family(family)
            .maybe_variant(variant)
            .maybe_version(version)
            .maybe_attachment(attachment)
            .maybe_open_weights(open_weights)
            .maybe_reasoning(reasoning)
            .maybe_structured_output(structured_output)
            .maybe_temperature(temperature)
            .maybe_tool_call(tool_call)
            .maybe_parameters(parameters)
            .maybe_release_date(release_date)
            .maybe_knowledge(knowledge)
            .maybe_last_updated(last_updated)
            .maybe_limit_context(limit_context)
            .maybe_limit_output(limit_output)
            .maybe_limit_input(limit_input)
            .maybe_modality_input(modality_input)
            .maybe_modality_output(modality_output)
            .maybe_cost_input(cost_input)
            .maybe_cost_output(cost_output)
            .maybe_cost_cache_read(cost_cache_read)
            .maybe_cost_cache_write(cost_cache_write)
            .maybe_cost_reasoning(cost_reasoning)
            .maybe_cost_input_audio(cost_input_audio)
            .maybe_cost_output_audio(cost_output_audio)
            .maybe_cost_over_200k(cost_over_200k)
            .maybe_cost_tiers(cost_tiers)
            .maybe_benchmarks(benchmarks_json)
            .build();
        row.weights = weights
            .unwrap_or_default()
            .infer_quantization(model_id.as_deref().unwrap_or_default())
            .as_ref()
            .and_then(|w| serde_json::to_string(w).ok());
        row
    }
}
impl From<ProviderDetails> for ProviderRow {
    fn from(details: ProviderDetails) -> Self {
        let provider_id = details.id;
        let name = details.name;
        let description = details.description;
        let endpoint = details.endpoint;
        let documentation = details.documentation;
        let authentication = details.authentication.as_ref().and_then(|a| serde_json::to_string(a).ok());
        let env = details.env.filter(|e| !e.is_empty()).map(|e| e.join(","));
        let npm = details.npm;
        let url = details.url;
        let established_date = details.established_date;
        let last_updated = details.last_updated;
        let models = details
            .models
            .as_ref()
            .map(|m| m.iter().filter_map(|d| d.id.clone()).collect::<Vec<_>>())
            .filter(|ids| !ids.is_empty())
            .map(|ids| ids.join(","));
        ProviderRow::init()
            .maybe_provider_id(provider_id)
            .maybe_name(name)
            .maybe_description(description)
            .maybe_endpoint(endpoint)
            .maybe_documentation(documentation)
            .maybe_authentication(authentication)
            .maybe_env(env)
            .maybe_npm(npm)
            .maybe_url(url)
            .maybe_established_date(established_date)
            .maybe_last_updated(last_updated)
            .maybe_models(models)
            .build()
    }
}
#[async_trait]
impl DatabasePersistence for CatalogResponse {
    async fn persist(self, database: Database<Table>) -> ApiResult<usize> {
        match self.models().persist(database.clone()).await {
            | Ok(model_count) => match self.providers().persist(database).await {
                | Ok(provider_count) => model_count
                    .checked_add(provider_count)
                    .ok_or_else(|| eyre!("Count overflow while persisting models.dev metadata")),
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
}
impl CatalogResponse {
    pub(crate) fn models(&self) -> Models {
        Models { models: self.models.clone() }
    }
    pub(crate) fn providers(&self) -> Providers {
        Providers {
            providers: self.providers.clone(),
        }
    }
}
#[async_trait]
impl DatabasePersistence for Models {
    async fn persist(self, database: Database<Table>) -> ApiResult<usize> {
        let models: Vec<ModelDetails> = self.models.into_values().collect();
        let message: fn(&ModelDetails) -> String = |item| format!("Saving \"{}\" model", item.id.as_deref().unwrap_or("unknown"));
        let operation = |item| async { database.insert(ModelRow::from(item)) };
        let finish = |count| format!("{}Saved metadata for {count} models", Label::CHECKMARK);
        with_progress(models, message, operation, finish, None, ProgressType::Bar)
            .await
            .map(|counts| counts.into_iter().sum::<usize>())
            .map_err(Report::msg)
    }
}
#[async_trait]
impl DatabasePersistence for Providers {
    async fn persist(self, database: Database<Table>) -> ApiResult<usize> {
        let providers: Vec<ProviderDetails> = self.providers.into_values().collect();
        let message: fn(&ProviderDetails) -> String = |item| format!("Saving \"{}\" provider", item.id.as_deref().unwrap_or("unknown"));
        let operation = |item| async { database.insert(ProviderRow::from(item)) };
        let finish = |count| format!("{}Saved metadata for {count} providers", Label::CHECKMARK);
        with_progress(providers, message, operation, finish, None, ProgressType::Bar)
            .await
            .map(|counts| counts.into_iter().sum::<usize>())
            .map_err(Report::msg)
    }
}
/// Download catalog data from models.dev
///
/// ## Example
/// ```ignore
/// use acorn::io::api::models_dev;
///
/// let catalog = models_dev::download().await.unwrap();
/// println!("Found {} providers and {} models", catalog.providers.len(), catalog.models.len());
/// ```
pub async fn download() -> ApiResult<CatalogResponse> {
    match INCLUDED_ENDPOINTS.find_by_name("models-dev") {
        | Some(endpoint) => {
            let response = endpoint.invoke("catalog", None).await;
            endpoint.handle::<CatalogResponse>(response)
        }
        | None => Err(eyre!("No models.dev endpoint found")),
    }
}
/// Download catalog data from models.dev, using a process-wide cache
pub async fn download_cached() -> ApiResult<&'static CatalogResponse> {
    match CATALOG_CACHE.get() {
        | Some(cached) => Ok(cached),
        | None => match download().await {
            | Ok(catalog) => {
                let _ = CATALOG_CACHE.set(catalog);
                match CATALOG_CACHE.get() {
                    | Some(cached) => Ok(cached),
                    | None => Err(eyre!("Cache unset after download — concurrent modification or OOM")),
                }
            }
            | Err(why) => Err(why),
        },
    }
}
