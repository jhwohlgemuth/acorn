use super::*;
use crate::io::mcp::tools::capability::{BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, CapabilitySelectionContext};
use crate::io::mcp::tools::needle::{model_catalog_schema, Family};
use acorn_schema::agent::tools::ToolEffects;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
struct Input {
    value: String,
}
#[derive(Debug, Deserialize, JsonSchema)]
struct NamedInput {
    description: String,
}
#[derive(Debug, JsonSchema, Serialize)]
struct OtherOutput {
    count: usize,
}
#[derive(Debug, JsonSchema, Serialize)]
struct Output {
    value: String,
}
fn binding(id: &str, kind: BindingKind, availability: BindingAvailability) -> BindingDefinition {
    BindingDefinition::new::<Input, Output>(id, "acorn.sample", version(), kind, effects(), availability)
}
fn effects() -> ToolEffects {
    ToolEffects {
        read_only: true,
        destructive: false,
        idempotent: true,
        open_world: false,
    }
}
fn register(registry: ToolRegistry, definition: BindingDefinition) -> ToolRegistry {
    registry
        .register_capability_binding::<Input, Output, _, _>(definition, |input| async move { Ok(Output { value: input.value }) })
        .expect("binding registration")
}
fn registry() -> ToolRegistry {
    ToolRegistry::default()
        .register_capability(CapabilityDefinition::new::<Input, Output>(
            "acorn.sample",
            version(),
            "A bounded sample operation",
            effects(),
        ))
        .expect("capability registration")
}

