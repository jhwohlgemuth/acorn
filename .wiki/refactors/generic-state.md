---
type: Plan
title: "Generic extensible State"
description: "Replace duplicated *State structs and snapshot pairs with generic composition types shared via acorn-core::state."
difficulty: medium
impact: medium
package: [acorn-core, acorn-lib, acorn-tui]
tags: [refactor, state, tui, webhooks, lifecycle]
status: planned
priority: next
---

# Goal

Give every `*State` one composition-based generic owner in `acorn-core::state` and make callers depend on that owner directly. Keep domain-specific policy where similarity is intentional (notably `GatherState`'s 16-field remote gather payload) while removing duplicated selection/loaded flags and runtime snapshot boilerplate.

# Current evidence

- `crates/acorn-tui/src/app.rs:72` `CheckState` (`items, selected_index, loaded`) and `crates/acorn-tui/src/app.rs:103` `DoctorState` (`data, selected_category, loaded, export_message`) both repeat `{selected, loaded, optional data}`. Constructors at `crates/acorn-tui/src/app.rs:438` and `:458` are near-identical. Only consumer is `crates/acorn-tui/src/app.rs:37` `App` and re-export in `crates/acorn-tui/src/lib.rs:9`.
- `crates/acorn-tui/src/app.rs:210` `State<T> { data: T, selected: usize }` is already generic and reused for `DashboardData:82`, `GgufPickerData:158`, `ThemePickerData:168`, `InterfaceData` via `App:37` (`dashboard`, `gguf_picker`, `theme_picker`). It proves composition works for TUI.
- `crates/acorn-tui/src/app.rs:127` `GatherState` is bespoke (16 fields: `checks, discoveries, input, input_count, selected_index, mode: GatherMode, remote_matches/total/has_more/offset/loading/error, organization_filter/role`) with single consumer `crates/acorn-tui/src/screens/gather.rs:32`. Only `selected_index`/`loaded`-like concerns overlap with the generic; remote paging is domain-specific and must stay bespoke.
- `crates/acorn-cli/src/io/mod.rs:17` `InputOutputStreamState` (`Piped|NotPiped`) is a binary `is_terminal` probe (`stdin_piped_status:51`, `stdout_piped_status:58`), not a lifecycle FSM. No dedup target beyond the two wrappers.
- `crates/acorn-schema/src/research_activity/logbook.rs:102` `LogbookState` (`Completed|InProgress|Upcoming`, `From<&str>:451` fallback to `InProgress`) is a temporal enum on `LogbookEntry:179`.
- `crates/acorn-lib/src/io/api/gitlab/mod.rs:96` `CommitStatusState` (`Running|Success|Failed`, `Display` lowercase, `From<bool>:1111`) is a GitLab API enum used in `crates/acorn-lib/src/io/api/gitlab/review.rs:424` and `effects.rs`.
- `crates/acorn-lib/src/io/api/gitlab/bot.rs:94` `SharedState = Arc<Mutex<State>>`, `crates/acorn-lib/src/io/api/gitlab/bot.rs:208` private `State { after, poll_count, processed_count, last_error }`, `crates/acorn-lib/src/io/api/gitlab/bot.rs:220` `StateSnapshot { after, poll_count, processed_count, last_error, webhook: WebhookRuntimeState }`, `crates/acorn-lib/src/io/api/gitlab/bot.rs:175` `RouterState`. Snapshot via `State::snapshot:820` for `GET /state` handler (`crates/acorn-lib/src/io/api/gitlab/bot.rs:801`).
- `crates/acorn-lib/src/io/api/webhooks/mod.rs:136` private `RuntimeState { accepting, active_claims, initialized, last_error_at, last_success_at }` (`Arc<Mutex<RuntimeState>>` in `WebhookRuntime:166`) with `active_claimed/released`, `snapshot:368`, `unavailable`, `updated_at:355`; `crates/acorn-lib/src/io/api/webhooks/mod.rs:171` `WebhookRuntimeState { accepting, active_claims, last_error_at, last_success_at, ready (= accepting && initialized) }` public observable, 6 callers (`bot.rs`, `webhooks/mod.rs:442 drain, 486 state`). Same `private Copy → snapshot() → public Copy` idiom as the bot state.
- `crates/acorn-lib/src/io/api/webhooks/store.rs:47` `OperationState` (`Queued|Running|Succeeded|RetryableFailure|TerminalFailure|Cancelled`, 7 variants) is the durable webhook FSM, 10 callers plus tests `crates/acorn-lib/src/io/api/gitlab/tests/{database,bot}.rs`, transitions in `fail:721`, `fail_callback:736`, `state:846` with `attempts.saturating_sub(1).min(8)` backoff.
- `crates/acorn-lib/src/analyzer/host/discovery/mod.rs:91` `LifecycleState` (`Found|Resolved|Unsupported|Failed|Created|Enriched|Unchanged|Conflict`, `strum Display lower`, `From<CandidateAction>:874`) drives `check():885` mapping `Failed→Error`, `Conflict|Unsupported→Warning`, else `Info`, and `RemoteMatch::link_checks:1345`.
- Redundancy: two snapshot pairs duplicate `active_claimed/released` + `snapshot()`; UI structs duplicate `selected/loaded`; outcome mapping duplicates `Retryable vs Terminal` logic.

# Project code boundaries

- **Update:** `crates/acorn-core/Cargo.toml`, `crates/acorn-core/src/lib.rs`, `crates/acorn-lib/src/lib.rs`, `crates/acorn-lib/src/prelude.rs`, `crates/acorn-tui/src/app.rs`, `crates/acorn-lib/src/io/api/webhooks/mod.rs`, `crates/acorn-lib/src/io/api/webhooks/store.rs`, `crates/acorn-lib/src/analyzer/host/discovery/mod.rs`, `crates/acorn-lib/src/io/api/gitlab/bot.rs`; optional leaf consumers `crates/acorn-tui/src/screens/gather.rs`, `crates/acorn-lib/src/io/api/gitlab/{review,effects}.rs` if `CommitStatusState` adopts the trait.
- **Create:** `crates/acorn-core/src/state.rs` (`Selectable`, `OutcomeKind`, `StateMachine`, `RuntimeCommon`, `backoff()` + `schemars` shim where needed); this plan file.
- **Delete:** duplicate `snapshot()` and `active_claimed/released` methods after generic lands; bespoke `CheckState`/`DoctorState` structs replaced by `Selectable` type aliases (immediate break).

# Dependencies and ordering

1. `acorn-core` already owns `acorn-core::options::{CommonRuntime, RuntimeOptions}` (commit `8ae5709e`); `acorn-core::state` follows the same `acorn-core` → `acorn-lib`/`acorn-tui` ownership direction. No `acorn-lib` facade.
2. Must land before further `acorn-tui` host extraction — `crates/acorn-tui/src/app.rs` is the TUI single-writer lock per [parallel execution](../../parallel-execution.md).
3. Serializes with `webhook`/`gitlab bot` lanes — shared `crates/acorn-lib/src/io/api/webhooks/{mod,store}.rs` and `bot.rs` are single-writer integration locks. Whitespace, feature, and schema locks do not apply.
4. Follows ownership rules in [crate boundary architecture](crate-boundary-architecture.md) and the prior [generic Options and Config](../../.wiki/refactors/generic-options-config.md) plan (now landed as `acorn-core::options`).

# Recommendation

## Shared core (`crates/acorn-core/src/state.rs`)

```rust
pub enum OutcomeKind { Success, Retryable, Terminal, Cancelled }
pub trait StateMachine: Copy + Eq {
    fn outcome(self) -> OutcomeKind;
    fn is_terminal(self) -> bool { matches!(self.outcome(), OutcomeKind::Terminal | OutcomeKind::Cancelled) }
    fn severity(self) -> CheckSeverity { CheckSeverity::Info } // overridden only by LifecycleState
}
#[derive(Clone, Debug)] pub struct Selectable<T> { pub data: Option<T>, pub selected: usize, pub loaded: bool }
#[derive(Clone, Copy, Debug)] pub struct RuntimeCommon { pub accepting: bool, pub active_claims: u64, pub initialized: bool, pub last_error_at: Option<Timestamp>, pub last_success_at: Option<Timestamp> }
pub fn backoff(attempts: u32) -> Duration { /* 1<<min(attempts-1,8) capped */ }
```

- Composition over inheritance: `GatherState { selectable: Selectable<()>, extension: GatherExt { checks, discoveries, input, mode, remote_* } }`. Flatten only on `extension` with `#[serde(default)]` if serialized, matching the `options.rs` constraint. `GatherExt` stays bespoke (16 fields, TUI-only).
- Manual `StateMachine` impls for leaf enums (`LogbookState`, `CommitStatusState`, `InputOutputStreamState`) — no `derive(StateMachine)` macro; 2–3 lines each (`Completed|Running|Piped → Success`, etc.).
- `LifecycleState` and `OperationState` get manual `StateMachine` where `outcome()` drives former `check()` severity / backoff. Former `OperationState::from(ClaimedOperation)` and `backoff` exponent logic in `store.rs:725` becomes `backoff(operation.attempts)` generic over `StateMachine`.
- Snapshot pairs collapse: `struct RuntimeState<E=()> { common: RuntimeCommon, extension: E }` with generic `snapshot() -> WebhookRuntimeState` + `active_claimed/released` on `RuntimeCommon`. Same for `bot::State` → `bot::StateSnapshot` (`RuntimeState<BotExt>`).

## Per-file migration

- `crates/acorn-tui/src/app.rs:72` `CheckState` → `type CheckState = Selectable<Vec<String>>`; `crates/acorn-tui/src/app.rs:103` `DoctorState` → `type DoctorState = Selectable<DoctorData>` with `DoctorExt { export_message }` flattened only if needed. Update `App::new()` and `load_doctor_data:337` to use `Selectable::new`.
- `crates/acorn-tui/src/app.rs:210` `State<T>` retained as alias to `Selectable<T>` for `DashboardData`/`GgufPickerData` consumers.
- `crates/acorn-lib/src/io/api/webhooks/store.rs:47` `OperationState` impl `StateMachine`; `crates/acorn-lib/src/analyzer/host/discovery/mod.rs:91` `LifecycleState` impl `StateMachine` replacing `check():885` severity mapping.
- `crates/acorn-lib/src/io/api/webhooks/mod.rs:136/171` and `crates/acorn-lib/src/io/api/gitlab/bot.rs:208/220` rewritten to `RuntimeState<()>` / `RuntimeState<BotExt>`; delete duplicate `snapshot()`.

# Validation

1. `cargo check --workspace --all-features`
2. `cargo check -p acorn-core --no-default-features` and `make check-wasm` (wasm32-unknown-unknown)
3. `cargo clippy --workspace --no-deps --all-targets` (no `unwrap_used`, `missing-docs`)
4. `cargo test -p acorn-core --lib`, `cargo test -p acorn-tui --lib`, `cargo nextest run -p acorn-lib -- io::api::webhooks` (covers `store.rs` FSM, `LifecycleState` checks)
5. Snapshot of `StateSnapshot` JSON (fields `after`, `poll_count`, `webhook.ready`) unchanged; `LifecycleState` check severity snapshot unchanged.

# Acceptance criteria

- `Selectable<T>` replaces `CheckState`/`DoctorState` bespoke structs; `State<T>` remains as alias to `Selectable<T>`.
- `GatherState` composition keeps `GatherExt` bespoke (no flattening of remote paging into generic), only `selected/loaded` extracted.
- `OperationState` and `LifecycleState` implement `StateMachine` manually; shared `backoff()` replaces duplicated exponent logic.
- `RuntimeState`/`WebhookRuntimeState` and `bot::State`/`StateSnapshot` share generic `RuntimeCommon` + `snapshot()`; duplicate `active_*` methods removed.
- All `#[serde(flatten)]` limited to `extension` with `#[serde(default)]`.
- `acorn-core` owns state types with `secrecy` retained where needed; no `acorn-lib` facade re-export.
- Portable feature matrix (`--no-default-features`, `check-wasm`) passes; `make lint` + `make test` green.

# Related plans

- [Crate boundary architecture](crate-boundary-architecture.md)
- [Generic extensible Options and Config](generic-options-config.md)
- [Workspace duplication consolidation](workspace-duplication-consolidation.md)
- [API roadmap](api/roadmap.md)
