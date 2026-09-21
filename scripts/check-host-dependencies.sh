#!/usr/bin/env bash
set -euo pipefail

readonly forbidden_host='^(acorn-lib|acorn-schema|acorn-tui|acorn-wasm) v'
readonly forbidden_command='^(acorn-core|acorn-host|acorn-lib|acorn-schema|acorn-tui|acorn-wasm) v'

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

check_tree acorn-cmd shell-lint "$forbidden_command"
for feature in config download fs http source terminal; do
    check_tree acorn-host "$feature" "$forbidden_host"
done
