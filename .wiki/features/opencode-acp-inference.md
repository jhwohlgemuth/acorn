---
type: Plan
title: "Coordinate ACP inference and verified fixes"
description: "Stage prompt hydration, interchangeable ACP/OpenAI-compatible inference, structured fix suggestions, and policy-governed deterministic remediation."
difficulty: high
impact: high
package: [workspace, acorn-lib, acorn-cli, acorn-schema, acorn-api, acorn-host, acorn-service]
tags: [feature, roadmap, agentic, acp, opencode, openai-compatible, prompts, inference, checks, fixes]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/schema/agent/`, `src/io/api/openai.rs`, `src/io/mcp/`, `src/doctor/`, `crates/acorn-cli/src/commands/{serve,doctor}/`, CLI parsing/help, assets, and focused tests.
- **Create:** the prompt, inference-port, ACP transport, fix-domain, and service modules shown in this plan; place portable contracts in `acorn-schema`/`acorn-service`, provider protocol in `acorn-api`, and process/HTTP implementations in `acorn-host` after extraction.
- **Delete:** no current integration; remove the `doctor --fix` not-implemented path only when the shared planner is functional.

# Dependencies

This is downstream of portable schema/error ownership, host traits, analyzer result stabilization, the API roadmap, and CLI parse/execution separation. It must not define the crate boundaries it consumes; see the [plan dependency network](../plan-network.md).

# Goal

Let ACORN hydrate embedded prompt templates with typed configuration and serializable data, then run them through either an Agent Client Protocol (ACP) agent or an OpenAI-compatible HTTP API. Use that inference capability to turn structured `Check` results into validated, reviewable `Fix` suggestions without applying changes automatically. OpenCode is the first ACP adapter, while Codex, Goose, Pi, and other ACP agents or bridges remain straightforward extensions.

# Scope and assumptions

- ACORN is the ACP client; OpenCode is the agent subprocess started with `opencode acp`.
- OpenCode is the first concrete adapter, not an assumption embedded in prompt, session, event, result, or permission types.
- The first vertical slice is one-shot, text-only inference with one new ACP session per invocation.
- The first slice denies permission requests for mutating tools by default. Broader agent tasks require an explicit permission policy and are not silently enabled by inference APIs.
- OpenCode remains responsible for provider credentials and its configured default model. ACORN may request an agent or model only when both the caller and negotiated ACP capabilities support it.
- Prompt input is any `serde::Serialize` value. Built-in templates continue to support their current convenience fields such as text, language, and limits.
- The existing OpenCode configuration synchronization path remains separate from ACP execution. The ACP module may read resolved OpenCode settings but must not rewrite them during inference.
- Inference-generated fixes are suggestion-only and can never become executable merely because a model returned them. Deterministic fixes from registered ACORN providers may be auto-applied only through an explicit policy and approval path.
- ACP and OpenAI-compatible APIs share prompt, request, result, capability, and fix schemas. Their process, authentication, streaming, and wire concerns remain in separate adapters.
- Deterministic fixes produced by an analyzer take precedence over model-generated suggestions. Inference is used only where a check lacks a complete deterministic correction or the user explicitly requests alternatives.
- The first release rejects ACP inference in offline mode because ACORN cannot prove that OpenCode's selected provider is local. Prompt hydration/dry-run remains available, and OpenAI-compatible HTTP is allowed offline only for an explicitly configured loopback endpoint. A later verified local-only agent descriptor may relax ACP policy without changing the request contract.

# Current evidence

- `crates/acorn-lib/src/io/mod.rs` already gates agent integrations behind the `agentic` feature, but `io::mcp` is only an empty placeholder and there is no ACP module.
- The existing `agentic` feature is enabled by default and currently selects the Axum/RMCP/Tower server stack. Adding an ACP client SDK to that feature would make an unrelated native process dependency part of default library builds unless ACP receives a separate capability feature.
- `schema::agent::PromptTemplate` embeds six `.prompt` assets and renders them with Tera through `PromptTemplateConfiguration`.
- `schema::agent` currently imports I/O, database, terminal, embedded-asset, Tera, and other host-facing code. Extending that mixed module with inference services would conflict with the in-progress crate-boundary plan, which assigns portable agent models to `acorn-schema` and agentic orchestration to `acorn-service`.
- Current prompt tests prove that known asset names can be loaded and that rendering returns the same success state for enum, string, and owned-string selectors. They do not assert that concrete input values appear in the rendered output or that unresolved variables are rejected.
- The prompt assets mix nested `config.*`, kebab-case keys, and underscore variable names. Their frontmatter also contains fields and shapes not represented by the current `schema::agent::FrontMatter`, so ACP execution must not assume the current frontmatter type can parse every embedded asset.
- `schema::agent::opencode::Config` already models OpenCode agents, models, permissions, commands, and configuration discovery. `io::sync::opencode` already owns configuration mutation and should remain the only writer.
- `analyzer::Check` already records category, severity, URI, locator, context, message, and check-specific errors. `AnalysisReport` retains checks by standard, batch, and category, but there is no parallel `Fix` model or fix report.
- `Check` is not currently a serialization contract and contains private, renderer-specific payloads. Inference should receive a normalized check reference DTO rather than serializing the internal check object or its entire source file.
- `io::api::openai` already supports OpenAI-compatible `/responses` and `/chat/completions` endpoints, configurable domains, optional bearer tokens, and local llama-swap routers. Its baseline response type is raw `serde_json::Value`, so fix generation needs typed request/response mapping and capability-aware structured output.
- `acorn doctor` already accepts `--fix` and `--interactive`, but `--fix` currently returns `Autofix is not implemented yet`. Software diagnostics detect ACORN, Git, Node.js, npm, npx, Pandoc, and Vale, but expose installation state as a display-oriented table rather than structured findings.
- Several deterministic remediation mechanisms already exist: `ResearchActivity::format_with` normalizes RAD metadata, Vale output carries action parameters and exact spans, Vale resolution can download its binary, and PDF export can install Chromium. These should become registered fix providers/executors instead of being reimplemented by an inference layer.
- Official OpenAI documentation describes Structured Outputs as enforcing a supplied JSON Schema and exposing refusals programmatically, making it appropriate for a typed fix envelope when the selected endpoint and model support it ([OpenAI Structured Outputs documentation](https://developers.openai.com/api/docs/guides/structured-outputs)). Compatibility servers may implement a smaller subset, so ACORN must still validate every returned fix locally.
- The official ACP Rust SDK is the intended starting point for Rust clients. Its API now lives in the dedicated [`agentclientprotocol/rust-sdk`](https://github.com/agentclientprotocol/rust-sdk) repository, while wire compatibility remains negotiated through `protocolVersion`; the current stable wire protocol is version 1 ([ACP protocol repository](https://github.com/agentclientprotocol/agent-client-protocol)). Pin and test an SDK release rather than coding against an unversioned example.
- OpenCode exposes an ACP-compatible subprocess over JSON-RPC on stdio through `opencode acp` and retains its built-in tools, MCP servers, project rules, agents, and permissions ([OpenCode ACP documentation](https://opencode.ai/docs/acp/)).

# Architecture review and viability

The design is viable, but it is an initiative-sized feature rather than one safe implementation change. Treat each delivery phase as an independently reviewable vertical slice and do not start package installation or inference-generated fixes in the ACP transport pull request.

Recommendation: proceed through prompt hydration, the narrow inference contract, and the fake ACP/OpenCode transport. Treat OpenAI compatibility, fix suggestions, and deterministic auto-fix as successive go/no-go milestones. Do not authorize real package installation until a document-only fix has demonstrated planning, preview, stale-input rejection, execution, and post-fix verification end to end.

| Review area | Assessment | Required adjustment |
|---|---|---|
| Prompt assets | Viable after normalization | Make representative rendering of every existing asset the first gate; current tests can pass without proving successful value substitution, and current frontmatter is richer than `FrontMatter`. |
| ACP/OpenCode | Viable with moderate integration risk | Pin the official Rust SDK, wrap SDK types at `io::acp`, test against a fake agent, and separately gate real OpenCode interoperability. |
| OpenAI-compatible APIs | Viable using the existing endpoint client | Reuse API endpoint/authentication work, model Responses and Chat Completions separately, and require configured capabilities rather than guessing structured-output support from `/models`. |
| Shared inference contract | Viable if kept narrow | Share request/result/provenance and non-mutation intent; do not flatten ACP permissions or HTTP failures into a false common protocol. |
| Fix suggestions | Viable and relatively low risk | Normalize checks into bounded DTOs, validate all model output locally, and keep generated proposals non-executable. |
| Deterministic auto-fix | Viable after the proposal model stabilizes | Land document fixes before host package installation, and require policy, preview, verification, and trusted recipe resolution. |
| Doctor package installation | Viable but highest operational risk | Support one verified OS/package-manager recipe at a time; tests use fake process runners and never mutate the developer or CI host. |
| Crate extraction | Compatible only with revised ownership | Keep schema models, service orchestration, API protocol DTOs, and native adapters separable now so the in-progress crate split remains mechanical. |

Primary delivery risks are bounded as follows:

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| ACP SDK API changes while the protocol remains wire-compatible | Medium | Medium | Pin a reviewed SDK release, expose only ACORN DTOs, and keep one fake-agent contract suite around the adapter. |
| Existing prompt assets fail or render unintended defaults | High | High | Make exact representative hydration tests a prerequisite and normalize one asset contract before transport work. |
| Repository or sensitive data is sent unintentionally | Medium | High | Require bounded explicit excerpts, preview transmitted data, redact secrets, and never default to whole-file/repository context. |
| Child processes survive timeout or cancellation | Medium | High | Give one cleanup guard ownership, use kill-on-drop plus explicit reap, and test early-exit/timeout behavior on Windows and Unix. |
| Default dependency or binary size grows unexpectedly | Medium | Medium | Keep `acp` non-default, inspect feature trees, and measure the CLI feature under the binary-size plan. |
| Model output crosses into executable actions | Low after type boundary | Critical | Keep executable types private and non-deserializable; resolve only registered recipe IDs under policy. |
| Package installation alters or destabilizes the host | Medium | Critical | Require interactive confirmation, fixed vectors, effect declarations, fake-runner tests, and post-install diagnostics; unsupported platforms remain manual. |

Use three readiness levels rather than one Boolean:

1. `InstallationReadiness`: the executable resolves and its version probe succeeds.
2. `ProtocolReadiness`: an opt-in bounded ACP initialize/handshake succeeds and records negotiated capabilities and advertised authentication methods without sending a user prompt.
3. `InferenceReadiness`: a separately authorized smoke request reaches a configured model. This may incur network use or cost and must not run as an ordinary doctor check.

`AcpClient::infer` always requires installation and protocol initialization, but `acorn doctor` should not claim model/provider readiness merely because `opencode --version` succeeds. Likewise, an OpenAI-compatible `/models` response is not evidence that a model supports Structured Outputs; endpoint mode and structured-output support remain explicit configuration unless a provider-specific capability contract proves otherwise.

## Relationship to other plans

This feature must consume the following plans as governing constraints instead of recreating their abstractions:

| Related plan | Relationship and ordering |
|---|---|
| [Crate boundary architecture](../refactors/crate-boundary-architecture.md) | Governs ownership. Portable agent configuration stays in the schema boundary; prompt/inference/fix orchestration moves toward `acorn-service`; ACP, process execution, doctor probes, and fix executors move toward `acorn-host`; OpenAI protocol DTOs move toward `acorn-api`; `acorn-lib` eventually re-exports compatibility paths only. Boundary preparation is a prerequisite, but full crate extraction is not. |
| [Portability roadmap](../refactors/portability/roadmap.md) and [host abstraction traits](../refactors/portability/host-abstraction-traits.md) | Define the split between pure hydration/planning and native process, filesystem, HTTP, clock, and package-manager adapters. New ports should match these plans rather than introduce a second host abstraction family. |
| [Dependency feature defaults](../refactors/portability/dependency-feature-defaults.md) and [portability roadmap](../refactors/portability/roadmap.md) | Require explicit imports and optional native dependencies. ACP receives a separate non-default `acp` capability; minimal, alloc, and WASM graphs must remain unchanged. |
| [Safe path](../refactors/core/safe-path.md) | Owns path normalization and containment. Document fix targets should adopt `SafePath` when available; until then, use one local compatibility wrapper designed for replacement rather than creating a competing public path type. |
| [Analyzer result normalization](../refactors/analyzer/result-boundary-normalization.md) and [check batch](../refactors/analyzer/check-batch-monoid.md) | Are not blockers. Fix suggestion code consumes `AnalysisReport` through one normalization adapter and should use the implemented analysis context split without changing check construction solely for inference. |
| [API client roadmap](../refactors/api/roadmap.md), [response DTOs](../refactors/api/response-dtos.md), [environment bootstrap](../refactors/api/environment-bootstrap.md), and [provider capability matrix](../refactors/api/provider-capability-matrix.md) | Govern endpoint/authentication/error/capability mechanics. The OpenAI adapter may add its narrow typed payloads first, but must not duplicate shared environment precedence, endpoint resolution, health, or provider-capability machinery. |
| [Extract acorn-cmd](../refactors/acorn-cmd-extraction.md) | Owns reusable command construction/execution helpers. ACP and fix executors may use the in-tree helpers initially, then depend on `acorn-cmd` after extraction; neither should invent shell-string execution. |
| [CLI parse/execution separation](../refactors/core/cli-parse-execution-models.md), [startup separation](../refactors/core/cli-startup-separation.md), and [CLI error boundary](../refactors/core/cli-error-boundary.md) | Require clap types to normalize into library options, concrete backend/executor selection to remain in the CLI composition root, and failures to return through the shared outer error boundary. They do not block library-first slices. |
| [macOS binary-size plan](../refactors/portability/macos-compact-unwind-size.md) | Requires ACP SDK/process additions to remain optional and measured independently because the default `agentic` server stack is already a suspected binary-size contributor. |

The implementation order should follow the governing boundary work, but it should not wait for every refactor to finish. Each slice records whether it uses a temporary in-tree compatibility path and the future owning crate shown below.

# Target design

## Proposed code layout

Start with a small number of cohesive modules and split files only when the first vertical slice makes the boundary useful. The intended end state is:

```text
crates/acorn-lib/src/
├── agent/
│   ├── mod.rs              # prompt/inference use case and transport-neutral DTOs
│   ├── backend.rs          # consumer-owned InferenceBackend port
│   └── prompt.rs           # embedded catalog hydration and metadata normalization
├── fix/
│   ├── mod.rs              # finding, proposal, report, and policy DTOs
│   ├── provider.rs         # deterministic finding-to-fix providers
│   └── planner.rs          # trusted proposal resolution and immutable plans
├── schema/agent/
│   └── mod.rs              # portable agent models, prompt selectors, and config
├── io/
│   ├── acp/
│   │   ├── mod.rs          # public ACP surface
│   │   ├── client.rs       # shared ACP session lifecycle
│   │   ├── installation.rs # executable discovery and version probe
│   │   └── opencode.rs     # thin OpenCode process descriptor
│   ├── fix/
│   │   ├── mod.rs          # executor/verifier ports
│   │   ├── document.rs     # atomic document changes and rollback
│   │   ├── download.rs     # verified existing download integrations
│   │   ├── package.rs      # fixed-vector package-manager execution
│   │   └── registry.rs     # trusted recipe lookup
│   └── api/openai.rs       # existing API plus typed inference adapter
└── doctor/
    ├── mod.rs              # existing diagnostics and presentation
    └── finding.rs          # structured DoctorFinding conversion

