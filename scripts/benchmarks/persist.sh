#!/usr/bin/env bash
set -euo pipefail

required_variables=(CI_API_V4_URL CI_PROJECT_ID CI_JOB_TOKEN CI_PIPELINE_ID CI_JOB_ID)
for variable in "${required_variables[@]}"; do
    if [[ -z "${!variable:-}" ]]; then
        echo "${variable} is required to persist benchmark history" >&2
        exit 1
    fi
done

package_name="${BENCHMARK_PACKAGE_NAME:-acorn-benchmarks}"
package_version="${CI_COMMIT_TAG:+${CI_COMMIT_TAG}-}${CI_PIPELINE_ID}-${CI_JOB_ID}"
target_dir="${CARGO_TARGET_DIR:-target}"
criterion_dir="${target_dir}/criterion"
output_dir="${target_dir}/benchmark-results"
transport_dir="${target_dir}/benchmark-transport"
project_api="${CI_API_V4_URL}/projects/${CI_PROJECT_ID}"
package_api="${project_api}/packages"
generic_api="${project_api}/packages/generic/${package_name}"

mkdir -p "${transport_dir}"
curl --fail --silent --show-error \
    --header "JOB-TOKEN: ${CI_JOB_TOKEN}" \
    --get \
    --data-urlencode "package_type=generic" \
    --data-urlencode "package_name=${package_name}" \
    --data-urlencode "order_by=created_at" \
    --data-urlencode "sort=desc" \
    --data-urlencode "per_page=20" \
    --output "${transport_dir}/packages.json" \
    "${package_api}"

previous_version="$(cargo xtask benchmark-history latest-package "${transport_dir}/packages.json")"
if [[ -n "${previous_version}" ]]; then
    curl --fail --silent --show-error \
        --header "JOB-TOKEN: ${CI_JOB_TOKEN}" \
        --output "${transport_dir}/previous-history.json" \
        "${generic_api}/${previous_version}/history.json"
    cargo xtask benchmark-history build "${criterion_dir}" "${output_dir}" "${transport_dir}/previous-history.json"
else
    cargo xtask benchmark-history build "${criterion_dir}" "${output_dir}"
fi

tar -czf "${output_dir}/criterion.tar.gz" -C "${criterion_dir}" .
for artifact in history.json run.json history.csv report.html criterion.tar.gz; do
    curl --fail --silent --show-error \
        --header "JOB-TOKEN: ${CI_JOB_TOKEN}" \
        --upload-file "${output_dir}/${artifact}" \
        "${generic_api}/${package_version}/${artifact}"
done

echo "Published benchmark history package ${package_name}/${package_version}"
