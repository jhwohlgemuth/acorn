#[cfg(feature = "exec")]
use super::*;
#[cfg(feature = "exec")]
use acorn::io::mcp::{CapabilitySelectionContext, ExecutionContext};
#[cfg(feature = "exec")]
use acorn::io::sidecar::needle::Needle;

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
            assert!(scope.bytes.checked_bytes().is_some_and(|bytes| bytes <= 22 * 1024));
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
        })
        .take(limit)
    {
        let id = case.get("id").and_then(serde_json::Value::as_str).expect("case ID");
        let prompt = case.get("prompt").and_then(serde_json::Value::as_str).expect("case prompt");
        let expected = case
            .get("expected")
            .and_then(|value| value.get("capability_id"))
            .and_then(serde_json::Value::as_str);
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
        let scope = registry.scope_for_prompt(prompt, selection).expect("bounded routing scope");
        let selected_ids = explicit_scope.clone().or(scope.ids.clone());
        let catalog = registry.capability_catalog(selected_ids.as_deref(), selection).expect("selected catalog");
        let catalog_bytes = catalog.len();
        if let Some(path) = std::env::var_os("ACORN_ROUTING_CATALOG") {
            std::fs::write(path, catalog).expect("write routing catalog");
        }
        let shortlist_contains_expected = expected.map(|capability| selected_ids.as_ref().is_none_or(|ids| ids.iter().any(|id| id == capability)));
        let started = std::time::Instant::now();
        let routed = Needle::start_with_scoped_registry(None, offline, registry.clone(), selected_ids).await;
        let result = match routed {
            | Ok(sidecar) => {
                sidecar
                    .route_once(
                        prompt.to_string(),
                        ExecutionContext {
                            selection,
                            preview: true,
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
                let selected_call = result
                    .response
                    .as_ref()
                    .and_then(|response| response.function_calls.first())
                    .map(|call| call.name.clone());
                let response_error = result.response.as_ref().and_then(|response| response.error.clone());
                let response_error_code = result.response.as_ref().and_then(|response| response.error_code.clone());
                let response_success = result.response.as_ref().map(|response| response.success);
                serde_json::json!({
                    "id": id,
                    "split": case.get("split"),
                    "class": case.get("class"),
                    "expected_capability": expected,
                    "selected_call": selected_call,
                    "selected_capability": result.selected.as_ref().map(|selected| &selected.capability_id),
                    "response_error": response_error,
                    "response_error_code": response_error_code,
                    "response_success": response_success,
                    "outcome": result.outcome,
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
                "selected_call": null,
                "outcome": "transport-failed",
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
}
