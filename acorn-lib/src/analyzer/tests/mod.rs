#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::analyzer::check::{highlighted, highlighted_array, highlighted_from_params, nearest_space_boundary, truncate, Standard};
use crate::analyzer::error::ErrorCode;
use crate::analyzer::error::ErrorKind;
use crate::analyzer::vale::{preprocess_vale_output, ValeConfig, ValeOutput, ValeOutputItem, ValeOutputItemSeverity};
use crate::analyzer::{
    checks_to_dataframe, collect_validation_checks, summary, Analysis, Check, CheckCategory, CheckSeverity, IntoChecks, StaticAnalyzerConfig,
};
use crate::io::License;
use crate::prelude::{create_dir_all, remove_file, write, HashMap, PathBuf};
use crate::schema::standard::cff::Cff;
use crate::schema::standard::crosswalk::ConversionWarning;
use crate::test::utils::unique_path;
use crate::{check, check_ok};
use ariadne::Color;
use ariadne::ReportKind;
use ariadne::Source;
use serde_json::Value;
use validator::{Validate, ValidationError, ValidationErrors, ValidationErrorsKind};

#[derive(Validate)]
struct FieldModel {
    #[validate(length(min = 3, max = 3))]
    name: String,
}
#[derive(Validate)]
struct ChildModel {
    #[validate(email)]
    email: String,
}
#[derive(Validate)]
struct ParentModel {
    #[validate(nested)]
    child: ChildModel,
}
#[derive(Validate)]
struct ParentListModel {
    #[validate(nested)]
    children: Vec<ChildModel>,
}
#[derive(Validate)]
struct RepeatedChildModel {
    #[validate(email)]
    email: String,
}
#[derive(Validate)]
struct RepeatedNameModel {
    #[validate(nested)]
    child: RepeatedChildModel,
}
#[derive(Validate)]
struct RepeatedNameParentModel {
    #[validate(nested)]
    child: RepeatedNameModel,
}
#[derive(Validate)]
struct ArrayParamModel {
    #[validate(length(min = 3))]
    tags: Vec<String>,
}

