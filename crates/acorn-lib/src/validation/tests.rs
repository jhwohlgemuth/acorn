use super::{Validate, ValidationError, ValidationReport};
use crate::prelude::alloc::{Box, String, ToString, Vec};
use crate::schema::OneOrMany;
use crate::util::SemanticVersion;
use alloc::collections::BTreeMap;
use core::future;
use serde::Serialize;

fn not_bad(value: &str) -> Result<(), ValidationError> {
    match value {
        | "bad" => Err(ValidationError::new("custom_code").with_message("custom message".into())),
        | _ => Ok(()),
    }
}
async fn not_bad_async(value: &str) -> Result<(), ValidationError> {
    match value {
        | "bad" => Err(ValidationError::new("async_custom").with_message("async custom message".into())),
        | _ => Ok(()),
    }
}
fn schema_sync(_: &MixedAsyncModel) -> Result<(), ValidationError> {
    Err(ValidationError::new("schema_sync"))
}
async fn schema_async(_: &MixedAsyncModel) -> Result<(), ValidationError> {
    Err(ValidationError::new("schema_async"))
}
async fn skipped_schema_async(_: &SkippedAsyncSchema) -> Result<(), ValidationError> {
    Err(ValidationError::new("should_be_skipped"))
}
#[derive(Serialize, Validate)]
#[validate(
    async(function = "schema_async", skip_on_field_errors = false),
    schema(function = "schema_sync", skip_on_field_errors = false)
)]
struct MixedAsyncModel {
    #[validate(
        custom(function = "not_bad"),
        async(function = "not_bad_async", code = "overridden", message = "overridden message"),
        length(min = 4)
    )]
    value: String,
    #[validate(async(function = "not_bad_async"))]
    optional: Option<String>,
}
#[derive(Serialize, Validate)]
#[validate(async(function = "skipped_schema_async"))]
struct SkippedAsyncSchema {
    #[validate(length(min = 4))]
    value: String,
}
#[derive(Clone, Serialize, Validate)]
struct AsyncChild {
    #[validate(async(function = "not_bad_async"))]
    value: String,
}
#[derive(Serialize, Validate)]
struct AsyncParent {
    #[validate(nested)]
    children: Vec<AsyncChild>,
    #[validate(nested)]
    array: [AsyncChild; 1],
    #[validate(nested)]
    boxed: Box<AsyncChild>,
}
#[derive(Serialize, Validate)]
struct AsyncEachModel<'a> {
    #[validate(each(async(function = "not_bad_async")))]
    values: Vec<String>,
    #[validate(each(async(function = "not_bad_async")))]
    array: [String; 2],
    #[validate(each(async(function = "not_bad_async")))]
    slice: &'a [String],
    #[validate(each(async(function = "not_bad_async")))]
    one_or_many: OneOrMany<String>,
    #[validate(each(async(function = "not_bad_async")))]
    map: BTreeMap<String, String>,
    #[validate(each(async(function = "not_bad_async")))]
    optional: Option<Vec<String>>,
}
struct ManualValidation;
impl Validate for ManualValidation {
    fn validate(&self) -> Result<(), ValidationReport> {
        Err(ValidationReport::from_error("", ValidationError::new("manual_sync")))
    }
}
struct ManualAsyncValidation;
impl Validate for ManualAsyncValidation {
    fn validate(&self) -> Result<(), ValidationReport> {
        Ok(())
    }
    fn validate_async(&self) -> impl future::Future<Output = Result<(), ValidationReport>> + '_ {
        future::ready(Err(ValidationReport::from_error("", ValidationError::new("manual_async"))))
    }
}
#[derive(Serialize, Validate)]
struct EachModel {
    #[validate(length(min = 1), each(custom(function = "not_bad")))]
    values: Vec<String>,
    #[validate(each(url))]
    optional: Option<Vec<String>>,
    #[validate(each(email))]
    array: [String; 2],
    #[validate(each(custom(function = "not_bad")))]
    one_or_many: OneOrMany<String>,
}
#[derive(Serialize, Validate)]
struct Child {
    #[validate(email)]
    email: String,
}
#[derive(Serialize, Validate)]
struct Parent {
    #[validate(nested)]
    children: Vec<Child>,
}
#[derive(Serialize, Validate)]
struct DoiModel {
    #[validate(doi)]
    doi: String,
}
#[derive(Serialize, Validate)]
struct MultipleFailures {
    #[validate(length(min = 10), email)]
    value: String,
}
#[derive(Serialize, Validate)]
struct FirstClassRules {
    #[validate(date)]
    date: String,
    #[validate(phone)]
    phone: String,
    #[validate(commit)]
    commit: String,
    #[validate(latitude)]
    latitude: f64,
    #[validate(longitude)]
    longitude: f64,
    #[validate(state)]
    state: String,
    #[validate(kebabcase)]
    kebab: String,
    #[validate(camelCase)]
    camel: String,
    #[validate(snake_case)]
    snake: String,
    #[validate(year)]
    year: i32,
}
#[test]
fn each_reports_every_index_in_rule_order() {
    let model = EachModel {
        values: vec!["bad".to_string(), "ok".to_string(), "bad".to_string()],
        optional: Some(vec!["invalid".to_string(), "https://example.org".to_string(), "also invalid".to_string()]),
        array: ["valid@example.org".to_string(), "invalid".to_string()],
        one_or_many: OneOrMany::Many(vec!["bad".to_string(), "bad".to_string()]),
    };
    let report = model.validate().expect_err("each rules should fail");
    let paths = report.iter().map(|issue| issue.path.as_str()).collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            "values[0]",
            "values[2]",
            "optional[0]",
            "optional[2]",
            "array[1]",
            "one_or_many[0]",
            "one_or_many[1]"
        ]
    );
    let first = report.iter().next().expect("expected first issue");
    assert_eq!(first.error.code, "custom_code");
    assert_eq!(first.error.message.as_deref(), Some("custom message"));
    assert_eq!(first.error.params.get("value").and_then(|value| value.as_str()), Some("bad"));
}
#[test]
fn empty_and_absent_each_collections_pass() {
    let model = EachModel {
        values: vec!["ok".to_string()],
        optional: None,
        array: ["first@example.org".to_string(), "second@example.org".to_string()],
        one_or_many: OneOrMany::Many(Vec::new()),
    };
    assert!(model.validate().is_ok());
}
#[tokio::test]
async fn async_validation_preserves_mixed_declaration_order_and_overrides() {
    let model = MixedAsyncModel {
        value: "bad".to_string(),
        optional: None,
    };
    let sync_codes = model
        .validate()
        .expect_err("synchronous rules should fail")
        .into_issues()
        .into_iter()
        .map(|issue| issue.error.code)
        .collect::<Vec<_>>();
    assert_eq!(sync_codes, ["custom_code", "length", "schema_sync"]);
    let report = model.validate_async().await.expect_err("the complete validation pass should fail");
    let codes = report.iter().map(|issue| issue.error.code.as_str()).collect::<Vec<_>>();
    assert_eq!(codes, ["custom_code", "overridden", "length", "schema_async", "schema_sync"]);
    let async_issue = report.iter().nth(1).expect("expected async issue");
    assert_eq!(async_issue.error.message.as_deref(), Some("overridden message"));
    assert_eq!(async_issue.error.params.get("value").and_then(|value| value.as_str()), Some("bad"));
}
#[tokio::test]
async fn async_schema_validation_skips_after_field_errors_by_default() {
    let report = SkippedAsyncSchema { value: "bad".to_string() }
        .validate_async()
        .await
        .expect_err("field validation should fail");
    let codes = report.iter().map(|issue| issue.error.code.as_str()).collect::<Vec<_>>();
    assert_eq!(codes, ["length"]);
}
#[tokio::test]
async fn async_each_supports_all_collection_shapes_and_every_index() {
    let slice = ["bad".to_string(), "ok".to_string()];
    let model = AsyncEachModel {
        values: vec!["bad".to_string(), "ok".to_string(), "bad".to_string()],
        array: ["ok".to_string(), "bad".to_string()],
        slice: &slice,
        one_or_many: OneOrMany::Many(vec!["bad".to_string(), "bad".to_string()]),
        map: BTreeMap::from([("first".to_string(), "bad".to_string()), ("second".to_string(), "ok".to_string())]),
        optional: Some(vec!["ok".to_string(), "bad".to_string()]),
    };
    assert!(model.validate().is_ok(), "sync validation must skip async rules");
    let report = model.validate_async().await.expect_err("async each rules should fail");
    let paths = report.iter().map(|issue| issue.path.as_str()).collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            "values[0]",
            "values[2]",
            "array[1]",
            "slice[0]",
            "one_or_many[0]",
            "one_or_many[1]",
            "map[0]",
            "optional[1]"
        ]
    );
}
#[tokio::test]
async fn async_validation_propagates_through_nested_and_blanket_implementations() {
    let child = || AsyncChild { value: "bad".to_string() };
    let parent = AsyncParent {
        children: vec![child()],
        array: [child()],
        boxed: Box::new(child()),
    };
    let paths = parent
        .validate_async()
        .await
        .expect_err("nested async rules should fail")
        .into_issues()
        .into_iter()
        .map(|issue| issue.path)
        .collect::<Vec<_>>();
    assert_eq!(paths, ["children[0].value", "array[0].value", "boxed.value"]);
    let nested = vec![vec![child()]];
    let paths = nested
        .validate_async()
        .await
        .expect_err("blanket async validation should recurse")
        .into_issues()
        .into_iter()
        .map(|issue| issue.path)
        .collect::<Vec<_>>();
    assert_eq!(paths, ["[0][0].value"]);
}
#[tokio::test]
async fn manual_validate_implementations_use_default_and_custom_async_futures() {
    let default = ManualValidation
        .validate_async()
        .await
        .expect_err("default async validation should call validate");
    assert_eq!(default.iter().next().map(|issue| issue.error.code.as_str()), Some("manual_sync"));
    let overridden = ManualAsyncValidation
        .validate_async()
        .await
        .expect_err("manual async override should run");
    assert_eq!(overridden.iter().next().map(|issue| issue.error.code.as_str()), Some("manual_async"));
}
#[test]
fn nested_collection_paths_are_indexed() {
    let parent = Parent {
        children: vec![
            Child {
                email: "valid@example.org".to_string(),
            },
            Child {
                email: "invalid".to_string(),
            },
            Child {
                email: "also-invalid".to_string(),
            },
        ],
    };
    let report = parent.validate().expect_err("nested children should fail");
    let paths = report.iter().map(|issue| issue.path.as_str()).collect::<Vec<_>>();
    assert_eq!(paths, ["children[1].email", "children[2].email"]);
}
#[test]
fn report_display_and_serialization_are_deterministic() {
    let parent = Parent {
        children: vec![Child {
            email: "invalid".to_string(),
        }],
    };
    let report = parent.validate().expect_err("nested child should fail");
    assert_eq!(report.to_string(), "children[0].email: email");
    assert_eq!(
        serde_json::to_string(&report).expect("report should serialize"),
        r#"[{"path":"children[0].email","error":{"code":"email","message":null,"params":{"value":"invalid"}}}]"#
    );
}
#[test]
fn multiple_failures_on_one_field_follow_rule_order() {
    let report = MultipleFailures { value: "bad".to_string() }
        .validate()
        .expect_err("both field rules should fail");
    let mut issues = report.iter();
    let first = issues.next().expect("expected the length issue");
    assert_eq!(first.path, "value");
    assert_eq!(first.error.code, "length");
    let second = issues.next().expect("expected the email issue");
    assert_eq!(second.path, "value");
    assert_eq!(second.error.code, "email");
    assert!(issues.next().is_none());
}
#[test]
fn first_class_scalar_rules_delegate_to_schema_validation() {
    let valid = FirstClassRules {
        date: "2025-01-01".to_string(),
        phone: "+1 865-576-7658".to_string(),
        commit: "0123456789abcdef".to_string(),
        latitude: 35.9,
        longitude: -84.3,
        state: "TN".to_string(),
        kebab: "acorn-validation".to_string(),
        camel: "acornValidation".to_string(),
        snake: "acorn_validation".to_string(),
        year: 2025,
    };
    assert!(valid.validate().is_ok());
    let invalid = FirstClassRules {
        date: "1949-01-01".to_string(),
        phone: "not a phone".to_string(),
        commit: "xyz".to_string(),
        latitude: 91.0,
        longitude: -181.0,
        state: "Tennessee".to_string(),
        kebab: "not_kebab".to_string(),
        camel: "not-camel".to_string(),
        snake: "not-snake".to_string(),
        year: 3000,
    };
    let report = invalid.validate().expect_err("all first-class rules should fail");
    let codes = report.iter().map(|issue| issue.error.code.as_str()).collect::<Vec<_>>();
    assert_eq!(
        codes,
        [
            "date",
            "phone",
            "commit",
            "latitude",
            "longitude",
            "state",
            "kebabcase",
            "camelcase",
            "snake_case",
            "year"
        ]
    );
}
#[test]
fn exact_versions_accept_only_canonical_abbreviated_forms() {
    ["0", "1", "1.2", "1.2.3", "4294967295.4294967295.4294967295"]
        .into_iter()
        .for_each(|value| assert!(SemanticVersion::parse_exact(value).is_ok(), "expected valid version {value}"));
    [
        "",
        ".",
        "1.",
        ".1",
        "1..2",
        "1.2.3.4",
        "+1",
        "-1",
        "01",
        "1.02",
        "v1.2.3",
        "version 1.2.3",
        "1.2.3-alpha",
        "1.2.3+build",
        "4294967296",
        " 1.2.3 ",
    ]
    .into_iter()
    .for_each(|value| assert!(SemanticVersion::parse_exact(value).is_err(), "expected invalid version {value}"));
}
#[test]
fn lossy_version_extraction_remains_available() {
    assert_eq!(SemanticVersion::from("tool v1.2.3 (build 9)"), SemanticVersion::from((1, 2, 3)));
}
#[test]
fn pid_rules_delegate_to_canonical_validation() {
    assert!(DoiModel {
        doi: "https://doi.org/10.1000/182".to_string()
    }
    .validate()
    .is_ok());
    let report = DoiModel {
        doi: "10/not-a-doi".to_string(),
    }
    .validate()
    .expect_err("malformed DOI should fail");
    assert_eq!(report.iter().next().map(|issue| issue.path.as_str()), Some("doi"));
}
