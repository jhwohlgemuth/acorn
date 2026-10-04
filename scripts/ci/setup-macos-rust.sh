#!/usr/bin/env bash
set -euo pipefail

rust_version="$(awk -F ' = ' '$1 ~ /rust-version/ { gsub(/["\r]/, "", $2); printf("%s", $2) }' Cargo.toml)"
if [[ -z "${rust_version}" ]]; then
  echo "Cargo.toml does not declare workspace.package.rust-version" >&2
  exit 65
fi
export CARGO_HOME="${CARGO_HOME:-${HOME}/.cargo}"
export PATH="${CARGO_HOME}/bin:/opt/homebrew/bin:${PATH}"
if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none
fi
rustup toolchain install "${rust_version}" --profile minimal --component clippy --component rustfmt
rustup default "${rust_version}"
rustc --version
cargo --version
