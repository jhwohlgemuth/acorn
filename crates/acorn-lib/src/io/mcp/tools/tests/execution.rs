use super::*;
use acorn_schema::agent::tools::ToolEffects;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;

#[derive(Debug, Deserialize, JsonSchema)]
struct Input {
    value: String,
    destination: Option<String>,
}
#[derive(Debug, JsonSchema, Serialize)]
struct Output {
    value: String,
    artifacts: Vec<String>,
}
fn effects(read_only: bool) -> ToolEffects {
    ToolEffects {
        read_only,
        destructive: false,
        idempotent: read_only,
        open_world: false,
    }
}
fn grounded() -> ExecutionContext {
    ExecutionContext {
        grounded: true,
        ..ExecutionContext::default()
    }
}
fn registry(read_only: bool, calls: Arc<AtomicUsize>) -> (ToolRegistry, SelectedBinding) {
    let effect = effects(read_only);
    let registry = ToolRegistry::default()
        .register_capability(CapabilityDefinition::new::<Input, Output>(
            "acorn.execute",
            version(),
            "Execute a bounded operation",
            effect,
        ))
        .expect("capability")
        .register_capability_binding::<Input, Output, _, _>(
            BindingDefinition::new::<Input, Output>(
                "native:execute",
                "acorn.execute",
                version(),
                BindingKind::Native,
                effect,
                BindingAvailability::Ready,
            ),
            move |input| {
                let calls = calls.clone();
                async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok(Output {
                        value: input.value,
                        artifacts: input.destination.into_iter().collect(),
                    })
                }
            },
        )
        .expect("binding");
    let selected = registry
        .select_capability_binding("acorn.execute", None, None, CapabilitySelectionContext::default())
        .expect("selection");
    (registry, selected)
}

