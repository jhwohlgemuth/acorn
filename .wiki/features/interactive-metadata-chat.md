---
type: Plan
title: Interactive metadata chat and terminal prompts
description: Add inline, persistent metadata chat to acorn ask and reusable terminal feedback to format.
difficulty: high
impact: high
package: [acorn-lib, acorn-cli, acorn-schema]
tags: [feature, ask, metadata, chat, terminal, enrichment]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Make `acorn ask --interactive` a multi-turn conversation for understanding, checking, and fixing metadata across the standards supported by `acorn check`. Keep its terminal interaction inline at the row where the command starts. Save each conversation in ACORN's standard platform data directory. Provide reusable Choose, Confirm, and Spin controls for other CLI commands, with `acorn format --interactive` as the first additional consumer.

The first usable interface is separate from `acorn tui`. The later chat-controller phase leaves a UI-neutral event contract that a future TUI screen can consume; it does not add that screen.

# Current evidence

Checked on 2026-10-21 against the current implementation:

- `ask` is a visible alias of `infer`. The current positional or piped prompt runs a one-shot Needle sidecar and prints JSON; `infer run` hydrates an embedded prompt and uses ACP or an OpenAI-compatible endpoint. Neither mode keeps chat state.
- The [native routing plan](needle-router-native-parity.md) owns the future caller-owned `ask` registry and CLI contract. RTR-03 freezes `ask` syntax; PAR-02 exposes full analysis checks. The [generic adapter plan](ask-executor-adapters.md) owns configured external executor bindings. This plan consumes those contracts and does not create another general tool registry.
- `analyzer/host/service::analyze_paths` dispatches local analysis across ACRE, CFF, CKAN, DataCite, DCAT, Document, Dublin Core, HuWise, Invenio, Kitfile, ModelKit, RAiD, RAD, and Text. OCI artifact checking follows a separate `check` path. `check --suggest-fixes` already produces bounded proposals and previews; it does not apply them.
- `acorn-lib::io::chat_history::ChatSession` creates private, versioned JSONL files under `ask/sessions`, flushes complete records, redacts recognized credentials, and starts a new file on clear. The first chat command still needs to wire this storage into its event flow.
- Plain and ACRE formatting can write in place. Enriched formatting builds a `Workflow::Plan` before application, including companion artifacts and configured hook output. Ambiguous ORCID/ROR enrichment currently becomes text conflicts. Vocabulary reports already retain bounded candidates.
- The CLI has a command-owned inline viewport with bounded scrolling, text input, Choose, two-button Confirm, asynchronous Spin, resize handling, TTY rejection, and terminal cleanup. `acorn-lib::agent` exposes a terminal-independent chat reducer with confirmed target identity and bounded context selection.
- The existing Confirm renderer shows arrow-marked `[Yes]` and `[No]` labels in green and red. Its PTY test checks that both labels appear, but the buttons do not yet have the filled, adjacent active/inactive treatment shown in the [user-supplied Gum Confirm animation](../assets/gum-confirm-reference.gif).
- `acorn-lib::agent::slash` defines typed command names, aliases, argument parsing, and context-specific suggestions shared by the TUI dashboard and GitLab webhook. Chat will consume the same contract in IMC-06. GitLab still requires `/acorn` and retains its existing note authorization and operation paths.
- Enrichment results and workflow document changes carry typed unresolved decisions alongside conflicts and failures. Candidate identity, a five-choice bound, explicit skip, and stale-source validation are defined; providers do not yet populate selectable choices.
- The requested visual and key behavior is modeled on [Gum Choose, Confirm, and Spin](https://github.com/charmbracelet/gum#commands), especially its two-button confirmation. This is an ACORN-native Rust implementation, not a Gum process dependency.

# Boundaries and dependencies

- **Owned here:** inline terminal controls, chat state and transcript storage, metadata attachment and analysis presentation, grounded conversational replies, reviewable fix application, typed uncertain-enrichment feedback, and interactive formatting.
- **Consumed, not replaced:** existing analysis, fix, workflow transaction, inference backend, and routed capability contracts. `infer` without `--interactive`, `infer run`, and the separate TUI remain available.
- IMC-09 may start against the present tree. IMC-04 consumes PAR-02; IMC-05/06 also consume RTR-03. If those contracts change, recheck the exact write set before starting. No phase depends on generic external adapter release or model-profile availability.
- Serialize IMC-04 through IMC-07 with RTR-03, ADP-09, and other inference-command work on shared files. Serialize IMC-06 and IMC-10 with the CLI parser/help lock. Serialize IMC-09 through IMC-11 with structured enrichment or format work on their exact library and command files. The TUI screen itself remains a later separate outcome.

# Experience and contracts

## CLI and conversation

- `acorn ask --interactive [PROMPT]` starts a fresh session; the optional prompt is its first user turn. The same flag is available through the `infer` spelling because `ask` is currently an alias. Keep positional/stdin exclusivity and existing one-shot syntax. Reject non-TTY input/output before entering raw mode; do not silently launch the full-screen TUI.
- Accept the inference selectors used by `infer run` in interactive mode: ACP with OpenCode by default, or an explicitly configured ACP agent or OpenAI-compatible endpoint. Validate selected backend, model, timeout, and offline policy before the first model request. Reject explicitly supplied Needle-only `--custom-location`, `--threshold`, and `--max-rounds` flags in interactive mode with a specific error.
- Use the shared context-aware slash parser: `/open PATH` attaches one file or a folder; `/clear` closes the active transcript and starts a fresh context; `/history` prints the current and archived transcript paths; `/help` lists keys and commands; `/exit` leaves cleanly. Unknown or malformed slash commands are local errors and never become model text. Natural-language paths are resolved against the working directory and require confirmation of the exact path and detected standard before use. Never use a model-invented or unconfirmed path as an operation target.
- File attachment is explicit context, not permission to mutate. For folders, use existing check discovery and stable path ordering, identify each supported standard, and report skipped or unsupported inputs. When extension or content detection is ambiguous, Choose offers only standards that the relevant operation supports. The metadata baseline is the named local standards in `analyze_paths`; Document and Text remain analysis inputs. OCI artifact references retain their existing `check` command path rather than being presented as a local metadata document.
- Conversation turns carry at most the latest 16 turns and 32 KiB of selected, relevant text/tool results into the next backend request. Older turns remain on disk; the UI indicates when context has been omitted. Do not send whole folders or silently truncate a file into an apparent complete document. Backend failures leave the session usable and record a failure event without manufacturing an answer.
- Route metadata actions through the frozen caller-owned `ask` capability contract. The chat controller may request a check, explanation, or fix proposal; it validates the action, attached target, standard, and authorization before dispatch. ACP or endpoint prose is grounded in typed results and cites file/field/check locations where available. Model output cannot register tools, choose arbitrary executables, grant permission, or write a file.

## Inline terminal controls

- Own a bounded viewport starting at the command's cursor row. Reserve space once, redraw only that region, keep chat history scrollable inside it, and restore the cursor, raw mode, and colors on normal exit, error, or interruption. Handle terminal resize and narrow widths without leaving fragments. Do not enter the alternate screen or let spinner/log output move the interaction downward.
- The reusable controls are CLI presentation functions with typed outcomes: Choose returns one selected item or cancellation, Confirm returns yes/no/cancellation, and Spin runs while a supplied asynchronous operation is pending. Choose supports arrow keys, `j`/`k`, Enter, and Escape, with scrolling. Confirm presents the question above two adjacent, filled **Yes** and **No** buttons when width allows, like the supplied Gum animation: the focused button has a bright magenta/pink background and high-contrast text; the other has a muted dark background. Narrow terminals stack or compact both choices without hiding either. Moving focus updates both buttons in place. Write approval starts with No focused, so Enter cannot approve by default; arrows/Tab change focus, `y`/`n` select directly, Enter selects the focused button, and Escape cancels. Spin stops on completion, failure, or cancellation.
- Use shape/text as well as color to identify focus and outcome; in no-color mode, retain two distinct button shapes and a visible focus marker. Honor the existing no-color setting and terminals without true color. A cancelled prompt never implies consent. The renderer owns terminal output while active; operations return structured events instead of printing over the viewport.

## History and changes

- Store transcripts under `project_data_directory("org", "ornl", "acorn", "ask/sessions")`, one newly created JSONL file per session. Records have a version, session ID, timestamp, event type, user/assistant text, selected standard/path references, concise tool result or decision, and secret-free backend provenance. Flush complete records so an interrupted session remains readable; treat a trailing partial record as recoverable. Protect files with user-only permissions where the platform supports them. Redact recognized bearer tokens, credential fields, and URL user info before persistence; do not persist entire attached documents or unbounded tool payloads.
- `/clear` archives the current file and opens a new one. New command invocations do not automatically resume old sessions. The terminal shows the saved path on clear and exit. Failure to create or append the transcript is visible and ends the affected session before another model/tool action; a user must not think history was saved when it was not.
- For chat fixes, use existing bounded fix proposals and previews. Show target, source finding, proposed diff, and effects; require Confirm before each selected write. Recheck source content and target authorization immediately before applying through the existing fix plan. Decline, stale source, failed validation, and rollback each produce explicit outcomes in UI and history.
- For `format --interactive`, deterministic formatting and accepted high-confidence vocabulary normalization proceed automatically under existing command rules. Ambiguous enrichment presents typed candidates with provider, field, source, and reason plus **Skip**. A choice is validated against the original source and standard before it enters the plan; a provider without candidates leaves the field unchanged and reports why. Keep non-interactive enrichment behavior unchanged. Evaluate configured hooks before the final reviewed plan is applied; source and companion artifacts use the existing transaction and rollback path. `--dry-run --interactive` may show decisions but performs no writes. For batches, review uncertain values per affected file; deterministic files need no confirmation.

# Validation and acceptance

- PTY validation proves the chat remains in its original inline region through multiple turns, long responses, spinner updates, resize, cancellation, and failures. Confirm renders the adjacent filled buttons and visible focus change in color, limited-color, and no-color modes, including narrow terminals; Escape and default Enter never approve a write. Terminal state is restored after interruption.
- Fixtures cover each local check-supported metadata standard, mixed-standard folders, ambiguous detection, unsupported inputs, offline operation, and a model response that mentions an unconfirmed path. Check findings and explanations retain source/field references. No model or provider call occurs for a deterministic command that does not need it.
- Fake ACP/endpoint tests cover bounded multi-turn context, refusal, timeout, transport failure, offline rejection, invalid action requests, and transcript write failures. Full history remains readable after context trimming and `/clear`; each invocation starts a new session.
- Fix tests cover accepted and declined diffs, stale files, invalid replacement, per-target authorization, and failed write/rollback. Format tests cover deterministic auto-apply, ambiguous candidate choice/skip, absent candidates, mixed batches, enrichment, configured hooks, companion artifacts, dry run, and non-interactive regression.
- The chat controller exposes events/actions without `crossterm`, terminal globals, or CLI-only state. A headless second consumer proves the future TUI can drive the same conversation logic. The release updates CLI help and inference/format documentation and passes focused checks, serialized lint/full tests, and wiki validation.

# Execution phases

## IMC-04 — Attach and analyze metadata targets

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: PAR-02 from the native routing plan
- Exact write set: `crates/acorn-cli/src/commands/infer/metadata.rs`, `crates/acorn-cli/src/commands/infer/tests/metadata.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`, `crates/acorn-cli/src/commands/infer/mod.rs`
- Primary integration lock: metadata analysis adapter
- Outcome: Resolve confirmed files/folders to typed standard-aware analysis inputs and present bounded structured findings through the landed check capability; no CLI subprocess or duplicate validator.
- Focused validation: Standard matrix, mixed folders, stable ordering, missing/ambiguous paths, unsupported formats, and offline fixtures.
- Handoff: Record the metadata operation contract for the chat driver; remove IMC-04.

## IMC-05 — Add grounded conversational replies

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: IMC-04, RTR-03 from the native routing plan
- Exact write set: `crates/acorn-schema/assets/prompts/metadata-chat.prompt`, `crates/acorn-schema/src/agent/mod.rs`, `crates/acorn-schema/src/agent/tests/mod.rs`, `crates/acorn-schema/src/agent/tests/snapshots/acorn_schema__agent__tests__prompt_asset_file_names.snap`, `crates/acorn-schema/src/agent/tests/snapshots/acorn_schema__agent__tests__prompt_asset_template_headers.snap`, `crates/acorn-cli/src/commands/infer/chat.rs`, `crates/acorn-cli/src/commands/infer/tests/chat.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`
- Primary integration lock: ask conversation composition
- Outcome: Drive bounded routed metadata actions and grounded ACP/endpoint prose over the chat controller; keep session failures and cancellations explicit.
- Focused validation: Fake backends for multi-turn context, typed findings, refusal, invalid route, timeout, backend failure, and no unauthorized action.
- Handoff: Freeze the chat-driver entry point for CLI syntax; remove IMC-05.

## IMC-06 — Expose `ask --interactive`

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: IMC-05; coordinate RTR-03 and ADP-09 ownership before editing shared roots
- Exact write set: `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/cli/tests/mod.rs`, `crates/acorn-cli/src/commands/mod.rs`, `crates/acorn-cli/src/commands/infer/mod.rs`, `CommandLineHelp.md`
- Primary integration lock: CLI contract
- Outcome: Add the flag, interactive backend selectors, TTY guard, first-turn handling, and the shared slash-command parser for `/open`, `/clear`, `/history`, `/help`, and `/exit` while preserving one-shot `ask`/`infer` and `infer run`.
- Focused validation: Parser/help and PTY command tests for aliases, stdin conflicts, non-TTY error, backend/offline selection, and clean exit.
- Handoff: Record a runnable inline chat smoke command and release the CLI lock; remove IMC-06.

## IMC-07 — Review and apply chat fix proposals

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: IMC-04, IMC-06, IMC-14
- Exact write set: `crates/acorn-cli/src/commands/infer/chat.rs`, `crates/acorn-cli/src/commands/infer/tests/chat.rs`
- Primary integration lock: ask fix application
- Outcome: Present existing fix previews, confirm individual writes, recheck targets, apply through the existing fix plan, and record outcomes.
- Focused validation: Proposed/accepted/declined/stale/invalid/failed-write fixtures, including no write on Escape or backend instruction alone.
- Handoff: Preserve the consent and stale-source behavior in tests; remove IMC-07.

## IMC-14 — Match Gum confirmation buttons

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-cli/src/io/interactive/mod.rs`, `crates/acorn-cli/src/io/interactive/tests.rs`
- Primary integration lock: inline terminal control renderer
- Outcome: Render Confirm as an inline question with adjacent filled buttons, a bright active button and muted inactive button, and a visible no-color focus state while preserving No as the safe initial selection and distinct yes/no/cancel outcomes.
- Focused validation: Renderer and PTY tests for initial No, focus movement, direct `y`/`n`, Enter, Escape, no-color and limited-color output, narrow/resize redraw, and terminal cleanup.
- Handoff: Record the Gum-like visual and keyboard contract in tests; remove IMC-14 before IMC-07 consumes Confirm.

## IMC-09 — Populate selectable enrichment candidates

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none; coordinate exact overlap with structured-enrichment work
- Exact write set: `crates/acorn-lib/src/io/enrichment.rs`, `crates/acorn-lib/src/io/tests/enrichment.rs`, `crates/acorn-lib/src/io/workflow/mod.rs`, `crates/acorn-lib/src/io/workflow/tests/mod.rs`
- Primary integration lock: enrichment providers
- Outcome: Supply bounded choices for ambiguous ORCID/ROR lookups and vocabulary findings where candidate evidence exists; retain unresolved diagnostics when it does not.
- Focused validation: Ambiguous/missing/provider-failed fixtures, duplicate candidates, provenance, and unchanged passive-mode output.
- Handoff: Record which providers and standards can present a choice; remove IMC-09.

## IMC-10 — Add interactive formatting for deterministic paths

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: IMC-09; claim the CLI contract lock after IMC-06
- Exact write set: `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/cli/tests/mod.rs`, `crates/acorn-cli/src/commands/mod.rs`, `crates/acorn-cli/src/commands/format/mod.rs`, `crates/acorn-cli/src/commands/format/acre.rs`, `crates/acorn-cli/src/commands/format/tests/mod.rs`, `CommandLineHelp.md`
- Primary integration lock: CLI format contract
- Outcome: Add `format --interactive`, TTY validation, and the reusable renderer to plain/ACRE formatting; deterministic changes continue through existing writes. Keep enriched/hook runs explicitly unavailable in this intermediate phase.
- Focused validation: Parser, file/folder formatting, no-change, dry run, non-TTY, and non-interactive regression tests.
- Handoff: Record the new flag contract and the enriched-mode prerequisite; remove IMC-10.

## IMC-11 — Review enrichment in the full format workflow

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: IMC-09, IMC-10
- Exact write set: `crates/acorn-cli/src/commands/format/mod.rs`, `crates/acorn-cli/src/commands/format/tests/mod.rs`, `crates/acorn-lib/src/io/workflow/mod.rs`, `crates/acorn-lib/src/io/workflow/tests/mod.rs`
- Primary integration lock: format workflow application
- Outcome: Resolve candidate choices per file before applying one validated plan; preserve configured hooks, companion artifacts, deterministic writes, and transaction rollback.
- Focused validation: Mixed accept/skip, missing candidates, changed source, enrich+hook, companion output, dry run, provider failure, and rollback fixtures.
- Handoff: Record all supported interactive format modes and artifact behavior; remove IMC-11.

## IMC-12 — Prove the future TUI consumer seam

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: IMC-07, IMC-11
- Exact write set: `crates/acorn-lib/src/agent/chat.rs`, `crates/acorn-lib/src/agent/chat/tests.rs`, `docs/src/inference/README.md`
- Primary integration lock: chat controller interface
- Outcome: Stabilize UI-neutral action/event snapshots and a headless second consumer; document how a future `acorn-tui` screen will render them without sharing terminal drawing code.
- Focused validation: Headless consumer drives attach, check, prompt, decision, clear, and cancel events without `crossterm` or CLI dependencies.
- Handoff: Record the frozen consumer contract and remaining TUI screen work outside this plan; remove IMC-12.

## IMC-13 — Document, validate, and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: IMC-12
- Exact write set: `docs/src/cli-reference/commands/infer.md`, `docs/src/cli-reference/commands/format.md`, `CommandLineHelp.md`, `.wiki/features/interactive-metadata-chat.md`, `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: interactive chat release gate
- Outcome: Publish chat keys, transcript location, supported standards and format behavior; complete acceptance evidence and retire this active plan.
- Focused validation: Serialized focused and full tests, lint, PTY smoke, generated help, documentation links, wiki validator, and `git diff --check`.
- Handoff: Delete the plan and remove every index, network, and lane reference only after durable documentation and acceptance pass.
