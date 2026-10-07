#!/usr/bin/env bash
set -euo pipefail

: "${CARGO_REGISTRY_TOKEN:?CARGO_REGISTRY_TOKEN is required}"

version="$(awk -F ' = ' '$1 ~ /^version/ { gsub(/["\r]/, "", $2); printf("%s", $2) }' Cargo.toml)"
crates=(acorn-macros acorn-core acorn-diff acorn-cmd acorn-host acorn-schema acorn-analysis acorn-service acorn-lib acorn-server acorn-wasm acorn-tui acorn-cli)
crates_api="https://crates.io/api/v1/crates"

published() {
    curl --fail --silent --show-error --user-agent "acorn-ci/${version}" "${crates_api}/$1/${version}" >/dev/null
}

wait_until_published() {
    local crate="$1"
    local _
    for _ in $(seq 1 30); do
        if published "${crate}"; then
            return 0
        fi
        sleep 10
    done
    echo "${crate} ${version} was not available from crates.io after 5 minutes" >&2
    return 1
}

for crate in "${crates[@]}"; do
    if published "${crate}"; then
        echo "${crate} ${version} is already published"
    else
        cargo publish --package "${crate}"
    fi
    wait_until_published "${crate}"
done
