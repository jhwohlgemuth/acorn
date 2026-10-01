#!/usr/bin/env bash
set -euo pipefail

readonly allowed_diff='^(acorn-diff|itoa|memchr|proc-macro2|quote|serde|serde_core|serde_derive|serde_json|similar|syn|unicode-ident|zmij) v'
readonly forbidden='^(ammonia|anydoc|arboard|ariadne|async-trait|axum|bat|color-eyre|comfy-table|console|directories|dotenvy|flate2|futures|glob|http|httpdate|indicatif|infer|is_executable|lychee-lib|nanoid|owo-colors|pdf-inspector|percy-dom|polars|rand|rayon|reqwest|ring|rsa|rusqlite|rust-embed|rust-ini|sevenz-rust2|shell-words|shuck-linter|shuck-parser|similar|tar|tera|tokio|tower|which|zip) v'
readonly forbidden_analysis='^(axum|duckdb|resvg|rmcp|shuck-linter|shuck-parser|sysinfo) v'

check_diff_tree() {
    local features
    local std_features
    local tree
    local unexpected
    tree="$(cargo tree --locked -p acorn-diff --no-default-features --features alloc --edges normal --prefix none)"
    unexpected="$(printf '%s\n' "$tree" | grep -Ev "$allowed_diff" || true)"
    if [[ -n "$unexpected" ]]; then
        printf 'Unexpected packages leaked into the acorn-diff alloc graph:\n%s\n' "$unexpected" >&2
        exit 1
    fi
    features="$(cargo tree --locked -p acorn-diff --no-default-features --features alloc --edges normal,features --prefix none)"
    std_features="$(printf '%s\n' "$features" | grep -E ' feature "std"( \(\*\))?$' || true)"
    if [[ -n "$std_features" ]]; then
        printf 'Standard-library features leaked into the acorn-diff alloc graph:\n%s\n' "$std_features" >&2
        exit 1
    fi
}

check_analysis_tree() {
    local tree
    local matches
    tree="$(cargo tree --locked -p acorn-lib --no-default-features --features analysis --edges normal --prefix none)"
    matches="$(printf '%s\n' "$tree" | grep -E "$forbidden_analysis" || true)"
    if [[ -n "$matches" ]]; then
        printf 'Unrelated host dependencies leaked into the analysis graph:\n%s\n' "$matches" >&2
        exit 1
    fi
}

check_tree() {
    local label="$1"
    shift
    local tree
    local matches
    tree="$(cargo tree --locked -p acorn-lib --no-default-features --features analyzer --edges normal --prefix none "$@")"
    matches="$(printf '%s\n' "$tree" | grep -E "$forbidden" || true)"
    if [[ -n "$matches" ]]; then
        printf 'Host-only dependencies leaked into the %s analyzer graph:\n%s\n' "$label" "$matches" >&2
        exit 1
    fi
}

check_wasm_tree() {
    local tree
    local matches
    tree="$(cargo tree --locked -p acorn-wasm --edges normal --prefix none --target wasm32-unknown-unknown)"
    matches="$(printf '%s\n' "$tree" | grep -E "$forbidden" || true)"
    if [[ -n "$matches" ]]; then
        printf 'Host-only dependencies leaked into the acorn-wasm graph:\n%s\n' "$matches" >&2
        exit 1
    fi
}

check_analysis_tree
check_diff_tree
check_tree native
check_tree wasm --target wasm32-unknown-unknown
check_wasm_tree