crates/acorn-cli/src/
├── commands/prompt/        # prompt preview and one-shot inference
├── commands/check/         # optional fix suggestion orchestration
└── commands/doctor/        # shared fix preview/apply lifecycle
```

The first implementation need not create every leaf file. In particular, `agent/mod.rs`, `fix/mod.rs`, `io/acp/mod.rs`, and the existing `io/api/openai.rs` can hold the initial vertical slice. Split provider, planner, or adapter files only when a second implementation would otherwise mix unrelated responsibilities.

The in-tree `agent` and `fix` roots are service boundaries intended to move to `acorn-service`. `io::acp`, process-backed doctor probes, and concrete fix executors are native adapters intended to move to `acorn-host`. OpenAI wire DTOs remain with `io::api::openai` until they move to `acorn-api`; the implementation of the service-owned inference port then belongs in the native composition layer. `schema::agent` retains only portable configuration, selectors, and metadata in the target architecture.

Keep `schema::agent::PromptTemplate` limited to its schema-owned representation during boundary preparation; do not make it the long-term owner of RustEmbed/Tera hydration. Move asset loading and hydration behind `agent::prompt`, migrate callers directly, and remove the displaced render path at the ownership cutover. This reuses the existing prompt assets without adding service behavior to the schema module.

Wire the portable and native boundaries explicitly at their existing roots:

```rust
// crates/acorn-lib/src/lib.rs
#[cfg(feature = "std")]
pub mod agent;
pub mod fix;

