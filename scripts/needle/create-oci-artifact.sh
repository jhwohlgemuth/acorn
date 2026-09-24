#!/bin/sh
set -eu

usage() {
    echo "Usage: $0 SOURCE_DIRECTORY REGISTRY_REFERENCE [BUNDLE_DIRECTORY] [PLATFORM]" >&2
    echo "Example: $0 ./needle3 harbor.example.org/research/needle:modelkit ./needle-modelkit linux-x86_64" >&2
}

source_directory="${1:-}"
reference="${2:-}"
bundle="${3:-needle-modelkit}"
platform="${4:-}"
acorn_bin="${ACORN_BIN:-acorn}"
oras_bin="${ORAS_BIN:-oras}"
source_revision="${NEEDLE_SOURCE_REVISION:-b274efcb211a9eef48c9a88da4b43bd569696a39}"

if [ -z "${source_directory}" ] || [ -z "${reference}" ]; then
    usage
    exit 2
fi
if [ ! -d "${source_directory}" ]; then
    echo "Needle 3 source directory does not exist: ${source_directory}" >&2
    exit 1
fi
if [ -e "${bundle}" ]; then
    echo "Refusing to overwrite bundle path: ${bundle}" >&2
    exit 1
fi
if ! command -v "${acorn_bin}" >/dev/null 2>&1; then
    echo "ACORN executable is unavailable: ${acorn_bin}" >&2
    exit 1
fi
if ! command -v "${oras_bin}" >/dev/null 2>&1; then
    echo "ORAS executable is unavailable: ${oras_bin}" >&2
    exit 1
fi
if command -v sha256sum >/dev/null 2>&1; then
    checksum() { sha256sum "$1" | awk '{print $1}'; }
elif command -v shasum >/dev/null 2>&1; then
    checksum() { shasum -a 256 "$1" | awk '{print $1}'; }
else
    echo "A SHA-256 utility (sha256sum or shasum) is required" >&2
    exit 1
fi
file_size() { wc -c < "$1" | tr -d '[:space:]'; }
if [ "${#source_revision}" -ne 40 ] || printf '%s' "${source_revision}" | grep -Eq '[^0-9a-fA-F]'; then
    echo "NEEDLE_SOURCE_REVISION must be a 40-character hexadecimal revision" >&2
    exit 1
fi

if [ -z "${platform}" ]; then
    system="$(uname -s)"
    machine="$(uname -m)"
    case "${system}/${machine}" in
        Linux/x86_64|Linux/amd64) platform="linux-x86_64" ;;
        Linux/aarch64|Linux/arm64) platform="linux-arm64" ;;
        Darwin/arm64|Darwin/aarch64) platform="macos-arm64" ;;
        MINGW*/*x86_64|MSYS*/*x86_64) platform="windows-x86_64" ;;
        MINGW*/*arm64|MSYS*/*arm64) platform="windows-arm64" ;;
        *)
            echo "Cannot infer a supported Needle platform from ${system}/${machine}; pass PLATFORM explicitly" >&2
            exit 1
            ;;
    esac
fi
case "${platform}" in
    windows-x86_64|windows-arm64) runner_name="needle.exe" ;;
    linux-x86_64|linux-arm64|macos-arm64) runner_name="needle" ;;
    *)
        echo "Unsupported Needle platform: ${platform}" >&2
        exit 1
        ;;
esac
runner="${source_directory}/${runner_name}"
model="${source_directory}/needle3.cact"
license="${source_directory}/LICENSE"
for required_file in "${runner}" "${model}" "${license}"; do
    if [ ! -f "${required_file}" ]; then
        echo "Needle 3 bundle input does not exist: ${required_file}" >&2
        exit 1
    fi
done

registry_reference="${reference#oci://}"
case "${registry_reference}" in
    *@*)
        echo "Publish to a tag, not an existing digest: ${reference}" >&2
        exit 1
        ;;
esac

