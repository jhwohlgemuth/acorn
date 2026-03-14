use crate::analyzer::error::ErrorCode;
use crate::analyzer::vale::{preprocess_vale_output, ValeOutput, ValeOutputItem, ValeOutputItemSeverity};
use crate::analyzer::{
  checks_to_dataframe,
  highlighted_span,
  highlighted_span_for_array,
  highlighted_span_from_params,
  Check,
  CheckCategory,
  CheckSeverity,
};
use crate::prelude::{HashMap, PathBuf};
use ariadne::Color;
use ariadne::ReportKind;
use ariadne::Source;
use serde_json::Value;

#[test]
fn test_checks_to_dataframe() {
    let check = Check::init().category(CheckCategory::Prose).success(true).build();
    let checks = vec![check.clone(), check.clone(), check];
    let df = checks_to_dataframe(checks.clone());
    let reason = "Failed to convert checks to dataframe";
    assert_eq!(df.expect(reason).shape(), (checks.len(), 6));
}
#[test]
fn test_highlighted_span() {
    let source = Source::from(String::from("line one\nline two\nline three"));
    let span = &[1, 5];
    let line_number = 2;
    let result = highlighted_span(source, span, line_number);
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
    let result = highlighted_span_for_array(source.clone(), params);
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
    let result = highlighted_span_from_params(source.clone(), code, params);
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
    let result = highlighted_span_from_params(source, code, params);
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
    let result = highlighted_span_from_params(source, code, params);
    assert!(result.is_some());
}
#[test]
fn test_highlighted_span_from_params_no_match() {
    let source = Source::from(String::from("{\"key\": \"value\"}"));
    let code = ErrorCode::Date;
    let params: HashMap<String, Value> = HashMap::new();
    let result = highlighted_span_from_params(source, code, params);
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
  let check = Check::init().category(CheckCategory::Prose).success(true).build();
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
  matrix
    .into_iter()
    .for_each(|(input, expected)| assert_eq!(Color::from(input), expected));
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
fn test_with_context_and_with_uri_preserve_severity() {
  let check = Check::init()
    .category(CheckCategory::Link)
    .success(false)
    .severity(CheckSeverity::Warning)
    .message("link check")
    .build();
  let with_context = check.clone().with_context("content/index.json".to_string());
  let with_uri = check.with_uri("https://example.org".to_string());
  assert_eq!(with_context.severity, CheckSeverity::Warning);
  assert_eq!(with_uri.severity, CheckSeverity::Warning);
}