#[test]
fn test_catalog_is_deterministic_and_requires_explicit_narrowing_when_oversized() {
    let short = CapabilityDefinition::new::<Input, Output>("acorn.short", version(), "Short", effects());
    let long = CapabilityDefinition::new::<Input, Output>("acorn.long", version(), "x".repeat(40_000), effects());
    let registry = ToolRegistry::default()
        .register_capability(short)
        .expect("short")
        .register_capability(long)
        .expect("long")
        .register_capability_binding::<Input, Output, _, _>(
            BindingDefinition::new::<Input, Output>(
                "native:short",
                "acorn.short",
                version(),
                BindingKind::Native,
                effects(),
                BindingAvailability::Ready,
            ),
            |input| async move { Ok(Output { value: input.value }) },
        )
        .expect("short binding")
        .register_capability_binding::<Input, Output, _, _>(
            BindingDefinition::new::<Input, Output>(
                "native:long",
                "acorn.long",
                version(),
                BindingKind::Native,
                effects(),
                BindingAvailability::Ready,
            ),
            |input| async move { Ok(Output { value: input.value }) },
        )
        .expect("long binding");
    let context = CapabilitySelectionContext::default();
    let error = registry.capability_catalog(None, context).expect_err("oversized catalog must fail");
    assert!(error.to_string().contains("narrow"));
    let scoped = registry
        .capability_catalog(Some(&["acorn.short".to_string()]), context)
        .expect("scoped catalog");
    assert!(scoped.contains("acorn.short"));
    assert!(!scoped.contains("acorn.long"));
    assert!(registry.capability_catalog(Some(&["acorn.missing".to_string()]), context).is_err());
    assert!(registry.capability_catalog(Some(&[]), context).is_err());
}
#[test]
fn test_catalog_preserves_native_union_argument_shapes() {
    let root = tempfile::tempdir().expect("root");
    let registry = ToolRegistry::default()
        .register_diff_capability(root.path())
        .and_then(|registry| registry.register_gather_capabilities(root.path(), true))
        .expect("native registry");
    let context = CapabilitySelectionContext {
        offline: true,
        allow_remote: false,
    };
    let catalog: Value = serde_json::from_str(&registry.capability_catalog(None, context).expect("catalog")).expect("JSON catalog");
    for (capability, valid, invalid) in [
        (
            "acorn.diff",
            serde_json::json!({
                "before":{"kind":"text","value":"before"},
                "after":{"kind":"file","value":"after.json"},
                "operation":{"kind":"similarity","method":"jaccard"}
            }),
            serde_json::json!({"before":"before","after":"after","operation":"similarity"}),
        ),
        (
            "acorn.gather",
            serde_json::json!({"input":{"kind":"directory","value":"data"}}),
            serde_json::json!({"input":"data"}),
        ),
    ] {
        let parameters = &catalog
            .as_array()
            .expect("tools")
            .iter()
            .find(|tool| tool["name"] == capability)
            .expect("tool")["parameters"];
        let advertised = jsonschema::validator_for(parameters).expect("advertised schema");
        let selected = registry.select_capability_binding(capability, None, None, context).expect("binding");
        let canonical = jsonschema::validator_for(&selected.capability.input_schema).expect("canonical schema");
        assert!(advertised.is_valid(&valid), "{capability}: {valid}");
        assert!(canonical.is_valid(&valid), "{capability}: {valid}");
        assert!(!advertised.is_valid(&invalid), "{capability}: {invalid}");
        assert!(!canonical.is_valid(&invalid), "{capability}: {invalid}");
        let mut wrong_tag = valid;
        let field = if capability == "acorn.diff" { "operation" } else { "input" };
        *wrong_tag.pointer_mut(&format!("/{field}/kind")).expect("discriminator") = serde_json::json!("invalid");
        assert!(!advertised.is_valid(&wrong_tag), "{capability}: discriminator");
    }
}
#[test]
fn test_catalog_projection_preserves_references_and_discriminators() {
    let schema = serde_json::json!({
        "type": "object",
        "$defs": {"Nested": {
            "description": "type prose",
            "anyOf": [
                {"type":"null"},
                {"type":"object","properties":{
                    "description":{"type":"string","description":"field prose"},
                    "kind":{"const":"nested"},
                    "value":{"oneOf":[{"type":"integer","minimum":1},{"const":{"title":"literal","description":"data"}}]}
                },"required":["kind","value"],"additionalProperties":false}
            ]
        }},
        "properties": { "nested": { "$ref": "#/$defs/Nested" } },
        "required": ["nested"]
    });
    let projected = model_catalog_schema(&schema);
    let mut expected = schema.clone();
    expected
        .pointer_mut("/$defs/Nested")
        .and_then(Value::as_object_mut)
        .expect("definition")
        .remove("description");
    expected
        .pointer_mut("/$defs/Nested/anyOf/1/properties/description")
        .and_then(Value::as_object_mut)
        .expect("property")
        .remove("description");
    assert_eq!(projected, expected);
    assert!(jsonschema::validator_for(&projected)
        .expect("projected schema")
        .is_valid(&serde_json::json!({
            "nested":{"kind":"nested","value":{"title":"literal","description":"data"}}
        })));
}
#[test]
fn test_catalog_projection_preserves_named_properties() {
    let registry = ToolRegistry::default()
        .register_capability(CapabilityDefinition::new::<NamedInput, Output>(
            "acorn.named",
            version(),
            "Named input",
            effects(),
        ))
        .expect("capability")
        .register_capability_binding::<NamedInput, Output, _, _>(
            BindingDefinition::new::<NamedInput, Output>(
                "native:named",
                "acorn.named",
                version(),
                BindingKind::Native,
                effects(),
                BindingAvailability::Ready,
            ),
            |input| async move { Ok(Output { value: input.description }) },
        )
        .expect("binding");
    let catalog = registry.capability_catalog(None, CapabilitySelectionContext::default()).expect("catalog");
    let value: serde_json::Value = serde_json::from_str(&catalog).expect("JSON catalog");
    assert_eq!(
        value
            .get(0)
            .and_then(|entry| entry.pointer("/parameters/properties/description/type"))
            .and_then(serde_json::Value::as_str),
        Some("string")
    );
}
#[test]
fn test_compatible_implementations_select_deterministically() {
    let registry = register(
        register(registry(), binding("local:sample", BindingKind::Local, BindingAvailability::Ready)),
        binding("native:sample", BindingKind::Native, BindingAvailability::Ready),
    );
    let context = CapabilitySelectionContext::default();
    assert_eq!(
        registry
            .select_capability_binding("acorn.sample", None, None, context)
            .expect("native default")
            .binding
            .id,
        "native:sample"
    );
    assert_eq!(
        registry
            .select_capability_binding("acorn.sample", None, Some("local:sample"), context)
            .expect("configured preference")
            .binding
            .id,
        "local:sample"
    );
    assert_eq!(
        registry
            .select_capability_binding("acorn.sample", Some("local:sample"), None, context)
            .expect("explicit selection")
            .binding
            .id,
        "local:sample"
    );
}
#[test]
fn test_duplicate_and_mismatched_contracts_are_rejected() {
    assert!(registry()
        .register_capability(CapabilityDefinition::new::<Input, Output>(
            "acorn.sample",
            version(),
            "duplicate",
            effects()
        ))
        .is_err());
    let registered = register(registry(), binding("native:sample", BindingKind::Native, BindingAvailability::Ready));
    assert!(registered
        .clone()
        .register_capability_binding::<Input, Output, _, _>(
            binding("native:sample", BindingKind::Local, BindingAvailability::Ready),
            |input| async move { Ok(Output { value: input.value }) }
        )
        .is_err());
    let wrong_schema = BindingDefinition::new::<Input, OtherOutput>(
        "native:other",
        "acorn.sample",
        version(),
        BindingKind::Native,
        effects(),
        BindingAvailability::Ready,
    );
    assert!(registered
        .clone()
        .register_capability_binding::<Input, OtherOutput, _, _>(wrong_schema, |input| async move { Ok(OtherOutput { count: input.value.len() }) })
        .is_err());
    let mut wrong_effects = binding("native:effect", BindingKind::Native, BindingAvailability::Ready);
    wrong_effects.effects.open_world = true;
    assert!(registered
        .clone()
        .register_capability_binding::<Input, Output, _, _>(wrong_effects, |input| async move { Ok(Output { value: input.value }) })
        .is_err());
    let mut wrong_version = binding("native:version", BindingKind::Native, BindingAvailability::Ready);
    wrong_version.version = SemanticVersion::from((2, 0, 0));
    assert!(registered
        .register_capability_binding::<Input, Output, _, _>(wrong_version, |input| async move { Ok(Output { value: input.value }) })
        .is_err());
}
#[test]
fn test_equally_preferred_remote_bindings_are_ambiguous() {
    let registry = register(
        register(registry(), binding("remote:a", BindingKind::Remote, BindingAvailability::Ready)),
        binding("remote:b", BindingKind::Remote, BindingAvailability::Ready),
    );
    let context = CapabilitySelectionContext {
        offline: false,
        allow_remote: true,
    };
    assert!(registry.select_capability_binding("acorn.sample", None, None, context).is_err());
    assert_eq!(
        registry
            .select_capability_binding("acorn.sample", Some("remote:b"), None, context)
            .expect("explicit remote")
            .binding
            .id,
        "remote:b"
    );
}
#[test]
fn test_runtime_schemas_must_be_valid_json_schema() {
    let mut definition = CapabilityDefinition::new::<Input, Output>("acorn.runtime", version(), "Runtime schema", effects());
    definition.output_schema = serde_json::json!({"type": 7});
    assert!(ToolRegistry::default()
        .register_capability(definition)
        .expect_err("invalid runtime schema")
        .to_string()
        .contains("output schema"));
}
#[test]
fn test_scoped_capability_ids_keep_their_family() {
    assert_eq!(Family::from("acorn.format::apply"), Family::Format);
    assert_eq!(Family::from("provider:acorn.format::apply"), Family::Format);
}
#[test]
fn test_unavailable_and_remote_bindings_do_not_fall_back_from_explicit_choice() {
    let registry = register(
        register(registry(), binding("native:sample", BindingKind::Native, BindingAvailability::Ready)),
        binding(
            "local:sample",
            BindingKind::Local,
            BindingAvailability::Unavailable {
                reason: "model asset missing".to_string(),
            },
        ),
    );
    assert!(registry
        .select_capability_binding("acorn.sample", Some("local:sample"), None, CapabilitySelectionContext::default())
        .expect_err("missing asset")
        .to_string()
        .contains("model asset missing"));
    let registry = register(registry, binding("remote:sample", BindingKind::Remote, BindingAvailability::Ready));
    assert!(registry
        .select_capability_binding("acorn.sample", Some("remote:sample"), None, CapabilitySelectionContext::default())
        .is_err());
    assert_eq!(
        registry
            .select_capability_binding(
                "acorn.sample",
                Some("remote:sample"),
                None,
                CapabilitySelectionContext {
                    offline: false,
                    allow_remote: true
                }
            )
            .expect("permitted remote")
            .binding
            .id,
        "remote:sample"
    );
    assert!(registry
        .select_capability_binding(
            "acorn.sample",
            Some("remote:sample"),
            None,
            CapabilitySelectionContext {
                offline: true,
                allow_remote: true
            }
        )
        .is_err());
}