#[test]
fn test_standard_detection_uses_dcat_jsonld_type() {
    let value: Value = serde_json::json!({
        "@type": "dcat:Dataset",
        "metadata": {}
    });
    let object = value.as_object().unwrap();
    assert_eq!(Standard::from(object), Standard::Dcat);
}
#[test]
fn test_cff_validation_issues_invalid_doi() {
    let cff = Cff {
        title: "Invalid DOI".to_string(),
        message: "Message".to_string(),
        doi: Some("not-a-doi".to_string()),
        ..Cff::default()
    };
    let issues = collect_validation_checks(&cff);
    assert!(!issues.is_empty());
    assert_eq!(issues[0].category, CheckCategory::Schema);
}
#[tokio::test]
async fn test_cff_check_schema_returns_errors_for_invalid_file() {
    let path = unique_path("cff-schema-invalid", "cff");
    if let Some(parent) = path.parent() {
        create_dir_all(parent).expect("failed to create test_artifacts directory");
    }
    let cff = Cff {
        title: "Invalid DOI".to_string(),
        message: "Message".to_string(),
        doi: Some("not-a-doi".to_string()),
        ..Cff::default()
    };
    let yaml = serde_norway::to_string(&cff).expect("failed to serialize CFF fixture");
    write(path.clone(), yaml).expect("failed to write CFF fixture");
    let checks = Cff::check_schema(core::slice::from_ref(&path), None).await;
    assert!(!checks.is_empty());
    assert!(checks.iter().any(|check| check.category == CheckCategory::Schema));
    let _cleanup = remove_file(path);
}
#[tokio::test]
async fn test_cff_check_quality_returns_ok_for_readable_file() {
    let path = unique_path("cff-quality-valid", "cff");
    if let Some(parent) = path.parent() {
        create_dir_all(parent).expect("failed to create test_artifacts directory");
    }
    let cff = Cff {
        title: "Valid CFF".to_string(),
        message: "Message".to_string(),
        ..Cff::default()
    };
    let yaml = serde_norway::to_string(&cff).expect("failed to serialize CFF fixture");
    write(path.clone(), yaml).expect("failed to write CFF fixture");
    let checks = Cff::check_quality(core::slice::from_ref(&path), None).await;
    assert_eq!(checks.len(), 1);
    assert!(checks[0].success);
    assert_eq!(checks[0].category, CheckCategory::Quality);
    let _cleanup = remove_file(path);
}
#[test]
fn test_checks_to_dataframe() {
    let check = check_ok!(CheckCategory::Prose);
    let checks = vec![check.clone(), check.clone(), check];
    let df = checks_to_dataframe(&checks);
    let reason = "Failed to convert checks to dataframe";
    assert_eq!(df.expect(reason).shape(), (checks.len(), 7));
}
#[test]
fn test_conversion_warnings_use_into_checks_trait() {
    let warnings = vec![ConversionWarning::no_equivalent("datacite", "dcat", "attributes.publisher")];
    let checks = warnings.to_checks(Some("record.json".to_string()));
    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].category, CheckCategory::Crosswalk);
    assert_eq!(checks[0].severity, CheckSeverity::Warning);
    assert_eq!(checks[0].issue_count(), 1);
    assert_eq!(checks[0].uri.as_deref(), Some("record.json"));
    assert_eq!(checks[0].locator.as_deref(), Some("attributes.publisher"));
}
#[test]
fn test_summary_includes_crosswalk_warnings() {
    let warnings = vec![
        ConversionWarning::no_equivalent("datacite", "dcat", "attributes.publisher"),
        ConversionWarning::no_equivalent("datacite", "dcat", "attributes.subjects"),
    ];
    let rows = summary(warnings.to_checks(None));
    assert!(rows.contains(&vec!["Crosswalk items found".to_string(), "2".to_string()]));
}
#[test]
fn test_highlighted_span() {
    let span = &[1, 5];
    let line_number = 2;
    let result = highlighted("line one\nline two\nline three", span, line_number);
    assert_eq!(result, 9..=13);
}
#[test]
fn test_highlighted_span_for_array() {
    let source = Source::from(String::from("{\n  \"key\": [\n    \"a\",\n    \"b\",\n    \"c\"\n  ]\n}"));
    let index = 1;
    let value = serde_json::json!(["a", "b", "c"]);
    let mut params: HashMap<String, Value> = HashMap::new();
    params.insert("index".to_string(), Value::Number(index.into()));
    params.insert("value".to_string(), value);
    let result = highlighted_array(&source, &params);
    assert!(result.is_some());
    let span = result.unwrap();
    assert!(*span.start() < source.text().len());
    assert!(*span.end() <= source.text().len());
}
#[test]
fn test_highlighted_span_from_params_length() {
    let source = Source::from(String::from("{\"key\": \"value\"}"));
    let code = ErrorCode::Length;
    let mut params: HashMap<String, Value> = HashMap::new();
    params.insert("max".to_string(), Value::Number(10.into()));
    let result = highlighted_from_params(&source, &code, &params);
    assert!(result.is_some());
    let span = result.unwrap();
    assert!(span.contains(&(source.text().len() - 2)));
}
#[test]
fn test_highlighted_span_from_params_section() {
    let source = Source::from(String::from("{\n  \"sections\": [\n    \"intro\",\n    \"methods\"\n  ]\n}"));
    let code = ErrorCode::Section;
    let index = 1;
    let value = serde_json::json!(["intro", "methods"]);
    let mut params: HashMap<String, Value> = HashMap::new();
    params.insert("index".to_string(), Value::Number(index.into()));
    params.insert("value".to_string(), value);
    let result = highlighted_from_params(&source, &code, &params);
    assert!(result.is_some());
}
#[test]
fn test_highlighted_span_from_params_with_index() {
    let source = Source::from(String::from("{\n  \"items\": [\n    \"first\",\n    \"second\"\n  ]\n}"));
    let code = ErrorCode::Other;
    let index = 0;
    let value = serde_json::json!(["first", "second"]);
    let mut params: HashMap<String, Value> = HashMap::new();
    params.insert("index".to_string(), Value::Number(index.into()));
    params.insert("value".to_string(), value);
    let result = highlighted_from_params(&source, &code, &params);
    assert!(result.is_some());
}
#[test]
fn test_highlighted_span_from_params_no_match() {
    let source = Source::from(String::from("{\"key\": \"value\"}"));
    let code = ErrorCode::Date;
    let params: HashMap<String, Value> = HashMap::new();
    let result = highlighted_from_params(&source, &code, &params);
    assert!(result.is_none());
}
#[test]
fn test_parse_vale_output() {
    let path = "/root/.cache/acorn/acornProject";
    let data = r#"
{
  "/root/.cache/acorn/acornProject": [
    {
      "Action": {
        "Name": "",
        "Params": null
      },
      "Span": [
        192,
        254
      ],
      "Check": "Google.OxfordComma",
      "Description": "",
      "Link": "https://developers.google.com/style/commas",
      "Message": "Use the Oxford comma in 'Once created, there is often no version control, stewardship or'.",
      "Severity": "warning",
      "Match": "Once created, there is often no version control, stewardship or",
      "Line": 8
    },
    {
      "Action": {
        "Name": "",
        "Params": null
      },
      "Span": [
        360,
        46
      ],
      "Check": "Vale.Avoid",
      "Description": "",
      "Link": "",
      "Message": "Avoid using 'geo-spatial'.",
      "Severity": "error",
      "Match": "geo-spatial",
      "Line": 170
    },
    {
      "Action": {
        "Name": "",
        "Params": null
      },
      "Span": [
        36,
        46
      ],
      "Check": "Vale.Avoid",
      "Description": "",
      "Link": "",
      "Message": "Avoid using 'geo-spatial'.",
      "Severity": "suggestion",
      "Match": "geo-spatial",
      "Line": 17
    }
  ]
}
    "#;
    let parsed: Vec<ValeOutputItem> = ValeOutput::parse(data, PathBuf::from(path));
    assert_eq!(parsed.len(), 3);
}
#[test]
#[cfg(any(unix, target_os = "wasi", target_os = "redox"))]
fn test_preprocess_vale_output_unix() {
    let path = PathBuf::from("/tmp/acorn/cache/check/test-file");
    let input = r#"{
  "/tmp/acorn/cache/check/test-file": [
    {
      "Action": {
        "Name": "",
        "Params": null
      },
      "Span": [192, 254],
      "Check": "Google.OxfordComma",
      "Description": "",
      "Link": "https://developers.google.com/style/commas",
      "Message": "Use the Oxford comma in 'Once created, there is often no version control, stewardship or'.",
      "Severity": "warning",
      "Match": "Once created, there is often no version control, stewardship or",
      "Line": 38
    }
  ]
}"#;

    let result = preprocess_vale_output(path, input);
    assert!(result.contains("\"items\":"));
    assert!(!result.contains("/tmp/acorn/cache/check/test-file"));

    // Verify the JSON is still valid after preprocessing
    let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&result);
    assert!(parsed.is_ok(), "Preprocessed output should be valid JSON");
}
#[test]
#[cfg(windows)]
fn test_preprocess_vale_output_windows() {
    let path = PathBuf::from(r"C:\Users\jason\AppData\Local\ornl\acorn\cache\check\cyzdN-Q4Nt\invalid-project-a");
    let input = r#"{
  "\\\\?\\C:\\Users\\jason\\AppData\\Local\\ornl\\acorn\\cache\\check\\cyzdN-Q4Nt\\invalid-project-a": [
    {
      "Action": {
        "Name": "",
        "Params": null
      },
      "Span": [192, 254],
      "Check": "Google.OxfordComma",
      "Description": "",
      "Link": "https://developers.google.com/style/commas",
      "Message": "Use the Oxford comma in 'Once created, there is often no version control, stewardship or'.",
      "Severity": "warning",
      "Match": "Once created, there is often no version control, stewardship or",
      "Line": 38
    }
  ]
}"#;

    let result = preprocess_vale_output(path.clone(), input);

    assert!(result.contains("\"items\":"), "Result should contain 'items' key");
    assert!(!result.contains("\\\\"), "Result should not contain double backslashes");
    assert!(!result.contains("C:/Users/jason"), "Result should not contain the path");
    assert!(!result.contains("//?/"), "Result should not contain extended path prefix");

    // Verify the JSON is still valid after preprocessing
    let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&result);
    assert!(parsed.is_ok(), "Preprocessed output should be valid JSON");
}
#[test]
#[cfg(windows)]
fn test_preprocess_vale_output_windows_full_example() {
    let path = PathBuf::from(r"C:\Users\jason\AppData\Local\ornl\acorn\cache\check\cyzdN-Q4Nt\invalid-project-a");
    let input = r#"{
  "\\\\?\\C:\\Users\\jason\\AppData\\Local\\ornl\\acorn\\cache\\check\\cyzdN-Q4Nt\\invalid-project-a": [
    {
      "Action": {
        "Name": "",
        "Params": null
      },
      "Span": [192, 254],
      "Check": "Google.OxfordComma",
      "Description": "",
      "Link": "https://developers.google.com/style/commas",
      "Message": "Use the Oxford comma in 'Once created, there is often no version control, stewardship or'.",
      "Severity": "warning",
      "Match": "Once created, there is often no version control, stewardship or",
      "Line": 38
    },
    {
      "Action": {
        "Name": "suggest",
        "Params": ["spellings"]
      },
      "Span": [23, 31],
      "Check": "Vale.Spelling",
      "Description": "",
      "Link": "",
      "Message": "Did you really mean 'Indexable'?",
      "Severity": "error",
      "Match": "Indexable",
      "Line": 47
    },
    {
      "Action": {
        "Name": "",
        "Params": null
      },
      "Span": [80, 82],
      "Check": "Google.Acronyms",
      "Description": "",
      "Link": "https://developers.google.com/style/abbreviations",
      "Message": "Spell out 'MNA', if it's unfamiliar to the audience.",
      "Severity": "suggestion",
      "Match": "MNA",
      "Line": 48
    },
    {
      "Action": {
        "Name": "",
        "Params": null
      },
      "Span": [84, 87],
      "Check": "Google.Will",
      "Description": "",
      "Link": "https://developers.google.com/style/tense",
      "Message": "Avoid using 'will'.",
      "Severity": "warning",
      "Match": "will",
      "Line": 48
    },
    {
      "Action": {
        "Name": "replace",
        "Params": ["geospatial"]
      },
      "Span": [36, 46],
      "Check": "Science.use-instead",
      "Description": "",
      "Link": "",
      "Message": "Use 'geospatial' instead of 'geo-spatial'.",
      "Severity": "error",
      "Match": "geo-spatial",
      "Line": 49
    },
    {
      "Action": {
        "Name": "suggest",
        "Params": ["spellings"]
      },
      "Span": [9, 15],
      "Check": "Vale.Spelling",
      "Description": "",
      "Link": "",
      "Message": "Did you really mean 'Jasdrey'?",
      "Severity": "error",
      "Match": "Jasdrey",
      "Line": 61
    },
    {
      "Action": {
        "Name": "suggest",
        "Params": ["spellings"]
      },
      "Span": [17, 23],
      "Check": "Vale.Spelling",
      "Description": "",
      "Link": "",
      "Message": "Did you really mean 'Wohlson'?",
      "Severity": "error",
      "Match": "Wohlson",
      "Line": 61
    },
    {
      "Action": {
        "Name": "replace",
        "Params": ["email"]
      },
      "Span": [3, 7],
      "Check": "Google.WordList",
      "Description": "",
      "Link": "https://developers.google.com/style/word-list",
      "Message": "Use 'email' instead of 'Email'.",
      "Severity": "warning",
      "Match": "Email",
      "Line": 62
    }
  ]
}"#;
    let result = preprocess_vale_output(path, input);
    // Verify the JSON is valid and can be parsed as ValeOutput
    let parsed: serde_json::Result<ValeOutput> = serde_json::from_str(&result);
    assert!(parsed.is_ok(), "Should parse as valid ValeOutput");
    if let Ok(vale_output) = parsed {
        assert_eq!(vale_output.items.len(), 8, "Should contain 8 items");
        assert_eq!(vale_output.items[0].check, "Google.OxfordComma");
        assert_eq!(vale_output.items[1].check, "Vale.Spelling");
        assert_eq!(vale_output.items[1].line, 47);
        assert_eq!(vale_output.items[7].check, "Google.WordList");
    }
}
#[test]
fn test_vale_output_parse_empty() {
    let path = PathBuf::from("test-file");
    let empty_output = "{}";
    let result = ValeOutput::parse(empty_output, path);
    assert_eq!(result.len(), 0, "Empty output should return empty vector");
}
#[test]
fn test_vale_output_severity_colored() {
    let warning = ValeOutputItemSeverity::Warning;
    let error = ValeOutputItemSeverity::Error;
    let suggestion = ValeOutputItemSeverity::Suggestion;
    assert!(!warning.colored().is_empty());
    assert!(!error.colored().is_empty());
    assert!(!suggestion.colored().is_empty());
}
#[test]
fn test_check_severity_default() {
    let check = check_ok!(CheckCategory::Prose);
    assert_eq!(check.severity, CheckSeverity::Error);
}
#[test]
fn test_check_severity_from_vale_output_item_severity_matrix() {
    let matrix = [
        (ValeOutputItemSeverity::Error, CheckSeverity::Error),
        (ValeOutputItemSeverity::Suggestion, CheckSeverity::Suggestion),
        (ValeOutputItemSeverity::Warning, CheckSeverity::Warning),
    ];
    matrix
        .into_iter()
        .for_each(|(input, expected)| assert_eq!(CheckSeverity::from(input), expected));
}
#[test]
fn test_check_severity_to_ariadne_color_matrix() {
    let matrix = [
        (CheckSeverity::Error, Color::Red),
        (CheckSeverity::Suggestion, Color::Blue),
        (CheckSeverity::Warning, Color::Yellow),
    ];
    matrix.into_iter().for_each(|(input, expected)| assert_eq!(Color::from(input), expected));
}
#[test]
fn test_check_severity_to_ariadne_report_kind_matrix() {
    let error_kind: ReportKind<'static> = CheckSeverity::Error.into();
    let suggestion_kind: ReportKind<'static> = CheckSeverity::Suggestion.into();
    let warning_kind: ReportKind<'static> = CheckSeverity::Warning.into();
    assert!(matches!(error_kind, ReportKind::Error));
    assert!(matches!(warning_kind, ReportKind::Warning));
    match suggestion_kind {
        | ReportKind::Custom(label, color) => {
            assert_eq!(label, "Suggestion");
            assert_eq!(color, Color::Blue);
        }
        | _ => panic!("Suggestion severity should map to custom report kind"),
    }
}
#[test]
fn test_report_schema_with_missing_value_param_does_not_panic() {
    let mut validation_errors = ValidationErrors::new();
    let mut error = ValidationError::new("license");
    error.params.clear();
    validation_errors.add("license", error);
    let issue = Check::init()
        .index(0)
        .category(CheckCategory::Schema)
        .severity(CheckSeverity::Error)
        .errors(ErrorKind::Validator(ValidationErrorsKind::Struct(Box::new(validation_errors))))
        .uri("fixtures/CITATION.cff")
        .build();
    issue.report();
}
#[test]
fn test_with_context_and_with_uri_preserve_severity() {
    let check = check!(
      CheckCategory::Link,
      false,
      severity: CheckSeverity::Warning,
      message: "link check"
    );
    let with_context = check.clone().with_context("content/index.json".to_string());
    let with_uri = check.with_uri(Some("https://example.org".to_string()));
    assert_eq!(with_context.severity, CheckSeverity::Warning);
    assert_eq!(with_uri.severity, CheckSeverity::Warning);
}
#[test]
fn test_collect_validation_checks_preserves_nested_indexed_paths_in_display() {
    #[derive(Validate)]
    struct ChildModel {
        #[validate(email)]
        email: String,
    }
    #[derive(Validate)]
    struct ParentListModel {
        #[validate(nested)]
        children: Vec<ChildModel>,
    }
    let value = ParentListModel {
        children: vec![
            ChildModel {
                email: "valid@example.com".to_string(),
            },
            ChildModel {
                email: "invalid".to_string(),
            },
        ],
    };
    let checks = collect_validation_checks(&value);
    assert_eq!(checks.len(), 1);
    let rendered = format!("{}", checks[0]);
    assert!(
        rendered.contains("children[1].email"),
        "expected nested indexed path in display output: {rendered}"
    );
    assert!(rendered.contains("EMAIL"), "expected EMAIL code in display output: {rendered}");
}
#[test]
fn test_collect_validation_checks_avoids_duplicate_wrapper_field_paths_in_display() {
    let cff = Cff {
        title: "Invalid License".to_string(),
        message: "Message".to_string(),
        license: Some(License::Single("not-a-license".to_string())),
        ..Cff::default()
    };
    let checks = collect_validation_checks(&cff);
    let schema_check = checks
        .iter()
        .find(|check| check.category == CheckCategory::Schema)
        .expect("expected schema check");
    let rendered = format!("{}", schema_check);
    assert_eq!(schema_check.context.as_deref(), Some("license"));
    assert!(rendered.contains("license"), "expected license path in display output: {rendered}");
    assert!(
        !rendered.contains("license.license"),
        "expected duplicated wrapper path to be removed: {rendered}"
    );
}
#[test]
fn test_nearest_space_boundary_returns_first_boundary_at_or_after_index() {
    let value = "Alpha Bravo Charlie";
    let result = nearest_space_boundary(value, 8);
    assert_eq!(result, Some(12));
}