// crates/acorn-lib/src/io/mod.rs
#[cfg(feature = "acp")]
pub mod acp;
pub mod fix;
```

The exact `agent` feature gate should follow its actual dependencies after prompt normalization; Tera/RustEmbed make the current implementation `std`-only. `fix` DTOs should remain alloc-capable where practical, while planner ports that require async/native types can be gated or split. If fix DTOs require types currently hidden behind `analysis` or `doctor`, copy only stable scalar values into reference DTOs rather than feature-gating the entire fix domain. For example, store a serialized severity or doctor category in the reference instead of retaining an `analyzer::Check` or `doctor` object.

## Representative prompt API

These examples specify ownership and dependency direction; exact names may be adjusted to match the existing `PromptTemplate` implementation. Public types require rustdoc and the derives needed by the current schema conventions.

```rust
#[derive(Clone, Debug, Serialize)]
pub struct PromptContext<T> {
    pub config: PromptTemplateConfiguration,
    pub data: T,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HydratedPrompt {
    pub asset: PromptFileAsset,
    pub body: String,
    pub metadata: PromptMetadata,
}

pub struct PromptHydrator;

impl PromptHydrator {
    pub fn hydrate<T>(
        asset: PromptFileAsset,
        context: &PromptContext<T>,
    ) -> ApiResult<HydratedPrompt>
    where
        T: Serialize,
    {
        load_prompt_asset(&asset)
            .and_then(|source| render_prompt_strict(&source, context))
            .and_then(parse_hydrated_prompt)
    }
}
```

`render_prompt_strict` should be the one normalization point that creates the `config` and `data` namespaces and any temporary aliases required by existing assets. It should configure Tera to fail on missing values, then reject a rendered body that still contains `{{`, `{%`, or other unresolved syntax. `parse_hydrated_prompt` should deserialize normalized frontmatter separately from the body so transport code never receives raw prompt-file syntax.

During the first in-tree slice, `PromptHydrator` may use the existing `PromptTemplate::get` asset access while all new parsing/rendering remains in `agent::prompt`. Keep `PromptTemplate::render` behavior covered but do not make the schema implementation depend back on the service. When the crate-boundary work relocates the embedded catalog to `acorn-service`, migrate callers to that owner and remove the old rendering surface from `acorn-lib`.

The new fix asset should consume normalized references, not internal analyzer objects:

```rust
#[derive(Clone, Debug, Serialize)]
pub struct SuggestFixesData {
    pub schema_version: String,
    pub findings: Vec<CheckReference>,
}
```

## Representative inference port

Keep prompt selectors and configuration in `schema::agent`, then place inference DTOs and the consumer-owned port in the in-tree `agent` service boundary:

```rust
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceRequest {
    pub prompt: HydratedPrompt,
    pub response_schema: Option<serde_json::Value>,
    pub model: Option<String>,
    pub agent: Option<String>,
    pub timeout: Duration,
    pub permission_policy: PermissionPolicy,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceResult {
    pub text: String,
    pub structured: Option<serde_json::Value>,
    pub finish_reason: InferenceFinishReason,
    pub usage: Option<InferenceUsage>,
    pub provenance: InferenceProvenance,
}

pub trait InferenceBackend {
    fn infer<'a>(
        &'a self,
        request: &'a InferenceRequest,
    ) -> BoxFuture<'a, ApiResult<InferenceResult>>;
}
```

Do not force ACP-only concepts such as permission request IDs or HTTP-only concepts such as status codes into these DTOs. Adapter-specific errors retain those details and convert only stable output into `InferenceResult`. A small CLI-owned selector can provide runtime dispatch without closing the library extension point:

```rust
pub enum ConfiguredInferenceBackend {
    Acp(AcpClient),
    OpenAi(OpenAiInferenceClient),
}

impl InferenceBackend for ConfiguredInferenceBackend {
    fn infer<'a>(
        &'a self,
        request: &'a InferenceRequest,
    ) -> BoxFuture<'a, ApiResult<InferenceResult>> {
        match self {
            Self::Acp(client) => client.infer(request),
            Self::OpenAi(client) => client.infer(request),
        }
    }
}
```

OpenAI request mapping should be a pure function that can be tested without HTTP:

```rust
fn responses_request(
    request: &InferenceRequest,
    model: &str,
) -> OpenAiResponsesRequest {
    OpenAiResponsesRequest {
        model: model.to_owned(),
        input: request.prompt.body.clone(),
        text: request.response_schema.clone().map(|schema| OpenAiTextConfig {
            format: json_schema_format(schema),
        }),
    }
}
```

The HTTP call then maps a typed response or refusal back to `InferenceResult`. Chat Completions uses a separate mapper because its structured-output field shape differs from Responses. Compatible servers that lack structured output return text which is parsed and validated through the same fix-envelope function used by ACP.

## Representative ACP implementation

Represent agent differences as immutable launch data until two working integrations prove that behavioral polymorphism is required:

```rust
#[derive(Clone, Debug)]
pub struct AgentProcessConfig {
    pub agent_id: String,
    pub executable: PathBuf,
    pub acp_arguments: Vec<OsString>,
    pub version_arguments: Vec<OsString>,
    pub working_directory: PathBuf,
    pub environment: BTreeMap<OsString, OsString>,
    pub timeout: Duration,
    pub permission_policy: PermissionPolicy,
}

pub fn opencode_process(
    executable: Option<PathBuf>,
    working_directory: PathBuf,
) -> AgentProcessConfig {
    AgentProcessConfig {
        agent_id: "opencode".to_owned(),
        executable: executable.unwrap_or_else(|| PathBuf::from("opencode")),
        acp_arguments: vec![OsString::from("acp")],
        version_arguments: vec![OsString::from("--version")],
        working_directory,
        environment: BTreeMap::new(),
        timeout: Duration::from_secs(120),
        permission_policy: PermissionPolicy::DenyMutations,
    }
}
```

`AgentProcessConfig` is trusted application configuration, not model output. Do not derive `Deserialize` for it unless a later user-owned configuration format requires that capability and validates executable paths separately.

Installation checking is reusable and does not open an ACP session:

```rust
pub fn check_installation(
    config: &AgentProcessConfig,
) -> ApiResult<AgentInstallation> {
    resolve_executable(&config.executable)
        .and_then(|path| {
            probe_version(&path, &config.version_arguments).map(|version| {
                AgentInstallation {
                    agent_id: config.agent_id.clone(),
                    executable: path,
                    version,
                }
            })
        })
}
```

The production implementation should return the resolved absolute executable in `AgentInstallation`, use the repository's existing command discovery/version helpers where possible, and distinguish not found, not executable, probe failure, timeout, and unparsable version. `AcpClient::infer` calls this function even if the caller previously ran doctor.

The ACP session implementation should read as a bounded lifecycle rather than a collection of OpenCode branches:

```rust
impl InferenceBackend for AcpClient {
    fn infer<'a>(
        &'a self,
        request: &'a InferenceRequest,
    ) -> BoxFuture<'a, ApiResult<InferenceResult>> {
        Box::pin(self.infer_checked(request))
    }
}
```

`infer_checked` owns the ordered preflight, spawn, initialize, session, prompt, and shutdown operations. Its exact calls will depend on the ACP SDK API. One cleanup guard must own the child process from spawn through completion so timeout, cancellation, protocol error, or early exit always closes stdin and reaps the child. Stream updates should fold into result state, and permission requests should be answered by `PermissionPolicy` rather than the agent adapter.

Codex, Goose, or Pi support should normally add another descriptor constructor and an interoperability test:

```rust
pub fn goose_process(/* verified options */) -> AgentProcessConfig;
pub fn codex_process(/* verified options */) -> AgentProcessConfig;
pub fn pi_process(/* verified options */) -> AgentProcessConfig;
```

Do not implement these constructors until the actual direct ACP command or maintained bridge, version probe, and permission behavior have been verified from current primary documentation.

## Representative fix domain

The serialization boundary must make an inferred suggestion structurally incapable of becoming a host command:

```rust
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum FindingReference {
    Check(CheckReference),
    Doctor(DoctorReference),
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FixAction {
    DocumentChange(DocumentChangeProposal),
    RegisteredRecipe { recipe_id: String },
    Manual { instructions: String },
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Fix {
    pub id: String,
    pub finding: FindingReference,
    pub summary: String,
    pub rationale: String,
    pub confidence: f32,
    pub actions: Vec<FixAction>,
    pub automation: FixAutomation,
    pub risk: FixRisk,
    pub effects: FixEffects,
    pub provenance: FixProvenance,
}
```

Validate confidence as a finite value from zero through one, require every finding ID to exist in the request batch, confine document targets to caller-approved roots, and compare source fingerprints before presenting an action as current. Unknown fields should fail rather than being ignored.

Executable types remain private to trusted planning and execution code and intentionally omit `Deserialize`:

```rust
pub struct ExecutableFix {
    id: String,
    action: ExecutableAction,
    preconditions: Vec<Precondition>,
    effects: FixEffects,
    verification: VerificationSpec,
}

enum ExecutableAction {
    RewriteDocument(DocumentTransform),
    VerifiedDownload(DownloadRecipe),
    InstallPackage(PackageInstallRecipe),
}
```

Only deterministic providers and the trusted recipe registry can construct `ExecutableFix`. Keep fields private and expose read-only preview methods so an executor cannot receive a `FixAction` accidentally.

## Representative provider, planner, and executor boundaries

Use separate phases so inference, analysis, doctor, and host mutation do not depend directly on one another:

```rust
pub trait FixProvider {
    fn propose(&self, finding: &FindingReference) -> ApiResult<Vec<Fix>>;
}

pub trait FixPlanner {
    fn plan(
        &self,
        fixes: &[Fix],
        policy: &FixPolicy,
    ) -> ApiResult<FixPlan>;
}

pub trait FixExecutor {
    fn supports(&self, fix: &ExecutableFix) -> bool;

    fn execute<'a>(
        &'a self,
        fix: &'a ExecutableFix,
    ) -> BoxFuture<'a, ApiResult<ExecutionRecord>>;
}

pub trait FixVerifier {
    fn verify<'a>(
        &'a self,
        fix: &'a ExecutableFix,
        record: &'a ExecutionRecord,
    ) -> BoxFuture<'a, ApiResult<VerificationResult>>;
}
```

Planning is a pure transformation where practical:

```text
Fix proposals
  -> reject stale or unsupported proposals
  -> resolve known recipe IDs
  -> enforce policy and allowed effects
  -> coalesce dependencies and remove duplicates
  -> detect conflicts
  -> topologically order actions
  -> return immutable FixPlan and exact preview
