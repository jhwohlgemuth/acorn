#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
docs_dir="${root_dir}/docs/src"
summary="${docs_dir}/SUMMARY.md"
command_index="${docs_dir}/cli-reference/commands/README.md"
generated_help="${root_dir}/CommandLineHelp.md"
failed=0

commands="$({
    awk '
        /^###### \*\*Subcommands:\*\*/ { in_commands = 1; next }
        in_commands && /^###### / { exit }
        in_commands && /^\* `/ {
            sub(/^\* `/, "")
            sub(/`.*/, "")
            print
        }
    ' "${generated_help}"
})"

if [[ -z "${commands}" ]]; then
    echo "No top-level commands found in CommandLineHelp.md"
    exit 1
fi

while IFS= read -r command; do
    page="${docs_dir}/cli-reference/commands/${command}.md"
    if [[ ! -f "${page}" ]]; then
        echo "Missing command page: ${page#"${root_dir}/"}"
        failed=1
    fi
    if ! grep -Fq "(./cli-reference/commands/${command}.md)" "${summary}"; then
        echo "Missing SUMMARY entry for command: ${command}"
        failed=1
    fi
    if ! grep -Fq "(./${command}.md)" "${command_index}"; then
        echo "Missing command index entry for command: ${command}"
        failed=1
    fi
done <<< "${commands}"

for page in "${docs_dir}"/cli-reference/commands/*.md; do
    page_name="$(basename "${page}" .md)"
    case "${page_name}" in
        README|model-workflows) continue ;;
    esac
    if ! grep -Fxq "${page_name}" <<< "${commands}"; then
        echo "Command page has no generated top-level command: ${page#"${root_dir}/"}"
        failed=1
    fi
done

if grep -ERni "work in progress|WIP" "${docs_dir}/cli-reference/commands" --include='*.md'; then
    echo "Command documentation must not contain WIP markers"
    failed=1
fi

while IFS= read -r -d '' file; do
    while IFS= read -r markdown_link; do
        target="${markdown_link#']('}"
        target="${target%')'}"
        target="${target%%#*}"
        target="${target%%\?*}"
        target="${target#<}"
        target="${target%>}"
        case "${target}" in
            ""|http://*|https://*|mailto:*|tel:*|data:*) continue ;;
            /*) candidate="${docs_dir}/${target#/}" ;;
            *) candidate="$(dirname "${file}")/${target}" ;;
        esac
        if [[ "${candidate}" == *.html ]]; then
            candidate="${candidate%.html}.md"
        fi
        if [[ -d "${candidate}" ]]; then
            candidate="${candidate}/README.md"
        elif [[ ! -e "${candidate}" && "$(basename "${candidate}")" == "index.md" ]]; then
            candidate="$(dirname "${candidate}")/README.md"
        fi
        if [[ ! -e "${candidate}" ]]; then
            echo "Broken local link in ${file#"${root_dir}/"}: ${target}"
            failed=1
        fi
    done < <(grep -Eo '\]\([^)]+\)' "${file}" || true)
done < <(find "${docs_dir}" -type f -name '*.md' -print0)

exit "${failed}"
