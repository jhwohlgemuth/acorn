#![allow(clippy::panic)]

use super::{automation_engine, host_platform, tree_matches};
use rhai::{Array, Dynamic};

#[test]
fn host_platform_uses_needle_platform_names() {
    assert!(!host_platform().is_empty());
}
#[test]
fn needle_manifest_primitives_evaluate() {
    let result = automation_engine(&[]).eval::<String>(
        r#"
            let transport = array_append(["--ca-file"], ["certificate.pem"]);
            json(#{ transport: transport, reference: strip_prefix("oci://registry/acorn:latest", "oci://") })
        "#,
    );
    assert_eq!(
        result.expect("script should evaluate"),
        "{\n  \"reference\": \"registry/acorn:latest\",\n  \"transport\": [\n    \"--ca-file\",\n    \"certificate.pem\"\n  ]\n}"
    );
}
#[test]
fn repository_automation_scripts_compile() {
    let engine = automation_engine(&[]);
    [
        (
            "check-host-dependencies.rhai",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts/check-host-dependencies.rhai")),
        ),
        (
            "check-portable-dependencies.rhai",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts/check-portable-dependencies.rhai")),
        ),
        (
            "create-oci-artifact.rhai",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts/needle/create-oci-artifact.rhai")),
        ),
    ]
    .into_iter()
    .for_each(|(name, source)| {
        if let Err(why) = engine.compile(source) {
            panic!("{name}: {why}");
        }
    });
}
#[test]
fn tree_matches_returns_only_named_packages() {
    let matches = tree_matches("acorn-core v0.1.0\nacorn-lib v0.1.0", Array::from([Dynamic::from("acorn-lib")])).expect("tree matches should parse");
    assert_eq!(matches.len(), 1);
}
#[test]
fn trusted_scripts_can_use_registered_host_functions() {
    let result = automation_engine(&["value".to_string()]).eval::<String>(r#"argument(0, "missing")"#);
    assert_eq!(result.expect("script should evaluate"), "value");
}
