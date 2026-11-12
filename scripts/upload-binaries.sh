#!/bin/sh
set -eu

: "${CI_JOB_TOKEN:?CI_JOB_TOKEN is required to upload release binaries}"
: "${PKG_REGISTRY_URL:?PKG_REGISTRY_URL is required to upload release binaries}"
: "${DEFAULT_ARM64_TARGET:?DEFAULT_ARM64_TARGET is required to upload release binaries}"
: "${DEFAULT_TARGET:?DEFAULT_TARGET is required to upload release binaries}"
: "${DEFAULT_WINDOWS_TARGET:?DEFAULT_WINDOWS_TARGET is required to upload release binaries}"
: "${BINARY_NAME:?BINARY_NAME is required to upload release binaries}"
: "${PKG_VERSION:?PKG_VERSION is required to upload release binaries}"

upload() {
    cargo xtask upload-artifact "$1" "$2"
}

upload \
    "bin/${BINARY_NAME}-v${PKG_VERSION}" \
    "${PKG_REGISTRY_URL}/${DEFAULT_TARGET}/v${PKG_VERSION}/${BINARY_NAME}"
upload \
    "bin/${BINARY_NAME}-v${PKG_VERSION}.exe" \
    "${PKG_REGISTRY_URL}/${DEFAULT_WINDOWS_TARGET}/v${PKG_VERSION}/${BINARY_NAME}.exe"
upload \
    "bin/${BINARY_NAME}-v${PKG_VERSION}.sha256" \
    "${PKG_REGISTRY_URL}/${DEFAULT_TARGET}/v${PKG_VERSION}/${BINARY_NAME}.sha256"
upload \
    "bin/${BINARY_NAME}-v${PKG_VERSION}.exe.sha256" \
    "${PKG_REGISTRY_URL}/${DEFAULT_WINDOWS_TARGET}/v${PKG_VERSION}/${BINARY_NAME}.exe.sha256"
upload \
    "bin/${BINARY_NAME}-v${PKG_VERSION}-aarch64.AppImage" \
    "${PKG_REGISTRY_URL}/${DEFAULT_ARM64_TARGET}/v${PKG_VERSION}/${BINARY_NAME}-v${PKG_VERSION}-aarch64.AppImage"
upload \
    "bin/${BINARY_NAME}-v${PKG_VERSION}-aarch64.AppImage.sha256" \
    "${PKG_REGISTRY_URL}/${DEFAULT_ARM64_TARGET}/v${PKG_VERSION}/${BINARY_NAME}-v${PKG_VERSION}-aarch64.AppImage.sha256"
upload \
    "bin/${BINARY_NAME}-v${PKG_VERSION}-x86_64.AppImage" \
    "${PKG_REGISTRY_URL}/${DEFAULT_TARGET}/v${PKG_VERSION}/${BINARY_NAME}-v${PKG_VERSION}-x86_64.AppImage"
upload \
    "bin/${BINARY_NAME}-v${PKG_VERSION}-x86_64.AppImage.sha256" \
    "${PKG_REGISTRY_URL}/${DEFAULT_TARGET}/v${PKG_VERSION}/${BINARY_NAME}-v${PKG_VERSION}-x86_64.AppImage.sha256"
upload \
    "bin/${BINARY_NAME}_${PKG_VERSION}-1_amd64.deb" \
    "${PKG_REGISTRY_URL}/${DEFAULT_TARGET}/v${PKG_VERSION}/${BINARY_NAME}_${PKG_VERSION}-1_amd64.deb"
upload \
    "bin/${BINARY_NAME}_${PKG_VERSION}-1_amd64.deb.sha256" \
    "${PKG_REGISTRY_URL}/${DEFAULT_TARGET}/v${PKG_VERSION}/${BINARY_NAME}_${PKG_VERSION}-1_amd64.deb.sha256"
upload \
    "bin/${BINARY_NAME}_${PKG_VERSION}-1_arm64.deb" \
    "${PKG_REGISTRY_URL}/${DEFAULT_ARM64_TARGET}/v${PKG_VERSION}/${BINARY_NAME}_${PKG_VERSION}-1_arm64.deb"
upload \
    "bin/${BINARY_NAME}_${PKG_VERSION}-1_arm64.deb.sha256" \
    "${PKG_REGISTRY_URL}/${DEFAULT_ARM64_TARGET}/v${PKG_VERSION}/${BINARY_NAME}_${PKG_VERSION}-1_arm64.deb.sha256"
upload \
    "bin/${BINARY_NAME}-v${PKG_VERSION}-${DEFAULT_TARGET}.mcpb" \
    "${PKG_REGISTRY_URL}/${DEFAULT_TARGET}/v${PKG_VERSION}/${BINARY_NAME}.mcpb"
upload \
    "bin/${BINARY_NAME}-v${PKG_VERSION}-${DEFAULT_TARGET}.mcpb.sha256" \
    "${PKG_REGISTRY_URL}/${DEFAULT_TARGET}/v${PKG_VERSION}/${BINARY_NAME}.mcpb.sha256"
upload \
    "bin/${BINARY_NAME}-v${PKG_VERSION}-${DEFAULT_WINDOWS_TARGET}.mcpb" \
    "${PKG_REGISTRY_URL}/${DEFAULT_WINDOWS_TARGET}/v${PKG_VERSION}/${BINARY_NAME}.mcpb"
upload \
    "bin/${BINARY_NAME}-v${PKG_VERSION}-${DEFAULT_WINDOWS_TARGET}.mcpb.sha256" \
    "${PKG_REGISTRY_URL}/${DEFAULT_WINDOWS_TARGET}/v${PKG_VERSION}/${BINARY_NAME}.mcpb.sha256"
