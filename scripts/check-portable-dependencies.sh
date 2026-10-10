#!/usr/bin/env bash
set -euo pipefail

readonly allowed_diff='^(acorn-diff|itoa|memchr|proc-macro2|quote|serde|serde_core|serde_derive|serde_json|similar|syn|unicode-ident|zmij) v'
readonly forbidden='^(ammonia|anydoc|arboard|ariadne|async-trait|axum|bat|color-eyre|comfy-table|console|directories|dotenvy|flate2|glob|http|httpdate|indicatif|infer|is_executable|lychee-lib|nanoid|owo-colors|pdf-inspector|percy-dom|polars|rand|rayon|reqwest|ring|rsa|rusqlite|rust-embed|rust-ini|sevenz-rust2|shell-words|shuck-linter|shuck-parser|tar|tera|tokio|tower|which|zip) v'
readonly forbidden_portable='^(acorn-host|acorn-lib|acorn-server|arboard|axum|color-eyre|directories|dotenvy|duckdb|lychee-lib|polars|reqwest|resvg|rmcp|rusqlite|shuck-linter|shuck-parser|sysinfo|tokio|tonic|tracing-subscriber|which) v'

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

check_portable_tree() {
    local package="$1"
    local tree
    local matches
    tree="$(cargo tree --locked -p "$package" --no-default-features --features alloc --edges normal --prefix none)"
    matches="$(printf '%s\n' "$tree" | grep -E "$forbidden_portable" || true)"
    if [[ -n "$matches" ]]; then
        printf 'Host dependencies leaked into the %s alloc graph:\n%s\n' "$package" "$matches" >&2
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

check_portable_tree acorn-analysis
check_portable_tree acorn-service
check_diff_tree
check_wasm_tree
