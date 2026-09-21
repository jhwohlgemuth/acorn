#!/bin/sh
set -eu

CARGO_ZIGBUILD_AARCH64_SHA256=25cf2cdc6cde848b33a761ff2cf46d1efe6ba7e1adfc4b6f3ee2d570cacb5419
CARGO_ZIGBUILD_VERSION=0.23.4
CARGO_ZIGBUILD_X86_64_SHA256=6b69963040818ca1ef573d8135a95aa902e17cf7f873c32c6426642dadfe7f59
ZIG_AARCH64_SHA256=f7a654acc967864f7a050ddacfaa778c7504a0eca8d2b678839c21eea47c992b
ZIG_VERSION=0.14.1
ZIG_X86_64_SHA256=24aeeec8af16c381934a6cd7d95c807a8cb2cf7df9fa40d359aa884195c4716c

install_root=${1:?Usage: install-zigbuild.sh INSTALL_ROOT}
bin_dir=${install_root}/bin
zig_dir=${install_root}/zig

if [ -x "${bin_dir}/cargo-zigbuild" ] && [ -x "${bin_dir}/zig" ]; then
    exit 0
fi

machine=$(uname -m)
case "${machine}" in
    aarch64 | arm64)
        cargo_zigbuild_sha256=${CARGO_ZIGBUILD_AARCH64_SHA256}
        target=aarch64-unknown-linux-gnu
        zig_arch=aarch64
        zig_sha256=${ZIG_AARCH64_SHA256}
        ;;
    amd64 | x86_64)
        cargo_zigbuild_sha256=${CARGO_ZIGBUILD_X86_64_SHA256}
        target=x86_64-unknown-linux-gnu
        zig_arch=x86_64
        zig_sha256=${ZIG_X86_64_SHA256}
        ;;
    *)
        echo "Unsupported cargo-zigbuild architecture: ${machine}" >&2
        exit 1
        ;;
esac

temporary_dir=$(mktemp -d)
trap 'rm -rf "${temporary_dir}"' EXIT HUP INT TERM

mkdir -p "${bin_dir}" "${zig_dir}"
curl -fsSL --retry 3 \
    "https://github.com/rust-cross/cargo-zigbuild/releases/download/v${CARGO_ZIGBUILD_VERSION}/cargo-zigbuild-${target}.tar.xz" \
    -o "${temporary_dir}/cargo-zigbuild.tar.xz"
printf '%s  %s\n' "${cargo_zigbuild_sha256}" "${temporary_dir}/cargo-zigbuild.tar.xz" | sha256sum --check
tar -xJf "${temporary_dir}/cargo-zigbuild.tar.xz" \
    -C "${bin_dir}" \
    --strip-components=1 \
    "cargo-zigbuild-${target}/cargo-zigbuild"

curl -fsSL --retry 3 \
    "https://ziglang.org/download/${ZIG_VERSION}/zig-${zig_arch}-linux-${ZIG_VERSION}.tar.xz" \
    -o "${temporary_dir}/zig.tar.xz"
printf '%s  %s\n' "${zig_sha256}" "${temporary_dir}/zig.tar.xz" | sha256sum --check
tar -xJf "${temporary_dir}/zig.tar.xz" -C "${zig_dir}" --strip-components=1
ln -sf "../zig/zig" "${bin_dir}/zig"
