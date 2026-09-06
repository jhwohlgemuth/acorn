#!/bin/sh
set -eu

: "${CI_JOB_TOKEN:?CI_JOB_TOKEN is required to upload release binaries}"
: "${PKG_REGISTRY_URL:?PKG_REGISTRY_URL is required to upload release binaries}"
: "${DEFAULT_TARGET:?DEFAULT_TARGET is required to upload release binaries}"
: "${DEFAULT_WINDOWS_TARGET:?DEFAULT_WINDOWS_TARGET is required to upload release binaries}"
: "${BINARY_NAME:?BINARY_NAME is required to upload release binaries}"
: "${PKG_VERSION:?PKG_VERSION is required to upload release binaries}"

transport_dir="$(mktemp -d)"
trap 'rm -rf "${transport_dir}"' EXIT

upload() {
    source="$1"
    destination="$2"
    existing="${transport_dir}/$(basename "${source}")"
    status="$(curl --silent --show-error --location \
        --header "JOB-TOKEN: ${CI_JOB_TOKEN}" \
        --output "${existing}" \
        --write-out "%{http_code}" \
        "${destination}" || true)"
    case "${status}" in
        200)
            cmp -s "${source}" "${existing}" || {
                echo "Refusing to overwrite ${destination} with different content" >&2
                return 1
            }
            echo "Release artifact already exists with matching content: ${destination}"
            ;;
        404)
            curl --fail-with-body --show-error --retry 3 --retry-all-errors \
                --header "JOB-TOKEN: ${CI_JOB_TOKEN}" \
                --upload-file "${source}" \
                "${destination}"
            ;;
        *)
            echo "Unable to inspect ${destination}: HTTP ${status}" >&2
            return 1
            ;;
    esac
}

upload \
    "bin/${BINARY_NAME}-v${PKG_VERSION}" \
    "${PKG_REGISTRY_URL}/${DEFAULT_TARGET}/v${PKG_VERSION}/${BINARY_NAME}"
upload \
    "bin/${BINARY_NAME}-v${PKG_VERSION}.exe" \
    "${PKG_REGISTRY_URL}/${DEFAULT_WINDOWS_TARGET}/v${PKG_VERSION}/${BINARY_NAME}.exe"
