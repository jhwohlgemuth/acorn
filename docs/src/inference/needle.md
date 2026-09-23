# Needle sidecar

ACORN integrates the [Cactus Compute Needle 2](https://github.com/cactus-compute/needle) standalone runner as a managed local subprocess. Needle is used for compact tool selection; ACORN owns downloading, verification, process cleanup, confidence policy, bounded tool execution, and result handling.

Needle's standalone interface is native rather than OpenAI-compatible. ACORN starts its documented local server and sends `POST /complete` requests through the loopback-only `acorn::io::api` client. The client rejects non-loopback hosts, unexpected schemes, oversized responses, and endpoints without an explicit port.

## Run inference

```console
acorn infer "Return the running ACORN version"
```

The first online run downloads the runner for the current supported platform from the immutable Needle revision pinned by ACORN. It verifies both the published byte length and SHA-256 before execution and caches the result outside the repository. Later runs can reuse that exact binary with global `--offline`:

Download and verify the engine without starting inference:

```console
acorn download needle
```

```console
acorn --offline infer "Return the running ACORN version"
```

Tool calls execute only when Needle supplies a confidence at or above the default `0.8` threshold. Both controls are configurable:

```console
acorn infer "Validate this research activity" \
  --threshold 0.9 \
  --max-rounds 4
```

ACORN does not commit or distribute Needle binaries in its source repository. Keeping upstream executables in the verified cache avoids repository bloat, platform ambiguity, and accidental redistribution of stale artifacts.

## Export `tools.json`

Generate the exact deterministic catalog given to Needle:

```console
acorn export --tools --output ./bundle
```

This atomically creates `./bundle/tools.json` and refuses to overwrite an existing catalog. Use `--dry-run` to preview the destination. Each entry contains Needle's native `name`, `description`, and JSON Schema `parameters` fields.

## Use a local ModelKit

Custom assets are disabled unless `--custom-location` is present. A local bundle is a directory with this layout:

```text
needle-bundle/
├── needle-modelkit.json
├── tools.json
└── needle             # needle.exe on Windows
```

Create `needle-modelkit.json` with safe bundle-relative paths:

```json
{
  "manifestVersion": "v1.0.0",
  "sourceRevision": "98fbd955b0347e78059be0c253cc1ffa09b87bc7",
  "platform": "linux-x86_64",
  "sidecar": {
    "kind": "needle",
    "protocolVersion": 1
  },
  "runner": {
    "path": "needle",
    "size": 14884256,
    "sha256": "REPLACE_WITH_SHA256"
  },
  "assets": [
    {
      "name": "tools",
      "path": "tools.json",
      "size": 1234,
      "sha256": "REPLACE_WITH_SHA256"
    }
  ],
  "tools": "tools"
}
```

`manifestVersion` follows ACORN's pinned Kitfile manifest version; the outer OCI manifest retains the OCI-defined numeric `schemaVersion: 2`. Use the exact platform value expected by the current host: `windows-x86_64`, `windows-arm64`, `linux-x86_64`, `linux-arm64`, or `macos-arm64`. The standalone runner already contains Needle's model; its published CLI does not accept separate tuned weights. ACORN validates the manifest, sidecar protocol, platform, byte size, SHA-256 digest, exact tool catalog, and every relative path before starting the process.

```console
acorn infer "Return the ACORN version" --custom-location ./needle-bundle
```

The same directory can be supplied as a `file://` URI. An `https://` location may point to the
manifest itself; ACORN downloads its safe relative runner and every declared asset into the verified
cache before starting Needle.

The session always regenerates its tool index from ACORN's current canonical catalog, so a stale bundled index cannot change the exposed tools.

## Publish an OCI-compatible ModelKit to Harbor

The bundle can be published as an OCI artifact that preserves the same files. Install ACORN and [ORAS 1.3 or newer](https://oras.land/docs/installation/), authenticate to the private registry, and use the repository script for your shell. The scripts refuse to overwrite the output directory, export the exact active ACORN tool catalog, copy and hash the supplied runner and tools, write `needle-modelkit.json`, push all three files, resolve the immutable manifest digest, and print the complete `--custom-location` value.

On Linux or macOS:

```console
oras login harbor.example.org
runner="$(acorn download needle)"
sh scripts/needle/create-oci-artifact.sh \
  "$runner" \
  harbor.example.org/research/needle:modelkit-2026-09 \
  ./needle-modelkit
```

On Windows PowerShell:

```powershell
oras login harbor.example.org
$runner = acorn download needle | Select-Object -Last 1
./scripts/needle/create-oci-artifact.ps1 `
  -Runner $runner `
  -Reference harbor.example.org/research/needle:modelkit-2026-09 `
  -BundleDirectory ./needle-modelkit
```

Pass `PLATFORM` as the fourth shell argument or `-Platform` in PowerShell when building for a host other than the machine running the script. Supported values are `windows-x86_64`, `windows-arm64`, `linux-x86_64`, `linux-arm64`, and `macos-arm64`; the supplied runner must actually match that platform. Override `ACORN_BIN`, `ORAS_BIN`, or `NEEDLE_SOURCE_REVISION` for the shell script, or use the corresponding PowerShell parameters.

For publication to Harbor with a private certificate authority, set `ORAS_CA_FILE` for the shell script or pass `-CaFile` in PowerShell. `ORAS_REGISTRY_CONFIG`/`-RegistryConfig` selects a non-default credential file for the script. Development-only HTTP registries require `ORAS_PLAIN_HTTP=1` or `-PlainHttp`; do not use plaintext transport for production credentials. These settings affect publication only: ACORN's subsequent pull uses ORAS's default credential store and the system trust store, so perform `oras login` without a custom config and install a private Harbor CA into the host trust store before using `--custom-location`.

The final output resembles:

```text
Bundle: ./needle-modelkit
Custom location: oci://harbor.example.org/research/needle@sha256:0123...
Run: acorn infer "Return the ACORN version" --custom-location oci://harbor.example.org/research/needle@sha256:0123...
```

Keep the local bundle to test the same content without a registry:

```console
acorn infer "Return the ACORN version" --custom-location ./needle-modelkit
```

### Manual publication

To inspect or reproduce the script manually, first create `tools.json` and the manifest using the local ModelKit instructions above. Then push the three files with ORAS:

```console
oras login harbor.example.org

oras push harbor.example.org/research/needle:modelkit-2026-09 \
  --artifact-type application/vnd.acorn.needle.modelkit.v1 \
  needle-modelkit.json:application/vnd.acorn.needle.modelkit.manifest.v1+json \
  tools.json:application/json \
  needle:application/vnd.acorn.needle.runner

oras resolve harbor.example.org/research/needle:modelkit-2026-09
```

Run with the immutable digest printed by `oras resolve`, never the mutable publication tag. The scripts use `oras resolve --full-reference` to avoid manually reconstructing the repository name:

```console
acorn infer "Validate this" \
  --custom-location oci://harbor.example.org/research/needle@sha256:REPLACE_WITH_MANIFEST_DIGEST
```

ACORN rejects tags for custom bundle pulls. It delegates authentication to ORAS-compatible credential configuration, pulls into a staging directory, publishes the cache entry atomically, then applies the same local ModelKit validation. A custom-location failure never falls back to the public runner.

This layout is an ACORN Needle ModelKit carried by the OCI Distribution protocol; it is not a CNCF ModelPack and does not claim interchangeability with arbitrary KitOps manifests. If an organization also packages the directory with KitOps, retain `needle-modelkit.json` and the same root-relative file names, then verify that `oras pull` reconstructs that layout before relying on it.

For production Harbor deployments, attach an SBOM and sign the digest with the organization's existing OCI signing policy. ACORN currently requires the digest and validates every declared file size and checksum, but signature-policy enforcement remains a registry/deployment responsibility.

For an air-gapped system, transfer the validated directory through the approved media process and use its local path with `--custom-location`; global `--offline` prevents registry or public download access.

## MCP relationship

`acorn serve mcp` exposes the canonical ACORN registry over MCP stdio. `tools/list` and Needle's `tools.json` are generated from the same names, descriptions, schemas, and exposure metadata. `tools/call` and Needle-selected calls reach the same dispatcher and handlers.

MCP is the interoperability surface for an external host or agent. Needle is the local inference implementation that decides whether to call a tool. Neither protocol bypasses ACORN's allowlist, and neither exposes arbitrary shell execution or recursive Needle inference.
