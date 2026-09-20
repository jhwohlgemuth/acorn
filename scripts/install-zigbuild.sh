#!/bin/sh
set -eu

CARGO_ZIGBUILD_SHA256=6b69963040818ca1ef573d8135a95aa902e17cf7f873c32c6426642dadfe7f59
CARGO_ZIGBUILD_VERSION=0.23.4
ZIG_SHA256=24aeeec8af16c381934a6cd7d95c807a8cb2cf7df9fa40d359aa884195c4716c
ZIG_VERSION=0.14.1

install_root=${1:?Usage: install-zigbuild.sh INSTALL_ROOT}
bin_dir=${install_root}/bin
zig_dir=${install_root}/zig

if [ -x "${bin_dir}/cargo-zigbuild" ] && [ -x "${bin_dir}/zig" ]; then
    exit 0
fi

temporary_dir=$(mktemp -d)
trap 'rm -rf "${temporary_dir}"' EXIT HUP INT TERM

mkdir -p "${bin_dir}" "${zig_dir}"
curl -fsSL --retry 3 \
    "https://github.com/rust-cross/cargo-zigbuild/releases/download/v${CARGO_ZIGBUILD_VERSION}/cargo-zigbuild-x86_64-unknown-linux-gnu.tar.xz" \
    -o "${temporary_dir}/cargo-zigbuild.tar.xz"
printf '%s  %s\n' "${CARGO_ZIGBUILD_SHA256}" "${temporary_dir}/cargo-zigbuild.tar.xz" | sha256sum --check
tar -xJf "${temporary_dir}/cargo-zigbuild.tar.xz" \
    -C "${bin_dir}" \
    --strip-components=1 \
    cargo-zigbuild-x86_64-unknown-linux-gnu/cargo-zigbuild

curl -fsSL --retry 3 \
    "https://ziglang.org/download/${ZIG_VERSION}/zig-x86_64-linux-${ZIG_VERSION}.tar.xz" \
    -o "${temporary_dir}/zig.tar.xz"
printf '%s  %s\n' "${ZIG_SHA256}" "${temporary_dir}/zig.tar.xz" | sha256sum --check
tar -xJf "${temporary_dir}/zig.tar.xz" -C "${zig_dir}" --strip-components=1
ln -sf "../zig/zig" "${bin_dir}/zig"
