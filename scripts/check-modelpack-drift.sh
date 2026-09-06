#!/bin/sh
set -eu

expected_spec="d7a6481cddb3bd30d646dd5ea5e700171baaeadb"
expected_kitops="v1.15.0"
actual_spec="$(git ls-remote https://github.com/modelpack/model-spec.git refs/heads/main | awk '{print $1}')"
actual_kitops="$(git ls-remote --refs --tags https://github.com/kitops-ml/kitops.git 'refs/tags/v*' | sed 's#.*refs/tags/##' | sort -V | tail -n 1)"

if [ "$actual_spec" != "$expected_spec" ]; then
  echo "ModelPack specification drifted: expected $expected_spec, found $actual_spec" >&2
  exit 1
fi
if [ "$actual_kitops" != "$expected_kitops" ]; then
  echo "KitOps release drifted: expected $expected_kitops, found $actual_kitops" >&2
  exit 1
fi
echo "ModelPack and KitOps compatibility baselines are current"
