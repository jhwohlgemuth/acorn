# KitOps v1.15.0 fixtures

These small compatibility fixtures are copied verbatim from KitOps v1.15.0 commit `6b8162ae5da4d46f1d2af2beb43e7fb077f052f4`:

- `parameters-data-types.yaml`: `pkg/artifact/testdata/parameters/test-data-types.yaml`
- `mcpserver-valid.yaml`: `pkg/artifact/testdata/validation/mcpserver-valid.yaml`

KitOps is licensed under Apache-2.0. The fixtures exercise its producer-owned YAML/JSON normalization and validation expectations without invoking the `kit` executable from `acorn-schema`.

`Kitfile` exercises every supported authoring field. `modelkit.json` is its resolved OCI configuration counterpart and includes complete layer identity.
