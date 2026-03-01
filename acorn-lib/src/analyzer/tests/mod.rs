use crate::analyzer::vale::{preprocess_vale_output, ValeOutput, ValeOutputItem, ValeOutputItemSeverity};
use crate::analyzer::{checks_to_dataframe, Check, CheckCategory};
use crate::prelude::PathBuf;
#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn test_checks_to_dataframe() {
    let check = Check::init().category(CheckCategory::Prose).success(true).build();
    let checks = vec![check.clone(), check.clone(), check];
    let df = checks_to_dataframe(checks.clone());
    let reason = "Failed to convert checks to dataframe";
    assert_eq!(df.expect(reason).shape(), (checks.len(), 6));
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
