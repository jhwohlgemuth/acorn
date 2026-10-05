#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" || "$(uname -m)" != "arm64" ]]; then
  echo "SLM host validation requires macOS ARM64" >&2
  exit 69
fi
xcode_version="$(xcodebuild -version | awk 'NR == 1 { print $2 }')"
swift_version="$(swift --version | sed -n 's/.*Apple Swift version \([0-9][0-9.]*\).*/\1/p' | head -n 1)"
swift_major="${swift_version%%.*}"
swift_remainder="${swift_version#*.}"
swift_minor="${swift_remainder%%.*}"
if [[ "${xcode_version}" != 26 && "${xcode_version}" != 26.* ]]; then
  echo "SLM runners require Xcode 26.x; found ${xcode_version}" >&2
  exit 1
fi
if [[ ! "${swift_major}" =~ ^[0-9]+$ || ! "${swift_minor}" =~ ^[0-9]+$ ]] || (( swift_major < 6 || (swift_major == 6 && swift_minor < 2) )); then
  echo "SLM runners require Swift 6.2 or newer; found ${swift_version}" >&2
  exit 1
fi
report_directory="${CARGO_TARGET_DIR:-target}/slm-host"
mkdir -p "${report_directory}"
printf 'print("slm-host-ready")\n' > "${report_directory}/smoke.swift"
swiftc "${report_directory}/smoke.swift" -o "${report_directory}/smoke"
"${report_directory}/smoke"
{
  echo "architecture=$(uname -m)"
  echo "macos=$(sw_vers -productVersion)"
  echo "swift=${swift_version}"
  echo "xcode=${xcode_version}"
} | tee "${report_directory}/summary.txt"
