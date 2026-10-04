# ...But Is It Agentic?

ACORN is developing agent capabilities through Intelligence in Depth (IID), an architecture that layers prompts, workflows, context, memory, and agent supervision. Each layer remains useful on its own, and a system can stop at the shallowest layer that meets its needs.

> [!NOTE]
> IID terminology is internal to ACORN and is being prepared for formal publication in the [Journal of Open Research Software (JORS)](https://openresearchsoftware.metajnl.com). Cite this page as a pre-publication description and contact the authors about prior-art claims.

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>IID organizes model use into six layers, from a single prompt through a supervised agent harness. The architecture keeps durable work in files and supports local operation.</p>
</blockquote>

IID adapts defense in depth, an information security strategy that combines people, technology, and operations across multiple layers.[^defense-in-depth] It distributes intelligence across reusable prompts, controlled workflows, skills and context, memory, and an interoperable agent harness.

```pikchr
scale = 0.75
Prompting: box "One-time" "prompting" wid 0.92in ht 0.55in rad 0.08in fill white thickness 0
arrow right 0.35in thick color 0x00662C
Templates: box "Prompt" "templates" wid 0.92in ht 0.55in rad 0.08in fill white thickness 0
arrow right 0.35in thick color 0x00662C
Loops: box "Workflow" "composition" wid 0.92in ht 0.55in rad 0.08in fill white thickness 0
arrow right 0.35in thick color 0x00662C
Context: box "Skills + context" "(+ MCP)" wid 1.15in ht 0.55in rad 0.08in fill white thickness 0
arrow right 0.35in thick color 0x00662C
Memory: box "Connected" "memory" wid 0.92in ht 0.55in rad 0.08in fill white thickness 0
arrow right 0.35in thick color 0x00662C
Harness: box "Harness" "(+ ACP)" wid 0.92in ht 0.55in rad 0.08in fill white thickness 0
```

The arrows show composition and control flow. IID is distinct from the [ASPECT maturity](aspect/maturity.md) scale. IID describes layered control, while [autonomy](aspect/autonomy.md) describes how humans and machines divide responsibility.

## Principles

IID uses two principles across every layer:

- Local-first operation keeps data, models, validation, and orchestration on the user's machine by default. Network and remote services remain explicit choices.[^local-first]
- File over app keeps durable instructions, context, state, and results in files that users can retrieve, read, and move between applications.[^file-over-app]

Files remain the source of truth when model APIs or protocols connect IID to external capabilities. The Model Context Protocol (MCP) can give agents access to prompts, resources, and tools.[^mcp] The Agent Client Protocol (ACP) can let clients communicate with and supervise external agents.[^acp]

Local-first operation can include remote services when users select them. It differs from [portability](aspect/portability.md), which describes where a complete capability can run after accounting for model, tool, and service dependencies. Databases may provide rebuildable indexes and operational storage. Credentials belong in environment variables or dedicated secret stores, not portable project files.

## Context awareness in IID

Within IID, context awareness means that a model or workflow receives the structured information needed for its task. For ACORN, that can include a research record, its schema, persistent identifiers, provenance, relationships, source material, prior decisions, validation results, and allowed tools.

This supplied context gives a model evidence for interpreting the project and tracing connections between records. It can also expose missing fields, conflicting observations, or unsupported relationships. The harness must preserve the boundary between recorded evidence and model inference so generated suggestions do not become authoritative data without validation or review.

## How IID guides ACORN

ACORN keeps schema validation, identifier checks, formatting, linking, and artifact generation deterministic. AI can consume or produce data around those operations, while the deterministic rules provide validation and provenance.

IID guides ACORN development in these ways:

- ACORN schemas, validators, persistent identifiers, and provenance provide machine-checkable context for model-assisted work.
- Prompts, skills, model APIs, local-model configuration, and future tools remain useful separately and can be composed when a workflow needs them.
- Iteration has exit criteria and budgets. Memory has provenance and lifecycle rules. Tools have permissions, and consequential actions require validation or human approval.
- Core operations and locally available models continue to work without a remote AI service. Network use respects ACORN's offline boundary.
- Research records, `.prompt` templates, `SKILL.md` instructions, configuration, and generated artifacts remain portable and under user control.

MCP and ACP are optional extensions. MCP defines what context and capabilities an agent can access. ACP defines how a client communicates with and supervises an agent. Skills remain portable instruction and resource packages independent of both protocols, while model APIs provide direct model invocation.

| Optional extension | Initial ACORN role | Purpose |
| --- | --- | --- |
| Protobuf/gRPC | Server and client library, available | Expose the shared operation registry through a versioned binary contract and establish reusable transport, policy, deadline, and cancellation boundaries for protocol integrations. |
| MCP | Server and allowlisted client, available | Let external agents call deterministic ACORN tools, and let ACORN applications call configured remote MCP tools through bounded one-operation sessions. |
| ACP | Client, planned | Let ACORN invoke and supervise interchangeable agents, beginning with OpenCode. |
| OpenAI-compatible API | Client, available | Let ACORN applications call a model server through supported OpenAI-style HTTP endpoints and JSON formats without requiring a full agent. |

The [Protobuf/gRPC transport](../packages/grpc.md) is an implementation stepping stone for ACP and MCP integrations. It proves that protocol adapters can share registered operations, validation, effect metadata, authentication context, offline and mutation policy, and bounded operation errors. Its transport adapter also establishes deadline and cancellation handling. ACP sessions and MCP tool semantics remain native to their respective protocols rather than being encoded as gRPC calls.

See [Intelligence in Depth layers](agentic-layers.md) for the six layers, examples, and current implementation status.

[^defense-in-depth]: National Institute of Standards and Technology, ["defense-in-depth," *Computer Security Resource Center Glossary*](https://csrc.nist.gov/glossary/term/defense_in_depth). The glossary traces the definition to CNSSI 4009-2015 and multiple NIST publications, including SP 800-53 Rev. 5.
[^mcp]: Model Context Protocol, ["Architecture"](https://modelcontextprotocol.io/specification/2025-06-18/architecture) and ["Server features"](https://modelcontextprotocol.io/specification/2025-06-18/server/index).
[^acp]: Agent Client Protocol, ["Architecture"](https://agentclientprotocol.com/get-started/architecture).
[^local-first]: Martin Kleppmann, Adam Wiggins, Peter van Hardenberg, and Mark McGranaghan, ["Local-first software: You own your data, in spite of the cloud"](https://www.inkandswitch.com/essay/local-first/), Ink & Switch, 2019, doi: [10.1145/3359591.3359737](https://doi.org/10.1145/3359591.3359737).
[^file-over-app]: Steph Ango, ["File over app"](https://stephango.com/file-over-app), 2023.
