# ...But Is It Agentic?

🌱 **It is growing in that direction.** ACORN approaches agentic development through **Intelligence in Depth (IID)**: the deliberate layering of AI capabilities so that each layer adds structure, context, continuity, or control.

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a Nutshell</p>
<p>IID layers prompting, templates, workflows, skills and context, memory, and agent harnesses. Each layer adds capability and control while staying local-first and file-oriented; MCP and ACP are optional extensions.</p>
</blockquote>

IID adapts the idea of defense in depth: an information security strategy that integrates people, technology, and operations across multiple layers.[^defense-in-depth] Instead of expecting one prompt or one model to carry an entire workflow, IID distributes intelligence across reusable prompts, controlled workflows, skills and connected capabilities, memory, and an interoperable agent harness. The layers reinforce one another, but each remains useful on its own:

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

*Arrows indicate composition and control flow, not increasing maturity or quality.*

IID is an ordered architecture, not an [ASPECT maturity](aspect/maturity.md) scale. The sequence shows how each layer can structure, supply, or govern the layers before it. A system is not more mature for using an agent harness; it should use the shallowest sequence that reliably satisfies the task. IID is therefore not a synonym for maximum [autonomy](aspect/autonomy.md).

## Intelligence in Depth

### Guiding Principles

Two principles constrain every IID layer:

- **Local-first.** Data, models, validation, and orchestration remain on the user's machine by default. The network and remote services are explicit choices, not prerequisites. Local-first preserves user ownership and offline operation without prohibiting synchronization or collaboration.[^local-first]
- **File over app.** Durable instructions, context, state, and results live in files the user can control, retrieve, and read. Applications are replaceable tools that operate on those files rather than the only place the work can exist.[^file-over-app]

Together they produce a third rule: **protocols are adapters, not dependencies**. MCP, ACP, and model APIs can connect IID to external capabilities without becoming the source of truth.

| IID layer | Local-first and file-over-app application |
| --- | --- |
| **One-time prompting** | Prefer a local model when it satisfies the task; make remote transmission explicit and allow the prompt and result to be saved. |
| **Prompt templates** | Keep reusable instructions in portable, version-controlled `.prompt` files rather than an application-only prompt library. |
| **Workflow composition and iteration** | Persist inputs, control flow, exit criteria, budgets, checkpoints, and results so a workflow is inspectable and resumable outside one application session. |
| **Skills, context, and capabilities** | Package instructions and references in `SKILL.md`-based directories; use MCP only when live resources or tools add value. |
| **Connected memory** | Preserve selected facts, decisions, and provenance in inspectable records; databases may index them without becoming an opaque source of truth. |
| **Agent harness and interoperability** | Configure workflows through files and CLI options; treat applications and ACP agents as replaceable interfaces. |

1. **One-time prompting** sends a direct instruction to a model through a library or API. It is appropriate for a single transformation or answer whose inputs and outputs are already well defined.

   *Example:* An application sends one research narrative to `acorn-lib`'s OpenAI-compatible chat-completion API with the instruction "Summarize this activity," then returns the model's response.

2. **Prompt templates** turn instructions into reusable, version-controlled `.prompt` files with defined inputs, configuration, and expected outputs. Templates make model behavior easier to share, test, and improve. Google's Dotprompt specification documents this executable `.prompt` file pattern.[^dotprompt]

   *Example:* ACORN renders its embedded `summarize.prompt` template with a source narrative, maximum word count, and output structure so different callers reuse the same instructions.

3. **Workflow composition and iteration** coordinates model calls, deterministic operations, and tools through explicit control flow. A workflow may sequence, select, repeat, delegate, or combine bounded tasks, but its path and safeguards remain defined by the application. Anthropic distinguishes these predefined workflows from agents that dynamically direct their own process and tool use.[^effective-agents] Repeating workflows require clear exit conditions, measurable progress, state passed between calls when needed, and limits on iterations, time, tokens, and cost.

   *Example (planned):* ACORN asks a model to address gaps in a research narrative, runs deterministic validation, and supplies any remaining findings to another bounded pass until no high-severity issue remains or the workflow reaches its iteration, token, or time limit.

