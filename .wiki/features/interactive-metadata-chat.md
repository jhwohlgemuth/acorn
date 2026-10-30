---
type: Plan
title: Interactive metadata chat and terminal prompts
description: Add inline, persistent metadata chat to acorn ask and reusable terminal feedback to format.
difficulty: high
impact: high
package: [acorn-lib, acorn-cli, acorn-schema]
tags: [feature, ask, metadata, chat, terminal, enrichment]
status: in-progress
execution: phased
phase_budget_hours: 5
---

# Goal

Make `acorn ask --interactive` a multi-turn conversation for understanding, checking, and fixing metadata across the standards supported by `acorn check`. Keep its terminal interaction inline at the row where the command starts. Save each conversation in ACORN's standard platform data directory. Provide reusable Choose, Confirm, and Spin controls for other CLI commands, with `acorn format --interactive` as the first additional consumer.

The first usable interface is separate from `acorn tui`. The later chat-controller phase leaves a UI-neutral event contract that a future TUI screen can consume; it does not add that screen.

# Current evidence

Checked on 2026-10-29 against the current implementation:

- `ask` is a visible alias of `infer`. One-shot Needle and `infer run` remain available. `ask --interactive` now uses the configured ACP or OpenAI-compatible backend in an inline terminal viewport and saves a fresh JSONL transcript.
- The [native routing plan](needle-router-native-parity.md) now has the caller-owned `ask` registry, opt-in route syntax, and a local `acorn.check` binding over `analyze_paths`. PAR-02 retains nonlocal and fix-preview parity work. The [generic adapter plan](ask-executor-adapters.md) owns configured external executor bindings. This plan consumes those contracts and does not create another general tool registry.
- `analyzer/host/service::analyze_paths` dispatches local analysis across ACRE, CFF, CKAN, DataCite, DCAT, Document, Dublin Core, HuWise, Invenio, Kitfile, ModelKit, RAiD, RAD, and Text. OCI artifact checking follows a separate `check` path. `check --suggest-fixes` already produces bounded proposals and previews; it does not apply them.
- `acorn-lib::io::chat_history::ChatSession` creates private, versioned JSONL files under `ask/sessions`, flushes complete records, redacts recognized credentials, and starts a new file on clear. The interactive command persists chat events and shows the saved path on exit.
- Plain and ACRE formatting can write in place. Enriched formatting builds a `Workflow::Plan` before application, including companion artifacts and configured hook output. Ambiguous enrichment now retains bounded typed decisions alongside its existing text diagnostics.
- The CLI has a command-owned inline viewport with bounded scrolling, text input, Choose, two-button Confirm, asynchronous Spin, resize handling, TTY rejection, and terminal cleanup. `acorn-lib::agent` exposes a terminal-independent chat reducer with confirmed target identity and bounded context selection.
- The confirmed metadata adapter resolves local files and folders under a bounded root, skips unsupported formats, calls the typed native check binding, and returns ordered structured results plus an 8 KiB context summary. Mixed standards require an explicit choice before attachment. The chat command resolves and confirms paths, uses the grounded driver, and offers `/check`, `/clear`, `/fix`, `/help`, `/history`, `/open`, and `/exit` through the shared slash parser. `/fix` rechecks typed sources, previews each document diff, defaults confirmation to No, and records each outcome. Recipe and manual actions are excluded.
- Confirm renders adjacent filled buttons with magenta focus, a muted inactive state, visible text markers without color, and No focused initially. Focused renderer and PTY tests cover its visual and keyboard behavior against the [user-supplied Gum Confirm animation](../_assets/gum-confirm-reference.gif).
- `acorn-lib::agent::slash` defines typed command names, aliases, argument parsing, and context-specific suggestions shared by chat, the TUI dashboard, and GitLab. GitLab still requires `/acorn` and retains its existing note authorization and operation paths. Focused CLI, chat-driver, fix-plan, and slash tests pass; a PTY smoke run exercised `/help` and clean `/exit` with a saved transcript.
- Enrichment results and workflow document changes carry typed decisions with a five-choice bound, explicit skip, stale-source validation, and distinct external-provider or embedded-vocabulary origins. Ambiguous ORCID matches can provide choices for CFF, DataCite, Invenio, and RAD; ambiguous ROR and complete ambiguous or unmatched vocabulary reports can provide choices for RAD. Missing or incomplete evidence retains diagnostics without a choice. Passive formatting still writes only deterministic results.
- The requested visual and key behavior is modeled on [Gum Choose, Confirm, and Spin](https://github.com/charmbracelet/gum#commands), especially its two-button confirmation. This is an ACORN-native Rust implementation, not a Gum process dependency.

# Boundaries and dependencies

- **Owned here:** inline terminal controls, chat state and transcript storage, metadata attachment and analysis presentation, grounded conversational replies, reviewable fix application, typed uncertain-enrichment feedback, and interactive formatting.
- **Consumed, not replaced:** existing analysis, fix, workflow transaction, inference backend, and routed capability contracts. `infer` without `--interactive`, `infer run`, and the separate TUI remain available.
- No remaining phase depends on generic external adapter release or model-profile availability.
- Serialize IMC-10 with the CLI parser/help lock and IMC-10/11 with structured enrichment or format work on their exact library and command files. ADP-09 and other inference-command work must still coordinate shared roots. The TUI screen itself remains a later separate outcome.

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
- For `format --interactive`, deterministic formatting and accepted high-confidence vocabulary normalization proceed automatically under existing command rules. Uncertain enrichment presents typed candidates with origin, field, source, and reason plus **Skip**. A choice is validated against the original source and standard before it enters the plan; a source without candidates leaves the field unchanged and reports why. Keep non-interactive enrichment behavior unchanged. Evaluate configured hooks before the final reviewed plan is applied; source and companion artifacts use the existing transaction and rollback path. `--dry-run --interactive` may show decisions but performs no writes. For batches, review uncertain values per affected file; deterministic files need no confirmation.

# Validation and acceptance

- PTY validation proves the chat remains in its original inline region through multiple turns, long responses, spinner updates, resize, cancellation, and failures. Confirm renders the adjacent filled buttons and visible focus change in color, limited-color, and no-color modes, including narrow terminals; Escape and default Enter never approve a write. Terminal state is restored after interruption.
- Fixtures cover each local check-supported metadata standard, mixed-standard folders, ambiguous detection, unsupported inputs, offline operation, and a model response that mentions an unconfirmed path. Check findings and explanations retain source/field references. No model or provider call occurs for a deterministic command that does not need it.
- Fake ACP/endpoint tests cover bounded multi-turn context, refusal, timeout, transport failure, offline rejection, invalid action requests, and transcript write failures. Full history remains readable after context trimming and `/clear`; each invocation starts a new session.
- Fix tests cover accepted and declined diffs, stale files, invalid replacement, per-target authorization, and failed write/rollback. Format tests cover deterministic auto-apply, ambiguous candidate choice/skip, absent candidates, mixed batches, enrichment, configured hooks, companion artifacts, dry run, and non-interactive regression.
- The chat controller exposes events/actions without `crossterm`, terminal globals, or CLI-only state. A headless second consumer proves the future TUI can drive the same conversation logic. The release updates CLI help and inference/format documentation and passes focused checks, serialized lint/full tests, and wiki validation.

# Execution phases

## IMC-10 — Add interactive formatting for deterministic paths

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: claim the CLI contract lock
- Exact write set: `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/cli/tests/mod.rs`, `crates/acorn-cli/src/commands/mod.rs`, `crates/acorn-cli/src/commands/format/mod.rs`, `crates/acorn-cli/src/commands/format/acre.rs`, `crates/acorn-cli/src/commands/format/tests/mod.rs`, `CommandLineHelp.md`
- Primary integration lock: CLI format contract
- Outcome: Add `format --interactive`, TTY validation, and the reusable renderer to plain/ACRE formatting; deterministic changes continue through existing writes. Keep enriched/hook runs explicitly unavailable in this intermediate phase.
- Focused validation: Parser, file/folder formatting, no-change, dry run, non-TTY, and non-interactive regression tests.
- Handoff: Record the new flag contract and the enriched-mode prerequisite; remove IMC-10.

## IMC-11 — Review enrichment in the full format workflow

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: IMC-10
- Exact write set: `crates/acorn-cli/src/commands/format/mod.rs`, `crates/acorn-cli/src/commands/format/tests/mod.rs`, `crates/acorn-lib/src/io/workflow/mod.rs`, `crates/acorn-lib/src/io/workflow/tests/mod.rs`
- Primary integration lock: format workflow application
- Outcome: Resolve candidate choices per file before applying one validated plan; preserve configured hooks, companion artifacts, deterministic writes, and transaction rollback.
- Focused validation: Mixed accept/skip, missing candidates, changed source, enrich+hook, companion output, dry run, provider failure, and rollback fixtures.
- Handoff: Record all supported interactive format modes and artifact behavior; remove IMC-11.

## IMC-12 — Prove the future TUI consumer seam

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: IMC-11
- Exact write set: `crates/acorn-lib/src/agent/chat/mod.rs`, `crates/acorn-lib/src/agent/chat/tests.rs`, `docs/src/inference/README.md`
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
