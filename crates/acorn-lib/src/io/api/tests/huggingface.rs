use crate::io::api::huggingface::{
    extract_sha256, next_page, target_path_from_sidecar, Candidate, CandidateSelection, Candidates, Downloaded, ModelInfo, ModelInfoExtension,
    Options, RepositoryResolution, SearchOptions,
};
use crate::io::api::Endpoint;
use crate::schema::agent::{ModelDetails, Weight, Weights};
use crate::util::constants::app::DEFAULT_HUGGINGFACE_DOMAIN;
use axum::http::{HeaderMap, HeaderValue};
use serde_json::{json, Value};

fn model(value: Value) -> ModelInfo {
    serde_json::from_value(value).unwrap()
}
#[test]
fn test_request_options_use_default_hugging_face_domain() {
    assert_eq!(Options::init().build().domain, DEFAULT_HUGGINGFACE_DOMAIN);
    assert_eq!(
        SearchOptions::init().identifier("acme/model").basename("model").build().domain,
        DEFAULT_HUGGINGFACE_DOMAIN
    );
}

#[test]
fn test_has_gguf_files() {
    let gguf = model(json!({"id": "mozilla/test-llama", "siblings": [{"rfilename": "tiny-llama.gguf"}]}));
    assert!(gguf.has_gguf_files());
    let non_gguf = model(json!({"id": "openai/gpt-oss-20b", "siblings": [{"rfilename": "model.safetensors"}]}));
    assert!(!non_gguf.has_gguf_files());
}
#[test]
fn test_is_declared_derivative() {
    let candidate = model(json!({"id": "community/quantized", "baseModels": ["OpenAI/GPT-OSS-2B"]}));
    assert!(candidate.is_declared_derivative_of("openai/gpt-oss-2b"));
    let candidate = model(json!({"id": "community/quantized", "tags": ["base_model:quantized:openai/gpt-oss-2b"]}));
    assert!(candidate.is_declared_derivative_of("openai/gpt-oss-2b"));
    let candidate = model(json!({
        "id": "community/quantized",
        "baseModels": ["other/model"],
        "tags": ["base_model:quantized:other/model"]
    }));
    assert!(!candidate.is_declared_derivative_of("openai/gpt-oss-2b"));
}
#[test]
fn test_base_models_accepts_legacy_current_and_null_shapes() {
    let legacy = model(json!({"id": "community/legacy", "baseModels": ["acme/base"]}));
    let current = model(json!({
        "id": "community/current",
        "baseModels": {"relation": "quantized", "models": [{"_id": "internal", "id": "acme/base"}]}
    }));
    let missing = model(json!({"id": "community/missing", "baseModels": null}));
    assert_eq!(legacy.base_models, Some(vec!["acme/base".to_string()]));
    assert_eq!(current.base_models, Some(vec!["acme/base".to_string()]));
    assert_eq!(missing.base_models, None);
    assert!(current.is_declared_derivative_of("ACME/BASE"));
}
#[test]
fn test_next_page_reads_hugging_face_link_header() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "link",
        HeaderValue::from_static("<https://huggingface.co/api/models?p=1>; rel=\"next\", <https://huggingface.co/api/models?p=2>; rel=\"last\""),
    );
    let endpoint = Endpoint::default().with_domain("huggingface.co");
    assert_eq!(
        next_page(&headers, &endpoint).unwrap(),
        Some("https://huggingface.co/api/models?p=1".to_string())
    );
}
#[test]
fn test_next_page_rejects_cross_origin_link() {
    let mut headers = HeaderMap::new();
    headers.insert("link", HeaderValue::from_static("<https://example.org/models?p=1>; rel=\"next\""));
    let endpoint = Endpoint::default().with_domain("huggingface.co");
    assert!(next_page(&headers, &endpoint).is_err());
}
#[test]
fn test_is_declared_variant_accepts_decorated_base_model_names() {
    let candidate = model(json!({
        "id": "unsloth/NVIDIA-Nemotron-3-Super-120B-A12B-GGUF",
        "cardData": {"base_model": ["nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-BF16"]},
        "tags": ["base_model:quantized:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-BF16"]
    }));
    assert!(candidate.is_declared_variant_of("nvidia/nemotron-3-super-120b-a12b"));
    assert!(!candidate.is_declared_variant_of("other/nemotron-3-super-120b-a12b"));
    assert!(!candidate.is_declared_variant_of("nvidia/different-model"));
}
#[test]
fn test_is_declared_variant_accepts_meta_publisher_alias_and_catalog_suffixes() {
    let candidate = model(json!({
        "id": "unsloth/Llama-4-Maverick-17B-128E-Instruct-GGUF",
        "tags": ["base_model:quantized:meta-llama/Llama-4-Maverick-17B-128E-Instruct"]
    }));
    assert!(candidate.is_declared_variant_of("meta/llama-4-maverick-17b-128e-instruct"));
    assert!(candidate.is_declared_variant_of("meta/llama-4-maverick-17b-128e-instruct-fp8"));
    assert!(candidate.is_declared_variant_of("meta/llama-4-maverick-17b-128e-instruct-maas"));
    assert!(!candidate.is_declared_variant_of("other/llama-4-maverick-17b-128e-instruct"));
}
#[test]
fn test_is_declared_variant_accepts_nvidia_version_separator_aliases() {
    let ultra = model(json!({
        "id": "bartowski/nvidia_Llama-3_1-Nemotron-Ultra-253B-v1-GGUF",
        "tags": ["base_model:quantized:nvidia/Llama-3_1-Nemotron-Ultra-253B-v1"]
    }));
    let super_model = model(json!({
        "id": "bartowski/nvidia_Llama-3_3-Nemotron-Super-49B-v1_5-GGUF",
        "tags": ["base_model:quantized:nvidia/Llama-3_3-Nemotron-Super-49B-v1_5"]
    }));
    assert!(ultra.is_declared_variant_of("nvidia/llama-3.1-nemotron-ultra-253b"));
    assert!(super_model.is_declared_variant_of("nvidia/llama-3.3-nemotron-super-49b-v1.5"));
    assert!(!super_model.is_declared_variant_of("nvidia/llama-nemotron-rerank-vl-1b-v2"));
}
#[test]
fn test_gguf_candidate_includes_sorted_unique_quantizations() {
    let candidate = Candidate::from(model(json!({
        "id": "community/quantized",
        "downloads": 42,
        "likes": 7,
        "siblings": [
            {"rfilename": "model-Q5_K_M.gguf"},
            {"rfilename": "model-Q4_K_M.gguf"},
            {"rfilename": "model-Q4_K_M-00001-of-00002.gguf"}
        ]
    })));
    assert_eq!(candidate.downloads, 42);
    assert_eq!(candidate.likes, Some(7));
    assert_eq!(candidate.quantizations, vec!["Q4_K_M", "Q5_K_M"]);
    assert_eq!(candidate.to_string(), "community/quantized");
}
#[test]
fn test_gguf_candidate_excludes_unrecognized_quantizations() {
    let candidate = Candidate::from(model(json!({
        "id": "community/unsupported",
        "siblings": [{"rfilename": "model-tq1_0.gguf"}]
    })));
    assert!(candidate.quantizations.is_empty());
}
#[test]
fn test_fallback_candidates_apply_inclusive_minimum_download_count() {
    let candidate = |id: &str, downloads: Option<u64>| {
        model(json!({
            "id": id,
            "downloads": downloads,
            "tags": ["base_model:quantized:acme/base"],
            "siblings": [{"rfilename": "model-Q4_K_M.gguf"}]
        }))
    };
    let options = SearchOptions::init()
        .identifier("acme/base")
        .basename("base")
        .minimum_download_count(100)
        .build();
    let candidates = Candidates::fallback(
        vec![
            candidate("acme/above-GGUF", Some(101)),
            candidate("acme/boundary-GGUF", Some(100)),
            candidate("acme/below-GGUF", Some(99)),
            candidate("acme/missing-GGUF", None),
        ],
        &options,
    );
    assert_eq!(
        candidates.iter().map(|candidate| candidate.id.as_str()).collect::<Vec<_>>(),
        vec!["acme/above-GGUF", "acme/boundary-GGUF"]
    );
}
#[test]
fn test_weights_to_source_uses_hugging_face_repository_identifier() {
    let source = Weights(vec![Weight {
        label: "Hugging Face".to_string(),
        url: "https://huggingface.co/openai/gpt-oss-20b".to_string(),
        is_open: Some(true),
        quantization: None,
        size: None,
    }])
    .to_source(Some("GPT OSS 20B".to_string()))
    .unwrap();
    assert_eq!(source.identifier(), "openai/gpt-oss-20b");
    assert_eq!(source.name(), "GPT OSS 20B");
}
#[test]
fn test_weights_to_source_keeps_direct_hugging_face_file_url() {
    let url = "https://huggingface.co/openai/gpt-oss-20b/resolve/main/model.gguf".to_string();
    let source = Weights(vec![Weight {
        label: "GGUF".to_string(),
        url: url.to_string(),
        is_open: Some(true),
        quantization: None,
        size: None,
    }])
    .to_source(None)
    .unwrap();
    assert_eq!(source.identifier(), url);
    assert_eq!(source.name(), "GGUF");
}
#[test]
fn test_repository_resolution_identifies_direct_and_fallback_values() {
    let direct = RepositoryResolution::direct("acme/model", 1);
    assert!(!direct.is_fallback());
    assert_eq!(direct.requested(), "acme/model");
    assert_eq!(direct.resolved(), "acme/model");
    assert_eq!(*direct.value(), 1);
    let fallback = RepositoryResolution::new("acme/model", "community/model-GGUF", 2);
    assert!(fallback.is_fallback());
    assert_eq!(fallback.requested(), "acme/model");
    assert_eq!(fallback.resolved(), "community/model-GGUF");
    assert_eq!(fallback.into_parts(), ("acme/model".to_string(), "community/model-GGUF".to_string(), 2));
}
#[test]
fn test_repository_resolution_transforms_values_without_losing_identity() {
    let mapped = RepositoryResolution::new("acme/model", "community/model-GGUF", 2).map(|value| value.to_string());
    assert_eq!(mapped.requested(), "acme/model");
    assert_eq!(mapped.resolved(), "community/model-GGUF");
    assert_eq!(mapped.value(), "2");
    let mapped = RepositoryResolution::direct("acme/model", 2).try_map(|value| if value > 0 { Ok(value * 2) } else { Err("invalid") });
    assert_eq!(
        mapped.map(RepositoryResolution::into_parts),
        Ok(("acme/model".to_string(), "acme/model".to_string(), 4))
    );
}
#[test]
fn test_downloaded_into_resolution_uses_one_identifier() {
    let downloaded = Downloaded::init().identifier("acme/model").revision("main").files(Vec::new()).build();
    let resolution = downloaded.into_resolution("acme/model");
    assert!(!resolution.is_fallback());
    assert_eq!(resolution.requested(), "acme/model");
    assert_eq!(resolution.resolved(), "acme/model");
}
#[test]
fn test_model_details_from_repository_resolution_preserves_fallback() {
    let resolution = RepositoryResolution::new("acme/model", "community/model-GGUF", ModelDetails::default());
    let details = ModelDetails::from(resolution);
    assert_eq!(details.id.as_deref(), Some("community/model-GGUF"));
    assert_eq!(details.fallback.as_deref(), Some("acme/model"));
}
#[test]
fn test_target_path_from_sidecar() {
    assert_eq!(target_path_from_sidecar("model.gguf.sha256"), Some("model.gguf".to_string()));
    assert_eq!(
        target_path_from_sidecar("nested/model.gguf.sha256sum"),
        Some("nested/model.gguf".to_string())
    );
}
#[test]
fn test_extract_sha256() {
    let digest = "ABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCD";
    assert_eq!(
        extract_sha256(format!("{digest}  model.gguf").as_str()).as_deref(),
        Some("abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd"),
    );
    let digest = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    assert_eq!(
        extract_sha256(format!("SHA256 ({digest}) = {digest} model.gguf").as_str()).as_deref(),
        Some(digest),
    );
    assert_eq!(extract_sha256("not-a-sha model.gguf"), None);
    assert_eq!(extract_sha256("0123456789abcdef"), None);
}
