use super::*;
use crate::io::mcp::tools::capability::{BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, CapabilitySelectionContext};
use acorn_schema::agent::tools::ToolEffects;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
struct Input {
    value: String,
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
