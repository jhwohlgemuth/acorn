#!/bin/sh
set -eu

appimagetool_version=1.9.1
runtime_version=20251108

if [ "$#" -gt 2 ]; then
    echo "Usage: $0 [install-root] [host-architecture]" >&2
    exit 2
fi

install_root="${1:-/usr/local}"
host_architecture="${2:-$(uname -m)}"
case "$host_architecture" in
    amd64 | x86_64)
        appimagetool_architecture=x86_64
        appimagetool_sha256=ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0
        ;;
    aarch64 | arm64)
        appimagetool_architecture=aarch64
        appimagetool_sha256=f0837e7448a0c1e4e650a93bb3e85802546e60654ef287576f46c71c126a9158
        ;;
    *)
        echo "Unsupported appimagetool host architecture: ${host_architecture}" >&2
        exit 2
        ;;
esac

if ! command -v curl >/dev/null 2>&1 || ! command -v sha256sum >/dev/null 2>&1; then
    echo "curl and sha256sum are required to install AppImage tools" >&2
    exit 1
fi

temporary_directory="$(mktemp -d "${TMPDIR:-/tmp}/acorn-appimage-tools.XXXXXX")"
trap 'rm -rf "${temporary_directory}"' EXIT

install_asset() {
    url="$1"
    sha256="$2"
    destination="$3"
    temporary="${temporary_directory}/$(basename -- "$destination")"

    if [ -f "$destination" ] && printf '%s  %s\n' "$sha256" "$destination" | sha256sum -c - >/dev/null 2>&1; then
        chmod 755 "$destination"
    else
        curl -fsSL "$url" -o "$temporary"
        printf '%s  %s\n' "$sha256" "$temporary" | sha256sum -c -
        mv "$temporary" "$destination"
        chmod 755 "$destination"
    fi
}

mkdir -p "${install_root}/bin" "${install_root}/lib/appimage"
install_asset \
    "https://github.com/AppImage/appimagetool/releases/download/${appimagetool_version}/appimagetool-${appimagetool_architecture}.AppImage" \
    "$appimagetool_sha256" \
    "${install_root}/bin/appimagetool"
install_asset \
    "https://github.com/AppImage/type2-runtime/releases/download/${runtime_version}/runtime-aarch64" \
    00cbdfcf917cc6c0ff6d3347d59e0ca1f7f45a6df1a428a0d6d8a78664d87444 \
    "${install_root}/lib/appimage/runtime-aarch64"
install_asset \
    "https://github.com/AppImage/type2-runtime/releases/download/${runtime_version}/runtime-x86_64" \
    2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d \
    "${install_root}/lib/appimage/runtime-x86_64"