#[tokio::test]
async fn test_cancellation_timeout_and_output_limit_do_not_retry() {
    let calls = Arc::new(AtomicUsize::new(0));
    let (registry, selected) = registry(true, calls.clone());
    let limited = ExecutionContext {
        max_output: acorn_schema::hardware::Memory::from(1_u64),
        ..grounded()
    };
    assert_eq!(
        registry.execute_selected(&selected, json!({"value":"safe"}), limited).await.outcome,
        ExecutionOutcome::ToolFailed
    );
    let expired = ExecutionContext {
        deadline: Some(Instant::now().checked_sub(Duration::from_millis(1)).expect("past instant")),
        ..grounded()
    };
    assert_eq!(
        registry.execute_selected(&selected, json!({"value":"safe"}), expired).await.outcome,
        ExecutionOutcome::TimedOut
    );
    let cancelled = ExecutionContext {
        cancellation: Some(Box::pin(async {})),
        ..grounded()
    };
    assert_eq!(
        registry.execute_selected(&selected, json!({"value":"safe"}), cancelled).await.outcome,
        ExecutionOutcome::Cancelled
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
#[cfg(feature = "mcp-client")]
#[tokio::test]
async fn test_cooperative_handler_finishes_cleanup_before_cancellation_returns() {
    let cleaned = Arc::new(AtomicUsize::new(0));
    let cleaned_in_handler = cleaned.clone();
    let effect = effects(true);
    let capability = CapabilityDefinition::new::<Input, Output>("acorn.cooperative", version(), "Cooperative check", effect);
    let binding = BindingDefinition::new::<Input, Output>(
        "remote:cooperative",
        "acorn.cooperative",
        version(),
        BindingKind::Remote,
        effect,
        BindingAvailability::Ready,
    );
    let handler = CapabilityHandler::cancellable(move |_arguments, cancellation| {
        let cleaned = cleaned_in_handler.clone();
        Box::pin(async move {
            cancellation.await;
            cleaned.fetch_add(1, Ordering::SeqCst);
            Err(color_eyre::eyre::eyre!("cancelled after cleanup"))
        })
    });
    let registry = ToolRegistry::default()
        .register_capability(capability)
        .expect("capability")
        .register_dynamic_binding(binding, handler)
        .expect("binding");
    let selected = registry
        .select_capability_binding(
            "acorn.cooperative",
            None,
            None,
            CapabilitySelectionContext {
                offline: false,
                allow_remote: true,
            },
        )
        .expect("selection");
    let context = ExecutionContext {
        cancellation: Some(Box::pin(tokio::time::sleep(Duration::from_millis(5)))),
        selection: CapabilitySelectionContext {
            offline: false,
            allow_remote: true,
        },
        ..grounded()
    };
    let result = registry.execute_selected(&selected, json!({"value":"safe"}), context).await;
    assert_eq!(result.outcome, ExecutionOutcome::Cancelled);
    assert_eq!(cleaned.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn test_denial_and_invalid_input_never_call_handler() {
    let calls = Arc::new(AtomicUsize::new(0));
    let (registry, selected) = registry(false, calls.clone());
    let arguments = json!({"value":"safe","destination":"out.json"});
    let denied = registry.execute_selected(&selected, arguments.clone(), grounded()).await;
    assert_eq!(denied.outcome, ExecutionOutcome::PermissionRequired);
    let wrong = ExecutionContext {
        mutation: Some(MutationAuthority {
            capability_id: "acorn.execute".to_string(),
            binding_id: "native:execute".to_string(),
            arguments: json!({"value":"different","destination":"out.json"}),
            destination: "out.json".to_string(),
        }),
        ..grounded()
    };
    assert_eq!(
        registry.execute_selected(&selected, arguments.clone(), wrong).await.outcome,
        ExecutionOutcome::PermissionRequired
    );
    let invalid = registry.execute_selected(&selected, json!({"value":42}), grounded()).await;
    assert_eq!(invalid.outcome, ExecutionOutcome::InvalidInput);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn test_exact_mutation_grant_runs_once_and_returns_bounded_artifact() {
    let calls = Arc::new(AtomicUsize::new(0));
    let (registry, selected) = registry(false, calls.clone());
    let arguments = json!({"value":"safe","destination":"out.json"});
    let context = ExecutionContext {
        mutation: Some(MutationAuthority {
            capability_id: "acorn.execute".to_string(),
            binding_id: "native:execute".to_string(),
            arguments: arguments.clone(),
            destination: "out.json".to_string(),
        }),
        ..grounded()
    };
    let result = registry.execute_selected(&selected, arguments, context).await;
    assert_eq!(result.outcome, ExecutionOutcome::Completed);
    assert_eq!(result.artifacts, vec![ArtifactReference { uri: "out.json".to_string() }]);
    assert_eq!(result.provenance.binding_id, "native:execute");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn test_handler_failure_does_not_leak_its_error_into_provenance() {
    let registry = ToolRegistry::default()
        .register_capability(CapabilityDefinition::new::<Input, Output>(
            "acorn.secret",
            version(),
            "A check",
            effects(true),
        ))
        .expect("capability")
        .register_capability_binding::<Input, Output, _, _>(
            BindingDefinition::new::<Input, Output>(
                "native:secret",
                "acorn.secret",
                version(),
                BindingKind::Native,
                effects(true),
                BindingAvailability::Ready,
            ),
            |_input| async { Err(color_eyre::eyre::eyre!("password=do-not-report")) },
        )
        .expect("binding");
    let selected = registry
        .select_capability_binding("acorn.secret", None, None, CapabilitySelectionContext::default())
        .expect("selected");
    let result = registry.execute_selected(&selected, json!({"value":"safe"}), grounded()).await;
    assert_eq!(result.outcome, ExecutionOutcome::ToolFailed);
    assert!(!serde_json::to_string(&result).expect("serialize").contains("do-not-report"));
}
#[tokio::test]
async fn test_in_flight_cancellation_drops_the_handler_future() {
    struct Cleanup(Arc<AtomicUsize>);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let started = Arc::new(AtomicUsize::new(0));
    let cleaned = Arc::new(AtomicUsize::new(0));
    let started_in_handler = started.clone();
    let cleaned_in_handler = cleaned.clone();
    let registry = ToolRegistry::default()
        .register_capability(CapabilityDefinition::new::<Input, Output>(
            "acorn.cancel",
            version(),
            "Cancellable check",
            effects(true),
        ))
        .expect("capability")
        .register_capability_binding::<Input, Output, _, _>(
            BindingDefinition::new::<Input, Output>(
                "native:cancel",
                "acorn.cancel",
                version(),
                BindingKind::Native,
                effects(true),
                BindingAvailability::Ready,
            ),
            move |_input| {
                let started = started_in_handler.clone();
                let cleaned = cleaned_in_handler.clone();
                async move {
                    let _cleanup = Cleanup(cleaned);
                    started.fetch_add(1, Ordering::SeqCst);
                    core::future::pending::<crate::error::ApiResult<Output>>().await
                }
            },
        )
        .expect("binding");
    let selected = registry
        .select_capability_binding("acorn.cancel", None, None, CapabilitySelectionContext::default())
        .expect("selected");
    let context = ExecutionContext {
        cancellation: Some(Box::pin(tokio::time::sleep(Duration::from_millis(5)))),
        ..grounded()
    };
    let result = registry.execute_selected(&selected, json!({"value":"safe"}), context).await;
    assert_eq!(result.outcome, ExecutionOutcome::Cancelled);
    assert!(result.effects_may_have_occurred);
    assert_eq!(started.load(Ordering::SeqCst), 1);
    assert_eq!(cleaned.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn test_stale_binding_and_ungrounded_input_never_call_handler() {
    let calls = Arc::new(AtomicUsize::new(0));
    let (registry, selected) = registry(true, calls.clone());
    let arguments = json!({"value":"safe"});
    assert_eq!(
        registry
            .execute_selected(&selected, arguments.clone(), ExecutionContext::default())
            .await
            .outcome,
        ExecutionOutcome::Withheld
    );
    let mut stale = selected.clone();
    stale.binding.version = SemanticVersion::from((2, 0, 0));
    assert_eq!(
        registry.execute_selected(&stale, arguments, grounded()).await.outcome,
        ExecutionOutcome::Unavailable
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}
