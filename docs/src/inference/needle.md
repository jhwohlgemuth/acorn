# Needle sidecar

ACORN integrates the [Cactus Compute Needle 3](https://github.com/cactus-compute/needle) engine and `needle3.cact` model as a managed local subprocess. Needle selects tools; ACORN owns asset verification, process cleanup, execution policy, bounded tool dispatch, and result handling.

The integration is pinned to upstream revision `b274efcb211a9eef48c9a88da4b43bd569696a39`. ACORN uses Needle's native loopback-only `POST /complete` API, sets `NEEDLE_TELEMETRY=0` and `DO_NOT_TRACK=1`, and rejects unexpected schemes, non-loopback hosts, missing ports, and oversized responses.

## Run inference

```console
acorn infer "Return the running ACORN version"
```

The prompt can also be read from standard input:

```console
echo "Return the running ACORN version" | acorn infer
```

The first online run downloads the engine for the current platform, the model, and its Apache-2.0 license. ACORN verifies the published byte length and SHA-256 of every file before execution and caches them outside the repository. Download and verify the bundle without starting inference:

```console
acorn download needle
```

The command prints the engine path. The adjacent cache directory also contains `needle3.cact` and `LICENSE`. Once cached, the bundle works offline:

```console
acorn --offline infer "Return the running ACORN version"
```

ACORN executes calls only when Needle returns a successful `call` response with finite calibrated confidence at or above the configured threshold. Calls are not executed when Needle reports negation, ungrounded arguments, or unusable confidence, and an unknown response type or a failed response is a failure. `suppressed_calls` remain diagnostic and are never executed.

Withholding is an abstention rather than a failure, so `acorn infer` still prints the native response and exits successfully. The `withheld` field of the result reports why execution was declined, and `results` stays empty because nothing ran:

```json
"withheld": { "reason": "confidence", "confidence": 0.1137, "threshold": 0.8 }
```

Lower `--threshold` to let Needle act on a call it is unsure about.

```console
acorn infer "Validate this research activity" \
  --threshold 0.9 \
  --max-rounds 4
```

The managed process uses a 20-layer model depth, a 4,096-token output cap, and input-overflow failure. ACORN does not commit or distribute Needle assets in its source repository.

## Pinned assets

The common model and license are:

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `needle3.cact` | 35,335,380 | `c9d915eca282ed42d1a09b143b592adb4cc6744ffe2d294adf5cfc5548170c38` |
| `LICENSE` | 11,358 | `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30` |

Supported engines are:

| Platform | File | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| `windows-x86_64` | `needle.exe` | 1,276,928 | `8dfa55f2a1280f4c7f2b9d7396ca2d3d4580ccc05c3bb3da62101b405323b062` |
| `windows-arm64` | `needle.exe` | 1,105,408 | `7211171111709a769bc2e858fd9967635a9571fad23fba57f369703f902ad618` |
| `linux-x86_64` | `needle` | 1,246,880 | `5eb163c5ed33bd914c103ef8eba2134c7bb2d97bafafdd69f410a4a3100e8c37` |
| `linux-arm64` | `needle` | 1,166,664 | `b8d20caa7b8412c5900e05cf23d7a30dd84b7b89357d5408df30246ea9c808a6` |
| `macos-arm64` | `needle` | 824,744 | `bfcc14c38a7ebf670bf7ade3849b617d268120dc4cc75d3aa12c88bc7c0ba75c` |

## Export `tools.json`

Generate the deterministic catalog given to Needle:

```console
acorn export --tools --output ./bundle
```

This atomically creates `./bundle/tools.json` and refuses to overwrite an existing catalog. Use `--dry-run` to preview the destination. The Needle catalog is capped at 32 KiB and uses compact input projections for large research-activity arguments. ACORN parses those projected JSON strings into the same typed values used by the canonical handlers. Destructive graduation application is excluded from Needle exposure.

## Use a local ModelKit

Custom assets are disabled unless `--custom-location` is present. A Needle 3 bundle has this layout:

```text
needle-bundle/
├── LICENSE
├── needle-modelkit.json
├── needle3.cact
├── tools.json
└── needle             # needle.exe on Windows
```

The manifest uses ACORN's `v1.0.0` ModelKit shape and Needle sidecar protocol 2:

```json
{
  "manifestVersion": "v1.0.0",
  "sourceRevision": "b274efcb211a9eef48c9a88da4b43bd569696a39",
  "platform": "linux-x86_64",
  "sidecar": {
    "kind": "needle",
    "protocolVersion": 2
  },
  "runner": {
    "path": "needle",
    "size": 1246880,
    "sha256": "5eb163c5ed33bd914c103ef8eba2134c7bb2d97bafafdd69f410a4a3100e8c37"
  },
  "assets": [
    {
      "name": "license",
      "path": "LICENSE",
      "size": 11358,
      "sha256": "cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30"
    },
    {
      "name": "model",
      "path": "needle3.cact",
      "size": 35335380,
      "sha256": "c9d915eca282ed42d1a09b143b592adb4cc6744ffe2d294adf5cfc5548170c38"
    },
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

Use the exact host platform shown in the pinned-assets table. ACORN validates the manifest, protocol, platform, named model and license assets, file sizes, SHA-256 digests, exact tool catalog, and safe relative paths before starting the process. Protocol 1 Needle 2 bundles are rejected with repackaging guidance.

This is a hard cutover: the current release has no Needle 2 selector. Roll back by reinstalling the prior ACORN release and using its pinned Needle 2 bundle. ACORN does not delete the prior release's cache, so that rollback remains available unless an operator removes it.

```console
acorn infer "Return the ACORN version" --custom-location ./needle-bundle
```

The same directory can be supplied as a `file://` URI. An `https://` location may point to the manifest itself; ACORN downloads its safe relative files into the verified cache. The session regenerates its tool index from the current engine, model, protocol, and catalog identity, so a stale bundled index cannot change the exposed tools.

## Save and publish an OCI ModelKit to Harbor

Install ACORN and [ORAS 1.3 or newer](https://oras.land/docs/installation/). Harbor implements the OCI Distribution API, so ORAS can publish the ModelKit without a Harbor-specific client.

### Build and publish in one step

The repository scripts copy and hash the required Needle 3 files, export `tools.json`, write the manifest, publish the artifact, resolve its immutable digest, and print the complete `--custom-location` value.

On Linux or macOS:

```console
oras login example.ornl.gov
engine="$(acorn download needle)"
sh scripts/needle/create-oci-artifact.sh \
  "$(dirname "$engine")" \
  example.ornl.gov/research-enablement/needle:modelkit-2026-09 \
  ./needle-modelkit
```

On Windows PowerShell:

```powershell
oras login example.ornl.gov
$engine = acorn download needle | Select-Object -Last 1
./scripts/needle/create-oci-artifact.ps1 `
  -SourceDirectory (Split-Path $engine) `
  -Reference example.ornl.gov/research-enablement/needle:modelkit-2026-09 `
  -BundleDirectory ./needle-modelkit
```

Pass `PLATFORM` as the fourth shell argument or `-Platform` in PowerShell when packaging for another host. The source directory must contain the matching engine, `needle3.cact`, and `LICENSE`. Override `ACORN_BIN`, `ORAS_BIN`, or `NEEDLE_SOURCE_REVISION` for the shell script, or use the corresponding PowerShell parameters.

For a private certificate authority, set `ORAS_CA_FILE` or pass `-CaFile`. `ORAS_REGISTRY_CONFIG`/`-RegistryConfig` selects a non-default credential file. Development-only HTTP registries require `ORAS_PLAIN_HTTP=1` or `-PlainHttp`; do not use plaintext transport for production credentials.

### Save an OCI layout before publishing

Start with the validated directory described in [Use a local ModelKit](#use-a-local-modelkit). Run ORAS from inside that directory so the OCI layer titles remain plain file names:

```console
(cd ./needle-modelkit && oras push --oci-layout ../needle-modelkit.oci:local \
  --artifact-type application/vnd.acorn.needle.modelkit.v2 \
  needle-modelkit.json:application/vnd.acorn.needle.modelkit.manifest.v2+json \
  LICENSE:text/plain \
  needle3.cact:application/vnd.acorn.needle.model.v3 \
  tools.json:application/json \
  needle:application/vnd.acorn.needle.runner.v3)

oras resolve --oci-layout ./needle-modelkit.oci:local
```

The `local` tag exists only inside the OCI layout. To store the layout as one transferable file, archive its contents and verify that ORAS resolves the same manifest digest:

```console
tar -cf needle-modelkit.oci.tar -C needle-modelkit.oci .
oras resolve --oci-layout ./needle-modelkit.oci.tar:local
```

Use the materialized `./needle-modelkit` directory with a local ACORN command. ACORN does not treat the OCI layout or its tar archive as a local ModelKit directory:

```console
acorn --offline infer "Return the ACORN version" --custom-location ./needle-modelkit
```

### Push a saved layout to Harbor

Authenticate, copy the saved artifact, and resolve the registry reference:

```console
oras login example.ornl.gov

oras cp --from-oci-layout ./needle-modelkit.oci.tar:local example.ornl.gov/research-enablement/needle:modelkit

oras resolve --full-reference example.ornl.gov/research-enablement/needle:modelkit
```

### Use the published bundle with ACORN

Resolve the publication tag before downloading or running the bundle. ACORN and ORAS both accept the resulting immutable repository and digest reference.

#### Download and run locally

`acorn download needle` retrieves ACORN's pinned public bundle. Use ORAS to download a bundle published to a private registry:

```console
reference="$(oras resolve --full-reference example.ornl.gov/research-enablement/needle:modelkit)"

oras pull --output ./needle-modelkit-pulled "${reference}"

acorn --offline infer "Return the ACORN version" \
  --custom-location ./needle-modelkit-pulled
```

ORAS reconstructs `needle-modelkit.json`, the engine, model, license, and `tools.json` in the output directory. ACORN validates those files before starting Needle.

#### Run directly from Harbor

Prefix the resolved reference with `oci://` and pass it to `--custom-location`:

```console
reference="$(oras resolve --full-reference example.ornl.gov/research-enablement/needle:modelkit)"

acorn infer "Return the ACORN version" \
  --custom-location "oci://${reference}"
```

On PowerShell:

```powershell
$reference = oras resolve --full-reference example.ornl.gov/research-enablement/needle:modelkit

acorn infer "Return the ACORN version" --custom-location "oci://$reference"
```

ACORN requires the digest reference because tags can move. It uses the registry credentials saved by `oras login`. A custom-location failure never falls back to the public bundle.

#### Run from an HTTPS manifest

ACORN can also download a manifest and its relative files from an HTTPS location:

```console
acorn infer "Return the ACORN version" --custom-location https://example.ornl.gov/needle/needle-modelkit.json
```

The URL must point to `needle-modelkit.json`. The engine, model, license, and tool catalog must be available at the relative paths declared in that manifest. A Harbor repository URL does not expose this static directory shape; use its digest-pinned `oci://` reference instead.

For air-gapped systems, transfer the validated directory through the approved media process and use its local path with `--custom-location`; global `--offline` prevents registry and public downloads.

## Compare Needle generations

The checked-in harness runs the same held-out JSONL prompts against explicit Needle 2 and Needle 3 assets, with telemetry disabled for both:

```console
python3 scripts/needle/compare-generations.py \
  --v2-runner /path/to/needle2 \
  --v3-runner /path/to/needle3 \
  --v3-model /path/to/needle3.cact \
  --tools /path/to/tools.json
```

It records exact native response/tool/argument matches, effective outcomes after ACORN's confidence and argument-validation policy, refusals, confidence, native peak RAM, installed bytes, and cold/warm wall time in `target/needle-generation-comparison.json`. The default exit gates allow no policy-level accuracy regression or policy escapes, cap latency at 1.5 times the Needle 2 baseline and Needle 3 peak memory at 128 MiB, and require zero Needle 3 process failures. Override the numeric limits explicitly when conducting a new evaluation; retain the resulting report with release evidence rather than presenting upstream benchmark claims as ACORN measurements.

The 2026-09-23 macOS ARM64 migration smoke used three prompts with one cold and one warm repetition:

| Generation | Raw exact match | Policy outcome match | Cold / warm mean | Peak RAM | Installed assets |
| --- | ---: | ---: | ---: | ---: | ---: |
| Needle 2 | 66.67% | 66.67% | 576 / 470 ms | 33.7 MB | 14,610,568 bytes |
| Needle 3 | 33.33% | 100% | 307 / 304 ms | 108.4 MB | 36,171,482 bytes |

Needle 3 proposed calls for the two negative prompts, but ACORN rejected both: one was below the confidence threshold and the other failed typed `activity_json` validation. The run had zero policy escapes or process failures and passed the documented gates. This is migration smoke evidence, not a general benchmark. This comparison ran on macOS ARM64; release-host results are recorded below as they pass.

On 2026-10-21, Oklahoma validated Linux x86_64 with `acorn-cli 0.5.1` from the locally built image `sha256:66ca4809a28cb2eadebc866a8886deaa98c5853579a07eee7d2cb62e81ee1119`. The cached engine, model, and license matched the pinned byte lengths and SHA-256 values above. Online and cached offline inference returned the running version through the tool-result handoff. One online run took 1.07 seconds wall time and reported 78.0 MB peak RAM. The unsupported weather prompt made no tool call; a proposed logbook-graduation call at 0.2465 confidence was withheld at the 0.8 threshold; an offline HTTPS ModelKit location was rejected before download.

For the cancellation checks, the same image binary ran directly on the Oklahoma host so its process tree could be observed. After Needle reported its loopback server ready, separate SIGTERM and SIGINT runs made ACORN exit with a cancellation error. Each captured Needle child had no descendants and was gone after ACORN exited; each session directory was removed. Normal inference also left no Needle process. This verifies Linux x86_64 cleanup without relying on container teardown.

On 2026-10-29, Dante validated Linux ARM64 with a native `acorn-cli 0.5.3` release build from revision `790ef808` on Ubuntu 24.04.5 (`aarch64`, Docker 29.1.3). `acorn download needle` fetched the engine, model, and license for Needle revision `b274efcb211a9eef48c9a88da4b43bd569696a39`; their byte lengths and SHA-256 values matched the pinned Linux ARM64 engine and common assets above. Online and cached offline inference returned `0.5.3` through the tool-result handoff. Native wall times were 0.80 and 0.68 seconds, respectively; Needle reported 73.9 MB peak RAM for both. The engine bound to `127.0.0.1` on an ephemeral port. The unsupported weather prompt executed no tool, and a proposed logbook-graduation call at 0.2311 confidence was withheld at the 0.8 threshold. Offline mode rejected an HTTPS ModelKit location before download.

Native SIGTERM and SIGINT runs were interrupted after Needle reported its loopback server ready. ACORN reported cancellation in both runs; each captured Needle child exited and its session directory was removed. Normal inference also left no Needle child or session directory. This directly verifies Linux ARM64 process cleanup on Dante.

The `validate-needle-windows-amd64` GitLab job is a manual release-host smoke for a native Windows x86_64 shell runner tagged `windows-x86_64`. It consumes the Windows executable from `build-release`, downloads and verifies the pinned engine, model, and license, exercises online and cached offline version-tool handoff, checks the unsupported and destructive prompts, rejects an offline network location, and records normal process cleanup. Its `target/needle-windows-x86_64.json` artifact includes the runner, revision, hashes, elapsed times, and peak memory reported by Needle. A passing job alone does not establish cancellation cleanup; observe a native Ctrl+C run and confirm the Needle child and session directory are removed before recording Windows x86_64 as verified here.

## MCP relationship

`acorn serve mcp` exposes the canonical ACORN registry and its full schemas over MCP stdio. Needle's `tools.json` retains the same safe tool names, dispatcher, and handlers, while using narrower routing descriptions and compact inference-oriented input schemas where needed. MCP is the interoperability surface for an external host or agent; Needle is the local tool selector. Neither exposes arbitrary shell execution or recursive Needle inference.
