#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use std::env::temp_dir;

use crate::io::api::{IntoBody, ParamStyle};
use crate::io::{
    command_exists, extract_zip, file_name_with_parent, git_branch_name, git_default_branch_name, parent, read_file, to_absolute_string, FromCommand,
    FromPath,
};
use crate::param;
use crate::prelude::{Path, PathBuf};
use crate::util::*;
use proptest::prelude::*;
#[cfg(test)]
use similar_asserts::assert_eq;

const FIXTURES: &str = "../tests/fixtures";

#[cfg(feature = "cmd")]
mod cmd;

#[test]
fn test_base32_crockford() {
    let value = 1234;
    let expected = "16j";
    assert_eq!(base32_crockford_encode(value), expected);
    let value = 123456789012345;
    let expected = "3g9230vqvs";
    assert_eq!(base32_crockford_encode(value), expected);
    // Round trip
    let value = 1234;
    let encoded = base32_crockford_encode(value);
    assert_eq!(base32_crockford_decode(&encoded), Some(value));
}
#[test]
fn test_chunk_string() {
    assert_eq!("abcdefghi".chunk(3), vec!["abc", "def", "ghi"]);
    assert_eq!("abcdefghi".chunk(2), vec!["ab", "cd", "ef", "gh", "i"]);
    assert_eq!("abcdefghi".to_string().chunk(3), vec!["abc", "def", "ghi"]);
}
#[test]
fn test_text_diff_changes_without_color_has_no_ansi_sequences() {
    let changes = text_diff_changes_with_color("old\n", "new\n", false);
    assert_eq!(
        changes,
        vec![
            (similar::ChangeTag::Delete, "- old\n".to_string()),
            (similar::ChangeTag::Insert, "+ new\n".to_string())
        ]
    );
    assert!(changes.iter().all(|(_, line)| !line.contains('\u{1b}')));
}
#[test]
fn test_last_values() {
    let values: Vec<String> = Constant::last_values("technology").collect();
    assert!(!values.is_empty());
    assert!(values.iter().all(|v| !v.is_empty()));
    assert!(values.contains(&"true".to_string()) || values.contains(&"false".to_string()));
    insta::assert_snapshot!(values[..5].join("\n"));
    let partners: Vec<String> = Constant::last_values("partners").collect();
    assert!(!partners.is_empty());
    assert!(partners.contains(&"041m9xr71".to_string()));
    let empty: Vec<String> = Constant::last_values("nonexistent-file").collect();
    assert!(empty.is_empty());
}
#[test]
fn test_nth_values() {
    let first_column: Vec<String> = Constant::nth_values("technology", 0).collect();
    assert!(!first_column.is_empty());
    assert!(first_column.iter().all(|v| !v.is_empty()));
    assert!(first_column.contains(&"airflow".to_string()));
    insta::assert_snapshot!(first_column[..5].join("\n"));
    let third_column: Vec<String> = Constant::nth_values("technology", 2).collect();
    assert!(!third_column.is_empty());
    assert!(third_column.contains(&"Apache Airflow".to_string()));
    insta::assert_snapshot!(third_column[..5].join("\n"));
    let partner_names: Vec<String> = Constant::nth_values("partners", 0).collect();
    assert!(partner_names.contains(&"Ames Laboratory".to_string()));
    let partner_abbrevs: Vec<String> = Constant::nth_values("partners", 1).collect();
    assert!(partner_abbrevs.contains(&"ANL".to_string()));
    let out_of_bounds: Vec<String> = Constant::nth_values("technology", 999).collect();
    assert!(out_of_bounds.is_empty());
    let empty: Vec<String> = Constant::nth_values("nonexistent-file", 0).collect();
    assert!(empty.is_empty());
}
#[test]
fn test_detect_json() {
    let valid = [
        r#"{"key": "value"}"#,
        r#"[1, 2, 3]"#,
        "{}",
        r#"  { "key" : "value" }  "#,
        r#"{"nested": {"key": "value"}}"#,
        r#"[{"id": 1}, {"id": 2}]"#,
        "true",
        "false",
        "null",
        "123",
        "42.5",
        r#""hello""#,
        r#"{"key": "value with \"escaped\" quotes"}"#,
        "{\"key\": \"value with all quotes escaped\"}",
    ];
    let invalid = [
        r#"Just a string"#,
        r#"{"key": "value""#, // Missing closing brace
        r#"<root><key>value</key></root>"#,
        "OK",
        "undefined",
        r#"{'key': 'value'}"#, // Single quotes not valid JSON
        r#"{"key": value}"#,   // Unquoted value
    ];
    for x in valid.iter() {
        assert!(detect_json(x), "=> [REASON] \"{x}\" is NOT detected as valid JSON");
    }
    for x in invalid.iter() {
        assert!(!detect_json(x), "=> [REASON] \"{x}\" IS detected as valid JSON");
    }
}
#[test]
fn test_detect_xml() {
    let valid = [
        r#"<root></root>"#,
        r#"<?xml version="1.0"?><root></root>"#,
        r#"<root><child></child></root>"#,
        r#"<root attr="value"></root>"#,
        r#"<root><child>text</child></root>"#,
        r#"<root><child attr="val">text</child></root>"#,
        r#"<root><child1></child1><child2></child2></root>"#,
        r#"  <root></root>  "#,
        r#"<root><!-- comment --></root>"#,
        r#"<root><![CDATA[data]]></root>"#,
    ];
    let invalid = [
        r#"Just a string"#,
        r#"<root></root"#,         // Missing closing >
        r#"<root><child></root>"#, // Mismatched tags
        r#"{"key": "value"}"#,
        "OK",
        r#"root></root>"#, // Missing opening <
        "",
        r#"<root"#,       // Incomplete tag
        r#"root</root>"#, // Missing opening <
    ];
    for x in valid.iter() {
        assert!(detect_xml(x), "=> [REASON] \"{x}\" is NOT detected as valid XML");
    }
    for x in invalid.iter() {
        assert!(!detect_xml(x), "=> [REASON] \"{x}\" IS detected as valid XML");
    }
}
#[test]
fn test_extract_zip() {
    let path = PathBuf::from(FIXTURES).join("data/highlight/reference.pptx");
    let result = extract_zip(path, None);
    println!("{result:?}");
    assert!(result.is_ok());
}
#[test]
fn test_file_extension() {
    assert_eq!(file_extension("hello.txt"), Some("txt".to_string()));
    assert_eq!(file_extension("README.md"), Some("md".to_string()));
    assert_eq!(file_extension("file.tar.gz"), Some("gz".to_string()));
    assert_eq!(file_extension("/path/to/folder/file.csv"), Some("csv".to_string()));
    assert_eq!(file_extension("./path/to/folder/file.csv"), Some("csv".to_string()));
    assert_eq!(file_extension(".dotfile"), None);
    assert_eq!(file_extension(".env"), None);
    assert_eq!(file_extension("filename"), None);
    assert_eq!(file_extension("/path/to/folder"), None);
    assert_eq!(file_extension("./path/to/folder"), None);
    assert_eq!(file_extension("/path/to/.hidden/folder"), None);
    assert_eq!(file_extension("path/to/.hidden/folder"), None);
}
#[test]
fn test_find_first() {
    let values = vec![
        ("a".to_string(), "b".to_string()),
        ("c".to_string(), "d".to_string()),
        ("e".to_string(), "f".to_string()),
    ];
    assert_eq!(find_first(values.clone(), "c"), Some(("c".to_string(), "d".to_string())));
    assert_eq!(find_first(values.clone(), "e"), Some(("e".to_string(), "f".to_string())));
    assert!(find_first(values.clone(), "does not exist").is_none());
}
#[test]
fn test_frontmatter_and_body() {
    let both = r#"---
name: Jason
type: person
---
body"#;
    let (frontmatter, body) = frontmatter_and_body(both);
    assert!(frontmatter.is_some());
    assert_eq!(frontmatter.unwrap(), "name: Jason\ntype: person".to_string());
    assert_eq!(body, "body".to_string());
    let no_body = r#"---
name: Jason
type: person
---"#;
    let (frontmatter, body) = frontmatter_and_body(no_body);
    assert!(frontmatter.is_some());
    assert_eq!(frontmatter.unwrap(), "name: Jason\ntype: person".to_string());
    assert!(body.is_empty());
    let no_frontmatter = r#"
body"#;
    let (frontmatter, body) = frontmatter_and_body(no_frontmatter);
    assert!(frontmatter.is_none());
    assert_eq!(body, "body".to_string());
    let empty_frontmatter = r#"---
---
body"#;
    let (frontmatter, body) = frontmatter_and_body(empty_frontmatter);
    assert!(frontmatter.is_none());
    assert_eq!(body, "body".to_string());
    let empty_frontmatter_no_body = r#"---
---"#;
    let (frontmatter, body) = frontmatter_and_body(empty_frontmatter_no_body);
    assert!(frontmatter.is_none());
    assert!(body.is_empty());
}
#[test]
fn test_generate_guid() {
    let id = generate_guid();
    assert_eq!(id.len(), 10);
}
#[ignore]
#[test]
fn test_git_branch() {
    assert_eq!(git_default_branch_name(), Some("main".to_string()));
    assert!(git_branch_name().is_some());
}
#[test]
fn test_mimetype() {
    let mime = MimeType::from("a.cff");
    assert_eq!(mime, MimeType::Cff);
    let mime = MimeType::from("a.txt");
    assert_eq!(mime, MimeType::Text);
    let mime = MimeType::from("font.otf".to_string());
    assert_eq!(mime, MimeType::Otf);
    let mime = MimeType::from("font.ttf");
    assert_eq!(mime, MimeType::Ttf);
    let mime = MimeType::from_path(Path::new("a.json"));
    assert_eq!(mime, MimeType::Json);
    let mime = MimeType::from_path(&PathBuf::from("file.yaml"));
    assert_eq!(mime, MimeType::Yaml);
    assert_eq!(MimeType::Json.file_type(), "json");
    assert_eq!(MimeType::Jsonc.file_type(), "jsonc");
    assert_eq!(MimeType::Ttf.file_type(), "ttf");
    assert_eq!(MimeType::Yaml.file_type(), "yaml");
    let mime = MimeType::from_path(Path::new("a.jsonc"));
    assert_eq!(mime, MimeType::Jsonc);
    let mime = MimeType::from("config.jsonc");
    assert_eq!(mime, MimeType::Jsonc);
}
#[test]
fn test_merge_chain_positional_only() {
    let positional = vec!["a".to_string(), "b".to_string()];
    let result = merge(&positional, None::<&[String]>);
    assert_eq!(result, vec!["a", "b"]);
}
#[test]
fn test_merge_chain_config_only() {
    let positional: Vec<String> = vec![];
    let config = Some(vec!["x".to_string(), "y".to_string()]);
    let result = merge(&positional, config.as_deref());
    assert_eq!(result, vec!["x", "y"]);
}
#[test]
fn test_merge_positional_takes_priority() {
    let positional = vec!["b".to_string()];
    let config = Some(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    let result = merge(&positional, config.as_deref());
    assert_eq!(result, vec!["b", "a", "c"]);
}
#[test]
fn test_merge_dedup_first_seen_wins() {
    let positional = vec!["m1".to_string(), "m2".to_string()];
    let config = Some(vec!["m2".to_string(), "m3".to_string(), "m1".to_string()]);
    let result = merge(&positional, config.as_deref());
    assert_eq!(result, vec!["m1", "m2", "m3"]);
}
#[test]
fn test_merge_filters_blank_entries() {
    let positional = vec!["", "  ", "a"];
    let config = Some(["", "b"]);
    let result = merge(positional, config);
    assert_eq!(result, vec!["a", "b"]);
}
#[test]
fn test_merge_all_blank_is_empty() {
    let positional = vec!["", "  "];
    let config = Some([""]);
    let result = merge(positional, config);
    assert!(result.is_empty());
}
#[test]
fn test_merge_trims_whitespace() {
    let positional = [" a ", "b"];
    let config = Some(vec!["b".to_string(), " c ".to_string()]);
    let result = merge(positional, config);
    assert_eq!(result, vec!["a", "b", "c"]);
}
#[test]
fn test_param_body_shorthand_no_comma() {
    let body = "{\"title\":\"ACORN\"}";
    let body_param = param!(Body & body);
    assert_eq!(body_param.name, "");
    let payload = vec![body_param].into_body();
    assert_eq!(payload, serde_json::json!({ "title": "ACORN" }));
}
#[test]
fn test_param_body_shorthand_with_comma() {
    let body = "plain text";
    let body_param = param!(Body, &body);
    assert_eq!(body_param.name, "");
    let payload = vec![body_param].into_body();
    assert_eq!(payload, serde_json::json!("plain text"));
}
#[test]
fn test_param_single_value() {
    let param = param!(ParamStyle::FieldList, "fl", "family-name");
    assert_eq!(param.name, "fl");
    assert_eq!(param.values, vec![vec![Some("family-name".to_string())]]);
}
#[test]
fn test_param_single_tuple() {
    let param = param!(ParamStyle::QueryPair, "filter", ("status", "inactive"));
    assert_eq!(param.name, "filter");
    assert_eq!(param.values, vec![vec![Some("status".to_string()), Some("inactive".to_string())]]);
}
#[test]
fn test_param_multiple_tuples() {
    let param = param!(
        ParamStyle::QueryPair,
        "q",
        (("affiliation-org-name", "Lyrasis"), ("ror-org-id", "https://ror.org/01qz5mb56"),)
    );
    assert_eq!(param.name, "q");
    assert_eq!(
        param.values,
        vec![
            vec![Some("affiliation-org-name".to_string()), Some("Lyrasis".to_string())],
            vec![Some("ror-org-id".to_string()), Some("https://ror.org/01qz5mb56".to_string())]
        ]
    );
}
#[test]
fn test_param_vec_notation() {
    let param = param!(ParamStyle::FieldList, "fields", vec![vec!["field1", "field2"], vec!["field3"],]);
    assert_eq!(param.name, "fields");
    assert_eq!(
        param.values,
        vec![
            vec![Some("field1".to_string()), Some("field2".to_string())],
            vec![Some("field3".to_string())]
        ]
    );
}
#[test]
fn test_param_template_value() {
    let param = param!(ParamStyle::TemplateValue, "identifier", "01qz5mb56");
    assert_eq!(param.name, "identifier");
    assert_eq!(param.values, vec![vec![Some("01qz5mb56".to_string())]]);
}
#[test]
fn test_param_with_quoted_value() {
    let param = param!(ParamStyle::QueryPair, "ror-id", "\"https://ror.org/01qz5mb56\"");
    assert_eq!(param.name, "ror-id");
    assert_eq!(param.values, vec![vec![Some("\"https://ror.org/01qz5mb56\"".to_string())]]);
}
#[test]
fn test_parent() {
    assert!(parent(FIXTURES).ends_with("tests"));
    assert!(parent(FIXTURES.to_string()).ends_with("tests"));
    assert!(parent(PathBuf::from(FIXTURES)).ends_with("tests"));
    assert!(parent(PathBuf::from("Cargo.toml")).ends_with("acorn-lib"));
    assert_eq!(parent(PathBuf::from("does-not-exist.yaml")), PathBuf::from("."));
}
#[test]
fn test_read() {
    let path = PathBuf::from(FIXTURES).join("test.txt");
    let content = read_file(path);
    assert!(content.is_ok());
    assert_eq!(content.unwrap(), "Hello world!");
    let path = PathBuf::from(FIXTURES).join("does-not-exist");
    let content = read_file(path);
    assert!(content.is_err());
}
#[test]
fn test_regex_capture_lookup() {
    let lookup = regex_capture_lookup(
        r"(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})",
        "2023-06-30",
        vec!["year", "month", "day"],
    );
    assert_eq!(lookup["year"], "2023");
    assert_eq!(lookup["month"], "06");
    assert_eq!(lookup["day"], "30");
}
#[test]
fn test_regex_inverse() {
    assert_eq!(regex_inverse("\\.json$"), "^(?!.*(?:\\.json$)).*$".to_string());
    let joined = regex_join(&["\\.json$".to_string(), "^README".to_string()]).unwrap();
    assert_eq!(regex_inverse(joined), "^(?!.*(?:(?:\\.json$)|(?:^README))).*$".to_string());
}
#[test]
fn test_regex_join() {
    assert_eq!(regex_join(&[]), None);
    assert_eq!(regex_join(&["".to_string()]), None);
    let patterns = vec!["\\.json$".to_string(), "^README".to_string()];
    assert_eq!(regex_join(&patterns), Some("(?:\\.json$)|(?:^README)".to_string()));
    let patterns = vec!["\\.json$".to_string(), "".to_string(), "^README".to_string()];
    assert_eq!(regex_join(&patterns), Some("(?:\\.json$)|(?:^README)".to_string()));
}
#[test]
fn test_regex_to_glob() {
    assert_eq!(regex_to_glob("\\.gguf$"), Some("*.gguf".to_string()));
    assert_eq!(regex_to_glob("Q4_K_M.*\\.gguf$"), Some("*Q4_K_M*.gguf".to_string()));
    assert_eq!(regex_to_glob("gguf$"), Some("*gguf".to_string()));
    assert_eq!(regex_to_glob("tiny\\.gguf"), Some("*tiny.gguf*".to_string()));
    assert_eq!(regex_to_glob("^model\\.bin"), Some("model.bin*".to_string()));
    assert_eq!(regex_to_glob("Q2_"), Some("*Q2_*".to_string()));
    assert_eq!(regex_to_glob("^model\\.bin$"), Some("model.bin".to_string()));
    assert!(regex_to_glob("[a-z]+\\.gguf$").is_none());
    assert!(regex_to_glob("\\.(gguf|safetensors)$").is_none());
    assert!(regex_to_glob("model+.gguf").is_none());
    assert!(regex_to_glob("(?=model).*\\.gguf").is_none());
    assert!(regex_to_glob("model$extra").is_none());
    assert!(regex_to_glob("").is_none());
    assert!(regex_to_glob("\\d+\\.gguf$").is_none());
    assert!(regex_to_glob("\\bmodel\\b").is_none());
}
#[test]
fn test_glob_matches() {
    assert!(glob_matches("model.Q4_K_M.gguf", "*.gguf"));
    assert!(!glob_matches("model.Q4_K_M.gguf", "*.safetensors"));
    assert!(glob_matches("model.gguf", "model.ggu?"));
    assert!(!glob_matches("model.gguf", "model.ggu??"));
    assert!(glob_matches("tiny.model.gguf", "*.gguf"));
    assert!(glob_matches("model.gguf", "model*"));
    assert!(glob_matches("model.Q4_K_M.gguf", "model*.gguf"));
    assert!(glob_matches("model.Q4_K_S.gguf", "model*.gguf"));
    assert!(!glob_matches("other.Q4_K_M.gguf", "model*.gguf"));
    assert!(glob_matches("model.gguf", "model.gguf"));
    assert!(!glob_matches("model.gguf", "other.gguf"));
    assert!(!glob_matches("model.gguf", ""));
    assert!(glob_matches("a.b.c.gguf", "*.gguf"));
    assert!(glob_matches("a.b.c.gguf", "*.*"));
}
#[test]
fn test_semantic_version() {
    let version = SemanticVersion::from("3");
    assert_eq!(version.major, 3);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);
    let version = SemanticVersion::from("2.1");
    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 1);
    assert_eq!(version.patch, 0);
    // Redundant case to ensure BagIt version is supported
    let version = SemanticVersion::from("1.0");
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);
    let version = SemanticVersion::from("1.2.3");
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    assert_eq!(format!("{}", version), "1.2.3");
    // cargo
    let version = SemanticVersion::from("cargo 1.88.0-nightly (d811228b1 2025-04-15)");
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 88);
    assert_eq!(version.patch, 0);
    // Git
    let version = SemanticVersion::from("git version 2.39.5");
    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 39);
    assert_eq!(version.patch, 5);
    // Pandoc
    let version = SemanticVersion::from(
        r#"
    pandoc 3.6.4
    Features: +server +lua
    Scripting engine: Lua 5.4
    User data directory: /root/.local/share/pandoc
    Copyright (C) 2006-2024 John MacFarlane. Web: https://pandoc.org
    This is free software; see the source for copying conditions. There is no
    warranty, not even for merchantability or fitness for a particular purpose.
    "#,
    );
    assert_eq!(version.major, 3);
    assert_eq!(version.minor, 6);
    assert_eq!(version.patch, 4);
    assert!(SemanticVersion::from_command("cargo").is_some());
    assert!(SemanticVersion::from_command("cargo").unwrap().major >= 1);
    assert!(SemanticVersion::from_command("not-a-real-command").is_none());
}
#[test]
fn test_snake_case() {
    assert_eq!(snake_case("FooBar"), "foo_bar");
    assert_eq!(snake_case("fooBar"), "foo_bar");
    assert_eq!(snake_case("fooBarBaz"), "foo_bar_baz");
    assert_eq!(snake_case("fooBarBaz".to_string()), "foo_bar_baz");
    assert_eq!(snake_case("A1B2"), "a_1_b_2");
}
#[test]
fn test_strip_suffixes() {
    let suffixes = &["-fp8", "-maas"];
    assert_eq!(strip_suffixes(suffixes, "model-fp8"), "model");
    assert_eq!(strip_suffixes(suffixes, "model-maas"), "model");
    assert_eq!(strip_suffixes(suffixes, "model"), "model");
}
#[test]
fn test_to_ascii_alphanumeric() {
    assert_eq!(to_ascii_alphanumeric("NVIDIA-Nemotron_3 Super"), "nvidianemotron3super");
}
#[test]
fn test_to_strings() {
    let list: Vec<PathBuf> = vec![];
    assert!(list.to_strings().is_empty());
    let list = vec![PathBuf::from("foo"), PathBuf::from("bar"), PathBuf::from("baz")];
    assert!(list.to_strings().contains(&"foo".to_string()));
    let list = vec![Path::new("foo"), Path::new("bar"), Path::new("baz")];
    assert!(list.to_strings().contains(&"foo".to_string()));
    let list = vec!["foo", "bar", "baz"];
    assert!(list.to_strings().contains(&"foo".to_string()));
}
#[test]
fn test_suffix() {
    assert_eq!(suffix(0), "s");
    assert_eq!(suffix(1), "");
    assert_eq!(suffix(2), "s");
}
#[test]
fn test_test_command() {
    let nonexistent = "does-not-exist".to_string();
    assert!(!command_exists(nonexistent));
    if cfg!(target_os = "windows") {
        assert!(command_exists("cmd".to_string()));
    } else {
        assert!(command_exists("ls"));
        assert!(command_exists("cargo".to_string()));
    }
}
#[test]
fn test_to_absolute_string() {
    let path = PathBuf::from(FIXTURES).join("test.txt");
    assert!(path.to_absolute_path().ends_with("test.txt"));
    assert!(to_absolute_string(path).ends_with("test.txt"));
    assert_eq!(to_absolute_string(PathBuf::from("foo/bar")), "foo/bar");
    assert_eq!(to_absolute_string(PathBuf::from("../../../does/not/exist")), "../../../does/not/exist");
    let path = "/root/dev/command/xylem-cli/Cargo.toml";
    match PathBuf::from(path).try_exists() {
        | Ok(true) => {
            assert_eq!(PathBuf::from("../xylem-cli/Cargo.toml").to_absolute_path(), path);
        }
        | Ok(false) | Err(_) => (),
    }
}
#[test]
fn test_file_name_with_parent() {
    let root = temp_dir().join(format!("acorn-test-path-{}", generate_guid()));
    // Folder
    let directory = root.join("folder");
    let created = std::fs::create_dir_all(&directory);
    assert!(created.is_ok());
    assert_eq!(file_name_with_parent(directory.clone()), "folder".to_string());
    // File
    let file = directory.join("example.txt");
    let written = std::fs::write(&file, "content");
    assert!(written.is_ok());
    assert_eq!(file_name_with_parent(file), "folder/example.txt".to_string());
    let removed = std::fs::remove_dir_all(&root);
    assert!(removed.is_ok());
}
#[test]
fn test_is_uri_or_path() {
    assert!(is_uri_or_path("https://example.org/style.zip"));
    assert!(is_uri_or_path("http://example.org/style.zip"));
    assert!(is_uri_or_path("file:///tmp/style"));
    assert!(is_uri_or_path("/tmp/style"));
    assert!(is_uri_or_path("./local/style"));
    assert!(is_uri_or_path("../parent/style"));
    assert!(!is_uri_or_path("Google"));
    assert!(!is_uri_or_path("proselint"));
    assert!(!is_uri_or_path("write-good"));
}
proptest! {
    #[test]
    fn prop_base32_round_trip(value in any::<u128>()) {
        let encoded = base32_crockford_encode(value);
        prop_assert_eq!(base32_crockford_decode(&encoded), Some(value));
    }
    #[test]
    fn prop_base32_decode_is_separator_and_case_insensitive(value in any::<u128>()) {
        let encoded = base32_crockford_encode(value).to_ascii_uppercase();
        let decorated = format!("  {encoded}-_  ");
        prop_assert_eq!(base32_crockford_decode(decorated), Some(value));
    }
    #[test]
    fn prop_file_extension_extracts_suffix_for_simple_names(
        stem in "[a-zA-Z0-9_]{1,32}",
        ext in "[a-zA-Z0-9]{1,10}"
    ) {
        let name = format!("{stem}.{ext}");
        prop_assert_eq!(file_extension(name), Some(ext));
    }
    #[test]
    fn prop_file_extension_returns_none_without_dot(name in "[a-zA-Z0-9_/-]{1,64}") {
        prop_assume!(!name.contains('.'));
        prop_assert_eq!(file_extension(name), None);
    }
    #[test]
    fn prop_suffix_only_singular_is_empty(value in any::<u16>()) {
        let expected = if value == 1 { "" } else { "s" };
        prop_assert_eq!(suffix(value), expected.to_string());
    }
    #[test]
    fn prop_regex_join_none_iff_all_patterns_empty(patterns in prop::collection::vec(".*", 0..20)) {
        let joined = regex_join(&patterns);
        let has_any_non_empty = patterns.iter().any(|x| !x.is_empty());
        prop_assert_eq!(joined.is_some(), has_any_non_empty);
    }
    #[test]
    fn prop_format_bytes_has_known_unit(value in any::<u64>()) {
        let formatted = format_bytes(value);
        let unit = formatted.split_whitespace().last();
        prop_assert!(matches!(unit, Some("B" | "KB" | "MB" | "GB" | "TB")));
    }
    #[test]
    fn prop_format_bytes_below_1024_stays_in_bytes(value in 0u64..1024) {
        prop_assert_eq!(format_bytes(value), format!("{value} B"));
    }
}