#[test]
fn test_nearest_space_boundary_returns_none_without_boundary_at_or_after_index() {
    let value = "Alpha Bravo Charlie";
    let result = nearest_space_boundary(value, 12);
    assert_eq!(result, None);
}
#[test]
fn test_process_returns_field_issue_tuple() {
    use crate::analyzer::error::{ErrorCode, ErrorKind};
    let value = FieldModel { name: "x".to_string() };
    let errors = value.validate().expect_err("expected validation failure");
    let issues = ErrorKind::Validator(ValidationErrorsKind::Struct(Box::new(errors))).process("");
    assert_eq!(issues.len(), 1);
    let issue = &issues[0];
    assert!(matches!(issue.code, ErrorCode::Length));
    assert_eq!(issue.path.as_deref(), Some("name"));
    assert_eq!(issue.message, "length");
    assert_eq!(issue.params.get("value").map(|v| v.as_str()), Some(Some("x")));
    assert_eq!(issue.params.get("min").and_then(|v| v.as_i64()), Some(3));
    assert_eq!(issue.params.get("max").and_then(|v| v.as_i64()), Some(3));
}
#[test]
fn test_process_returns_nested_struct_path() {
    use crate::analyzer::error::{ErrorCode, ErrorKind};
    let value = ParentModel {
        child: ChildModel {
            email: "not-an-email".to_string(),
        },
    };
    let errors = value.validate().expect_err("expected validation failure");
    let issues = ErrorKind::Validator(ValidationErrorsKind::Struct(Box::new(errors))).process("");
    assert_eq!(issues.len(), 1);
    let issue = &issues[0];
    assert!(matches!(issue.code, ErrorCode::Email));
    assert_eq!(issue.path.as_deref(), Some("child.email"));
    assert_eq!(issue.params.get("value").map(|v| v.as_str()), Some(Some("not-an-email")));
}
#[test]
fn test_process_returns_nested_list_index_path() {
    use crate::analyzer::error::{ErrorCode, ErrorKind};
    let value = ParentListModel {
        children: vec![
            ChildModel {
                email: "valid@example.com".to_string(),
            },
            ChildModel {
                email: "invalid".to_string(),
            },
        ],
    };
    let errors = value.validate().expect_err("expected validation failure");
    let issues = ErrorKind::Validator(ValidationErrorsKind::Struct(Box::new(errors))).process("");
    assert_eq!(issues.len(), 1);
    let issue = &issues[0];
    assert!(matches!(issue.code, ErrorCode::Email));
    assert_eq!(issue.path.as_deref(), Some("children[1].email"));
    assert_eq!(issue.params.get("value").map(|v| v.as_str()), Some(Some("invalid")));
}
#[test]
fn test_process_returns_none_path_for_root_field_errors() {
    use crate::analyzer::error::process;
    use crate::analyzer::error::ErrorCode;
    let value = FieldModel { name: "x".to_string() };
    let errors = value.validate().expect_err("expected validation failure");
    let field_errors = errors
        .into_errors()
        .remove("name")
        .and_then(|kind| match kind {
            | ValidationErrorsKind::Field(errors) => Some(errors),
            | _ => None,
        })
        .expect("expected field-level validation errors for name");
    let kind = ValidationErrorsKind::Field(field_errors);
    let issues = process("", &kind);
    assert_eq!(issues.len(), 1);
    let issue = &issues[0];
    assert!(matches!(issue.code, ErrorCode::Length));
    assert_eq!(issue.path.as_deref(), None);
    assert_eq!(issue.message, "length");
    assert_eq!(issue.params.get("value").map(|v| v.as_str()), Some(Some("x")));
    assert_eq!(issue.params.get("min").and_then(|v| v.as_i64()), Some(3));
}
#[test]
fn test_process_serializes_array_param_values() {
    use crate::analyzer::error::{ErrorCode, ErrorKind};
    let value = ArrayParamModel {
        tags: vec!["one".to_string(), "two".to_string()],
    };
    let errors = value.validate().expect_err("expected validation failure");
    let issues = ErrorKind::Validator(ValidationErrorsKind::Struct(Box::new(errors))).process("");
    assert_eq!(issues.len(), 1);
    let issue = &issues[0];
    assert!(matches!(issue.code, ErrorCode::Length));
    assert_eq!(issue.path.as_deref(), Some("tags"));
    assert_eq!(issue.message, "length");
    assert_eq!(
        issue.params.get("value").and_then(|v| v.as_array()).map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(|item| item.to_string())
                .collect::<Vec<String>>()
        }),
        Some(vec!["one".to_string(), "two".to_string()])
    );
    assert_eq!(issue.params.get("min").and_then(|v| v.as_i64()), Some(3));
}
#[test]
fn test_process_preserves_non_wrapper_repeated_segment_paths() {
    use crate::analyzer::error::ErrorKind;
    let value = RepeatedNameParentModel {
        child: RepeatedNameModel {
            child: RepeatedChildModel {
                email: "invalid".to_string(),
            },
        },
    };
    let errors = value.validate().expect_err("expected validation failure");
    let issues = ErrorKind::Validator(ValidationErrorsKind::Struct(Box::new(errors))).process("");
    assert_eq!(issues.len(), 1);
    let issue = &issues[0];
    assert_eq!(issue.path.as_deref(), Some("child.child.email"));
}
#[test]
fn test_resolve_vale_package_source() {
    let value = ValeConfig::resolve_package("Google");
    assert_eq!(value, "https://github.com/errata-ai/Google/releases/latest/download/Google.zip");
    let value = ValeConfig::resolve_package("https://example.org/custom-style.zip");
    assert_eq!(value, "https://example.org/custom-style.zip");
    let value = ValeConfig::resolve_package("file:///tmp/custom-style");
    assert_eq!(value, "file:///tmp/custom-style");
    let value = ValeConfig::resolve_package(" /tmp/custom-style ");
    assert_eq!(value, "/tmp/custom-style");
    let values = ["Google".to_string(), "https://example.org/custom-style.zip".to_string()];
    let resolved = values.iter().map(ValeConfig::resolve_package).collect::<Vec<String>>();
    assert_eq!(
        resolved,
        vec![
            "https://github.com/errata-ai/Google/releases/latest/download/Google.zip".to_string(),
            "https://example.org/custom-style.zip".to_string(),
        ]
    );
}
#[test]
fn test_source_text_formats_array_values_as_markdown_list() {
    use crate::analyzer::error::{ErrorCode, Location, ValidatorIssue};
    let mut params: HashMap<String, Value> = HashMap::new();
    params.insert("value".to_string(), serde_json::json!(["alpha", "beta"]));
    let issue = ValidatorIssue::init()
        .code(ErrorCode::Section)
        .maybe_path(Some("sections.impact".to_string()))
        .message("section")
        .params(params)
        .build();
    let source = issue.source_text();
    assert!(source.contains("- \"alpha\""));
    assert!(source.contains("- \"beta\""));
}
#[test]
fn test_truncate_keeps_prefix_at_or_below_maximum() {
    let source_text = "Alpha Bravo Charlie Delta Echo";
    let span = 20..=24;
    let (result_source, result_span) = truncate(span, source_text, 5);
    let retained_prefix_length = result_span.start().saturating_sub(3);
    assert_eq!(result_source.text(), "...Delta Echo");
    assert!(retained_prefix_length <= 5);
}
#[test]
fn test_truncate_source_before_span_no_truncation() {
    let source_text = "short line";
    let span = 2..=5;
    let (result_source, result_span) = truncate(span, source_text, 10);
    assert_eq!(result_source.text(), source_text);
    assert_eq!(result_span, 2..=5);
}
#[test]
fn test_truncate_source_before_span_with_ellipsis() {
    let source_text = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let span = 20..=24;
    let (result_source, result_span) = truncate(span, source_text, 5);
    let result_text = result_source.text();
    assert!(result_text.starts_with("..."));
    assert_eq!(result_span, 8..=12);
    assert_eq!(&result_text[8..=12], "KLMNO");
}

