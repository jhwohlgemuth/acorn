#!/usr/bin/env bash
set -euo pipefail

readonly forbidden='^(ammonia|anydoc|arboard|ariadne|async-trait|axum|bat|color-eyre|comfy-table|console|directories|dotenvy|flate2|futures|glob|http|httpdate|indicatif|infer|is_executable|lychee-lib|nanoid|owo-colors|pdf-inspector|percy-dom|polars|rand|rayon|reqwest|ring|rsa|rusqlite|rust-embed|rust-ini|sevenz-rust2|shell-words|shuck-linter|shuck-parser|similar|tar|tera|tokio|tower|which|zip) v'

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

check_tree native
check_tree wasm --target wasm32-unknown-unknown
check_wasm_tree