```

Execution is the only mutating phase. It rechecks preconditions immediately before each action, invokes an executor using an executable plus fixed argument vector rather than a shell string, records redacted output, and invokes the verifier. A process exit code of success produces an `executed` record; only successful verification produces a `resolved` outcome.

The recipe registry can initially use explicit matching instead of a plugin framework:

```rust
impl RecipeRegistry {
    pub fn resolve(
        &self,
        recipe_id: &str,
        platform: &PlatformContext,
    ) -> Option<ExecutableFix> {
        match (recipe_id, platform.package_manager.as_deref()) {
            ("software.node.install", Some("winget")) => {
                self.node_with_winget(platform)
            }
            ("software.vale.install", _) => self.vale_with_existing_resolver(platform),
            _ => None,
        }
    }
}
```

The concrete package identifier and flags in `node_with_winget` must be verified before implementation and remain compiled trusted data. If a platform/package-manager combination has no registered recipe, planning downgrades the proposal to manual instructions rather than guessing a command.

## Doctor integration example

Convert display-oriented diagnostic state into a stable finding before rendering it:

```rust
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DoctorFinding {
    pub reference: DoctorReference,
    pub status: DoctorStatus,
    pub details: BTreeMap<String, String>,
}
```

The software diagnostic provider can then remain deterministic:

```rust
impl FixProvider for SoftwareFixProvider {
    fn propose(&self, finding: &FindingReference) -> ApiResult<Vec<Fix>> {
        match finding {
            FindingReference::Doctor(reference) if reference.subject == "node" => {
                Ok(vec![node_installation_fix(reference)])
            }
            FindingReference::Doctor(reference) if reference.subject == "opencode" => {
                Ok(vec![opencode_installation_fix(reference)])
            }
            _ => Ok(Vec::new()),
        }
    }
}
```

`doctor --fix` should orchestrate the shared lifecycle rather than own installation logic:

```text
run diagnostics
  -> produce DoctorFinding values
  -> collect deterministic Fix proposals
  -> plan under FixPolicy
  -> print exact preview
  -> obtain required confirmation
  -> execute trusted actions
  -> rerun source diagnostics
  -> render FixOutcome values
