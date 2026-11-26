#!/bin/sh
set -eu

if [ "$#" -ne 5 ]; then
    echo "usage: package-acornd.sh VERSION TARGET ACORN_BINARY ACORND_BINARY OUTPUT_DIR" >&2
    exit 2
fi

version=$1
target=$2
acorn_binary=$3
acornd_binary=$4
output_dir=$5

if ! printf '%s\n' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "release version must be MAJOR.MINOR.PATCH" >&2
    exit 2
fi

case "$target" in
    x86_64-unknown-linux-musl|aarch64-unknown-linux-musl) ;;
    *) echo "unsupported acornd Linux release target: $target" >&2; exit 2 ;;
esac

: "${ACORND_SIGNING_KEY:?ACORND_SIGNING_KEY must name a protected Minisign secret-key file}"
minisign_bin=${MINISIGN_BIN:-minisign}
public_key=crates/acorn-cli/assets/acornd-release.pub

test -f "$acorn_binary"
test -f "$acornd_binary"
test -f "$ACORND_SIGNING_KEY"
test -f "$public_key"
mkdir -p "$output_dir"

archive_name="acornd-v${version}-${target}.tar.gz"
archive="$output_dir/$archive_name"
signature="$archive.minisig"
if [ -e "$archive" ] || [ -e "$signature" ]; then
    echo "refusing to replace an existing signed bundle: $archive" >&2
    exit 2
fi

work_dir=$(mktemp -d "$output_dir/.acornd-package.XXXXXXXX")
trap 'rm -rf "$work_dir"' EXIT HUP INT TERM
staged_archive="$work_dir/$archive_name"

python3 - "$version" "$target" "$acorn_binary" "$acornd_binary" "$staged_archive" <<'PY'
import gzip
import hashlib
import io
import json
import os
import re
import stat
import sys
import tarfile

version, target, acorn_path, acornd_path, output = sys.argv[1:]
if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version):
    raise SystemExit("release version must be MAJOR.MINOR.PATCH")

machine = 62 if target.startswith("x86_64-") else 183
binaries = []
sources = []
for name, path in (("acorn", acorn_path), ("acornd", acornd_path)):
    source = os.fdopen(os.open(path, os.O_RDONLY | os.O_NOFOLLOW), "rb")
    source_stat = os.fstat(source.fileno())
    if not stat.S_ISREG(source_stat.st_mode):
        raise SystemExit(f"{name} must be a regular file, not a symlink")
    header = source.read(20)
    if len(header) != 20 or header[:6] != b"\x7fELF\x02\x01" or int.from_bytes(header[18:20], "little") != machine:
        raise SystemExit(f"{name} is not a matching 64-bit Linux ELF binary")
    source.seek(0)
    hasher = hashlib.sha256()
    for chunk in iter(lambda: source.read(1024 * 1024), b""):
        hasher.update(chunk)
    digest = hasher.hexdigest()
    source.seek(0)
    binaries.append({"name": name, "sha256": digest, "size": source_stat.st_size})
    sources.append((name, source, source_stat.st_size))

manifest = json.dumps(
    {"formatVersion": 1, "version": version, "target": target, "binaries": binaries},
    sort_keys=True,
    separators=(",", ":"),
).encode("utf-8")

with open(output, "wb") as destination:
    with gzip.GzipFile(fileobj=destination, mode="wb", filename="", mtime=0) as compressed:
        with tarfile.open(fileobj=compressed, mode="w", format=tarfile.USTAR_FORMAT) as archive:
            for name, content, mode in (("manifest.json", io.BytesIO(manifest), 0o644),):
                member = tarfile.TarInfo(name)
                member.size = len(manifest)
                member.mode = mode
                member.mtime = 0
                archive.addfile(member, content)
            for name, source, size in sources:
                member = tarfile.TarInfo(name)
                member.size = size
                member.mode = 0o755
                member.mtime = 0
                archive.addfile(member, source)

for _, source, _ in sources:
    source.close()
PY

if [ -n "${ACORND_SIGNING_PASSPHRASE:-}" ]; then
    cargo xtask sign-acornd-bundle "$staged_archive"
else
    "$minisign_bin" -S -m "$staged_archive" -x "$staged_archive.minisig" -s "$ACORND_SIGNING_KEY" -t "ACORN acornd $version $target"
fi
"$minisign_bin" -V -H -m "$staged_archive" -x "$staged_archive.minisig" -p "$public_key"
mv "$staged_archive" "$archive"
mv "$staged_archive.minisig" "$signature"
printf '%s\n%s\n' "$archive" "$signature"