4. **Skills, context, and capability engineering** assembles the instructions, research data, evidence, examples, tools, constraints, and current state needed for each invocation. Agent Skills package specialized instructions and optional resources into portable, version-controlled directories that an agent can load when relevant.[^skills] As an optional extension, the Model Context Protocol (MCP) can standardize how applications connect agents to prompts, resources, and tools.[^mcp]

   *Example:* An agent loads the ACORN skill and receives the relevant `ResearchActivity` schema, source record, validation results, and allowed tools before proposing a correction. A future MCP server could expose the same authoritative context and deterministic checks through a standard interface.

5. **Connected memory** preserves useful state across iterations, sessions, or tasks. Memory should be selective and attributable: durable facts, decisions, provenance, and prior results are more valuable than an unfiltered transcript. Its origin, availability, modality, and quality are part of the capability's [data](aspect/data.md) profile. A memory service may be reached through MCP, but the protocol is the connection mechanism rather than the memory itself.

   *Example (planned):* A later session retrieves a previously reviewed project identity, identifier mapping, and provenance record instead of asking the model to infer them again from conversation history.

6. **Agent harness and interoperability** orchestrates models, skills, tools, workflows, context, memory, permissions, validation, recovery, observability, and human approval. The harness turns the lower layers into a controlled workflow and owns its budgets and stopping conditions. Its permissions and approval policy shape [autonomy](aspect/autonomy.md), while its perception, state, actions, and feedback shape [motivity](aspect/motivity.md). As an optional extension, the Agent Client Protocol (ACP) can standardize communication between the harness and external agents, including sessions, streamed updates, and permission requests.[^acp]

   *Example (planned):* ACORN launches OpenCode through ACP, sends a hydrated prompt, streams progress, mediates tool permissions, validates the returned suggestion, and asks for human approval before any change is applied.

## How IID Shapes ACORN

ACORN exists to maintain, validate, connect, analyze, and communicate research activity data. That work needs trustworthy structure and provenance before it needs [autonomy](aspect/autonomy.md). ACORN therefore keeps schema validation, identifier checks, formatting, linking, and artifact generation deterministic. AI is an optional consumer of—and contributor to—those workflows, not a replacement for their rules.

IID guides ACORN development in five ways:

- **Ground intelligence in research structure.** ACORN schemas, validators, persistent identifiers, and provenance provide authoritative context and machine-checkable boundaries for model-assisted work.
- **Make capability composable.** Prompts, skills, model APIs, local-model configuration, and future tools are useful independently and can be combined without forcing every user into a full agent stack.
- **Keep agency accountable.** Iteration needs explicit exit criteria and budgets; memory needs provenance and lifecycle rules; tools need permissions; and consequential actions need validation or human approval.
- **Work locally by default.** Deterministic core operations, prompt hydration, and locally available models should continue to work without a remote AI service. Network use must remain explicit and respect ACORN's offline boundary.
- **Keep files authoritative.** Research records, `.prompt` templates, `SKILL.md` instructions, configuration, and generated artifacts should remain portable. Applications consume or synchronize this state rather than owning its only copy.

Local-first does not mean local-only: ACORN can still synchronize applications and use remote services when selected. It is also distinct from [portability](aspect/portability.md), which describes where the complete capability can run after accounting for its model, tool, and service dependencies. File over app does not require every internal value to be a flat file. Databases may provide rebuildable indexes and operational storage, while credentials belong in environment variables or dedicated secret stores rather than portable project files.

MCP and ACP are optional, complementary extension points rather than requirements for IID. MCP answers **what context and capabilities an agent can access**; ACP answers **how a client communicates with and supervises an agent**. Skills remain portable instruction and resource packages independent of both protocols. A system can implement context engineering or an agent harness without MCP or ACP. The protocols also do not replace model APIs, which provide direct model invocation.

| Optional extension | Initial ACORN role | Purpose |
| --- | --- | --- |
| **MCP** | Server, planned | Let external agents discover and call deterministic ACORN resources and tools. |
| **ACP** | Client, planned | Let ACORN invoke and supervise interchangeable agents, beginning with OpenCode. |
| **OpenAI-compatible API** | Client, available | Let ACORN applications invoke a model endpoint directly without requiring a full agent. |

