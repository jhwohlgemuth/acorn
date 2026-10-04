#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "Homebrew installation smoke testing requires macOS" >&2
  exit 69
fi
if ! command -v brew >/dev/null 2>&1; then
  echo "Homebrew is required" >&2
  exit 69
fi
version="${1:-$(awk -F ' = ' '$1 ~ /^version/ { gsub(/["\r]/, "", $2); printf("%s", $2) }' Cargo.toml)}"
formula_directory="${CARGO_TARGET_DIR:-target}/homebrew/Formula"
formula_path="${formula_directory}/acorn-cli.rb"
mkdir -p "${formula_directory}"
"$(dirname "${BASH_SOURCE[0]}")/generate-formula.sh" "${version}" "${formula_path}"
brew install --build-from-source --formula "${formula_path}"
brew test "${formula_path}"
installed_binary="$(brew --prefix acorn-cli)/bin/acorn"
installed_version="$("${installed_binary}" --version)"
printf '%s\n' "${installed_version}"
if [[ "${installed_version}" != *"${version}"* ]]; then
  echo "Installed ACORN version does not match ${version}" >&2
  exit 1
fi