```

When Node, npm, and npx are all absent, the planner should recognize npm and npx as postconditions of the Node recipe, select one installation, and retain three verification probes. Package installation is never selected by safe-only, non-interactive policy because its declared effects include network, package-manager mutation, possible elevation, and limited rollback.

## Check-to-fix orchestration example

Keep orchestration in one service function so CLI presentation does not duplicate validation:

```rust
pub fn suggest_fixes<'a, B>(
    backend: &'a B,
    report: &'a AnalysisReport,
    options: &'a FixOptions,
) -> BoxFuture<'a, ApiResult<FixReport>>
where
    B: InferenceBackend + Sync,
{
    Box::pin(suggest_fixes_inner(backend, report, options))
}
```

`suggest_fixes_inner` should call individually testable `normalize_checks`, `build_suggest_fixes_prompt`, `build_inference_request`, `parse_fix_envelope`, and `validate_fixes` helpers around the one asynchronous backend call. Deterministic fixes are collected before inference, and already-resolved findings are omitted unless alternatives were explicitly requested.

## File-level first vertical slice

The smallest useful implementation should touch approximately these files:

1. `crates/acorn-lib/src/schema/agent/mod.rs`: normalize prompt selectors/configuration and reduce new dependencies; add `crates/acorn-lib/src/agent/mod.rs` for hydration, inference DTOs, and the consumer-owned backend port while preserving the legacy render path.
2. `crates/acorn-lib/assets/prompts/suggest-fixes.prompt` or the actual existing prompt asset directory: add the versioned fix request/output contract and register `PromptFileAsset::SuggestFixes`.
3. `crates/acorn-lib/src/fix/mod.rs`: add references, proposals, report grouping, validation, and deterministic-provider interfaces.
4. `crates/acorn-lib/src/lib.rs`: export the service and fix boundaries without pulling native process support into minimal builds.
5. `crates/acorn-lib/src/io/acp/mod.rs` plus `installation.rs` and `opencode.rs` if warranted: add the process descriptor, preflight, client, permission policy, and ACP result mapping.
6. `crates/acorn-lib/src/io/mod.rs` and workspace manifests: feature-gate ACP and its optional SDK/process dependencies.
7. `crates/acorn-lib/src/io/api/openai.rs`: retain raw functions and add typed request/result mapping implementing the same inference port.
8. Focused test modules and fake ACP/HTTP executables: prove hydration, backend equivalence, process cleanup, local validation, and inability to deserialize executable actions.

Do not add CLI apply behavior in this slice. Once suggestions are stable, adapt one deterministic document fix, then structure doctor findings and add one fake package recipe end to end before supporting real installation recipes.

## Prompt preparation using schema::agent

Add a transport-independent prompt document boundary instead of teaching the ACP adapter about Tera or embedded assets. Reuse `schema::agent` prompt selectors, configuration, and normalized metadata, but put new asset hydration in the in-tree `agent::prompt` service boundary so schema ownership moves in the direction required by the crate-boundary plan.

1. Introduce a serializable render context that clearly separates prompt configuration from caller data.
2. Add a hydrator that loads a `PromptFileAsset`, renders it in strict mode, splits frontmatter from the body, and returns a typed `HydratedPrompt` containing the body plus normalized metadata needed by inference.
3. Align the embedded asset variables and the schema representation. Preserve `PromptTemplate::render(asset, config)` behavior during the in-tree slice; migrate it to a facade wrapper only when asset ownership moves out of the schema implementation.
4. Reject missing required values, non-object data where an object is required, reserved-key collisions, invalid rendered frontmatter, and any remaining Tera delimiters before an agent process starts.
5. Keep hydration pure: it accepts values and returns a document or typed error without filesystem, process, network, or session side effects.

Do not expand the existing `FrontMatter` struct to mirror every dotprompt field in the first slice. Parse rendered YAML into a private raw value, extract and validate the stable fields inference needs (`name`, output format/schema, model hint, tools, and descriptive metadata), and preserve unconsumed keys in an `extra` map for forward compatibility. The normalized public `PromptMetadata` must not expose raw Tera syntax or imply that an unvalidated field is supported.

The initial context should expose stable `config` and `data` namespaces. Canonical Tera expressions use snake_case keys such as `config.max_tokens` and `data.text`; do not place kebab-case identifiers inside expressions. Migrate built-in assets to Tera's explicit `default(value=...)` syntax where defaults are needed. Compatibility aliases for existing top-level convenience fields may be produced by one normalization function and removed after all bundled assets and documented callers use the canonical namespaces; do not duplicate aliases in ACP or HTTP adapters.

## Fix model parallel to checks

Add a shared `acorn::fix` domain rooted at `crates/acorn-lib/src/fix/mod.rs` so analysis checks, doctor diagnostics, and future finding sources can use it without depending on one another. Keep host-side executors behind native I/O/service capability boundaries.

- `FindingReference`: a tagged reference to an analysis check or doctor diagnostic, designed to admit future finding sources without changing fix identity;
- `CheckReference`: request-local check identifier, standard, category, severity, URI, locator, concise message/context, and a fingerprint of the source value or excerpt used to generate the fix;
- `DoctorReference`: diagnostic category, subject such as `node`, observed/expected state, platform context, and a fingerprint of the observation used to generate the fix;
- `Fix`: fix identifier, `FindingReference`, target, summary, rationale, confidence, actions, status, automation, risk/effects, and provenance;
- `FixAction`: a serializable proposal limited to document-change descriptions, manual instructions, or an opaque reference to a registered recipe. It cannot contain an executable path, shell text, package repository, or argument vector;
- `ExecutableFix`: a non-inference execution type produced only by trusted deterministic providers or by resolving a known recipe ID. It may contain format-aware transforms, verified downloads, curated package installation, configuration writes, or directory creation, with fixed argument vectors, preconditions, declared effects, and expected postconditions;
- `FixStatus`: proposed, unsupported, rejected, stale, applied, or failed. Inference produces only proposed, unsupported, or rejected states; trusted executors own applied and failed transitions;
- `FixAutomation`: safe automatic, confirmation required, interactive only, or manual;
- `FixRisk` and `FixEffects`: risk level plus explicit filesystem, network, process, package-manager, elevation, restart, and reversibility effects;
- `FixProvenance`: deterministic provider or inference source, recipe/prompt version, backend kind, agent/model identifier, and request identifier without credentials or raw environment values;
- `FixPlan` and `FixOutcome`: trusted executable actions, dependency/conflict ordering, previewed effects, execution results, rollback information, and verification results;
- `CategoryFixes`, `FixBatch`, and `FixReport`: grouping by analysis standard/check category or doctor diagnostic category;
- `FixOptions`: included finding sources/severities/categories, batching and context limits, minimum confidence, backend selection, automation ceiling, and allowed effects.
- `IntoFixes`: conversion for analyzers that already provide deterministic replacements, parallel to `IntoChecks`, so those proposals enter the same report without inference.

Do not add generated prose directly to `Check`. A check remains an observation; a fix is a separately reviewable proposal linked through `CheckReference`. Use request-local identifiers for response correlation and source fingerprints for staleness detection rather than changing every existing check constructor to allocate a persistent identifier.

Mirror the useful report behavior rather than copying the implementation mechanically: `FixReport::fixes()` flattens grouped results like `AnalysisReport::checks()`, category summaries count proposed/unsupported/rejected/applied fixes, and JSON/CSV presentation consumes the structured report. Fix status, automation, risk, and confidence replace check success and severity because a remediation has different lifecycle semantics from an observation.

## Extensible fix providers and executors

Separate discovery, planning, execution, and verification so doctor remediations do not force host commands into analysis code:

- `FixProvider`: pure or read-only discovery that maps supported findings to declarative `Fix` values;
- `FixPlanner`: accepts trusted deterministic fixes or resolves proposal recipe references through the registry, filters by policy, coalesces duplicates such as Node satisfying npm/npx, resolves dependencies/conflicts, and produces an immutable preview of `ExecutableFix` values;
- `FixExecutor`: capability-scoped adapter for one trusted action family, such as document edits, verified downloads, or package installation;
- `FixVerifier`: reruns the originating check or doctor diagnostic and records whether the finding is resolved;
- `FixPolicy`: enforces offline mode, approval, allowed effects, maximum risk, elevation rules, and whether non-reversible actions are permitted.

Do not expose a general model-generated shell-command action, and do not implement `Deserialize` for executable host command specifications. Inference may propose document changes, manual remediation text, or a registered recipe ID, but process/package actions become executable only after registry resolution to a trusted `ExecutableFix`. Executors must avoid shell concatenation, print an exact dry-run preview, recheck preconditions immediately before mutation, and record stdout/stderr only after redacting secrets.

Document edits use expected original values or hashes, atomic writes, and recoverable backups where practical. Package installation is usually non-reversible, may require network access/elevation, and therefore defaults to confirmation or interactive mode. Successful execution never implies success by itself; the original finding must be rerun.

## Initial deterministic auto-fix candidates

| Finding | Candidate fix | Automation | Verification |
|---|---|---|---|
| RAD values differ from `ResearchActivity::format_with` output | Apply the existing format transform and atomically rewrite the same document | Safe automatic after preview when the source fingerprint still matches | Re-read, validate, and compare normalized output |
| Vale issue exposes a supported replacement action, exact span, and expected text | Apply the declared text replacement | Safe automatic after stale-span check | Rerun the same Vale rule |
| RAD lacks deterministic linked-data context already produced by the link workflow | Apply the existing context augmentation | Confirmation required because document structure changes | Re-read and rerun schema/link checks |
| Vale is missing and prose checking is requested | Use the existing verified Vale resolver/downloader | Confirmation required; network-sensitive | Resolve `vale`, read its version, and rerun prose preflight |
| Chromium is missing for PDF export | Reuse the existing Chromium installer path | Confirmation required; network and disk effects | Resolve the installed executable and run the existing browser preflight |
| OpenCode is missing for an ACP-selected workflow | Offer a curated, platform-specific installation recipe when one is registered; otherwise emit manual instructions | Interactive or manual | Run the shared OpenCode installation/version preflight |
| Node.js is missing | Select a curated recipe for a detected supported package manager; treat npm/npx as capabilities supplied by the Node installation | Interactive; network, package-manager, and possible elevation effects | Resolve `node`, `npm`, and `npx` and validate their versions |
| Git or Pandoc is missing for a workflow that requires it | Select a curated platform/package-manager recipe or provide manual instructions | Interactive or manual | Rerun the individual software diagnostic |
| OpenAI-compatible endpoint/model readiness fails | Correct only deterministic local configuration mistakes; authentication, remote service, or model-access failures remain manual | Confirmation or manual | Repeat a bounded readiness request without exposing credentials |

Do not auto-fix dead URLs, missing required scientific content, ambiguous schema values, readability findings, crosswalk loss, or inferred prose rewrites. Those remain suggestions because ACORN cannot prove the intended replacement.

Add an embedded `suggest-fixes.prompt` asset whose data is a bounded list of `CheckReference` values plus only the source excerpts needed to understand them. Its output contract is a versioned JSON fix envelope derived from the Rust fix schema. The prompt must allow an explicit unsupported result instead of forcing the model to invent a correction.

## Shared inference contract

With ACP and OpenAI-compatible HTTP as two real implementations, introduce one small inference port rather than letting fix orchestration branch on transports:

- `InferenceRequest`: hydrated prompt, optional JSON output schema, model/agent selection, timeout, and non-mutating policy;
- `InferenceCapabilities`: structured output, streaming, tools, attachments, and mutation/permission characteristics discovered or configured for the backend;
- `InferenceEvent`: text deltas and the minimal stable set of status/tool events shared by callers;
- `InferenceResult`: final text, optional structured JSON, finish reason, usage when available, request/session identifier, and backend provenance;
- `InferenceBackend`: one async inference operation implemented by ACP and OpenAI-compatible adapters.

Keep request/result DTOs and the consumer-owned port with the in-tree agent service boundary, consistent with the crate-boundary plan. While the repository remains a single library crate, adapters may be composed in the check/fix service; they must not leak ACP message types or provider response JSON into `Fix`.

Normalize only genuine common behavior. ACP sessions can request permissions and emit tool updates, whereas OpenAI-compatible endpoints may be single HTTP requests with no agent process. Capability checks must make those differences explicit rather than pretending the transports are identical.

## OpenAI-compatible inference adapter

Extend `io::api::openai` instead of creating a second HTTP client:

1. Add typed request and response DTOs for the subset needed by prompt inference and fix suggestions while retaining the existing raw endpoint functions for compatibility.
2. Support an explicit endpoint mode: Responses API or Chat Completions. Prefer Responses for the official OpenAI API, but do not silently fall back between endpoints after arbitrary errors; local compatibility servers select their supported mode through configuration or a readiness probe.
3. Map the optional fix JSON Schema to the endpoint's structured-output request shape when supported. Otherwise request JSON text, parse it, and run the same local schema and semantic validation used for ACP output.
4. Normalize response text, structured payloads, finish reasons, usage, model identifiers, refusals, and provider errors into `InferenceResult`.
5. Reuse `OPENAI_SERVER_HOST` and `OPENAI_API_KEY`, support an explicit base URL and model in `FixOptions`, and keep tokens in secret-bearing configuration that is never serialized, logged, or copied into provenance.
6. Respect offline mode: reject remote HTTP inference; allow only an explicitly configured loopback/local endpoint after applying the repository's existing offline policy.
7. Test against the in-repository HTTP server for both endpoint modes, missing/invalid credentials, structured and unstructured outputs, refusal/error envelopes, timeouts, and malformed compatible-server responses.

## ACP client in io::acp

Create `crates/acorn-lib/src/io/acp/mod.rs`, gated by a separate `acp` capability, with a small public surface:

- `AgentProcessConfig`: agent identifier, executable, ACP arguments, version arguments, working directory, environment overrides, timeout, and permission policy;
- an OpenCode constructor or thin adapter that supplies `opencode`, `acp`, and `--version` defaults without changing the shared transport;
- ACP mapping from the shared `InferenceRequest`, `InferenceEvent`, and `InferenceResult` types;
- `AgentInstallation`: agent identifier, resolved executable path, and detected version information from a successful preflight check;
- `AcpClient`: the `InferenceBackend` implementation operating on an `AgentProcessConfig`.

Implementation flow:

1. Run a read-only installation preflight before starting ACP: resolve the configured executable path or find it on `PATH`, verify it is executable, invoke its configured version arguments, and return `AgentInstallation`. For the OpenCode adapter this resolves `opencode` and invokes `opencode --version`. Fail with a dedicated not-installed or unusable-version error that identifies the agent, searched command/path, and executable override option.
2. Spawn the configured ACP command, initially `opencode acp`, with piped stdin/stdout, separate stderr, a controlled working directory, and kill-on-drop behavior.
3. Use the official `agent-client-protocol` runtime crate rather than maintaining JSON-RPC envelopes locally.
4. Initialize the connection, negotiate protocol version and capabilities, create a session, apply supported agent/model selections, and send only the hydrated prompt body as text content.
5. Fold streamed ACP session updates into immutable result state while optionally forwarding typed events to the caller.
6. Answer permission requests through the configured policy. The inference default denies writes, shell commands, and other mutating operations; an allow or interactive policy must be explicit.
7. On completion, cancellation, timeout, malformed protocol data, or child exit, close the connection and reap the subprocess. Include bounded stderr context in errors without logging prompt data, credentials, or environment values.

Keep the extension seam data-driven and protocol-focused: one immutable process descriptor feeds one ACP client. Do not add a provider-neutral behavior framework, speculative agent enum variants, or agent-specific branches in the session loop. A future Codex, Goose, or Pi integration should add a thin constructor or adapter only after its direct ACP command or required ACP bridge is verified. Extract a richer agent trait only when a second working integration proves behavior that cannot be represented by `AgentProcessConfig` and negotiated ACP capabilities.

Expose the generic installation preflight separately from `infer` so `acorn doctor`, the prompt CLI, and library consumers can check a selected agent without creating an ACP session. `infer` must call the same preflight internally so callers cannot accidentally bypass it. Reuse the repository's command-discovery and semantic-version utilities where their error detail is sufficient; do not implement a second `PATH` search. OpenCode-specific diagnostics may wrap the generic result but must not fork the discovery implementation.

Return raw version output plus an optional parsed semantic version rather than making semantic-version parsing a requirement for every future agent. Add a separate protocol-readiness call for users who explicitly request a handshake; installation preflight remains cheap and process-bounded, while inference readiness remains an opt-in smoke test.

## Feature and dependency wiring

1. Add the official `agent-client-protocol` crate as an optional workspace dependency with default features disabled and pin a reviewed compatible release.
2. Add only the Tokio process and I/O features required for stdio transport.
3. Add a separate, non-default `acp` library feature for the ACP SDK and Tokio process/I/O features. Do not add it to the existing default `agentic` server feature. The CLI may opt into `acorn-lib/acp` explicitly when shipping the command; minimal, alloc, default-library, and WASM builds must not resolve the ACP runtime or process support.
4. Run the license and duplicate-dependency checks before accepting the dependency update.

## CLI vertical slice

After the library client is covered by a deterministic fake agent, add one CLI route for end-to-end use. Prefer a focused shape such as:

```text
acorn prompt run summarize --data input.json
acorn prompt run translate --data input.yaml --set language=French
acorn prompt run summarize --data input.json --dry-run
acorn prompt run summarize --data input.json --agent opencode
acorn check activity.json --suggest-fixes --backend acp --agent opencode
acorn check activity.json --suggest-fixes --backend openai --model local-model
```

The final command spelling should be checked against the CLI vocabulary before implementation. Regardless of spelling, the first CLI slice must:

- select only known embedded `PromptFileAsset` values;
- expose an agent selector whose first accepted value is `opencode` and whose dispatch boundary can later register verified Codex, Goose, or Pi adapters without changing prompt hydration or ACP result handling;
- select ACP or OpenAI-compatible HTTP through an explicit backend option, with backend-specific readiness checks and no hidden fallback from one backend to another;
- load JSON or YAML object data and permit stdin for text-oriented prompts;
- make `--dry-run` print the hydrated prompt without launching OpenCode;
- check and report OpenCode installation readiness before inference, while allowing `--dry-run` to work without OpenCode installed;
- reuse the same readiness check from `acorn doctor`, reporting OpenCode as required only for ACP-backed workflows rather than for unrelated ACORN commands;
- write inference content to stdout and diagnostics through `tracing` with `Label` styling;
- accept explicit working-directory, agent, model, timeout, and non-mutating permission options without copying OpenCode credentials into ACORN arguments;
- produce a useful error when OpenCode is absent or lacks a required negotiated capability;
- render fix suggestions separately from checks, support a structured JSON `FixReport`, and never apply a suggestion unless a later explicitly authorized apply workflow is added;
- regenerate command help after the CLI contract is finalized.

# Delivery sequence

## 0. Align ownership, features, and upstream contracts

- Record the transitional in-tree module and future crate owner for each new public type or adapter using the ownership table in this plan.
- Add no implementation to `schema::agent` until the prompt selector/configuration surface is separated from its current I/O, database, rendering, and terminal imports sufficiently for the new code to depend in the intended direction.
- Select and pin a reviewed official ACP Rust SDK release, inspect its enabled dependency features and license, and base the adapter on its client example without exposing SDK types publicly.
- Define a separate non-default `acp` feature and confirm that the default library, minimal, alloc, and WASM dependency graphs do not select it.
- Confirm the existing API endpoint and environment helpers that the OpenAI adapter will reuse; record any dependency on the API roadmap instead of fixing unrelated API behavior in this feature.

Exit gate: ownership and feature tests show no new schema-to-host dependency, `acp` is independently selectable, default-library dependency resolution is unchanged, and the pinned SDK supports the client-to-agent stdio connection required by the fake-agent test design.

## 1. Normalize and verify prompt hydration

- Keep selectors/configuration under `schema::agent`; add render context and hydrated prompt types under the in-tree agent service boundary.
- Reconcile asset variables and frontmatter with the schema contract.
- Preserve the existing render entry point with regression coverage without introducing a schema-to-service dependency.
- Add value-level and failure-path tests for every embedded prompt.

Exit gate: all built-in assets render with representative data, contain the expected substituted values, contain no unresolved delimiters, and yield a non-empty body without any process or network access.

## 2. Establish the narrow inference contract

- Add request, capability, minimal event, result, and provenance DTOs plus the consumer-owned `InferenceBackend` port under the in-tree agent service boundary.
- Limit the first contract to one hydrated text prompt, optional output schema, configured model/agent selector, timeout, non-mutation intent, final text/structured value, finish reason, usage, and provenance.
- Add a deterministic in-memory backend so prompt orchestration can be tested before either external adapter exists.
- Keep endpoint errors, ACP permission request identifiers, and provider wire payloads out of the common DTOs.

Exit gate: a hydrated prompt runs through the in-memory backend and returns a normalized result without I/O, and serialization tests prove the contract contains neither credentials nor transport-specific messages.

## 3. Prove the ACP/OpenCode transport

- Add the optional official ACP dependency and feature wiring.
- Add the reusable agent installation preflight for explicit paths and `PATH` discovery, executable validation, and configurable version reporting; provide OpenCode defaults for `opencode --version`.
- Implement the ACP `InferenceBackend` mapping, process lifecycle, initialization, session creation, prompt sending, update folding, cancellation, timeout, and default permission denial.
- Return a typed result and typed errors that distinguish executable, spawn, negotiation, session, permission, protocol, timeout, and child-exit failures.
- Add separate installation and protocol-readiness operations; do not label either one as provider/model inference readiness.

Exit gate: installation preflight distinguishes missing, non-executable, version-command failure, and usable agent installations, the OpenCode adapter supplies the correct command defaults, and an in-repository fake ACP agent completes a one-shot streaming prompt deterministically on Windows and Unix without requiring OpenCode or model credentials.

## 4. Add the OpenAI-compatible adapter

- Extend `io::api::openai` with the narrow stable DTOs needed for Responses and Chat Completions while preserving its raw public functions and following the API response-DTO plan.
- Map the two endpoint modes independently; Responses structured output uses its endpoint-specific text-format shape, while Chat Completions uses its own response-format shape.
- Add explicit endpoint mode and capability configuration, secret handling, offline policy, typed refusals/errors, and local parsing/validation fallback.
- Reuse shared endpoint/environment/authentication helpers as they land rather than embedding a second precedence or URL policy in inference code.

Exit gate: the same hydrated prompt runs against deterministic fake `/responses` and `/chat/completions` endpoints, producing equivalent normalized results without external network access; configured capability differences remain visible rather than inferred from `/models`.

## 5. Add proposal-only fixes and check suggestions

- Add `FindingReference`, `CheckReference`, `Fix`, serializable `FixAction`, `FixStatus`, `FixAutomation`, `FixRisk`, `FixEffects`, `FixProvenance`, `FixOptions`, and grouped `FixReport` proposal types. Defer `ExecutableFix`, planning, execution, rollback, and outcome types to step 6.
- Add pure conversion from `AnalysisReport` to bounded serializable check references without exposing private renderer payloads or changing existing check constructors.
- Add `IntoFixes` only for analyzers that already expose deterministic replacement data, plus report flattening and category/status summaries.
- Add the `suggest-fixes.prompt` asset and generate or maintain its versioned JSON output schema from the proposal-only fix envelope.
- Add local validation for finding IDs, targets, SHA-256 source fingerprints, finite confidence bounds, allowed action variants, and invented or duplicate targets.

- Add a suggestion service that consumes an `AnalysisReport`, selects eligible unsuccessful checks, creates bounded batches, hydrates `suggest-fixes.prompt`, invokes an injected `InferenceBackend`, validates the returned envelope, and produces `FixReport`.
- Preserve deterministic analyzer corrections first and avoid sending already-resolved or unsupported check categories unless requested.
- Limit transmitted content to normalized checks and required excerpts; never send whole repositories or unrelated files by default.
- Treat partial backend failures as explicit rejected/unsupported fix records or batch diagnostics without changing the original checks.

Exit gate: the same analysis fixture produces structurally equivalent validated fixes through fake ACP and fake OpenAI-compatible backends; unknown findings, stale fingerprints, out-of-scope paths, malformed actions, and invented targets are rejected; no proposal type can deserialize an executable command.

## 6. Implement deterministic auto-fix planning and execution

- Add the provider, planner, executor, verifier, policy, recipe-registry, preview, and outcome boundaries.
- Add `DoctorReference`, structured `DoctorReport` findings, `ExecutableFix`, `FixPlan`, `FixOutcome`, and execution records now that the proposal contract is stable; retain existing doctor table and JSON outputs as views.
- Adapt RAD formatting/context augmentation, supported Vale replacement actions, Vale download, and Chromium installation as the first trusted providers rather than duplicating their implementations.
- Add structured doctor findings and curated installation recipes for supported platform/package-manager combinations. Model output may select or explain a registered recipe but may not create or alter its executable/arguments.
- Coalesce dependent software findings so one Node.js installation can satisfy Node, npm, and npx; reject conflicting plans.
- Define CLI policy: non-interactive `doctor --fix` may apply only safe, reversible fixes; package installation and other network/elevation/non-reversible actions require explicit interactive selection and confirmation.
- Recheck fingerprints and preconditions immediately before execution, then rerun the source diagnostic/check and record the verified outcome.

Exit gate: deterministic fixtures can preview, apply, verify, and where supported roll back document fixes; fake package-manager tests prove that Node/Vale installation recipes are selected, approved, invoked as fixed argument vectors, and verified without changing the test host.

## 7. Validate real interoperability

- Add an opt-in smoke test that launches a real `opencode acp` process only when explicitly enabled.
- Verify negotiated capabilities before asserting optional model, mode, or metadata behavior.
- Test against a local/sandbox model configuration where practical so CI does not send repository data to an external provider.
- Add a separately gated OpenAI-compatible smoke test that targets an explicitly configured endpoint and model; never run it from ordinary CI or infer credentials from test fixtures.

Exit gate: a hydrated prompt and one check-fix request produce non-empty, schema-valid results through OpenCode and through the configured OpenAI-compatible endpoint; the ACP process is reaped and neither route mutates source files.

## 8. Expose the CLI and reusable task hooks

- Add the chosen prompt-run command, data loading, dry-run behavior, streaming/output formatting, and CLI tests.
- Wire the shared installation preflight into `acorn doctor` with capability-scoped pass, warning, and failure messaging.
- Add explicit fix-suggestion CLI wiring after normal analysis, including backend/model/agent selection, structured output, batching controls, and suggestion-only messaging.
- Complete `doctor --fix` and interactive selection over the shared fix planner, clearly separating safe automatic actions from confirmation-required installation recipes.
- Expose event and permission hooks from the library for later tasks such as structured extraction, gap analysis, teaching, translation, or explicitly authorized repository work.
- Document the adapter checklist for a subsequent agent: verified ACP executable or bridge command, version probe, authentication/config ownership, capability differences, permission behavior, and an opt-in interoperability smoke test.
- Keep application of inference-generated fixes, arbitrary commands, multi-turn sessions, resume/fork, attachments, MCP-over-ACP, and automatic tool approval out of the first release unless separately accepted.

Exit gate: a user can preview and run an embedded prompt, request structured fix suggestions through either backend, and use doctor to preview or explicitly apply registered deterministic remediations with predictable output and verified outcomes.

# Validation

1. Add snapshot or exact-value tests for all embedded prompts with representative configuration and data, plus missing-value, unknown-asset, invalid-frontmatter, collision, and unresolved-template failures.
2. Add fix-domain tests for check/doctor correlation, grouping, serialization/schema round trips, confidence validation, source fingerprints, stale inputs, unsupported findings, duplicate actions, effect declarations, dependency/conflict planning, path confinement, and transport-free deterministic behavior. Prove inference JSON cannot deserialize into an executable command specification.
3. Add backend contract tests that run identical prompt and fix fixtures through fake ACP and fake OpenAI-compatible implementations and compare their normalized results.
4. Add fake-agent integration tests for initialization, capability negotiation, streamed text assembly, structured fix assembly, stop reasons, permission denial, timeout/cancellation, malformed messages, stderr diagnostics, early child exit, and process cleanup.
5. Add installation-preflight tests for an absent command, invalid explicit path, non-executable file where supported, failing version arguments, and a fake executable with a valid version response. Verify the same generic preflight works with at least two fake agent descriptors so the implementation is not accidentally coupled to the `opencode` binary name.
6. Add HTTP tests for Responses and Chat Completions modes, custom base URLs, missing authentication, structured-output capability, JSON fallback validation, refusals, usage, rate/provider errors, timeouts, and offline rejection of remote endpoints.
7. Add executor tests for atomic document edits and rollback, fixed command/argument invocation, offline/network policy, approval/elevation gates, timeouts, non-zero exits, redaction, and mandatory post-fix verification. Use fake package managers and isolated `PATH` values; never install software on the test host.
8. Add CLI parsing, command, and doctor tests for file input, stdin, dry-run without OpenCode installed, missing OpenCode during ACP inference, unavailable API endpoints, capability-scoped doctor output, fix JSON output, safe-only non-interactive fixes, interactive install selection, declined approval, invalid data, timeout, model/agent selection, and clean stdout.
9. Run focused tests for `schema::agent`, `agent`, `fix`, `analyzer`, `doctor`, `io::acp`, and `io::api::openai`, then `make lint` and `make test`.
10. Run `cargo check -p acorn-lib --no-default-features`, the existing alloc and WASM checks, `cargo check -p acorn-lib --no-default-features --features acp`, and independent `agentic`/`acp`/combined feature checks. Inspect feature trees to prove the default library and portable builds do not select the ACP SDK or Tokio process support. Confirm the OpenAI-compatible client and host fix executors stay behind native capability boundaries.
11. Run `make audit` or the repository's equivalent dependency policy checks after adding the ACP SDK.
12. Regenerate CLI help and build documentation when the CLI surface lands.

# Acceptance criteria

- `AnalysisReport` and structured `DoctorReport` findings can be transformed into a grouped `FixReport` whose proposals retain their originating finding and target provenance.
- Each model-generated fix references a supplied finding, includes a source fingerprint and expected original value where applicable, and passes local schema and semantic validation before it is exposed.
- Inference-generated fixes remain suggestions: they cannot grant mutating permissions, create executable commands, edit files, or mark findings resolved.
- Registered deterministic fixes can be previewed, policy-checked, explicitly applied, and verified through a shared planner/executor lifecycle.
- Non-interactive auto-fix runs execute only safe reversible actions; network, elevation, package-manager, restart, or non-reversible effects require interactive selection and confirmation.
- Doctor can represent missing Node.js, npm/npx, Git, Pandoc, Vale, OpenCode, and similar software as structured findings, then offer only curated platform/package-manager recipes or manual instructions.
- A single Node.js remediation can satisfy dependent npm/npx findings without duplicate installations.
- Model output cannot introduce package names, repositories, executables, or arguments into an executable action; host remediation resolves through trusted registered recipes.
- Executors accept only `ExecutableFix` plans produced by trusted providers or recipe resolution, never a deserialized inference `FixAction` directly.
- Every executed fix rechecks preconditions and reruns its originating check/diagnostic. Execution without successful verification is not reported as resolved.
- An unsupported or ambiguous check can produce an explicit no-fix result without forcing a fabricated operation.
- One shared inference request/result contract supports both ACP and OpenAI-compatible HTTP without leaking either wire format into checks or fixes.
- Public ACP execution code is rooted at `acorn::io::acp` and native process dependencies are feature-gated.
- Shared ACP prompt, request, event, result, permission, installation, process-lifecycle, and session code contains no OpenCode-specific branching; OpenCode supplies a thin launch configuration.
- Prompt selection, configuration, and normalized metadata reuse `schema::agent`; the in-tree agent service hydrates the existing embedded prompt assets without making ACP or HTTP adapters aware of Tera.
- Callers can hydrate every built-in prompt with concrete serializable data, and missing required data fails before OpenCode is launched.
- Callers can check OpenCode readiness without opening a session, and every inference call verifies that a usable `opencode` executable is installed before attempting ACP startup.
- Missing or unusable OpenCode installations produce an actionable typed error; dry-run prompt hydration remains available without OpenCode.
- `acorn doctor` reports OpenCode installation and version readiness in the ACP capability check without failing unrelated capability checks.
- Adding a verified Codex, Goose, or Pi ACP integration requires only a thin launch/probe adapter plus interoperability tests unless that agent demonstrates a genuinely different capability need.
- ACORN negotiates ACP capabilities, starts a session with `opencode acp`, sends the hydrated body, collects streaming output, and returns a typed result.
- The existing `io::api::openai` module can send the same hydrated prompt and fix schema through explicitly selected Responses or Chat Completions mode, using a configurable OpenAI-compatible base URL and model.
- Structured output is requested when supported, but all ACP and HTTP fix responses undergo the same local deserialization, validation, check-correlation, and path-confinement rules.
- Provider tokens, prompt source content, and raw environment values never appear in logs, serialized fix provenance, or error messages.
- Remote OpenAI-compatible inference is rejected in offline mode; any local exception requires an explicitly configured loopback endpoint.
- ACP inference is rejected in offline mode in the first release; hydration and dry-run do not require an agent or network.
- Inference denies mutating permission requests by default and never logs prompt data, credentials, or full environment values.
- Timeout, cancellation, protocol failure, and child exit always close and reap the OpenCode process.
- Tests do not require network access, OpenCode installation, or provider credentials; separately gated real-OpenCode and configured-API smoke tests cover interoperability.
- Existing prompt rendering and OpenCode configuration synchronization remain compatible.
- Minimal, alloc, and WASM builds do not resolve ACP or subprocess dependencies.
- Host fix executors remain outside portable schema/analysis builds, and tests never install packages or modify the real host environment.
- The CLI supports prompt preview, one-shot inference, structured fix suggestions, and policy-governed doctor auto-fix with stable output and documented diagnostics.

# Deferred decisions

- Whether the CLI should expose fix suggestions as `check --suggest-fixes`, `fix suggest`, or both through one shared service.
- Whether non-fix prompt outputs should opt into schema validation. Fix envelopes always require it.
- Which document action subset should be accepted from inference-generated suggestions. Deterministic providers may execute registered format-aware transforms, but generated text/structured edits remain review-only initially.
- Which package managers and installation recipes are supported on each OS/architecture. Unsupported combinations return manual instructions rather than guessed commands.
- Whether fixes should receive durable identifiers across separate runs. The first suggestion workflow needs request-local correlation plus source fingerprints, not a new persistent check identity system.
- Whether long-lived, resumed, or forked ACP sessions belong in the CLI, TUI, or a later service API.
- Which Codex, Goose, and Pi executables provide ACP directly versus requiring maintained bridge processes, and the exact invocation/version probes for each. Verify these against their current primary documentation before adding an adapter.

# Related plans

- [Crate boundary architecture](../refactors/crate-boundary-architecture.md)
- [Portability roadmap](../refactors/portability/roadmap.md)
- [Portability roadmap](../refactors/portability/roadmap.md)
- [Host abstraction traits](../refactors/portability/host-abstraction-traits.md)
- [API client roadmap](../refactors/api/roadmap.md)
- [Safe path](../refactors/core/safe-path.md)
- [CLI parse and execution models](../refactors/core/cli-parse-execution-models.md)
