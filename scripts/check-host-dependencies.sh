#!/usr/bin/env bash
set -euo pipefail

readonly forbidden_host='^(acorn-lib|acorn-schema|acorn-tui|acorn-wasm) v'
readonly forbidden_command='^(acorn-core|acorn-host|acorn-lib|acorn-schema|acorn-tui|acorn-wasm) v'
readonly forbidden_mcp='^(acorn-cmd|acorn-diff|ammonia|arboard|ariadne|axum|bat|comfy-table|console|directories|dotenvy|duckdb|flate2|glob|indicatif|infer|is_executable|jsonschema|lychee-lib|nanoid|pdf-inspector|percy-dom|polars|rayon|resvg|rusqlite|rust-embed|rust-ini|same-file|sevenz-rust2|shuck-linter|shuck-parser|similar|sysinfo|tar|tera|zip|zstd) v'
readonly forbidden_std='^(acorn-cmd|acorn-diff|agent-client-protocol|ammonia|anydoc|arboard|ariadne|axum|bat|comfy-table|console|directories|dotenvy|duckdb|flate2|glob|indicatif|infer|is_executable|jsonschema|lychee-lib|nanoid|pdf-inspector|percy-dom|polars|rayon|reqwest|resvg|ring|rmcp|rusqlite|rust-embed|rust-ini|same-file|sevenz-rust2|shuck-linter|shuck-parser|similar|sysinfo|tar|tera|tokio|tower|which|zip|zstd) v'

check_tree() {
    local package="$1"
    local features="$2"
    local forbidden="$3"
    local tree
    local matches
    tree="$(cargo tree --locked -p "$package" --no-default-features --features "$features" --edges normal --prefix none)"
    matches="$(printf '%s\n' "$tree" | grep -E "$forbidden" || true)"
    if [[ -n "$matches" ]]; then
        printf 'Unrelated ACORN packages leaked into %s [%s]:\n%s\n' "$package" "$features" "$matches" >&2
        exit 1
    fi
}

check_acorn_lib_tree() {
    local features="$1"
    local forbidden="$2"
    local tree
    local matches
    tree="$(cargo tree --locked -p acorn-lib --no-default-features --features "$features" --edges normal --prefix none)"
    matches="$(printf '%s\n' "$tree" | grep -E "$forbidden" || true)"
    if [[ -n "$matches" ]]; then
        printf 'Unrelated dependencies leaked into acorn-lib [%s]:\n%s\n' "$features" "$matches" >&2
        exit 1
    fi
}

check_tree acorn-cmd shell-lint "$forbidden_command"
for feature in config download fs http source terminal; do
    check_tree acorn-host "$feature" "$forbidden_host"
done
check_acorn_lib_tree mcp-client "$forbidden_mcp"
check_acorn_lib_tree std "$forbidden_std"
