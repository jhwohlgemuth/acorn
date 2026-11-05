#[cfg(feature = "exec")]
use super::*;
#[cfg(feature = "exec")]
use acorn::io::mcp::{
    AnalysisInput, AnalysisOutput, BindingAvailability, BindingDefinition, BindingKind, CapabilityError, CapabilitySelectionContext,
    DiffOperationInput, DiffOperationOutput, DoctorInput, DoctorOutput, ExecutionContext, GatherInput, GatherOutput, SwhidInput, SwhidOutput,
    ToolRegistry,
};
#[cfg(feature = "exec")]
use acorn::io::sidecar::needle::Needle;
#[cfg(feature = "exec")]
use acorn_core::util::constants::app::APPLICATION;
#[cfg(feature = "exec")]
use acorn_core::Location;

#[cfg(feature = "exec")]
fn corpus_binding_id(id: &str) -> String {
    match id.strip_prefix("native:") {
        | Some(alias) if matches!(alias, "check" | "diff" | "doctor" | "gather" | "swhid") => format!("native:{APPLICATION}.{alias}"),
        | _ => id.to_string(),
    }
}
#[cfg(feature = "exec")]
fn replay_binding_fixture(
    case: &serde_json::Value,
    capability_id: &str,
    native: &ToolRegistry,
    selection: CapabilitySelectionContext,
    binding_override: Option<&str>,
    binding_preference: Option<&str>,
) -> serde_json::Value {
    let Some(candidates) = case
        .get("route_context")
        .and_then(|context| context.get("candidate_bindings"))
        .and_then(serde_json::Value::as_array)
    else {
        return serde_json::Value::Null;
    };
    if candidates.is_empty() {
        return serde_json::json!({"status":"no-declared-bindings"});
    }
    if candidates.iter().any(|candidate| {
        candidate
            .as_str()
            .is_some_and(|id| id.ends_with("-incompatible") || id.ends_with("-v1") || id.ends_with("-v2"))
    }) {
        return serde_json::json!({"status":"missing-contract-evidence"});
    }
    let Ok(selected) = native.select_capability_binding(capability_id, None, None, CapabilitySelectionContext::default()) else {
        return serde_json::json!({"status":"no-canonical-contract"});
    };
    let capability = selected.capability;
    let mut registry = ToolRegistry::default()
        .register_capability(capability.clone())
        .expect("fixture capability");
    for candidate in candidates {
        let id = candidate.as_str().expect("candidate binding ID");
        let normalized = corpus_binding_id(id);
        let kind = if id.starts_with("native:") {
            BindingKind::Native
        } else if id.starts_with("local:") {
            BindingKind::Local
        } else {
            BindingKind::Remote
        };
        macro_rules! register {
            ($input:ty, $output:ty) => {{
                let definition = BindingDefinition::new::<$input, $output>(
                    normalized.clone(),
                    capability.id.clone(),
                    capability.version,
                    kind,
                    capability.effects.clone(),
                    BindingAvailability::Ready,
                );
                registry.register_capability_binding::<$input, $output, _, _>(definition, |_| async {
                    Err(color_eyre::eyre::eyre!("fixture handler must never execute"))
                })
            }};
        }
        let registered = match capability_id {
            | "acorn.check" => register!(AnalysisInput, AnalysisOutput),
            | "acorn.diff" => register!(DiffOperationInput, DiffOperationOutput),
            | "acorn.doctor" => register!(DoctorInput, DoctorOutput),
            | "acorn.gather" => register!(GatherInput, GatherOutput),
            | "acorn.swhid" => register!(SwhidInput, SwhidOutput),
            | _ => return serde_json::json!({"status":"unsupported-capability"}),
        };
        match registered {
            | Ok(updated) => registry = updated,
            | Err(why) => return serde_json::json!({"status":"invalid-contract","binding":normalized,"error":why.to_string()}),
        }
    }
    match registry.select_capability_binding(capability_id, binding_override, binding_preference, selection) {
        | Ok(selected) => serde_json::json!({"status":"selected","binding":selected.binding.id}),
        | Err(why) if matches!(why.downcast_ref::<CapabilityError>(), Some(CapabilityError::AmbiguousBindings(_))) => {
            serde_json::json!({"status":"needs-input"})
        }
        | Err(why) => serde_json::json!({"status":"unavailable","error":why.to_string()}),
    }
}