## Current ACORN Support

The table reports ACORN's implementation coverage for each IID layer. It does not score ACORN's maturity or imply that every workflow should use every layer.

| Layer | Status | ACORN support |
| --- | --- | --- |
| **1. One-time prompting** | Available as a library API | [`acorn-lib`](../packages/acorn-lib.md#ai-and-agent-apis) exposes OpenAI-compatible completion, chat-completion, response, embedding, image, and audio operations. Applications supply the model request and decide when to invoke it; ACORN does not yet provide a general-purpose interactive prompt command. |
| **2. Prompt templates** | Implemented | `acorn-lib` embeds and renders versionable `.prompt` templates for summarization, translation, teaching, simple explanation, claim extraction, and gap finding. Templates accept structured configuration such as source text, output limits, sampling parameters, and stop sequences. |
| **3. Workflow composition and iteration** | Not yet implemented | ACORN does not yet provide a general model workflow executor with explicit control flow, exit criteria, evolving state, progress checks, and resource budgets. Existing operational loops, such as file watching or service polling, are not IID model workflows. |
| **4. Skills, context, and capability engineering** | Partially implemented | [`acorn skill`](../cli-reference/commands/skill.md) distributes ACORN's bundled, task-oriented skill to compatible agents. ACORN also models agent, permission, reference, and MCP configuration for OpenCode. MCP is an optional extension at this layer; the CLI advertises a server path, but [`acorn serve mcp`](../cli-reference/commands/serve.md) currently returns a not-implemented error. |
| **5. Connected memory** | Available building blocks | ACORN's local database persists research candidates, activity history, model metadata, and other provenance-bearing records. These stores can become grounded inputs to agent memory, but ACORN does not yet manage conversational memory, cross-session agent recall, or memory selection and retention policies. |
| **6. Agent harness and interoperability** | Partially implemented | ACORN can manage local model metadata and weights, [synchronize model configuration](../cli-reference/model-workflows.md) into OpenCode, VS Code, Goose, and llama-swap, distribute its agent skill, and run a GitLab bot service. It also models harness and benchmark metadata. ACP is an optional extension at this layer; ACORN does not yet implement an ACP client. A unified runtime that composes prompts, workflows, context, connected memory, tools, agents, permissions, evaluation, and approvals remains future work. |

The sequence is composable, not all-or-nothing. ACORN can strengthen each layer independently—for example, by adding prompt evaluation before workflow execution, exposing deterministic ACORN operations as tools before granting broader agency, and defining memory provenance before retaining agent state. The result should be an agentic ACORN whose intelligence grows in depth without weakening the dependable research infrastructure at its core.

[^defense-in-depth]: National Institute of Standards and Technology, ["defense-in-depth," *Computer Security Resource Center Glossary*](https://csrc.nist.gov/glossary/term/defense_in_depth). The glossary traces the definition to CNSSI 4009-2015 and multiple NIST publications, including SP 800-53 Rev. 5.
[^effective-agents]: Anthropic, ["Building effective agents"](https://www.anthropic.com/engineering/building-effective-agents), 2024.
[^dotprompt]: Google, [*Dotprompt specification*](https://github.com/google/dotprompt/tree/main/spec) and [*Dotprompt reference documentation*](https://google.github.io/dotprompt/).
[^skills]: Agent Skills, [*Specification*](https://agentskills.io/specification).
[^mcp]: Model Context Protocol, ["Architecture"](https://modelcontextprotocol.io/specification/2025-06-18/architecture) and ["Server features"](https://modelcontextprotocol.io/specification/2025-06-18/server/index).
[^acp]: Agent Client Protocol, ["Architecture"](https://agentclientprotocol.com/get-started/architecture).
[^local-first]: Martin Kleppmann, Adam Wiggins, Peter van Hardenberg, and Mark McGranaghan, ["Local-first software: You own your data, in spite of the cloud"](https://www.inkandswitch.com/essay/local-first/), Ink & Switch, 2019, doi: [10.1145/3359591.3359737](https://doi.org/10.1145/3359591.3359737).
[^file-over-app]: Steph Ango, ["File over app"](https://stephango.com/file-over-app), 2023.
