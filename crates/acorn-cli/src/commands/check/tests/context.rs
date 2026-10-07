use crate::commands::check::{build_analysis_context, should_sync_analyzer, sync_analyzer_before_check};
use acorn::analyzer::host::context::{AnalysisContext, AnalysisOptions, OutputFormat};
use acorn_analysis::{readability::ReadabilityType, CheckCategory};
use acorn_schema::standard::Standard;
use jiff::SignedDuration;
use std::path::PathBuf;

#[test]
fn test_check_context_builder_assigns_every_option_to_its_owner() {
    let context = build_analysis_context(
        &[CheckCategory::Schema, CheckCategory::Link],
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        Standard::Datacite,
        ReadabilityType::ARI,
        false,
        true,
        true,
        SignedDuration::from_secs(7),
        Some(PathBuf::from("/opt/vale")),
    );
    assert_eq!(context.analysis.readability_metric, ReadabilityType::ARI);
    assert_eq!(context.analysis.skip, ["schema", "link"]);
    assert_eq!(context.analysis.standard, Standard::Datacite);
    assert!(context.execution.extension.disable_website_checks);
    assert!(context.execution.extension.ignore_sync_failure);
    assert!(context.execution.common.offline);
    assert!(context.execution.common.quiet);
    assert!(!context.execution.extension.skip_sync);
    assert!(context.execution.extension.skip_verify_checksum);
    assert_eq!(context.execution.extension.sync_retry_interval, SignedDuration::from_secs(7));
    assert_eq!(
        context.execution.extension.vale_config.as_ref().map(|config| config.path.clone()),
        Some(PathBuf::from("/opt/vale/.vale.ini"))
    );
    assert!(context.report.extension.all);
    assert!(context.report.extension.exit_on_first_error);
    assert_eq!(context.report.extension.format, OutputFormat::Text);
    assert!(context.report.extension.no_fail);
    assert!(context.report.common.quiet);
    assert!(context.report.extension.terse);
}
#[test]
fn test_check_context_builder_keeps_non_raw_output_visible() {
    let context = build_analysis_context(
        &[],
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        Standard::ResearchActivityData,
        ReadabilityType::FKGL,
        false,
        false,
        false,
        SignedDuration::from_secs(1),
        None,
    );
    assert!(!context.execution.common.quiet);
    assert!(!context.report.common.quiet);
}
#[test]
fn test_should_sync_analyzer_reflects_prose_selection() {
    assert!(should_sync_analyzer(&AnalysisContext::default()));
    let skipped = AnalysisContext::init()
        .analysis(AnalysisOptions::init().skip(vec!["prose".to_string()]).build())
        .build();
    assert!(!should_sync_analyzer(&skipped));
    let acre = AnalysisContext::init()
        .analysis(
            AnalysisOptions::init()
                .standard(Standard::ApplicationConfigurationResearchEnablement)
                .build(),
        )
        .build();
    assert!(!should_sync_analyzer(&acre));
}
#[tokio::test]
async fn test_sync_analyzer_skips_resolution_when_prose_is_omitted() {
    let context = AnalysisContext::init()
        .analysis(
            AnalysisOptions::init()
                .skip(vec!["prose".to_string()])
                .standard(Standard::Datacite)
                .build(),
        )
        .build();
    let resolved = sync_analyzer_before_check(&context)
        .await
        .expect("skipped prose should avoid analyzer resolution");
    assert_eq!(resolved.analysis.standard, Standard::Datacite);
    assert!(resolved.execution.extension.skip_sync);
}
