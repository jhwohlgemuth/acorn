#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "macOS unwind validation requires Darwin" >&2
  exit 69
fi

readonly maximum_bytes="${MAX_EH_FRAME_BYTES:-14680064}"
readonly target_directory="${CARGO_TARGET_DIR:-target}"
readonly report_directory="${target_directory}/macos-unwind"
readonly summary_path="${report_directory}/summary.txt"

mkdir -p "${report_directory}"

section_bytes() {
  local section="$1"
  local report="$2"
  awk -v target="${section}:" '$1 == "Section" && $2 == target { print $3; exit }' "${report}"
}

measure_artifact() {
  local label="$1"
  local binary_path="$2"
  local build_log="$3"
  local size_report="${report_directory}/${label}-size.txt"
  local otool_report="${report_directory}/${label}-otool.txt"
  local eh_frame_bytes
  local gcc_except_tab_bytes
  local otool_bytes
  local otool_hex
  local text_bytes
  local total_bytes
  local unwind_info_bytes

  size -m "${binary_path}" | tee "${size_report}"
  otool -l "${binary_path}" | tee "${otool_report}" >/dev/null
  text_bytes="$(section_bytes __text "${size_report}")"
  gcc_except_tab_bytes="$(section_bytes __gcc_except_tab "${size_report}")"
  unwind_info_bytes="$(section_bytes __unwind_info "${size_report}")"
  eh_frame_bytes="$(section_bytes __eh_frame "${size_report}")"
  otool_hex="$(awk '$1 == "sectname" && $2 == "__eh_frame" { found=1; next } found && $1 == "size" { print $2; exit }' "${otool_report}")"
  total_bytes="$(stat -f '%z' "${binary_path}")"

  if [[ ! "${eh_frame_bytes}" =~ ^[0-9]+$ || ! "${otool_hex}" =~ ^0x[0-9a-fA-F]+$ ]]; then
    echo "Unable to read __eh_frame from size and otool output for ${label}" >&2
    exit 65
  fi
  otool_bytes="$((otool_hex))"
  if [[ "${eh_frame_bytes}" -ne "${otool_bytes}" ]]; then
    echo "size and otool disagree about ${label} __eh_frame (${eh_frame_bytes} versus ${otool_bytes} bytes)" >&2
    exit 65
  fi

  {
    echo "${label}_binary=${binary_path}"
    echo "${label}_text_bytes=${text_bytes}"
    echo "${label}_gcc_except_tab_bytes=${gcc_except_tab_bytes}"
    echo "${label}_unwind_info_bytes=${unwind_info_bytes}"
    echo "${label}_eh_frame_bytes=${eh_frame_bytes}"
    echo "${label}_total_bytes=${total_bytes}"
  } | tee -a "${summary_path}"

  if grep -q "__eh_frame section too large" "${build_log}"; then
    echo "Apple linker reported that ${label} __eh_frame is too large for compact unwind offsets" >&2
    exit 1
  fi
  if [[ "${eh_frame_bytes}" -gt "${maximum_bytes}" ]]; then
    echo "${label} __eh_frame is ${eh_frame_bytes} bytes; the limit is ${maximum_bytes} bytes" >&2
    exit 1
  fi
}

readonly cli_binary="${target_directory}/debug/acorn"
readonly cli_build_log="${report_directory}/cli-build.log"
readonly test_build_json="${report_directory}/test-build.json"
readonly test_build_log="${report_directory}/test-build.log"
readonly test_build_stderr="${report_directory}/test-build.stderr"

{
  echo "commit=$(git rev-parse HEAD)"
  if git diff --quiet && git diff --cached --quiet; then
    echo "worktree=clean"
  else
    echo "worktree=dirty"
  fi
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "linker=$(xcrun ld -v 2>&1 | awk '/PROGRAM:ld/ && !found { print; found=1 }')"
  echo "architecture=$(uname -m)"
  echo "profiles=dev,test"
  echo "dependency_opt_level=2"
  echo "acorn_lib_opt_level=1"
  echo "measurement_mode=${MEASUREMENT_MODE:-incremental}"
  echo "maximum_bytes=${maximum_bytes}"
} | tee "${summary_path}"

cli_started="${SECONDS}"
cargo build --locked --package acorn-cli --bin acorn 2>&1 | tee "${cli_build_log}"
echo "cli_build_seconds=$((SECONDS - cli_started))" | tee -a "${summary_path}"
measure_artifact cli "${cli_binary}" "${cli_build_log}"

test_started="${SECONDS}"
cargo test --locked --package acorn-cli --bin acorn --no-run --message-format=json >"${test_build_json}" 2>"${test_build_stderr}"
jq -r 'select(.reason == "compiler-message") | .message.rendered // empty' "${test_build_json}" | tee "${test_build_log}"
tee -a "${test_build_log}" <"${test_build_stderr}" >&2
test_binary="$(jq -r 'select(.reason == "compiler-artifact" and .profile.test == true and (.target.kind | index("bin") != null)) | .executable // empty' "${test_build_json}" | tail -n 1)"
if [[ -z "${test_binary}" || ! -x "${test_binary}" ]]; then
  echo "Unable to locate the acorn CLI unit-test artifact" >&2
  exit 65
fi
echo "test_build_seconds=$((SECONDS - test_started))" | tee -a "${summary_path}"
measure_artifact test "${test_binary}" "${test_build_log}"
