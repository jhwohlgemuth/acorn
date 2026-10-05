use super::*;
use acorn::io::database::{schema::Table, Database, Operations};
use acorn_schema::agent::Weight;

fn weight(path: &str, quantization: Quantization, size: Option<u64>) -> Weight {
    Weight {
        label: path.to_string(),
        url: format!("https://huggingface.co/acme/model-GGUF/resolve/v1/{path}"),
        is_open: None,
        quantization: Some(quantization),
        size,
    }
}
#[tokio::test]
async fn test_no_local_database_skips_plan_lookup_and_constraint_lookup() {
    let invalid_database_path = PathBuf::from(".");
    let plans = resolve_plans(
        &ModelSelectors::from(vec!["acme/base".to_string()]),
        &[],
        &Default::default(),
        None,
        &Some(invalid_database_path.clone()),
        true,
        true,
    )
    .await
    .unwrap();
    let selected = ModelDownloadPlans(plans).constrain(
        &[Quantization::Q4kM],
        &[],
        &[],
        true,
        &Some("1GB".parse().unwrap()),
        &Some(invalid_database_path),
    );
    assert_eq!(selected.0.len(), 1);
    assert_eq!(selected.0.first().map(|plan| plan.selector.identifier()).as_deref(), Some("acme/base"));
}
#[tokio::test]
async fn test_optional_harbor_enrichment_falls_back_to_oci() {
    let profile = RegistryProfile {
        kind: RegistryKind::Harbor,
        endpoint: "http://127.0.0.1:1".to_string(),
        plain_http: true,
        ..Default::default()
    };
    let plan = ModelDownloadPlan::init()
        .selector(Source::from("oci://127.0.0.1:5000/ai/qwen:v1"))
        .auth(AuthenticationRequirement::Optional)
        .maybe_registry_profile(Some(profile.clone()))
        .build();
    let reference = OciReference::parse("oci://127.0.0.1:5000/ai/qwen:v1").unwrap();
    let resolution = OciResolution {
        registry: "127.0.0.1:5000".to_string(),
        repository: "ai/qwen".to_string(),
        requested_reference: "v1".to_string(),
        resolved_digest: format!("sha256:{}", "a".repeat(64)),
        artifact_type: None,
        package_format: oci::ModelArtifactKind::Annotated,
        inventory_status: oci::ModelInventoryStatus::Complete,
        total_size: 0,
        transfer_size: 0,
        installed_size: Some(0),
        layers: Vec::new(),
        ignored_layers: Vec::new(),
        files: Vec::new(),
        filter: Vec::new(),
        ignore: Vec::new(),
        quantization: Vec::new(),
    };
    assert!(plan.harbor_metadata(&reference, &resolution).await.unwrap().is_none());
    let required = ModelDownloadPlan {
        auth: AuthenticationRequirement::Required,
        ..plan
    };
    assert!(required.harbor_metadata(&reference, &resolution).await.is_err());
}
#[test]
fn test_plan_filter_defers_oci_patterns_to_manifest_files() {
    let plans = ModelDownloadPlans(vec![
        ModelDownloadPlan::new(Source::from("oci://registry.example.org/ai/qwen:v1")),
        ModelDownloadPlan::new(Source::from("acme/qwen")),
    ]);
    let filtered = plans.filter(&["\\.gguf$".to_string()], &["tokenizer".to_string()]).unwrap();
    assert_eq!(filtered.0.len(), 1);
    assert_eq!(
        filtered.0.first().map(|plan| plan.selector.identifier()).as_deref(),
        Some("oci://registry.example.org/ai/qwen:v1")
    );
}
#[tokio::test]
async fn test_resolve_plans_preserves_database_limits() {
    let directory = tempfile::tempdir().unwrap();
    let database_path = directory.path().join("acorn.db");
    let database = Database::<Table>::from_path(Some(database_path.clone()));
    database.migrate().unwrap();
    database
        .insert(
            ModelRow::init()
                .model_id("acme/model")
                .weights(r#"[{"label":"Hugging Face","url":"https://huggingface.co/acme/model"}]"#)
                .limit_context(131_072)
                .limit_input(65_536)
                .limit_output(8_192)
                .build(),
        )
        .unwrap();
    let plans = resolve_plans(
        &ModelSelectors::from(vec!["acme/model".to_string()]),
        &[],
        &Default::default(),
        None,
        &Some(database_path),
        false,
        true,
    )
    .await
    .unwrap();
    let limit = plans
        .first()
        .and_then(|plan| plan.details.as_ref())
        .and_then(|details| details.limit.as_ref())
        .expect("persisted model limits");
    assert_eq!(limit.context, 131_072);
    assert_eq!(limit.input, Some(65_536));
    assert_eq!(limit.output, Some(8_192));
}
#[test]
fn test_select_weight_group_memory_only_defaults_to_q4_k_m() {
    let groups = Weights(vec![
        weight("model-Q4_K_M.gguf", Quantization::Q4kM, Some(10)),
        weight("model-Q5_K_M.gguf", Quantization::Q5kM, Some(5)),
    ])
    .groups();
    let memory = "1GB".parse::<Memory>().unwrap();
    let selected = groups.select(&[], Some(&memory)).unwrap();
    assert_eq!(selected.quantization, Quantization::Q4kM);
}
#[test]
fn test_select_weight_group_rejects_aggregate_oversize_and_allows_unknown_size() {
    let oversized = Weights(vec![
        weight("a-Q4_K_M.gguf", Quantization::Q4kM, Some(800_000_000)),
        weight("b-Q4_K_M.gguf", Quantization::Q4kM, Some(800_000_000)),
    ])
    .groups();
    let memory = "1GB".parse::<Memory>().unwrap();
    assert!(oversized.select(&[Quantization::Q4kM], Some(&memory)).is_none());
    let unknown = Weights(vec![weight("model-Q4_K_M.gguf", Quantization::Q4kM, None)]).groups();
    assert!(unknown.select(&[Quantization::Q4kM], Some(&memory)).is_some());
}
#[test]
fn test_select_weight_group_uses_ordered_exact_allowlist() {
    let groups = Weights(vec![
        weight("model-Q4_K_M.gguf", Quantization::Q4kM, Some(10)),
        weight("model-Q5_K_M.gguf", Quantization::Q5kM, Some(20)),
    ])
    .groups();
    let selected = groups.select(&[Quantization::Q5kM, Quantization::Q4kM], None).unwrap();
    assert_eq!(selected.quantization, Quantization::Q5kM);
}
#[test]
fn test_selected_group_rewrites_repository_revision_shards_and_honors_filters() {
    let groups = Weights(vec![
        weight("model-Q4_K_M-00001-of-00002.gguf", Quantization::Q4kM, Some(10)),
        weight("model-Q4_K_M-00002-of-00002.gguf", Quantization::Q4kM, Some(20)),
        weight("model-Q5_K_M.gguf", Quantization::Q5kM, Some(40)),
    ])
    .groups();
    let plan = ModelDownloadPlan::new(Source::from("acme/base"));
    let selected = plan
        .clone()
        .constrain_with_groups(&groups, &[Quantization::Q4kM], Some(&"1GB".parse().unwrap()), &[], &[])
        .unwrap();
    assert_eq!(selected.selector.identifier(), "acme/model-GGUF");
    assert_eq!(selected.revision, "v1");
    assert_eq!(selected.required_paths.len(), 2);
    assert!(plan
        .constrain_with_groups(&groups, &[Quantization::Q4kM], None, &[], &["00002".to_string()])
        .is_none());
}
#[test]
fn test_weight_groups_aggregate_split_shards() {
    let groups = Weights(vec![
        weight("model-Q4_K_M-00001-of-00002.gguf", Quantization::Q4kM, Some(10)),
        weight("model-Q4_K_M-00002-of-00002.gguf", Quantization::Q4kM, Some(20)),
    ])
    .groups();
    assert_eq!(groups.0.len(), 1);
    assert_eq!(groups.0.first().map(|group| group.paths.len()), Some(2));
    assert_eq!(groups.0.first().and_then(|group| group.size), Some(30));
}
