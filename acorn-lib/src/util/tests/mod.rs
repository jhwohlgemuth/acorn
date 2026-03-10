use crate::io::api::ParamStyle;
use crate::io::{
    command_exists, extract_zip, file_name_with_parent, git_branch_name, git_default_branch_name, parent, read_file, to_absolute_string, FromCommand,
    FromPath,
};
use crate::prelude::{CommandOutput, Path, PathBuf};
use crate::util::*;
use crate::{cmd, param};
#[cfg(test)]
use similar_asserts::assert_eq;

const FIXTURES: &str = "../tests/fixtures";

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
fn test_cmd_output() {
    // Platform-specific shell tests
    #[cfg(unix)]
    {
        // Shell string
        let output = cmd!(sh "echo sh-output").expect("sh string");
        assert!(output.status.success());
        assert_eq!(output.stdout(), "sh-output");
        // Shell string with multiple args
        let output = cmd!(sh "echo one two three").expect("sh multi-arg");
        assert!(output.status.success());
        assert_eq!(output.stdout(), "one two three");
        // Shell string with format! interpolation
        let name = "world";
        let output = cmd!(sh format!("echo hello-{name}")).expect("sh format!");
        assert!(output.status.success());
        assert_eq!(output.stdout(), "hello-world");
        // Array literal
        let output = cmd!("echo", ["hello"]).expect("array literal");
        assert!(output.status.success());
        assert_eq!(output.stdout(), "hello");
        // Args variable
        let args = vec!["world"];
        let output = cmd!("echo", args).expect("args variable");
        assert!(output.status.success());
        assert_eq!(output.stdout(), "world");
        // CLI-style literals
        let output = cmd!("echo" "cli-style").expect("cli-style");
        assert!(output.status.success());
        assert_eq!(output.stdout(), "cli-style");
        // Stderr capture
        let output = cmd!("sh", ["-c", "echo oops >&2"]).expect("stderr");
        assert_eq!(output.stderr(), "oops");
        // Non-zero exit code
        let output = cmd!("sh", ["-c", "exit 42"]).expect("exit code");
        assert!(!output.status.success());
    }
    #[cfg(windows)]
    {
        // Windows shell string using cmd /c
        let output = cmd!(sh "cmd /c echo sh-output").expect("sh string");
        assert!(output.status.success());
        assert_eq!(output.stdout(), "sh-output");
        // Shell string with multiple args
        let output = cmd!(sh "cmd /c echo one two three").expect("sh multi-arg");
        assert!(output.status.success());
        assert_eq!(output.stdout(), "one two three");
        // Shell string with format! interpolation
        let name = "world";
        let output = cmd!(sh format!("cmd /c echo hello-{name}")).expect("sh format!");
        assert!(output.status.success());
        assert_eq!(output.stdout(), "hello-world");
        // Array literal - using 'cargo' which exists as executable
        let output = cmd!("cargo", ["--version"]).expect("array literal");
        assert!(output.status.success());
        assert!(output.stdout().contains("cargo"));
        // Stderr capture using cmd /c
        let output = cmd!("cmd", ["/c", "echo oops 1>&2"]).expect("stderr");
        assert!(output.stderr().contains("oops") || output.stdout().contains("oops"));
    }
}
#[test]
fn test_cmd_status() {
    // Platform-specific tests
    #[cfg(unix)]
    {
        // Shell string
        let status = cmd!(sh status "echo sh-status").expect("sh string");
        assert!(status.success());
        // Shell string with format! interpolation
        let arg = "check";
        let status = cmd!(sh status format!("echo {arg}")).expect("sh format!");
        assert!(status.success());
        // Array literal
        let status = cmd!(status "echo", ["hello"]).expect("array literal");
        assert!(status.success());
        // Args variable
        let args = vec!["hello"];
        let status = cmd!(status "echo", args).expect("args variable");
        assert!(status.success());
        // CLI-style literals
        let status = cmd!(status "echo" "status-check").expect("cli-style");
        assert!(status.success());
        // Non-zero exit code
        let status = cmd!(status "sh", ["-c", "exit 42"]).expect("exit code");
        assert!(!status.success());
    }
    #[cfg(windows)]
    {
        // Windows shell string using cmd /c
        let status = cmd!(sh status "cmd /c echo sh-status").expect("sh string");
        assert!(status.success());
        // Shell string with format! interpolation
        let arg = "check";
        let status = cmd!(sh status format!("cmd /c echo {arg}")).expect("sh format!");
        assert!(status.success());
        // Array literal - using 'cargo' which exists as executable
        let status = cmd!(status "cargo", ["--version"]).expect("array literal");
        assert!(status.success());
    }
}
#[test]
fn test_cmd_error() {
    // Nonexistent binary — array style
    assert!(cmd!("not-a-real-command-xyz", ["arg"]).is_err());
    // Nonexistent binary — sh style
    assert!(cmd!(sh "not-a-real-command-xyz arg").is_err());
}
#[test]
fn test_current_date() {
    let date = current_date();
    assert_eq!(date.len(), 10);
    assert_eq!(date.split("-").count(), 3);
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
fn test_license() {
    assert_eq!(License::from("MIT".to_string()), License::Mit);
    assert_eq!(License::from("MIT"), License::Mit);
    assert!(License::from("MIT").is_open_source());
    assert!(License::from("BSD-3-Clause").is_open_source());
    assert!(!License::from("Proprietary").is_open_source());
    assert!(!License::from("Not-a-real-license").is_open_source());
    assert_eq!(License::from_technology("apptainer"), Some(License::Bsd3Clause));
    assert!(License::from_technology("not-a-real-technology").is_none());
}
#[test]
fn test_mimetype() {
    let mime = MimeType::from("a.cff");
    assert_eq!(mime, MimeType::Yaml);
    let mime = MimeType::from("a.txt");
    assert_eq!(mime, MimeType::Text);
    let mime = MimeType::from("font.otf".to_string());
    assert_eq!(mime, MimeType::Otf);
    let mime = MimeType::from_path(Path::new("a.json"));
    assert_eq!(mime, MimeType::Json);
    let mime = MimeType::from_path(PathBuf::from("file.yaml"));
    assert_eq!(mime, MimeType::Yaml);
    assert_eq!(MimeType::Json.file_type(), "json");
    assert_eq!(MimeType::Yaml.file_type(), "yaml");
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
    assert!(path.to_absolute_string().ends_with("test.txt"));
    assert!(to_absolute_string(path).ends_with("test.txt"));
    assert_eq!(to_absolute_string(PathBuf::from("foo/bar")), "foo/bar");
    assert_eq!(to_absolute_string(PathBuf::from("../../../does/not/exist")), "../../../does/not/exist");
    let path = "/root/dev/command/xylem-cli/Cargo.toml";
    match PathBuf::from(path).try_exists() {
        | Ok(true) => {
            assert_eq!(PathBuf::from("../xylem-cli/Cargo.toml").to_absolute_string(), path);
        }
        | Ok(false) => (),
        | Err(_) => (),
    }
}

#[test]
fn test_file_name_with_parent() {
    let root = std::env::temp_dir().join(format!("acorn-test-path-{}", generate_guid()));
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
