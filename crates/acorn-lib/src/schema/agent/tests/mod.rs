#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]
use crate::io::InputOutput;
use crate::prelude::{env, remove_file, PathBuf};
use crate::schema::agent::{
    opencode, Benchmark, CostDetails, CostTier, FrontMatter, Harness, LimitDetails, Metric, Modalities, ModelDetails, ModelResolutionReason,
    ModelSelector, ModelSelectors, PromptFileAsset, PromptTemplate, PromptTemplateConfiguration, Quantization, TierInfo, Weight, Weights,
};
use crate::schema::research_activity::aspect::data::Modality;
use crate::schema::OneOrMany;
use crate::util::MarkdownSupport;

fn all_known_assets() -> [PromptFileAsset; 6] {
    [
        PromptFileAsset::Eli5,
        PromptFileAsset::ExtractClaim,
        PromptFileAsset::FindGaps,
        PromptFileAsset::Summarize,
        PromptFileAsset::Teach,
        PromptFileAsset::Translate,
    ]
}
fn all_known_quantizations() -> [Quantization; 12] {
    [
        Quantization::Q2k,
        Quantization::Q3kS,
        Quantization::Q3kM,
        Quantization::Q3kL,
        Quantization::Q4kM,
        Quantization::Q5kM,
        Quantization::Q6k,
        Quantization::Q8_0,
        Quantization::F16,
        Quantization::BF16,
        Quantization::F8,
        Quantization::IQ4_XS,
    ]
}