"${acorn_bin}" export --tools --output "${bundle}"
cp "${runner}" "${bundle}/${runner_name}"
cp "${model}" "${bundle}/needle3.cact"
cp "${license}" "${bundle}/LICENSE"
license_sha256="$(checksum "${bundle}/LICENSE")"
license_size="$(file_size "${bundle}/LICENSE")"
model_sha256="$(checksum "${bundle}/needle3.cact")"
model_size="$(file_size "${bundle}/needle3.cact")"
runner_sha256="$(checksum "${bundle}/${runner_name}")"
runner_size="$(file_size "${bundle}/${runner_name}")"
tools_sha256="$(checksum "${bundle}/tools.json")"
tools_size="$(file_size "${bundle}/tools.json")"

printf '%s\n' \
    '{' \
    '  "manifestVersion": "v1.0.0",' \
    "  \"sourceRevision\": \"${source_revision}\"," \
    "  \"platform\": \"${platform}\"," \
    '  "sidecar": {' \
    '    "kind": "needle",' \
    '    "protocolVersion": 2' \
    '  },' \
    '  "runner": {' \
    "    \"path\": \"${runner_name}\"," \
    "    \"size\": ${runner_size}," \
    "    \"sha256\": \"${runner_sha256}\"" \
    '  },' \
    '  "assets": [' \
    '    {' \
    '      "name": "license",' \
    '      "path": "LICENSE",' \
    "      \"size\": ${license_size}," \
    "      \"sha256\": \"${license_sha256}\"" \
    '    },' \
    '    {' \
    '      "name": "model",' \
    '      "path": "needle3.cact",' \
    "      \"size\": ${model_size}," \
    "      \"sha256\": \"${model_sha256}\"" \
    '    },' \
    '    {' \
    '      "name": "tools",' \
    '      "path": "tools.json",' \
    "      \"size\": ${tools_size}," \
    "      \"sha256\": \"${tools_sha256}\"" \
    '    }' \
    '  ],' \
    '  "tools": "tools"' \
    '}' > "${bundle}/needle-modelkit.json"

(
    cd "${bundle}"
    set -- push \
        --artifact-type application/vnd.acorn.needle.modelkit.v2 \
        "${registry_reference}" \
        needle-modelkit.json:application/vnd.acorn.needle.modelkit.manifest.v2+json \
        LICENSE:text/plain \
        needle3.cact:application/vnd.acorn.needle.model.v3 \
        tools.json:application/json \
        "${runner_name}:application/vnd.acorn.needle.runner.v3"
    if [ -n "${ORAS_REGISTRY_CONFIG:-}" ]; then set -- "$@" --registry-config "${ORAS_REGISTRY_CONFIG}"; fi
    if [ -n "${ORAS_CA_FILE:-}" ]; then set -- "$@" --ca-file "${ORAS_CA_FILE}"; fi
    if [ "${ORAS_PLAIN_HTTP:-0}" = "1" ]; then set -- "$@" --plain-http; fi
    "${oras_bin}" "$@"
)

set -- resolve --full-reference "${registry_reference}"
if [ -n "${ORAS_REGISTRY_CONFIG:-}" ]; then set -- "$@" --registry-config "${ORAS_REGISTRY_CONFIG}"; fi
if [ -n "${ORAS_CA_FILE:-}" ]; then set -- "$@" --ca-file "${ORAS_CA_FILE}"; fi
if [ "${ORAS_PLAIN_HTTP:-0}" = "1" ]; then set -- "$@" --plain-http; fi
resolved_reference="$("${oras_bin}" "$@")"
if ! printf '%s' "${resolved_reference}" | grep -Eq '@sha256:[0-9a-fA-F]{64}$'; then
    echo "ORAS returned an unexpected resolved reference: ${resolved_reference}" >&2
    exit 1
fi

echo "Bundle: ${bundle}"
echo "Custom location: oci://${resolved_reference}"
echo "Run: acorn infer \"Return the ACORN version\" --custom-location oci://${resolved_reference}"
