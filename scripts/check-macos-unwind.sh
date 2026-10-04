#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "macOS unwind validation requires Darwin" >&2
  exit 69
fi
maximum_bytes="${MAX_EH_FRAME_BYTES:-14680064}"
target_directory="${CARGO_TARGET_DIR:-target}"
report_directory="${target_directory}/macos-unwind"
binary_path="${target_directory}/debug/acorn"
build_log="${report_directory}/build.log"
mkdir -p "${report_directory}"
cargo build --locked --package acorn-cli --bin acorn 2>&1 | tee "${build_log}"
size -m "${binary_path}" | tee "${report_directory}/size.txt"
otool -l "${binary_path}" | tee "${report_directory}/otool.txt" >/dev/null
size_bytes="$(awk '$1 == "Section" && $2 == "__eh_frame:" { print $3; exit }' "${report_directory}/size.txt")"
otool_hex="$(awk '$1 == "sectname" && $2 == "__eh_frame" { found=1; next } found && $1 == "size" { print $2; exit }' "${report_directory}/otool.txt")"
if [[ ! "${size_bytes}" =~ ^[0-9]+$ || ! "${otool_hex}" =~ ^0x[0-9a-fA-F]+$ ]]; then
  echo "Unable to read __eh_frame from size and otool output" >&2
  exit 65
fi
otool_bytes="$((otool_hex))"
if [[ "${size_bytes}" -ne "${otool_bytes}" ]]; then
  echo "size and otool disagree about __eh_frame (${size_bytes} versus ${otool_bytes} bytes)" >&2
  exit 65
fi
{
  echo "binary=${binary_path}"
  echo "eh_frame_bytes=${size_bytes}"
  echo "maximum_bytes=${maximum_bytes}"
} | tee "${report_directory}/summary.txt"
if grep -q "__eh_frame section too large" "${build_log}"; then
  echo "Apple linker reported that __eh_frame is too large for compact unwind offsets" >&2
  exit 1
fi
if [[ "${size_bytes}" -gt "${maximum_bytes}" ]]; then
  echo "__eh_frame is ${size_bytes} bytes; the limit is ${maximum_bytes} bytes" >&2
  exit 1
fi
