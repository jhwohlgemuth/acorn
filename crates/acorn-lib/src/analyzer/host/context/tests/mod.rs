use super::*;

#[test]
fn test_analysis_context_builder_keeps_quiet_policies_separate() {
    let context = AnalysisContext::init()
        .execution(ExecutionOptions::init().quiet(true).build())
        .report(ReportOptions::init().quiet(false).build())
        .build();
    assert!(context.execution.quiet);
    assert!(!context.report.quiet);
}

#[test]
fn test_analysis_context_default_partitions_options() {
    let context = AnalysisContext::default();
    assert_eq!(context.analysis.readability_metric, ReadabilityType::default());
    assert_eq!(context.analysis.readability_threshold, None);
    assert!(context.analysis.skip.is_empty());
    assert_eq!(context.analysis.standard, Standard::ResearchActivityData);
    assert!(!context.execution.disable_website_checks);
    assert!(!context.execution.ignore_sync_failure);
    assert!(!context.execution.offline);
    assert!(!context.execution.quiet);
    assert!(!context.execution.skip_sync);
    assert!(!context.execution.skip_verify_checksum);
    assert_eq!(context.execution.sync_retry_interval, SignedDuration::from_secs(1));
    assert!(context.execution.vale_config.is_none());
    assert!(!context.report.all);
    assert!(!context.report.exit_on_first_error);
    assert!(!context.report.filter_by_verbosity);
    assert_eq!(context.report.format, OutputFormat::Json);
    assert!(!context.report.no_fail);
    assert!(!context.report.quiet);
    assert!(!context.report.terse);
    assert_eq!(context.report.verbosity, None);
}

#[test]
fn test_execution_options_without_sync_preserves_every_other_field() {
    let options = ExecutionOptions::init()
        .disable_website_checks(true)
        .ignore_sync_failure(true)
        .offline(true)
        .quiet(true)
        .skip_sync(false)
        .skip_verify_checksum(true)
        .sync_retry_interval(SignedDuration::from_secs(9))
        .build()
        .without_sync();
    assert!(options.disable_website_checks);
    assert!(options.ignore_sync_failure);
    assert!(options.offline);
    assert!(options.quiet);
    assert!(options.skip_sync);
    assert!(options.skip_verify_checksum);
    assert_eq!(options.sync_retry_interval, SignedDuration::from_secs(9));
    assert!(options.vale_config.is_none());
}
