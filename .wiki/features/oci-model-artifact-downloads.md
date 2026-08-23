---
okf_version: "0.1"
type: Plan
title: OCI model artifact downloads
description: Add portable OCI model artifact resolution and ORAS-backed downloads, with optional Harbor discovery and metadata integration.
difficulty: high
impact: high
package:
  - acorn-lib
  - acorn-cli
tags:
  - oci
  - oras
  - harbor
  - model
  - download
status: planned
---

# OCI model artifact downloads

## Goal

Extend `acorn download model` so an explicit OCI artifact reference can resolve and materialize model files from any OCI Distribution-compatible registry, including Harbor. Use the ORAS CLI as the initial content-transfer backend, add Harbor API support for registry-specific discovery and metadata, and preserve the existing Hugging Face/GGUF behavior for unqualified model selectors.

The first production scope is pull-only. Pushing, deleting, retagging, registry administration, vulnerability scanning, signature-policy enforcement, and recursive referrer downloads are separate capabilities and are not required for this plan.

## Difficulty and impact assessment

**Overall difficulty: high.** Calling `oras pull` is individually straightforward, but a production implementation must add an unambiguous reference grammar, keep secrets out of process arguments and logs, model tag-versus-digest identity, validate manifest media types and paths, preserve dry-run/offline/filter behavior, isolate partial downloads, and work consistently across Windows, Linux, and macOS. Harbor adds a second authenticated API surface with encoded project/repository/reference path segments and paginated results.

**Overall impact: high.** The feature would let ACORN consume internally governed and air-gap-adjacent model artifacts without republishing them to Hugging Face, reuse Harbor retention/immutability/access controls, and make model distribution portable across OCI registries. It also introduces an external executable dependency for this path, expands the public configuration schema, and increases the security consequences of model download handling.

| Work area | Difficulty | Impact | Reason |
| --- | --- | --- | --- |
| OCI reference and configuration model | medium | high | It defines backward-compatible routing and persisted provenance for every later phase. |
| ORAS command adapter and secure authentication | medium | high | The process call is small, but cross-platform lookup, version gating, secret redaction, cancellation, and error mapping are user-visible. |
| Manifest validation and safe materialization | high | high | OCI artifacts can contain arbitrary media types and paths; ACORN must prevent traversal, partial state, and accidental container-image extraction. |
| Harbor artifact discovery API | high | medium | It improves browsing and tag resolution for Harbor but is not needed for a known OCI reference. |
| Native OCI Distribution client | high | medium | It would remove the runtime ORAS dependency, but duplicates mature authentication, redirect, digest, and registry-compatibility behavior. |
| Documentation, fixtures, and registry integration tests | medium | high | Real interoperability and authentication failures are difficult to prove with unit tests alone. |

## Current evidence

