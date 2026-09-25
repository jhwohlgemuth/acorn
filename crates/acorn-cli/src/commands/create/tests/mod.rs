use super::mcp::run;
use super::secret::deliver;
use crate::cli::arguments;
use crate::test::util::{temp_test_dir, TestCleanup};
use acorn::io::Platform;
use acorn_schema::modelkit::kitfile::Kitfile;
use core::cell::RefCell;
use std::fs::{read_to_string, write};

#[test]
fn test_create_mcp_dry_run_validates_without_writing() {
    let root = temp_test_dir("create-mcp-dry-run");
    let _cleanup = TestCleanup::new(root.clone());
    let binary = root.join("acorn");
    let output = root.join("acorn.mcpb");
    write(&binary, b"test executable").expect("write executable");
    let arguments = arguments::CreateMcp {
        output: output.clone(),
        binary: Some(binary),
        platform: Some(Platform::Darwin),
        kitfile: None,
        clobber: false,
        dry_run: true,
    };
    run(&arguments).expect("dry-run bundle plan");
    assert!(!output.exists());
}
#[test]
fn test_create_mcp_upserts_kitfile_idempotently() {
    let root = temp_test_dir("create-mcp-kitfile");
    let _cleanup = TestCleanup::new(root.clone());
    let binary = root.join("acorn");
    let output = root.join("bundles/acorn.mcpb");
    let kitfile = root.join("Kitfile.yaml");
    std::fs::create_dir_all(output.parent().expect("bundle parent")).expect("create bundle parent");
    write(&binary, b"test executable").expect("write executable");
    write(&kitfile, "manifestVersion: v1.0.0\nmodel:\n  path: model.gguf\n").expect("write Kitfile");
    let arguments = arguments::CreateMcp {
        output: output.clone(),
        binary: Some(binary),
        platform: Some(Platform::Darwin),
        kitfile: Some(kitfile.clone()),
        clobber: true,
        dry_run: false,
    };
    run(&arguments).expect("create bundle");
    let first = read_to_string(&kitfile).expect("first Kitfile");
    run(&arguments).expect("repeat bundle");
    let second = read_to_string(&kitfile).expect("second Kitfile");
    assert_eq!(first, second);
    let parsed = Kitfile::from_yaml(&second).expect("valid authoring Kitfile");
    assert_eq!(parsed.mcp_servers.len(), 1);
    let server = parsed.mcp_servers.first().expect("ACORN MCP server");
    assert_eq!(server.path, "bundles/acorn.mcpb");
    assert!(server.layer.digest.is_none());
}
#[test]
fn test_deliver_combines_clipboard_and_piped_stdout() {
    let copied = RefCell::new(String::new());
    let written = RefCell::new(String::new());
    deliver(
        "generated-secret",
        true,
        true,
        |value| {
            copied.replace(value.to_string());
            Ok(())
        },
        |value| {
            written.replace(value.to_string());
        },
    )
    .expect("combined delivery");
    assert_eq!(copied.into_inner(), "generated-secret");
    assert_eq!(written.into_inner(), "generated-secret");
}
#[test]
fn test_deliver_copies_without_printing_the_secret() {
    let copied = RefCell::new(String::new());
    let written = RefCell::new(String::new());
    let result = deliver(
        "generated-secret",
        true,
        false,
        |value| {
            copied.replace(value.to_string());
            Ok(())
        },
        |value| {
            written.replace(value.to_string());
        },
    );
    assert!(result.is_ok());
    assert_eq!(copied.into_inner(), "generated-secret");
    assert!(written.into_inner().is_empty());
}
#[test]
fn test_deliver_writes_only_when_stdout_is_piped() {
    let silent = RefCell::new(String::new());
    deliver(
        "generated-secret",
        false,
        false,
        |_| Ok(()),
        |value| {
            silent.replace(value.to_string());
        },
    )
    .expect("silent delivery");
    assert!(silent.into_inner().is_empty());
    let written = RefCell::new(String::new());
    deliver(
        "generated-secret",
        false,
        true,
        |_| Ok(()),
        |value| {
            written.replace(value.to_string());
        },
    )
    .expect("piped delivery");
    assert_eq!(written.into_inner(), "generated-secret");
}