#[test]
fn test_prompt_file_asset_from_str_round_trips_for_known_assets() {
    all_known_assets().iter().for_each(|asset| {
        let name = asset.to_string();
        let mapped = PromptFileAsset::from(name.as_str());
        assert_eq!(mapped.to_string(), name);
    });
}
#[test]
fn test_prompt_file_asset_unknown_from_str_round_trips() {
    let value = "does-not-exist.prompt";
    let mapped = PromptFileAsset::from(value);
    assert_eq!(mapped.to_string(), value);
}
#[test]
fn test_quantization_from_str_round_trips_for_known_values() {
    all_known_quantizations().iter().for_each(|quantization| {
        let name = quantization.to_string();
        let mapped = Quantization::from(name.as_str());
        assert_eq!(mapped.to_string(), name);
    });
}
#[test]
fn test_quantization_unknown_from_str_round_trips() {
    let value = "Q3_K_X";
    let mapped = Quantization::from(value);
    assert_eq!(mapped.to_string(), value);
}
#[test]
fn test_quantization_lowercase_from_str_round_trips() {
    let value = "q2_k";
    let mapped = Quantization::from(value);
    assert_eq!(mapped.to_string(), "Q2_K");
}
#[test]
fn test_model_details_selector_prefers_id_and_falls_back_to_name() {
    let with_id = ModelDetails {
        id: Some(" acme/model ".to_string()),
        name: Some("display name".to_string()),
        ..Default::default()
    };
    let with_name = ModelDetails {
        name: Some(" acme/fallback ".to_string()),
        ..Default::default()
    };
    let empty = ModelDetails {
        id: Some(" ".to_string()),
        name: Some("ignored".to_string()),
        ..Default::default()
    };
    assert_eq!(with_id.selector().map(|selector| selector.to_string()), Ok("acme/model".to_string()));
    assert_eq!(with_name.selector().map(|selector| selector.to_string()), Ok("acme/fallback".to_string()));
    assert_eq!(empty.selector(), Err(ModelResolutionReason::MissingIdentifier));
}
#[test]
fn test_model_details_selector_explains_unresolved_models() {
    let not_open = ModelDetails {
        id: Some("acme/closed".to_string()),
        open_weights: Some(false),
        ..Default::default()
    };
    let no_open_weights = ModelDetails {
        open_weights: Some(true),
        ..Default::default()
    };
    let no_hugging_face_repository = ModelDetails {
        id: Some("acme/external".to_string()),
        weights: Some(Weights(vec![Weight::from("https://example.com/model")])),
        ..Default::default()
    };
    assert_eq!(not_open.selector(), Err(ModelResolutionReason::NotOpen));
    assert_eq!(no_open_weights.selector(), Err(ModelResolutionReason::NoOpenWeights));
    assert_eq!(no_hugging_face_repository.selector(), Err(ModelResolutionReason::NoHuggingFaceRepository));
    assert_eq!(ModelDetails::default().selector(), Err(ModelResolutionReason::MissingIdentifier));
}
#[test]
fn test_model_details_selector_uses_identifier_for_open_model_without_weight_sources() {
    let details = ModelDetails {
        id: Some("openai/gpt-oss-20b".to_string()),
        open_weights: Some(true),
        ..Default::default()
    };
    assert_eq!(
        details.selector().map(|selector| selector.to_string()),
        Ok("openai/gpt-oss-20b".to_string())
    );
}
#[test]
fn test_model_selector_normalizes_values_and_fallback_search_names() {
    let selector = ModelSelector::new(" nvidia/llama-3.1-nemotron-ultra-253b ").unwrap();
    assert_eq!(selector.as_str(), "nvidia/llama-3.1-nemotron-ultra-253b");
    assert_eq!(selector.fallback_search_name(), "llama-3_1-nemotron-ultra-253b-v1");
    let selector = ModelSelector::new("nvidia/llama-3.3-nemotron-super-49b-v1.5").unwrap();
    assert_eq!(selector.fallback_search_name(), "llama-3_3-nemotron-super-49b-v1_5");
    assert_eq!(ModelSelector::new(" "), None);
}
#[test]
fn test_model_selectors_reject_malformed_structured_input() {
    assert!(ModelSelectors::parse("models:\n  - missing required fields".to_string()).is_err());
    assert!(ModelSelectors::parse("[invalid".to_string()).is_err());
    assert_eq!(
        ModelSelectors::parse("https://example.com/model".to_string())
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .as_str(),
        "https://example.com/model"
    );
}
#[test]
fn test_model_selectors_parse_windows_path_as_plain_text() {
    let path = r"C:\models\tiny.gguf";
    let selectors = ModelSelectors::parse(path.to_string()).unwrap();
    assert_eq!(selectors.iter().next().unwrap().as_str(), path);
}
#[test]
fn test_quantization_compact_aliases_from_str_map_to_canonical_values() {
    [
        ("Q2K", Quantization::Q2k),
        ("Q3KS", Quantization::Q3kS),
        ("Q3KM", Quantization::Q3kM),
        ("Q3KL", Quantization::Q3kL),
        ("Q4KM", Quantization::Q4kM),
        ("Q5KM", Quantization::Q5kM),
        ("Q6K", Quantization::Q6k),
        ("Q80", Quantization::Q8_0),
        ("IQ4XS", Quantization::IQ4_XS),
    ]
    .iter()
    .for_each(|(alias, expected)| assert_eq!(&Quantization::from(*alias), expected));
}
#[test]
fn test_weights_infer_quantization_from_enum_variants() {
    [
        ("model-q4km", Quantization::Q4kM),
        ("model-bf16", Quantization::BF16),
        ("model-fp8", Quantization::F8),
        ("model-iq4_xs", Quantization::IQ4_XS),
    ]
    .into_iter()
    .for_each(|(model_id, expected)| {
        let weights = Weights::default().infer_quantization(model_id).expect("expected inferred quantization");
        assert_eq!(weights.0.first().and_then(|weight| weight.quantization.as_ref()), Some(&expected));
    });
}
#[test]
fn test_quantization_f8_aliases_deserialize_canonically() {
    ["F8", "f8", "FP8", "fp8", "Fp8", "fP8"].iter().for_each(|alias| {
        let decoded: Quantization = serde_json::from_str(&format!(r#""{alias}""#)).unwrap();
        assert!(matches!(decoded, Quantization::F8));
        assert_eq!(serde_json::to_string(&decoded).unwrap(), r#""F8""#);
    });
}
#[test]
fn test_quantization_from_gguf_filename_detects_known_and_custom_values() {
    assert_eq!(
        Quantization::from_gguf_filename("model-Q4_K_M.gguf").map(|value| value.to_string()),
        Some("Q4_K_M".to_string())
    );
    assert_eq!(
        Quantization::from_gguf_filename("model-IQ4_XS-00001-of-00002.gguf").map(|value| value.to_string()),
        Some("IQ4_XS".to_string())
    );
    assert_eq!(
        Quantization::from_gguf_filename("model-MXFP4-00001-of-00002.gguf").map(|value| value.to_string()),
        Some("MXFP4".to_string())
    );
    assert!(Quantization::from_gguf_filename("model-Q4_K_M.safetensors").is_none());
}
#[test]
fn test_weight_quantization_round_trips_in_json() {
    let weight = crate::schema::agent::Weight {
        label: "Q4_K_M".to_string(),
        url: "https://example.com/model.gguf".to_string(),
        is_open: Some(true),
        quantization: Some(Quantization::Q4kM),
        size: Some(42),
    };
    let json = serde_json::to_string(&weight).unwrap();
    assert!(json.contains(r#""quantization":"Q4_K_M""#));
    let decoded: crate::schema::agent::Weight = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.quantization.as_ref().map(ToString::to_string), Some("Q4_K_M".to_string()));
}
#[test]
fn test_weight_unknown_quantization_round_trips_in_json() {
    let json = r#"{"label":"MXFP4","url":"https://example.com/model.safetensors","quantization":"mxfp4"}"#;
    let decoded: crate::schema::agent::Weight = serde_json::from_str(json).unwrap();
    assert!(matches!(decoded.quantization.as_ref(), Some(Quantization::Other(value)) if value == "mxfp4"));
    assert_eq!(serde_json::to_string(&decoded).unwrap(), json);
}
#[test]
fn test_from_asset_returns_some_for_all_known_assets() {
    all_known_assets().iter().for_each(|asset| {
        let content = PromptTemplate::from_asset(&asset.to_string());
        assert!(content.is_some(), "Expected embedded asset to exist: {}", asset);
    });
}
#[test]
fn test_render_accepts_prompt_asset_str_and_string_inputs() {
    let config = PromptTemplateConfiguration::init()
        .text("source text".to_string())
        .max_items(3)
        .max_words(42)
        .build();
    let from_enum = PromptTemplate::render(PromptFileAsset::Summarize, &config).unwrap();
    let from_str = PromptTemplate::render("summarize", &config).unwrap();
    let from_string = PromptTemplate::render("summarize.prompt".to_string(), &config).unwrap();
    assert_eq!(from_enum, from_str);
    assert_eq!(from_str, from_string);
    assert!(from_enum.contains("under 42 words"));
    assert!(from_enum.contains("Limit key points to 3 bullets"));
    assert!(from_enum.contains("source text"));
    all_known_assets().iter().for_each(|asset| {
        assert!(PromptTemplate::render(asset.clone(), &config).is_ok(), "prompt should render: {asset}");
    });
}
#[test]
fn test_render_unknown_asset_returns_error() {
    let config = PromptTemplateConfiguration::init().build();
    let error = PromptTemplate::render("unknown.prompt", &config).expect_err("unknown prompt should fail");
    assert!(error.to_string().contains("unknown.prompt"));
}
#[test]
fn test_front_matter_to_markdown_includes_delimiters() {
    let frontmatter = FrontMatter::init()
        .name("summarize".to_string())
        .description("A prompt".to_string())
        .build();
    let markdown = frontmatter.to_markdown();
    assert!(markdown.starts_with("---\n"));
    assert!(markdown.contains("name: summarize"));
    assert!(markdown.ends_with("---"));
}
#[test]
fn test_model_details_to_markdown_includes_nested_capabilities() {
    let benchmark = Benchmark {
        name: "MMLU".to_string(),
        score: 0.91,
        metric: Some(Metric::Score),
        source: "https://example.com/benchmark".to_string(),
        date: None,
        dataset: Some("test-set".to_string()),
        harness: Some(Harness::OpenCode),
        variant: None,
        version: Some("1".to_string()),
    };
    let model = ModelDetails {
        benchmarks: Some(OneOrMany::One(benchmark)),
        limit: Some(LimitDetails {
            context: 128_000,
            input: None,
            output: Some(4_096),
        }),
        modalities: Some(Modalities {
            input: vec![Modality::Text, Modality::Image],
            output: vec![Modality::Text],
        }),
        cost: Some(CostDetails {
            input: Some(1.0),
            tiers: Some(vec![CostTier {
                input: 0.5,
                output: 1.0,
                cache_read: None,
                tier: TierInfo {
                    kind: "context".to_string(),
                    size: 200_000,
                },
            }]),
            ..CostDetails::default()
        }),
        weights: Some(Weights(vec![Weight {
            label: "Q4_K_M".to_string(),
            url: "https://example.com/model.gguf".to_string(),
            is_open: Some(true),
            quantization: Some(Quantization::Q4kM),
            size: Some(42),
        }])),
        ..ModelDetails::default()
    };
    let markdown = model.to_markdown().replace("\r\n", "\n");
    assert!(markdown.contains("- Benchmarks\n  - Benchmark\n    - Name: MMLU"), "{markdown}");
    assert!(markdown.contains("- Limits\n  - Context: 128000"));
    assert!(markdown.contains("- Modalities\n  - Input: text, image"));
    assert!(markdown.contains("- Cost\n  - Input: 1"));
    assert!(markdown.contains("- Tiers\n    - Tier\n      - Input: 0.5"));
    assert!(markdown.contains("- Weights\n  - Weight\n    - Label: Q4_K_M"));
}
#[test]
fn test_snapshot_prompt_asset_file_names() {
    let names: Vec<String> = all_known_assets().iter().map(ToString::to_string).collect();
    insta::assert_yaml_snapshot!("prompt_asset_file_names", names);
}
#[test]
fn test_snapshot_prompt_asset_template_headers() {
    let headers: Vec<String> = all_known_assets()
        .iter()
        .filter_map(|asset| PromptTemplate::from_asset(&asset.to_string()).map(|content| content.lines().take(6).collect::<Vec<&str>>().join("\n")))
        .collect();
    insta::assert_yaml_snapshot!("prompt_asset_template_headers", headers);
}
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}
#[test]
fn test_opencode_config_from_jsonc_file() {
    let path = fixtures_dir().join("config").join("opencode.jsonc");
    let config = opencode::Config::from_path(&path).unwrap();
    assert_eq!(config.model.as_deref(), Some("anthropic/claude-sonnet-4-5"));
    let agents = config.agent.unwrap();
    let build = agents.get("build").unwrap();
    assert_eq!(build.steps, Some(50));
}
#[test]
fn test_opencode_config_from_jsonc_string() {
    let content = r#"{
        // MCP server config
        "mcp": {
            "local-tool": {
                "type": "local",
                "command": ["npx", "my-tool"],
                "enabled": true
            }
        }
    }"#;
    let config = opencode::Config::parse_jsonc(content).unwrap();
    let mcp = config.mcp.unwrap();
    assert!(mcp.contains_key("local-tool"));
}
#[test]
fn test_opencode_config_from_jsonc_with_trailing_commas() {
    let content = r#"{
        "model": "openai/gpt-4o",
        "logLevel": "DEBUG",
    }"#;
    let config = opencode::Config::parse_jsonc(content).unwrap();
    assert_eq!(config.model.as_deref(), Some("openai/gpt-4o"));
}
#[test]
fn test_opencode_config_from_jsonc_rejects_invalid() {
    let content = r#"{ model: invalid }"#;
    assert!(opencode::Config::parse_jsonc(content).is_err());
}
#[test]
fn test_opencode_config_read_json_round_trip() {
    let json = r#"{"model":"anthropic/claude-sonnet-4-5","logLevel":"INFO"}"#;
    let config: opencode::Config = serde_json::from_str(json).unwrap();
    let temp = env::temp_dir().join("acorn_test_opencode.json");
    config.write_json(&temp).unwrap();
    let loaded = opencode::Config::read_json(temp.clone()).unwrap();
    assert_eq!(loaded.model.as_deref(), Some("anthropic/claude-sonnet-4-5"));
    assert!(loaded.log_level.is_some());
    remove_file(temp).ok();
}
#[test]
fn test_opencode_config_write_dispatches_by_extension() {
    let json = r#"{"model":"test"}"#;
    let config: opencode::Config = serde_json::from_str(json).unwrap();
    let temp_json = env::temp_dir().join("acorn_test_opencode_dispatch.json");
    let temp_jsonc = env::temp_dir().join("acorn_test_opencode_dispatch.jsonc");
    config.write(&temp_json).unwrap();
    config.write(&temp_jsonc).unwrap();
    let loaded_json = opencode::Config::read(&temp_json).unwrap();
    let loaded_jsonc = opencode::Config::read(&temp_jsonc).unwrap();
    assert_eq!(loaded_json.model.as_deref(), Some("test"));
    assert_eq!(loaded_jsonc.model.as_deref(), Some("test"));
    remove_file(temp_json).ok();
    remove_file(temp_jsonc).ok();
}
#[test]
fn test_opencode_config_read_jsonc_file() {
    let path = fixtures_dir().join("config").join("opencode.jsonc");
    let config = opencode::Config::read(path).unwrap();
    assert_eq!(config.model.as_deref(), Some("anthropic/claude-sonnet-4-5"));
}
