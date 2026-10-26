# Intelligence in Depth layers

Intelligence in Depth (IID) organizes automation and model-assisted systems into seven layers, numbered 0 through 6. The layers are composable: use the shallowest sequence that reliably satisfies the task, starting with deterministic automation. See [...But Is It Agentic?](agentic.md) for the architecture's principles.

## 0. Deterministic automation

Explicit rules and code handle tasks with defined inputs, transformations, and acceptance criteria. Schemas describe accepted data structures, validators enforce constraints, and conversion code maps fields or serializes records according to known rules. With fixed inputs, rule versions, and configuration, these operations produce repeatable results without model inference.

For example, ACORN can [validate a research record](../cli-reference/commands/check.md), [normalize its formatting](../cli-reference/commands/format.md), or [convert DataCite metadata to DCAT](../cli-reference/commands/export.md#standard-crosswalks) using implemented field mappings. Schema generation, identifier syntax and checksum checks, and artifact generation also belong here.

A task can finish at layer 0. A script or pipeline that composes deterministic operations stays at this layer; model workflow composition begins at layer 3. Higher layers can call layer 0 operations as tools and use the same validators to check model-generated data. Validation establishes conformance to the encoded rules; factual accuracy and ambiguous mappings may still require source evidence or human review. Conversions can be lossy, so their documented limitations and reported warnings remain part of interpreting the result.

## 1. One-time prompting

An application sends a direct instruction to a model through a library or API. This layer suits a single transformation or answer whose inputs and outputs are already defined.

For example, an application can send one research narrative to `acorn-lib`'s OpenAI-compatible chat-completion API with the instruction "Summarize this activity," then return the response.

"OpenAI-compatible" describes the connection format, not the model provider. A compatible server accepts supported OpenAI-style HTTP endpoints and JSON requests and responses. The server may be OpenAI's service or another implementation, including llama-swap, vLLM, or LiteLLM. ACORN uses this format because local and hosted servers widely support it, which reduces custom client code.

## 2. Prompt templates

Prompt templates store reusable instructions in version-controlled `.prompt` files with defined inputs, configuration, and expected outputs. Google's Dotprompt specification documents this executable file pattern.[^dotprompt]

For example, ACORN renders its embedded `summarize.prompt` template with a source narrative, maximum word count, and output structure so callers can reuse the same instructions.

## 3. Workflow composition and iteration

A workflow coordinates model calls, deterministic operations, and tools through explicit control flow. It may sequence, select, repeat, delegate, or combine bounded tasks, but the application defines its path and safeguards. Anthropic distinguishes these predefined workflows from agents that direct their own process and tool use.[^effective-agents]

Repeated workflows need clear exit conditions, measurable progress, state passed between calls when required, and limits on iterations, time, tokens, and cost.

A planned ACORN workflow could ask a model to address gaps in a research narrative, run deterministic validation, and pass remaining findings to another bounded attempt. It would stop when no high-severity issue remains or when it reaches an iteration, token, or time limit.

## 4. Skills, context, and capabilities

This layer assembles the instructions, research data, evidence, examples, tools, constraints, and current state needed for an invocation. Agent Skills package specialized instructions and optional resources into portable, version-controlled directories.[^skills] MCP can connect an agent to live prompts, resources, and tools.[^mcp]

This is the layer where an application makes a model context-aware for a specific task. The model can use supplied relationships and provenance to explain connections or identify gaps, but it cannot treat an unsupported inference as source data.

For example, an agent can load the ACORN skill with a `ResearchActivity` schema, source record, validation results, and allowed tools before proposing a correction. The local stdio MCP server exposes the same canonical tool registry and deterministic checks through a standard interface.

## 5. Connected memory

Connected memory preserves selected state across iterations, sessions, or tasks. Useful memory includes attributable facts, decisions, provenance, and prior results. Its origin, availability, modality, and quality are part of its [data](aspect/data.md) profile.

A planned workflow could retrieve a previously reviewed project identity, identifier mapping, and provenance record instead of reconstructing them from conversation history.

## 6. Agent harness and interoperability

An agent harness orchestrates models, skills, tools, workflows, context, memory, permissions, validation, recovery, observability, and human approval. It owns resource budgets and stopping conditions. Permissions and approvals shape [autonomy](aspect/autonomy.md), while perception, state, actions, and feedback shape [motivity](aspect/motivity.md). ACP can standardize sessions, streamed updates, and permission requests between a harness and external agents.[^acp]

ACORN launches OpenCode through ACP for bounded fix suggestions, sends the normalized check results as context, streams progress, and applies a fixed permission policy. The returned suggestions remain reviewable data; `acorn check` does not apply them.

ACORN's [Protobuf/gRPC transport](../packages/grpc.md) provides part of the foundation for that work. Its shared operation registry, policy context, authentication, deadlines, and cancellation behavior can support more protocol adapters. ACP will still use ACP sessions, streaming, and permission messages, while MCP keeps its own tool discovery and invocation semantics.

## Applying the principles

| # | IID layer | Local-first, file over app, and user sovereignty |
| --- | --- | --- |
| 0 | Deterministic automation | Validate and convert data locally using schemas and code. Keep inputs, outputs, rule versions, and configuration inspectable and versionable so users can reproduce the work without a model service. |
| 1 | One-time prompting | Prefer a local model when it satisfies the task. Make remote transmission explicit. Let users save the prompt and result independently of the model service. |
| 2 | Prompt templates | Keep reusable instructions in portable, version-controlled `.prompt` files. |
| 3 | Workflow composition and iteration | Persist inputs, control flow, exit criteria, budgets, checkpoints, and results so the workflow remains inspectable and resumable. |
| 4 | Skills, context, and capabilities | Package reusable instructions and references in `SKILL.md` directories. Use MCP for live resources and tools while keeping durable context in files. |
| 5 | Connected memory | Preserve selected facts, decisions, and provenance in inspectable records. Databases may index them without becoming the only source. |
| 6 | Agent harness and interoperability | Configure workflows through files and CLI options. Treat applications and ACP agents as replaceable interfaces. |

## Current ACORN support

| # | Layer | Status | ACORN support |
| --- | --- | --- | --- |
| 0 | Deterministic automation | Implemented | [Schemas and validators](../packages/acorn-schema.md), identifier checks, canonical formatting, [metadata crosswalks](../cli-reference/commands/export.md#standard-crosswalks), and artifact generation run without model inference. These operations are available through library APIs and commands such as `acorn check`, `acorn format`, and `acorn export`. |
| 1 | One-time prompting | Available as a library API | [`acorn-lib`](../packages/acorn-lib.md#ai-and-agent-apis) exposes completion, chat-completion, response, embedding, image, and audio operations through supported OpenAI-style HTTP endpoints and JSON formats. Applications decide when to invoke a model. ACORN does not provide a general-purpose interactive prompt command. |
| 2 | Prompt templates | Implemented | `acorn-lib` embeds and renders versionable `.prompt` templates for summarization, translation, teaching, simple explanation, claim extraction, and gap finding. Templates accept source text, output limits, sampling parameters, and stop sequences. |
| 3 | Workflow composition and iteration | Not implemented | ACORN does not provide a general model workflow executor with explicit control flow, exit criteria, evolving state, progress checks, and resource budgets. File watching and service polling are operational loops, not IID model workflows. |
| 4 | Skills, context, and capabilities | Implemented | [`acorn skill`](../cli-reference/commands/skill.md) distributes task-oriented instructions. [`acorn serve mcp`](../cli-reference/commands/serve.md), MCPB packaging, and opt-in client synchronization expose the canonical policy-filtered tool registry. [`acorn mcp`](../cli-reference/commands/mcp.md) discovers and invokes configured remote tools without automatically re-exporting them. |
| 5 | Connected memory | Building blocks available | ACORN's local database persists research candidates, activity history, model metadata, and provenance-bearing records. ACORN does not manage conversational memory, cross-session agent recall, or memory selection and retention policies. |
| 6 | Agent harness and interoperability | Partially implemented | ACORN manages local model metadata and weights, [synchronizes model configuration](../cli-reference/model-workflows.md) into OpenCode, VS Code, Goose, and llama-swap, distributes its agent skill, runs a GitLab bot service, provides a versioned gRPC boundary over its shared operation registry, and supervises one-shot OpenCode sessions through ACP. It does not provide a unified general-purpose agent runtime. |

ACORN can develop each layer independently. Layer 0 remains useful without any model integration, and its deterministic operations can become tools for higher layers. Prompt evaluation can precede workflow execution, and memory provenance can be defined before agent state is retained.

[^dotprompt]: Google, [*Dotprompt specification*](https://github.com/google/dotprompt/tree/main/spec) and [*Dotprompt reference documentation*](https://google.github.io/dotprompt/).
[^effective-agents]: Anthropic, ["Building effective agents"](https://www.anthropic.com/engineering/building-effective-agents), 2024.
[^skills]: Agent Skills, [*Specification*](https://agentskills.io/specification).
[^mcp]: Model Context Protocol, ["Architecture"](https://modelcontextprotocol.io/specification/2025-06-18/architecture) and ["Server features"](https://modelcontextprotocol.io/specification/2025-06-18/server/index).
[^acp]: Agent Client Protocol, ["Architecture"](https://agentclientprotocol.com/get-started/architecture).
