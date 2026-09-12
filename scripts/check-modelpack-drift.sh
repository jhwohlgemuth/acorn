#!/bin/sh
set -eu

expected_spec="d7a6481cddb3bd30d646dd5ea5e700171baaeadb"
expected_kitops_commit="6b8162ae5da4d46f1d2af2beb43e7fb077f052f4"
expected_kitops_tag="v1.15.0"
repository_root="$(CDPATH='' cd -- "$(dirname "$0")/.." && pwd)"
temporary_root="$(mktemp -d)"
trap 'rm -rf "$temporary_root"' EXIT HUP INT TERM

checkout_kitops() {
  git -c advice.detachedHead=false clone --quiet --depth 1 --branch "$expected_kitops_tag" https://github.com/kitops-ml/kitops.git "$temporary_root/kitops"
  actual_commit="$(git -C "$temporary_root/kitops" rev-parse HEAD)"
  if [ "$actual_commit" != "$expected_kitops_commit" ]; then
    echo "KitOps tag moved: expected $expected_kitops_commit, found $actual_commit" >&2
    exit 1
  fi
}

if [ "${1:-}" = "upstream-test" ]; then
  checkout_kitops
  (cd "$temporary_root/kitops" && go test ./pkg/artifact)
  exit 0
fi

actual_spec="$(git ls-remote https://github.com/modelpack/model-spec.git refs/heads/main | awk '{print $1}')"
actual_kitops="$(git ls-remote --refs --tags https://github.com/kitops-ml/kitops.git 'refs/tags/v*' | sed 's#.*refs/tags/##' | sort -V | tail -n 1)"

if [ "$actual_spec" != "$expected_spec" ]; then
  echo "ModelPack specification drifted: expected $expected_spec, found $actual_spec" >&2
  exit 1
fi
if [ "$actual_kitops" != "$expected_kitops_tag" ]; then
  echo "KitOps release drifted: expected $expected_kitops_tag, found $actual_kitops" >&2
  exit 1
fi
checkout_kitops
diff -ru "$temporary_root/kitops/pkg/artifact/testdata/parameters" "$repository_root/tests/fixtures/modelkit/parameters"
diff -ru "$temporary_root/kitops/pkg/artifact/testdata/validation" "$repository_root/tests/fixtures/modelkit/validation"
(cd "$repository_root" && cargo nextest run -p acorn-schema modelkit)
echo "ModelPack and KitOps compatibility baselines and fixtures are current"