#[test]
fn test_truncate_source_before_span_uses_nearest_space() {
    let source_text = "Alpha Bravo Charlie Delta Echo";
    let span = 20..=24;
    let (result_source, result_span) = truncate(span, source_text, 5);
    let result_text = result_source.text();
    assert_eq!(result_text, "...Delta Echo");
    assert_eq!(result_span, 3..=7);
}
#[test]
fn test_vale_config_unwrap_or_default_uses_default_values() {
    let value: Option<ValeConfig> = Vec::<ValeConfig>::new().into_iter().next();
    let config = value.unwrap_or_default();
    assert_eq!(config.path, PathBuf::from("./.vale/.vale.ini"));
    assert!(!config.packages.is_empty());
    assert!(!config.vocabularies.is_empty());
    assert!(!config.disabled.is_empty());
}
#[test]
fn test_vale_output_item_location_matches_prose_display_format() {
    use crate::analyzer::error::Location;
    use crate::analyzer::vale::{ValeOutputItem, ValeOutputItemAction, ValeOutputItemSeverity};
    let item = ValeOutputItem {
        action: ValeOutputItemAction {
            name: "replace".to_string(),
            params: None,
        },
        check: "Vale.Spelling".to_string(),
        description: "".to_string(),
        line: 17,
        link: "".to_string(),
        message: "Did you really mean 'Indexable'?".to_string(),
        severity: ValeOutputItemSeverity::Error,
        span: vec![23, 31],
        word_match: "Indexable".to_string(),
    };
    assert_eq!(item.locator(), "Line 17, Character 23");
}
#[test]
fn test_vale_output_item_source_text_uses_word_match() {
    use crate::analyzer::error::Location;
    use crate::analyzer::vale::{ValeOutputItem, ValeOutputItemAction, ValeOutputItemSeverity};
    let item = ValeOutputItem {
        action: ValeOutputItemAction {
            name: "replace".to_string(),
            params: None,
        },
        check: "Science.use-instead".to_string(),
        description: "".to_string(),
        line: 19,
        link: "".to_string(),
        message: "Use 'geospatial' instead of 'geo-spatial'.".to_string(),
        severity: ValeOutputItemSeverity::Error,
        span: vec![36, 46],
        word_match: "geo-spatial".to_string(),
    };
    assert_eq!(item.source_text(), "geo-spatial");
}
#[test]
fn test_standard_to_schema_does_not_panic() {
    let standards = [
        Standard::ResearchActivityData,
        Standard::CitationFileFormat,
        Standard::Datacite,
        Standard::Dcat,
        Standard::Huwise,
        Standard::Invenio,
        Standard::Raid,
        Standard::Text,
        Standard::Docx,
    ];
    for standard in standards {
        standard.to_schema("json");
    }
}
