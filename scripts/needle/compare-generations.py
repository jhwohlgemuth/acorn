#!/usr/bin/env python3
"""Compare Needle 2 and Needle 3 against ACORN's checked-in prompt corpus."""

from __future__ import annotations

import argparse
import json
import math
import os
import statistics
import subprocess
import tempfile
import time
from pathlib import Path
from typing import Any


DEFAULT_TIMEOUT_SECONDS = 120


def parse_arguments() -> argparse.Namespace:
    """Parse explicit runner, model, catalog, and output locations."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixtures", default="tests/fixtures/needle/evaluation.jsonl", type=Path)
    parser.add_argument("--execution-threshold", default=0.8, type=float)
    parser.add_argument("--max-accuracy-regression", default=0.0, type=float)
    parser.add_argument("--max-latency-ratio", default=1.5, type=float)
    parser.add_argument("--max-v3-memory-mb", default=128.0, type=float)
    parser.add_argument("--output", default="target/needle-generation-comparison.json", type=Path)
    parser.add_argument("--repetitions", default=2, type=int)
    parser.add_argument("--timeout", default=DEFAULT_TIMEOUT_SECONDS, type=int)
    parser.add_argument("--tools", required=True, type=Path)
    parser.add_argument("--v2-runner", required=True, type=Path)
    parser.add_argument("--v3-model", required=True, type=Path)
    parser.add_argument("--v3-runner", required=True, type=Path)
    return parser.parse_args()


def parse_json_output(stdout: str) -> dict[str, Any]:
    """Decode either a single JSON document or the final JSON line."""
    try:
        value = json.loads(stdout)
    except json.JSONDecodeError:
        candidates = (line for line in reversed(stdout.splitlines()) if line.strip())
        for candidate in candidates:
            try:
                value = json.loads(candidate)
                break
            except json.JSONDecodeError:
                continue
        else:
            raise ValueError("runner stdout did not contain a JSON response") from None
    if not isinstance(value, dict):
        raise ValueError("runner response must be a JSON object")
    return value


def load_fixtures(path: Path) -> list[dict[str, Any]]:
    """Load non-empty JSONL records."""
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]


def command_for(generation: str, arguments: argparse.Namespace, prompt: str, tool_index: Path) -> list[str]:
    """Build the documented one-shot invocation for one generation."""
    common = ["--tools", str(arguments.tools), "--tool-index", str(tool_index), "--prompt", prompt]
    if generation == "needle2":
        return [str(arguments.v2_runner), *common]
    return [
        str(arguments.v3_runner),
        "--model",
        str(arguments.v3_model),
        *common,
        "--depth",
        "20",
        "--max",
        "4096",
        "--fail-input-overflow",
    ]


def raw_matches(fixture: dict[str, Any], response: dict[str, Any]) -> bool:
    """Score exact native response kind, tool, and any declared expected arguments."""
    expected = fixture["expected"]
    calls = response.get("function_calls") or []
    expected_tool = expected.get("tool")
    matching = [call for call in calls if isinstance(call, dict) and call.get("name") == expected_tool]
    arguments_match = "arguments" not in expected or any(call.get("arguments") == expected["arguments"] for call in matching)
    return response.get("type") == expected["type"] and (bool(matching) and arguments_match if expected_tool is not None else not calls)


def arguments_are_valid(call: dict[str, Any], tools: dict[str, dict[str, Any]]) -> bool:
    """Apply the deterministic argument checks relevant to the compact ACORN catalog."""
    definition = tools.get(str(call.get("name")))
    arguments = call.get("arguments")
    if definition is None or not isinstance(arguments, dict):
        return False
    schema = definition.get("parameters") or {}
    properties = schema.get("properties") or {}
    if any(name not in arguments for name in schema.get("required") or []):
        return False
    if any(name not in properties for name in arguments):
        return False
    activity_json = arguments.get("activity_json")
    if activity_json is not None:
        try:
            return isinstance(activity_json, str) and isinstance(json.loads(activity_json), dict)
        except json.JSONDecodeError:
            return False
    return True


def authorized_tools(response: dict[str, Any], tools: dict[str, dict[str, Any]], threshold: float) -> list[str]:
    """Model ACORN's confidence, grounding, exposure, and argument-validation boundary."""
    confidence = response.get("confidence")
    validation = response.get("validation") or {}
    eligible = (
        response.get("success") is True
        and response.get("type") == "call"
        and isinstance(confidence, (int, float))
        and math.isfinite(confidence)
        and threshold <= confidence <= 1.0
        and validation.get("negation") is not True
        and not validation.get("ungrounded")
    )
    if not eligible:
        return []
    return [
        str(call["name"])
        for call in response.get("function_calls") or []
        if isinstance(call, dict) and "name" in call and arguments_are_valid(call, tools)
    ]


