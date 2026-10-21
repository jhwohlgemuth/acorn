#!/bin/sh
set -eu

usage() {
    echo "Usage: $0 <version> <rust-target> <debian-architecture> <appimage-architecture>" >&2
}

if [ "$#" -ne 4 ]; then
    usage
    exit 2
fi

version="$1"
target="$2"
debian_architecture="$3"
appimage_architecture="$4"

case "${target}:${debian_architecture}:${appimage_architecture}" in
    aarch64-unknown-linux-musl:arm64:aarch64 | x86_64-unknown-linux-musl:amd64:x86_64) ;;
    *)
        echo "Unsupported Linux package architecture tuple: ${target}/${debian_architecture}/${appimage_architecture}" >&2
        exit 2
        ;;
esac

repo_root="$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)"
manifest_version="$(awk -F ' = ' '$1 ~ /^version/ { gsub(/["\r]/, "", $2); printf("%s", $2); exit }' "${repo_root}/Cargo.toml")"

if [ "$version" != "$manifest_version" ]; then
    echo "Requested version ${version} does not match workspace version ${manifest_version}" >&2
    exit 2
fi

target_root="${CARGO_TARGET_DIR:-${repo_root}/target}"
case "$target_root" in
    /*) ;;
    *) target_root="${repo_root}/${target_root}" ;;
esac

binary="${target_root}/${target}/release/acorn"
if [ ! -x "$binary" ]; then
    echo "Release binary is missing or not executable: ${binary}" >&2
    exit 1
fi

appimagetool="${APPIMAGETOOL:-$(command -v appimagetool || true)}"
runtime_root="${APPIMAGE_RUNTIME_DIR:-/usr/local/lib/appimage}"
runtime="${runtime_root}/runtime-${appimage_architecture}"
if [ -z "$appimagetool" ] || [ ! -x "$appimagetool" ]; then
    echo "appimagetool is missing or not executable; run scripts/install-appimage-tools.sh" >&2
    exit 1
fi
if [ ! -r "$runtime" ]; then
    echo "AppImage runtime is missing: ${runtime}" >&2
    exit 1
fi

if [ -z "${SOURCE_DATE_EPOCH:-}" ]; then
    SOURCE_DATE_EPOCH="$(git -C "$repo_root" log -1 --format=%ct)"
fi
case "$SOURCE_DATE_EPOCH" in
    '' | *[!0-9]*)
        echo "SOURCE_DATE_EPOCH must be an unsigned integer" >&2
        exit 2
        ;;
esac
export LC_ALL=C SOURCE_DATE_EPOCH TZ=UTC

output_dir="${repo_root}/bin"
appimage_output="${output_dir}/acorn-v${version}-${appimage_architecture}.AppImage"
deb_output="${output_dir}/acorn_${version}-1_${debian_architecture}.deb"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/acorn-linux-package.XXXXXX")"
appdir="${work_dir}/Acorn.AppDir"
trap 'rm -rf "${work_dir}"' EXIT

mkdir -p \
    "${appdir}/usr/bin" \
    "${appdir}/usr/share/applications" \
    "${appdir}/usr/share/icons/hicolor/scalable/apps" \
    "${appdir}/usr/share/metainfo" \
    "$output_dir"
cp "$binary" "${appdir}/usr/bin/acorn"
cp "${repo_root}/packaging/linux/AppRun" "${appdir}/AppRun"
cp "${repo_root}/packaging/linux/gov.ornl.acorn.desktop" "${appdir}/gov.ornl.acorn.desktop"
cp "${repo_root}/packaging/linux/gov.ornl.acorn.desktop" "${appdir}/usr/share/applications/gov.ornl.acorn.desktop"
cp "${repo_root}/packaging/linux/gov.ornl.acorn.metainfo.xml" "${appdir}/usr/share/metainfo/gov.ornl.acorn.appdata.xml"
cp "${repo_root}/packaging/linux/gov.ornl.acorn-icon.svg" "${appdir}/gov.ornl.acorn.svg"
cp "${repo_root}/packaging/linux/gov.ornl.acorn-icon.svg" "${appdir}/usr/share/icons/hicolor/scalable/apps/gov.ornl.acorn.svg"
chmod 755 "${appdir}/AppRun" "${appdir}/usr/bin/acorn"

find "$appdir" -exec touch -h -d "@${SOURCE_DATE_EPOCH}" {} +
rm -f "$appimage_output" "$deb_output" "${appimage_output}.sha256" "${deb_output}.sha256"

cargo deb \
    --manifest-path "${repo_root}/crates/acorn-cli/Cargo.toml" \
    --no-build \
    --no-strip \
    --target "$target" \
    --deb-revision 1 \
    --output "$deb_output"

APPIMAGE_EXTRACT_AND_RUN=1 ARCH="$appimage_architecture" VERSION="$version" \
    "$appimagetool" \
    --runtime-file "$runtime" \
    --mksquashfs-opt -no-xattrs \
    --mksquashfs-opt -no-duplicates \
    --mksquashfs-opt -processors \
    --mksquashfs-opt 1 \
    "$appdir" \
    "$appimage_output"

write_sha256() {
    file="$1"
    directory="$(dirname -- "$file")"
    name="$(basename -- "$file")"
    if command -v sha256sum >/dev/null 2>&1; then
        (cd "$directory" && sha256sum "$name" > "${name}.sha256")
    elif command -v shasum >/dev/null 2>&1; then
        (cd "$directory" && shasum -a 256 "$name" > "${name}.sha256")
    else
        echo "A SHA-256 utility (sha256sum or shasum) is required" >&2
        exit 1
    fi
}

write_sha256 "$appimage_output"
write_sha256 "$deb_output"
