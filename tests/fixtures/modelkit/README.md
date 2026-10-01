# KitOps v1.15.0 fixtures

The `parameters/` and `validation/` directories are copied verbatim from KitOps v1.15.0 commit `6b8162ae5da4d46f1d2af2beb43e7fb077f052f4`:

- `parameters/`: all five files from `pkg/artifact/testdata/parameters/`
- `validation/`: all 39 files from `pkg/artifact/testdata/validation/`

The `check/` directory contains ACORN-owned, directly executable Kitfiles for CLI integration tests:

- `codec-issue/`: one unknown-field decoding issue
- `many-schema-issues/`: several independent semantic validation issues
- `one-schema-issue/`: one semantic validation issue
- `quality-issues/`: three packaging issues, including a malformed MCP bundle
- `three-schema-issues/`: three semantic validation issues in a resolved `modelkit.json`

KitOps is licensed under Apache-2.0. The fixtures exercise its producer-owned YAML/JSON normalization and validation expectations without invoking the `kit` executable from `acorn-schema`. Refresh both directories from the pinned source tree, update the commit here and in `scripts/check-modelpack-drift.sh`, then run that script to verify byte-for-byte provenance and both implementations' focused tests.

`Kitfile` exercises every supported authoring field. `modelkit.json` is its resolved OCI configuration counterpart and includes complete layer identity.

ACORN intentionally reports stable path-addressed validation codes instead of reproducing KitOps error prose. It also enforces the documented non-empty-content requirement, cleans Windows and POSIX paths portably, includes prompt paths in duplicate detection, and includes MCP Bundles in resolved layer-identity completeness.