#[test]
#[cfg(feature = "exec")]
fn test_check_model_catalog_is_compact() {
    let registry = native_routing_registry(true).expect("native registry");
    let catalog = registry
        .capability_catalog(
            Some(&["acorn.check".to_string()]),
            CapabilitySelectionContext {
                offline: true,
                allow_remote: false,
            },
        )
        .expect("check catalog");
    let tools: serde_json::Value = serde_json::from_str(&catalog).expect("catalog JSON");
    assert!(catalog.len() < 2 * 1024, "check catalog is {} bytes", catalog.len());
    assert_eq!(tools.pointer("/0/parameters/required/0"), Some(&serde_json::json!("input")));
    let variants = tools
        .pointer("/0/parameters/properties/input/properties/kind/enum")
        .and_then(serde_json::Value::as_array)
        .expect("input variants");
    for kind in ["directory", "file", "url"] {
        assert!(variants.iter().any(|variant| variant.as_str() == Some(kind)));
    }
}
#[test]
#[cfg(feature = "exec")]
fn test_calibration_shortlist_retains_supported_capabilities_within_needle_budget() {
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../../tests/fixtures/routing/cases.json")).expect("frozen routing corpus");
    assert_eq!(corpus.get("version").and_then(serde_json::Value::as_u64), Some(3));
    let cases = corpus.get("cases").and_then(serde_json::Value::as_array).expect("routing cases");
    let scores = cases
        .iter()
        .filter(|case| case.get("split").and_then(serde_json::Value::as_str) == Some("calibration"))
        .filter_map(|case| {
            case.get("expected")
                .and_then(|expected| expected.get("capability_id"))
                .and_then(serde_json::Value::as_str)
                .zip(case.get("prompt").and_then(serde_json::Value::as_str))
                .map(|(capability, prompt)| (case, capability, prompt))
        })
        .filter_map(|(case, capability, prompt)| {
            let offline = case
                .get("route_context")
                .and_then(|context| context.get("offline"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_else(|| prompt.to_ascii_lowercase().contains("offline"));
            let registry = native_routing_registry(offline).expect("native routing registry");
            let selection = CapabilitySelectionContext {
                offline,
                allow_remote: false,
            };
            let scope = registry.scope_for_prompt(prompt, selection).expect("bounded shortlist");
            assert!(scope.bytes.checked_bytes().is_some_and(|bytes| bytes <= 6 * 1024));
            let retained = scope.ids.as_ref().is_none_or(|ids| ids.iter().any(|id| id == capability));
            registry
                .capability_catalog(Some(&[capability.to_string()]), selection)
                .is_ok()
                .then_some((case.get("id").and_then(serde_json::Value::as_str).unwrap_or("unknown"), retained, scope))
        })
        .collect::<Vec<_>>();
    let misses = scores
        .iter()
        .filter(|(_, retained, _)| !retained)
        .map(|(id, _, scope)| format!("{id}:{}", scope.ids.as_ref().and_then(|ids| ids.first()).map_or("all", String::as_str)))
        .collect::<Vec<_>>();
    let retained = scores.len().saturating_sub(misses.len());
    assert!(
        retained.saturating_mul(100) >= scores.len().saturating_mul(95),
        "calibration shortlist missed {}",
        misses.join(", ")
    );
}
#[test]
#[cfg(feature = "exec")]
fn test_corpus_binding_aliases_resolve_without_changing_other_bindings() {
    let registry = native_routing_registry(true).expect("native registry");
    let context = CapabilitySelectionContext {
        offline: true,
        allow_remote: false,
    };
    for alias in ["check", "doctor", "gather", "swhid"] {
        let capability = format!("{APPLICATION}.{alias}");
        let binding = corpus_binding_id(&format!("native:{alias}"));
        assert!(registry.select_capability_binding(&capability, Some(&binding), None, context).is_ok());
        assert_eq!(corpus_binding_id(&binding), binding);
        let preferred = registry
            .select_capability_binding(&capability, None, Some(&binding), context)
            .expect("preference");
        assert_eq!(preferred.binding.id, binding);
    }
    for id in ["local:diff", "native:missing", "remote:check-a"] {
        assert_eq!(corpus_binding_id(id), id);
        assert!(registry.select_capability_binding("acorn.check", Some(id), None, context).is_err());
    }
}
#[test]
#[cfg(feature = "exec")]
fn test_calibration_alternative_binding_policy_replay() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!("../../../../../../tests/fixtures/routing/cases.json")).expect("frozen corpus");
    let registry = native_routing_registry(false).expect("native registry");
    let cases = corpus.get("cases").and_then(serde_json::Value::as_array).expect("cases");
    for (id, status, binding) in [
        ("BOUND-031", "selected", Some("native:acorn.check")),
        ("BOUND-032", "selected", Some("local:diff")),
        ("BOUND-033", "selected", Some("native:acorn.gather")),
        ("BOUND-034", "selected", Some("native:acorn.doctor")),
        ("BOUND-035", "selected", Some("local:swhid")),
        ("BOUND-036", "needs-input", None),
        ("BOUND-037", "unavailable", None),
        ("BOUND-038", "unavailable", None),
        ("BOUND-039", "missing-contract-evidence", None),
        ("BOUND-040", "missing-contract-evidence", None),
    ] {
        let case = cases
            .iter()
            .find(|case| case.get("id").and_then(serde_json::Value::as_str) == Some(id))
            .expect("fixture case");
        let capability_id = case
            .pointer("/expected/capability_id")
            .and_then(serde_json::Value::as_str)
            .expect("capability");
        let caller = case.pointer("/route_context/caller_selection");
        let selection = CapabilitySelectionContext {
            offline: false,
            allow_remote: caller
                .and_then(|value| value.get("allow_remote"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        };
        let override_id = caller
            .and_then(|value| value.get("binding_id"))
            .and_then(serde_json::Value::as_str)
            .map(corpus_binding_id);
        let preference_id = caller
            .and_then(|value| value.get("preferred_binding_id"))
            .and_then(serde_json::Value::as_str)
            .map(corpus_binding_id);
        let replay = replay_binding_fixture(
            case,
            capability_id,
            &registry,
            selection,
            override_id.as_deref(),
            preference_id.as_deref(),
        );
        assert_eq!(replay.get("status").and_then(serde_json::Value::as_str), Some(status), "{id}: {replay}");
        assert_eq!(replay.get("binding").and_then(serde_json::Value::as_str), binding, "{id}: {replay}");
    }
}
#[test]
#[cfg(feature = "exec")]
fn test_shortlist_ignores_family_keywords_in_input_paths() {
    let registry = native_routing_registry(true).expect("native registry");
    let context = CapabilitySelectionContext {
        offline: true,
        allow_remote: false,
    };
    for path in [
        "export/index.json",
        "download/index.json",
        "diff.json",
        "/export",
        "C:\\export\\index.json",
        "`gather/index.json`",
        "https://example.com/export/index.json",
    ] {
        let scope = registry
            .scope_for_prompt(&format!("Check the metadata in {path}"), context)
            .expect("shortlist");
        assert!(scope.ids.as_ref().expect("narrowed scope").iter().any(|id| id == "acorn.check"), "{path}");
    }
    let export = registry
        .scope_for_prompt("Export the metadata in check/index.json", context)
        .expect("export shortlist");
    assert!(export
        .ids
        .as_ref()
        .expect("narrowed scope")
        .iter()
        .all(|id| id.starts_with("acorn.export")));
}
#[tokio::test]
#[ignore = "runs the pinned Needle backend over the frozen routing corpus"]
#[cfg(feature = "exec")]
async fn test_measure_real_needle_routing_without_handler_execution() {
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../../tests/fixtures/routing/cases.json")).expect("frozen routing corpus");
    let cases = corpus.get("cases").and_then(serde_json::Value::as_array).expect("routing cases");
    let limit = std::env::var("ACORN_ROUTING_LIMIT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(cases.len());
    let selected_case = std::env::var("ACORN_ROUTING_CASE_ID").ok();
    let selected_split = std::env::var("ACORN_ROUTING_SPLIT").ok();
    let explicit_scope = std::env::var("ACORN_ROUTING_SCOPE")
        .ok()
        .map(|scope| scope.split(',').map(str::to_string).collect::<Vec<_>>());
    let output = std::env::var_os("ACORN_ROUTING_RESULTS")
        .map(PathBuf::from)
        .expect("routing results path");
    let mut rows = Vec::new();
    for case in cases
        .iter()
        .filter(|case| {
            selected_case
                .as_deref()
                .is_none_or(|id| case.get("id").and_then(serde_json::Value::as_str) == Some(id))
                && selected_split
                    .as_deref()
                    .is_none_or(|split| case.get("split").and_then(serde_json::Value::as_str) == Some(split))
        })
        .take(limit)
    {
        let id = case.get("id").and_then(serde_json::Value::as_str).expect("case ID");
        let prompt = case.get("prompt").and_then(serde_json::Value::as_str).expect("case prompt");
        let expected = case
            .get("expected")
            .and_then(|value| value.get("capability_id"))
            .and_then(serde_json::Value::as_str);
        let caller_arguments = case
            .get("inventory_id")
            .and_then(serde_json::Value::as_str)
            .filter(|inventory_id| inventory_id.starts_with("check."))
            .and_then(|_| case.get("input_refs"))
            .and_then(serde_json::Value::as_array)
            .and_then(|references| references.first())
            .and_then(serde_json::Value::as_str)
            .map(|input| {
                let kind = if Location::from(input).is_http_or_https() {
                    "url"
                } else if std::path::Path::new(input).is_dir() {
                    "directory"
                } else {
                    "file"
                };
                serde_json::json!({"input":{"kind":kind,"value":input}})
            });
        let offline = case
            .get("route_context")
            .and_then(|context| context.get("offline"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_else(|| prompt.to_ascii_lowercase().contains("offline"));
        let caller_selection = case.get("route_context").and_then(|context| context.get("caller_selection"));
        let candidate_bindings = case.get("route_context").and_then(|context| context.get("candidate_bindings"));
        let binding_override = caller_selection
            .and_then(|selection| selection.get("binding_id"))
            .and_then(serde_json::Value::as_str)
            .map(corpus_binding_id);
        let binding_preference = caller_selection
            .and_then(|selection| selection.get("preferred_binding_id"))
            .and_then(serde_json::Value::as_str)
            .map(corpus_binding_id);
        let allow_remote = caller_selection
            .and_then(|selection| selection.get("allow_remote"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let registry = native_routing_registry(offline).expect("native routing registry");
        let selection = CapabilitySelectionContext { offline, allow_remote };
        let binding_fixture = expected.map(|capability_id| {
            replay_binding_fixture(
                case,
                capability_id,
                &registry,
                selection,
                binding_override.as_deref(),
                binding_preference.as_deref(),
            )
        });
        let scope = registry.scope_for_prompt(prompt, selection).expect("bounded routing scope");
        let selected_ids = explicit_scope.clone().or(scope.ids.clone());
        let catalog = registry.selection_catalog(selected_ids.as_deref(), selection).expect("selected catalog");
        let catalog_bytes = catalog.len();
        if let Some(path) = std::env::var_os("ACORN_ROUTING_CATALOG") {
            std::fs::write(path, catalog).expect("write routing catalog");
        }
        let shortlist_contains_expected = expected.map(|capability| selected_ids.as_ref().is_none_or(|ids| ids.iter().any(|id| id == capability)));
        let model_prompt = caller_arguments.as_ref().map_or_else(
            || prompt.to_string(),
            |arguments| format!("{prompt}\n\nCaller-supplied structured arguments (copy exactly): {arguments}"),
        );
        let started = std::time::Instant::now();
        let routed = Needle::start_with_scoped_registry(None, offline, registry.clone(), selected_ids).await;
        let result = match routed {
            | Ok(sidecar) => {
                sidecar
                    .route_once(
                        model_prompt,
                        ExecutionContext {
                            selection,
                            preview: true,
                            binding_override: binding_override.clone(),
                            binding_preference: binding_preference.clone(),
                            caller_arguments,
                            ..ExecutionContext::default()
                        },
                    )
                    .await
            }
            | Err(why) => Err(why),
        };
        let latency_ms = started.elapsed().as_millis();
        let row = match result {
            | Ok(result) => {
                assert!(result
                    .execution
                    .as_ref()
                    .is_none_or(|execution| execution.content.is_none() && execution.artifacts.is_empty()));
                let would_execute = result.outcome == acorn::io::mcp::ExecutionOutcome::Proposed
                    && result.selected.as_ref().is_some_and(|selected| {
                        registry
                            .select_capability_binding(&selected.capability_id, Some(&selected.binding_id), None, selection)
                            .is_ok_and(|binding| binding.capability.effects.read_only)
                    });
                let selected_call = result
                    .response
                    .as_ref()
                    .and_then(|response| response.function_calls.first())
                    .map(|call| call.name.clone());
                let selected_arguments = result
                    .response
                    .as_ref()
                    .and_then(|response| response.function_calls.first())
                    .map(|call| &call.arguments);
                let response_error = result.response.as_ref().and_then(|response| response.error.clone());
                let response_error_code = result.response.as_ref().and_then(|response| response.error_code.clone());
                let response_success = result.response.as_ref().map(|response| response.success);
                let response_confidence = result.response.as_ref().and_then(|response| response.confidence);
                serde_json::json!({
                    "id": id,
                    "split": case.get("split"),
                    "class": case.get("class"),
                    "expected_capability": expected,
                    "binding_override": binding_override,
                    "binding_preference": binding_preference,
                    "candidate_bindings": candidate_bindings,
                    "binding_fixture": binding_fixture,
                    "selected_call": selected_call,
                    "selected_arguments": selected_arguments,
                    "selected_capability": result.selected.as_ref().map(|selected| &selected.capability_id),
                    "response_error": response_error,
                    "response_error_code": response_error_code,
                    "response_success": response_success,
                    "response_confidence": response_confidence,
                    "outcome": result.outcome,
                    "would_execute_without_preview": would_execute,
                    "withheld": result.withheld,
                    "shortlist_contains_expected": shortlist_contains_expected,
                    "catalog_bytes": catalog_bytes,
                    "unscoped_bytes": scope.unscoped_bytes,
                    "omitted_capabilities": scope.omitted,
                    "latency_ms": latency_ms,
                    "error": null,
                })
            }
            | Err(why) => serde_json::json!({
                "id": id,
                "split": case.get("split"),
                "class": case.get("class"),
                "expected_capability": expected,
                "binding_override": binding_override,
                "binding_preference": binding_preference,
                "candidate_bindings": candidate_bindings,
                "binding_fixture": binding_fixture,
                "selected_call": null,
                "outcome": "transport-failed",
                "would_execute_without_preview": false,
                "withheld": null,
                "shortlist_contains_expected": shortlist_contains_expected,
                "catalog_bytes": catalog_bytes,
                "unscoped_bytes": scope.unscoped_bytes,
                "omitted_capabilities": scope.omitted,
                "latency_ms": latency_ms,
                "error": why.to_string(),
            }),
        };
        rows.push(row);
        eprintln!("measured {id} ({}/{})", rows.len(), limit.min(cases.len()));
    }
    let data = serde_json::json!({
        "version": 1,
        "corpus_version": corpus.get("version"),
        "mode": "pinned-needle-route-preview",
        "cases": rows,
    });
    std::fs::write(output, serde_json::to_vec_pretty(&data).expect("encode routing results")).expect("write routing results");
    let unsafe_cases = rows
        .iter()
        .filter(|row| {
            matches!(
                row.get("class").and_then(serde_json::Value::as_str),
                Some("forbidden" | "negated" | "ungrounded")
            )
        })
        .filter(|row| row.get("would_execute_without_preview").and_then(serde_json::Value::as_bool) == Some(true))
        .filter_map(|row| row.get("id").and_then(serde_json::Value::as_str))
        .collect::<Vec<_>>();
    assert!(unsafe_cases.is_empty(), "negative cases eligible for execution: {unsafe_cases:?}");
}