def policy_matches(fixture: dict[str, Any], authorized: list[str]) -> bool:
    """Score the effective ACORN outcome after policy and typed argument validation."""
    expected_tool = fixture["expected"].get("tool")
    return expected_tool in authorized if expected_tool is not None else not authorized


def run_case(
    generation: str,
    fixture: dict[str, Any],
    phase: str,
    arguments: argparse.Namespace,
    tool_index: Path,
    tools: dict[str, dict[str, Any]],
) -> dict[str, Any]:
    """Run one isolated prompt with telemetry disabled and retain native metrics."""
    environment = {**os.environ, "DO_NOT_TRACK": "1", "NEEDLE_TELEMETRY": "0"}
    started = time.perf_counter()
    try:
        completed = subprocess.run(
            command_for(generation, arguments, fixture["prompt"], tool_index),
            capture_output=True,
            check=False,
            env=environment,
            text=True,
            timeout=arguments.timeout,
        )
        elapsed_ms = (time.perf_counter() - started) * 1_000
        response = parse_json_output(completed.stdout)
        authorized = authorized_tools(response, tools, arguments.execution_threshold)
        return {
            "authorized_tools": authorized,
            "confidence": response.get("confidence"),
            "elapsed_ms": round(elapsed_ms, 3),
            "error": response.get("error"),
            "exit_code": completed.returncode,
            "fixture": fixture["id"],
            "function_calls": response.get("function_calls") or [],
            "generation": generation,
            "peak_ram_mb": response.get("peak_ram_mb"),
            "phase": phase,
            "policy_matched": completed.returncode == 0 and policy_matches(fixture, authorized),
            "raw_matched": completed.returncode == 0 and raw_matches(fixture, response),
            "response_type": response.get("type"),
            "selected_tools": [call.get("name") for call in response.get("function_calls") or []],
            "stderr": completed.stderr.strip(),
            "suppressed_calls": response.get("suppressed_calls") or [],
            "validation": response.get("validation"),
        }
    except (OSError, subprocess.TimeoutExpired, ValueError) as error:
        return {
            "elapsed_ms": round((time.perf_counter() - started) * 1_000, 3),
            "error": str(error),
            "fixture": fixture["id"],
            "generation": generation,
            "phase": phase,
            "policy_matched": False,
            "raw_matched": False,
        }


def mean(values: list[float]) -> float | None:
    """Return a stable rounded mean when at least one metric exists."""
    return round(statistics.fmean(values), 3) if values else None


def summarize(generation: str, results: list[dict[str, Any]]) -> dict[str, Any]:
    """Aggregate quality and native performance metrics for one generation."""
    selected = [result for result in results if result["generation"] == generation]
    confidences = [float(result["confidence"]) for result in selected if isinstance(result.get("confidence"), (int, float))]
    memory = [float(result["peak_ram_mb"]) for result in selected if isinstance(result.get("peak_ram_mb"), (int, float))]
    cold = [float(result["elapsed_ms"]) for result in selected if result["phase"] == "cold"]
    warm = [float(result["elapsed_ms"]) for result in selected if result["phase"] == "warm"]
    return {
        "average_confidence": mean(confidences),
        "average_elapsed_ms": mean([float(result["elapsed_ms"]) for result in selected]),
        "average_peak_ram_mb": mean(memory),
        "cold_elapsed_ms": mean(cold),
        "failures": sum(result.get("exit_code") != 0 or bool(result.get("error")) for result in selected),
        "max_peak_ram_mb": round(max(memory), 3) if memory else None,
        "policy_accuracy": round(sum(bool(result["policy_matched"]) for result in selected) / len(selected), 4) if selected else None,
        "policy_escapes": sum(bool(result.get("authorized_tools")) and not result["policy_matched"] for result in selected),
        "raw_accuracy": round(sum(bool(result["raw_matched"]) for result in selected) / len(selected), 4) if selected else None,
        "total": len(selected),
        "warm_elapsed_ms": mean(warm),
    }


