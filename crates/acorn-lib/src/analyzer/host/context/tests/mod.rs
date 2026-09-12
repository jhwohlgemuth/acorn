use super::*;

#[test]
fn test_analysis_context_builder_keeps_quiet_policies_separate() {
    let context = AnalysisContext::init()
        .execution(
            ExecutionOptions::init()
                .common(CommonRuntime {
                    quiet: true,
                    ..Default::default()
                })
                .build(),
        )
        .report(ReportOptions::default())
        .build();
    assert!(context.execution.common.quiet);
    assert!(!context.report.common.quiet);
}

#[test]
fn test_analysis_context_default_partitions_options() {
    let context = AnalysisContext::default();
    assert_eq!(context.analysis.readability_metric, ReadabilityType::default());
    assert_eq!(context.analysis.readability_threshold, None);
    assert!(context.analysis.skip.is_empty());
    assert_eq!(context.analysis.standard, Standard::ResearchActivityData);
    assert!(!context.execution.extension.disable_website_checks);
    assert!(!context.execution.extension.ignore_sync_failure);
    assert!(!context.execution.common.offline);
    assert!(!context.execution.common.quiet);
    assert!(!context.execution.extension.skip_sync);
    assert!(!context.execution.extension.skip_verify_checksum);
    assert_eq!(context.execution.extension.sync_retry_interval, SignedDuration::from_secs(1));
    assert!(context.execution.extension.vale_config.is_none());
    assert!(!context.report.extension.all);
    assert!(!context.report.extension.exit_on_first_error);
    assert!(!context.report.extension.filter_by_verbosity);
    assert_eq!(context.report.extension.format, OutputFormat::Json);
    assert!(!context.report.extension.no_fail);
    assert!(!context.report.common.quiet);
    assert!(!context.report.extension.terse);
    assert_eq!(context.report.extension.verbosity, None);
}

#[test]
fn test_execution_options_without_sync_preserves_every_other_field() {
    let options = ExecutionOptions::init()
        .common(CommonRuntime {
            offline: true,
            quiet: true,
            ..Default::default()
        })
        .extension(
            ExecutionExtension::init()
                .disable_website_checks(true)
                .ignore_sync_failure(true)
                .skip_sync(false)
                .skip_verify_checksum(true)
                .sync_retry_interval(SignedDuration::from_secs(9))
                .build(),
        )
        .build();
    let options = ExecutionOptions {
        extension: options.extension.without_sync(),
        ..options
    };
    assert!(options.common.offline);
    assert!(options.common.quiet);
    assert!(options.extension.disable_website_checks);
    assert!(options.extension.ignore_sync_failure);
    assert!(options.extension.skip_sync);
    assert!(options.extension.skip_verify_checksum);
    assert_eq!(options.extension.sync_retry_interval, SignedDuration::from_secs(9));
    assert!(options.extension.vale_config.is_none());
}