- `crates/acorn-cli/src/cli/arguments/mod.rs` describes model inputs as IDs/names, local paths, or direct URLs. Unqualified remote identifiers flow into Hugging Face fallback search, so treating every `host/path:tag` string as OCI would be a breaking ambiguity.
- `crates/acorn-cli/src/commands/download/model/plan.rs` already separates selector planning, dry-run resolution, download execution, and post-download metadata. OCI should enter through this pipeline as another typed remote source rather than as a second command implementation.
- `ModelDownloadPlan` currently carries Hugging Face-specific revision and resolution behavior, while direct HTTP files use the shared `DownloadItem`/`DownloadItems` transport. An OCI artifact is a manifest plus one or more content-addressed blobs and does not fit the single-URL branch.
- `crates/acorn-lib/src/lib.rs::Repository` supports Git, GitHub, GitLab, and Hugging Face but has no OCI variant. `Location` also has no recognized OCI scheme.
- `crates/acorn-lib/src/io/config.rs::ModelEntryOptions` already carries source, revision, authentication requirement, filters, quantization, and local materialization options. This is the natural compatibility boundary for configured OCI sources.
- `crates/acorn-lib/src/util/cmd.rs` already centralizes external-command execution, rendering, and sensitive-argument redaction. The ORAS adapter should reuse and extend this boundary rather than invoke a shell string.
- OCI artifacts can use an image manifest with `artifactType`, arbitrary config and layer media types, annotations, and content digests. Consumers must tolerate unknown artifact types at the storage layer, but ACORN still needs an explicit model-download acceptance policy before materialization. [OCI image manifest specification](https://github.com/opencontainers/image-spec/blob/main/manifest.md)
- The portable Distribution API resolves manifests by tag or digest and downloads blobs by digest; blob endpoints may redirect and clients are expected to verify content identity. [OCI Distribution HTTP API V2](https://distribution.github.io/distribution/spec/api/)
- ORAS 1.3 provides stable `manifest fetch` and `pull` commands, supports tag and digest references, writes to an explicit output directory, rejects path traversal by default, supports custom CA/client certificates, and can read passwords or identity tokens from standard input. [ORAS manifest fetch](https://oras.land/docs/commands/oras_manifest_fetch/), [ORAS pull](https://oras.land/docs/commands/oras_pull/)
- ORAS can reuse Docker-compatible registry credential configuration without requiring a running Docker daemon. [ORAS authentication](https://oras.land/docs/how_to_guides/authentication/)
- Harbor's v2 API can list artifacts and retrieve one artifact by tag or digest under a project and repository. This is useful for discovery and Harbor metadata, but the portable registry protocol remains the correct content-transfer boundary. [Harbor v2 API](https://github.com/goharbor/harbor/blob/main/api/v2.0/swagger.yaml)

## Recommendation

Implement this as two cooperating adapters behind one typed OCI plan:

1. Use ORAS for manifest inspection and content pull in the first production release.
2. Use the Harbor v2 API only when a configured registry profile identifies the host as Harbor and ACORN needs discovery, tag selection, labels, or Harbor-specific metadata.
3. Keep the OCI Distribution reference and resolved digest as the canonical identity. Harbor metadata must enrich that identity, not replace it.
4. Do not implement a native blob downloader in the first release. Reassess it only after the ORAS-backed path has stable fixtures and measured demand for a dependency-free binary.

This division delivers generic Harbor downloads immediately because Harbor implements OCI Distribution, while avoiding a vendor-specific download path. It also gives ACORN a replaceable `OciTransport` boundary if a Rust-native client is justified later.

## Proposed user contract

### References

- Require `oci://` for positional OCI sources, for example `oci://harbor.example.org/ai/models/qwen:Q4_K_M` or `oci://harbor.example.org/ai/models/qwen@sha256:...`.
- Strip only the `oci://` routing prefix before passing the validated reference to ORAS.
- Continue treating unqualified `owner/repository` selectors as Hugging Face inputs.
- Accept a tag or digest, but never silently default to `latest`. A missing reference is an actionable error unless a configured model entry supplies `revision`.
- Resolve tags to a manifest digest during planning. Persist and report both the requested tag and resolved digest so dry runs, logs, database records, and synchronization are reproducible.
- Reject URL user-info, fragments, malformed digests, ambiguous hostless references, and insecure HTTP unless a registry profile explicitly enables development-only plain HTTP.

### Example use

These commands show the planned interface; OCI support is not yet implemented. The registry names, repositories, tags, and digests are illustrative.

Pull a model artifact from Harbor or any other OCI Distribution-compatible registry by tag:

```sh
acorn download model oci://harbor.example.org/ai/models/qwen:Q4_K_M
```

Pull an immutable artifact by digest:

```sh
acorn download model oci://harbor.example.org/ai/models/qwen@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
```

Resolve a mutable tag to its digest and preview the selected files without writing:

```sh
acorn download model oci://harbor.example.org/ai/models/qwen:Q4_K_M --dry-run
```

Emit the same dry-run resolution as structured JSON for automation:

```sh
acorn download model oci://harbor.example.org/ai/models/qwen:Q4_K_M --dry-run --raw --format json
```

Select a quantization and restrict the materialized files using the existing model filters:

```sh
acorn download model oci://harbor.example.org/ai/models/qwen:v1 --quantization Q4_K_M --filter '\.gguf$|tokenizer|config\.json$' --ignore 'imatrix|\.md$'
```

Write the verified artifact beneath a specific model directory:

```sh
acorn download model oci://harbor.example.org/ai/models/qwen:Q4_K_M --local-dir ./models
```

Load one or more `oci://` references from an existing plain-text, JSON, or YAML model-list document:

```sh
acorn download model --model-file ./oci-models.txt
```

Use OCI sources and the named Harbor registry profile from application configuration:

```sh
acorn download model --config ./.acorn.json
```

Download and then synchronize the resolved local model with one inference target:

```sh
acorn download model oci://harbor.example.org/ai/models/qwen:Q4_K_M --sync opencode
```

Remote OCI access remains subject to the global offline policy, so this command is expected to fail before ORAS or Harbor is contacted:

```sh
acorn --offline download model oci://harbor.example.org/ai/models/qwen:Q4_K_M
```

### Configuration

Add `Repository::OCI` and an optional named registry profile rather than a Harbor-only model source. A target shape to validate against existing schema conventions is:

```json
{
  "registries": {
    "ornl-harbor": {
      "kind": "harbor",
      "endpoint": "https://harbor.example.org",
      "credentialEnv": "ACORN_HARBOR_TOKEN"
    }
  },
  "models": [
    {
      "name": "qwen-oci",
      "source": {
        "provider": "oci",
        "location": "oci://harbor.example.org/ai/models/qwen:Q4_K_M",
        "registry": "ornl-harbor"
      },
      "auth": "required"
    }
  ]
}
```

- Treat this shape as a design checkpoint, not permission to store a secret in the configuration file.
- Registry profiles may name environment variables, an ORAS/Docker registry-config path, CA/client-certificate paths, and an explicit `plainHttp` development switch. They must not contain plaintext passwords or tokens.
- A profile with `kind: harbor` enables Harbor discovery. A profile omitted from a direct `oci://` source uses generic ORAS behavior and its normal credential store.
- Command-line overrides, if needed after the profile design is tested, should select a profile or credential file; do not add password/token flags whose values appear in process listings.

### Artifact acceptance and file selection

- Accept OCI image manifests used as artifacts and resolve image indexes only when exactly one explicitly selected platform or artifact manifest is available. Do not pull runnable container images merely because their layers are OCI-compliant.
- Define and document ACORN model artifact media types. During an initial compatibility period, also allow `application/octet-stream` and OCI layer media types only when every materialized file has a safe `org.opencontainers.image.title` annotation and matches model-file policy.
- Parse the stable raw manifest output from `oras manifest fetch`; do not depend on ORAS experimental formatted JSON as ACORN's internal contract.
- Apply `--filter`, `--ignore`, configured `required_paths`, and quantization constraints to annotated logical file paths before the pull. If the installed ORAS version cannot selectively materialize the accepted set, pull into a staging directory, validate the staged inventory, and atomically publish only accepted files.
- Preserve ORAS's default traversal rejection and add ACORN-side path validation. Never pass `--allow-path-traversal`.
- Pull into a unique sibling staging directory. Verify the manifest digest, expected layer digests/sizes, accepted filenames, required shard completeness, and destination collision policy before renaming into the final model directory. Clean only the exact staging directory on failure.
- Define clobber behavior before implementation. The recommended default is to reject a non-empty destination; a future explicit replacement flag may atomically replace it.

### Authentication and trust

- Prefer an existing ORAS/Docker credential store. For noninteractive automation, pass a configured token to ORAS through standard input and use existing command redaction for all diagnostics.
- Map authentication-required, forbidden, not-found, TLS, digest mismatch, unsupported manifest, and missing-ORAS failures into typed errors with the registry host and reference but no credentials.
- Make `--offline` reject remote OCI and Harbor operations before spawning ORAS or issuing HTTP.
- Keep TLS verification enabled by default. Custom CA and mutual-TLS files may be configured explicitly. Plain HTTP and insecure TLS must be separate, conspicuous opt-ins and must never be inferred from a failed HTTPS request.
- Digest verification is mandatory for OCI content and must not be disabled by the existing `--skip-verify-checksum` flag. That flag may continue to govern optional Hugging Face SHA-256 sidecars; OCI descriptor digests are part of artifact identity.
- Signature verification is a follow-up policy integration. The first release must clearly report that digest integrity is verified but publisher identity is not, and it must preserve the resolved digest needed for later Cosign/Notation enforcement.

## Implementation plan

### Phase 0: executable spike and contract fixtures

1. Publish a small OCI fixture containing a tiny fake `.gguf`, tokenizer/config files, model metadata annotations, and a multi-shard variant to a disposable local Distribution registry and a supported Harbor test instance.
2. Record the exact raw outputs and exit behavior of the minimum supported ORAS release for `version`, `manifest fetch`, and `pull` on public, private, missing, malformed, tag, digest, custom-CA, and interrupted requests.
3. Confirm selective-file behavior, destination collision behavior, traversal rejection, staging cleanup, tag-to-digest resolution, and Windows path handling before freezing the CLI/config contract.
4. Decide and document the minimum ORAS version. Reject older versions with an installation-oriented error rather than attempting best-effort flag compatibility.

Exit condition: the repository contains non-secret fixtures and a short compatibility record proving the proposed contract on Windows and one Unix platform.

### Phase 1: typed OCI source and resolution

1. Add an `OCI` repository variant and typed `OciReference` parser covering registry host, repository path, tag or digest, and safe display/redaction. Extend `Location`/`Source` conversion without changing existing HTTP, local, or Hugging Face inference.
2. Add registry-profile configuration with `eserde` multi-error coverage, `serde(deny_unknown_fields)` where appropriate, validation for endpoint/scheme/environment-variable names, and JSON schema/docs updates.
3. Refactor `ModelDownloadPlan` only enough to hold a provider-neutral source resolution. Keep Hugging Face fallback options in the Hugging Face branch and OCI manifest/digest data in the OCI branch.
4. Resolve an OCI tag to a manifest descriptor during planning; make `--dry-run` and `--raw` show registry, repository, requested reference, resolved digest, artifact type, total size, and selected files without writing.
5. Ensure whitelist matching operates on the configured model name and canonical OCI reference without making Harbor API calls for generic registries.

Exit condition: OCI sources parse, route, resolve, serialize, and dry-run deterministically while every existing model-resolution test remains unchanged.

### Phase 2: ORAS process adapter and safe materialization

1. Add a narrow `OciTransport` trait and an `OrasTransport` implementation under the `std` feature. Express inputs and outputs as typed values; keep command construction and process side effects at the adapter boundary.
2. Locate `oras`, validate its version once per command run, build argument arrays without a shell, reuse command logging/redaction, support cancellation, and translate stderr/exit codes into typed errors.
3. Fetch and parse the raw manifest, select accepted descriptors, create a staging destination, invoke `oras pull`, validate the staged paths and descriptor digests, and atomically publish the result.
4. Integrate the downloaded paths into existing `ModelDetails`, weight persistence, activity logging, and optional `--sync` behavior. Preserve the requested OCI tag as provenance and the digest as resolved identity.
5. Make quiet/raw/verbose output consistent with existing `Label` and tracing behavior. Never expose credentials or duplicate ORAS progress when ACORN is silent or piped.

Exit condition: `acorn download model oci://...@sha256:...` and tag-based pulls work against a generic local registry, are digest-verified, leave no partial destination on failure, and feed the same synchronization path as existing model downloads.

### Phase 3: Harbor discovery and metadata

1. Add a focused Harbor client in `acorn-lib/src/io/api/harbor.rs` using the existing HTTP policy/auth/error conventions. Implement only health/version capability probing, list artifacts, and get artifact by tag or digest.
2. Correctly percent-encode project, repository, and reference path values; implement pagination and bounded query filtering; deserialize only the response fields ACORN uses.
3. When a selected registry profile is `harbor`, enrich OCI planning with Harbor tags, push time, media/artifact type, labels, size, and digest. Use this metadata to produce clearer selection and policy errors, not to download blobs.
4. Add an optional explicit discovery surface only if a concrete UX is agreed after fixture testing. Do not make ordinary `download model oci://...` enumerate a whole project or guess among tags.
5. Fall back to generic OCI resolution when Harbor enrichment is unavailable only for optional authentication and transient capability failures. A configured required Harbor policy or an explicit discovery request must fail closed.

Exit condition: a configured Harbor source can resolve by tag or digest, paginate deterministic discovery results, enrich dry-run output, and hand the canonical OCI reference to the same ORAS transport.

### Phase 4: hardening, documentation, and release

1. Add a doctor check for ORAS availability/version and registry-profile configuration without contacting registries unless explicitly requested.
2. Regenerate command help and update `docs/src/cli-reference/commands/download.md` with OCI examples, the ORAS prerequisite, credential-store behavior, tag/digest reproducibility, Harbor profiles, offline behavior, TLS configuration, accepted media types, and the digest-versus-signature trust boundary.
3. Add migration notes for the public `Repository` schema and examples for both generic registries and Harbor.
4. Run focused tests, `make lint`, `make test`, `make help`, `make wasm`, `make book`, `make audit`, and registry interoperability tests on Windows and Linux.
5. Reassess a Rust-native OCI Distribution client only after measuring ORAS installation friction, startup overhead, and unsupported environments. If pursued, implement it behind `OciTransport` and require the same fixtures and acceptance criteria.

Exit condition: documentation, generated help, standard validation, security review, and cross-platform registry tests have accepted results.

## Validation strategy

### Unit and fixture tests

- Parse valid registry ports, nested repository names, tags, SHA-256 digests, IPv6 hosts, and `oci://` routing; reject missing references, invalid digests, user-info, fragments, traversal-like titles, and ambiguous Hugging Face selectors.
- Snapshot generic OCI and Harbor configuration parsing, schema serialization, `--help`, dry-run, raw JSON/YAML, and typed errors.
- Test manifest/index/artifact-type decoding, unknown annotations, missing titles, duplicate paths, filter/ignore precedence, quantization selection, required shard completeness, and size overflow.
- Test command arguments and redacted diagnostics with a fake ORAS executable. Cover success, missing binary, old version, authentication failure, TLS failure, not found, digest mismatch, nonzero exit, invalid UTF-8, interruption, and cleanup.
- Mock Harbor pagination, encoded nested repository names, tag and digest lookup, 401/403/404/429/5xx behavior, malformed responses, optional enrichment fallback, and required-policy failure.

### Integration tests

- Pull the same fixture by mutable tag and immutable digest from a disposable OCI Distribution registry, then prove both resolve to identical verified content and recorded digest.
- Pull public and authenticated fixtures from Harbor; cover robot-account or token authentication through standard input/credential store without recording secrets.
- Exercise custom CA, mutual TLS if supported by the test environment, explicitly enabled plain HTTP, redirects, range/resume behavior delegated to ORAS, multi-layer artifacts, multi-shard models, and destination collisions.
- Run interrupted and corrupted downloads and assert that the final destination and model database remain unchanged.
- Verify `--offline`, `--dry-run`, `--raw`, `--filter`, `--ignore`, `--quantization`, `--local-dir`, whitelist, configuration, activity logging, and `--sync` semantics across OCI and existing Hugging Face paths.
- Run at least Windows and Linux jobs because process invocation, credential helpers, atomic renames, executable discovery, and path validation differ materially.

## Risks and mitigations

- **Mutable tags weaken reproducibility.** Always resolve and persist the digest; display a warning when a tag is used and make digest pinning easy to copy from dry-run output.
- **Digest integrity is not publisher authenticity.** State the boundary clearly and leave a typed policy hook for future Cosign/Notation verification.
- **Arbitrary artifact contents can escape or overwrite destinations.** Stage, reject unsafe/duplicate paths, preserve ORAS traversal protection, validate before publish, and never enable traversal flags.
- **Secrets can leak through command arguments or logs.** Prefer credential helpers or standard input, redact all sensitive flags, and test captured diagnostics.
- **ORAS output/flags can drift.** Pin a minimum supported version, parse stable raw manifest JSON, keep experimental formatted output out of the contract, and test version-specific behavior.
- **Harbor API behavior can vary by deployment version.** Probe capabilities, deserialize a minimal response subset, treat Harbor as optional enrichment, and retain the portable Distribution/ORAS path.
- **OCI images may be mistaken for model artifacts.** Require an accepted artifact/config/layer media-type policy and safe logical filenames; reject ordinary runnable-image manifests by default.
- **Existing Hugging Face behavior could regress.** Route only explicit OCI sources and keep all existing unqualified-selector tests as compatibility gates.
- **The `acorn-lib` portability boundary could regress.** Gate process and network implementations behind `std`; keep reference parsing, manifest models, and selection logic pure where practical; retain `make wasm` validation.

## Acceptance criteria

- `acorn download model` accepts explicit `oci://` tag and digest references without changing how existing Hugging Face IDs, HTTP URLs, or local paths are interpreted.
- Generic OCI pulls use a version-checked ORAS executable through argument-array process invocation, never a composed shell command.
- OCI tags resolve to digests during planning, and the requested reference plus resolved digest appear in dry-run/raw output and persisted provenance.
- Artifact manifests and file paths are validated before an atomic final publish; digest mismatch, unsafe paths, interruption, and any pull failure leave no partial final model or database update.
- Remote OCI and Harbor access is rejected in offline mode before any process or HTTP request starts.
- Credentials are obtained from a credential store, named environment source, or standard input and never appear in configuration values, process arguments, logs, snapshots, or errors.
- Configured Harbor registries can list and retrieve artifacts by tag or digest through the Harbor v2 API, while all content transfer remains registry-portable through ORAS.
- The initial release rejects ordinary container images and undocumented artifact layouts; supported model artifact media types and annotation requirements are documented.
- Existing filters, quantization/shard completeness, whitelist, output-directory, raw/dry-run, activity logging, and synchronization behavior have explicit OCI tests.
- Focused unit/integration tests and `make lint`, `make test`, `make help`, `make wasm`, `make book`, and `make audit` have accepted results on the supported toolchain, with OCI integration coverage on Windows and Linux.