def metric_gate(baseline: float | None, candidate: float | None, maximum_ratio: float) -> bool:
    """Compare optional positive metrics without turning missing telemetry into a pass."""
    return baseline is not None and candidate is not None and candidate <= baseline * maximum_ratio


def evaluate_gates(summary: dict[str, dict[str, Any]], arguments: argparse.Namespace) -> dict[str, bool]:
    """Apply the predeclared correctness, latency, memory, and process-failure gates."""
    needle2 = summary["needle2"]
    needle3 = summary["needle3"]
    return {
        "accuracy": needle3["policy_accuracy"] + arguments.max_accuracy_regression >= needle2["policy_accuracy"],
        "cold_latency": metric_gate(needle2["cold_elapsed_ms"], needle3["cold_elapsed_ms"], arguments.max_latency_ratio),
        "memory": needle3["max_peak_ram_mb"] is not None and needle3["max_peak_ram_mb"] <= arguments.max_v3_memory_mb,
        "needle3_process": needle3["failures"] == 0,
        "policy": needle3["policy_escapes"] == 0,
        "warm_latency": metric_gate(needle2["warm_elapsed_ms"], needle3["warm_elapsed_ms"], arguments.max_latency_ratio),
    }


def main() -> int:
    """Run both generations in a fixed order and write one deterministic report shape."""
    arguments = parse_arguments()
    if arguments.repetitions < 2:
        raise ValueError("--repetitions must be at least 2 to measure cold and warm runs")
    fixtures = load_fixtures(arguments.fixtures)
    tool_definitions = json.loads(arguments.tools.read_text(encoding="utf-8"))
    tools = {definition["name"]: definition for definition in tool_definitions}
    with tempfile.TemporaryDirectory(prefix="acorn-needle-comparison-") as index_root:
        indexes = {generation: Path(index_root, f"{generation}.idx") for generation in ("needle2", "needle3")}
        results = [
            run_case(generation, fixture, "cold" if repetition == 0 else "warm", arguments, indexes[generation], tools)
            for repetition in range(arguments.repetitions)
            for fixture in fixtures
            for generation in ("needle2", "needle3")
        ]
    summary = {generation: summarize(generation, results) for generation in ("needle2", "needle3")}
    license_path = arguments.v3_model.parent / "LICENSE"
    summary["needle2"]["installed_bytes"] = arguments.v2_runner.stat().st_size
    summary["needle3"]["installed_bytes"] = (
        arguments.v3_runner.stat().st_size
        + arguments.v3_model.stat().st_size
        + (license_path.stat().st_size if license_path.exists() else 0)
    )
    gates = evaluate_gates(summary, arguments)
    report = {
        "fixtures": str(arguments.fixtures),
        "gates": gates,
        "limits": {
            "execution_threshold": arguments.execution_threshold,
            "max_accuracy_regression": arguments.max_accuracy_regression,
            "max_latency_ratio": arguments.max_latency_ratio,
            "max_v3_memory_mb": arguments.max_v3_memory_mb,
        },
        "results": results,
        "summary": summary,
        "tools": str(arguments.tools),
    }
    arguments.output.parent.mkdir(parents=True, exist_ok=True)
    arguments.output.write_text(f"{json.dumps(report, indent=2, sort_keys=True)}\n", encoding="utf-8")
    print(json.dumps({"gates": gates, "summary": summary}, indent=2, sort_keys=True))
    return 0 if all(gates.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
