#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-}"
OUT_PATH="${2:-Formula/acorn-cli.rb}"
TEMPLATE_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/formula.template.rb"
CRATE_URL="https://static.crates.io/crates/acorn-cli/acorn-cli-${VERSION}.crate"
if [[ -z "${VERSION}" ]]; then
  echo "Usage: $0 <version-without-v-prefix> [output-path]" >&2
  exit 64
fi
if [[ ! -f "${TEMPLATE_PATH}" ]]; then
  echo "Formula template not found at ${TEMPLATE_PATH}" >&2
  exit 66
fi
hash_file() {
  local file_path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${file_path}" | awk '{print $1}'
    return
  fi
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${file_path}" | awk '{print $1}'
    return
  fi
  if command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 "${file_path}" | awk '{print $NF}'
    return
  fi
  echo "No SHA-256 utility found (need one of: sha256sum, shasum, openssl)" >&2
  exit 69
}
tmp_file="$(mktemp)"
trap 'rm -f "${tmp_file}"' EXIT
curl -A "acorn-homebrew-formula/1.0" --retry 3 --retry-delay 1 -LsSf "${CRATE_URL}" -o "${tmp_file}"
SHA256="$(hash_file "${tmp_file}")"
mkdir -p "$(dirname "${OUT_PATH}")"
sed \
  -e "s|__VERSION__|${VERSION}|g" \
  -e "s|__SHA256__|${SHA256}|g" \
  "${TEMPLATE_PATH}" > "${OUT_PATH}"
echo "Wrote ${OUT_PATH}"
echo "Source URL: ${CRATE_URL}"
echo "sha256: ${SHA256}"
