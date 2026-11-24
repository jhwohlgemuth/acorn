# Scheduled GitLab and SharePoint sync

Two independent owner-run flows reconcile GitLab project choices into SharePoint
and mirror an approval-receipted SharePoint JSONL snapshot into a dedicated
GitLab file. Neither flow writes the Vergil intake inbox, changes review state,
finalizes batches, or deletes source receipts.

The November 24 hardening is **local source and generated definitions only**.
No target, connection, permission, readiness flag, or live schedule was changed.
The last recorded live versions were saved and Off; their current tenant state
has not been rechecked. They do not yet contain these changes. Remaining binding,
isolated acceptance, and deployment are tracked in the
[scheduled-sync plan](../../../scheduled-sync/index.md).

| Artifact | Purpose |
| --- | --- |
| `build-catalog-sync.py` / `catalog-sync.clipboard.json` | Catalog reconciliation scope, including complete-input validation, conditional writes, and readback. |
| `build-approved-sync.py` / `approved-sync.clipboard.json` | Stable approved-source capture and conditional, append-only GitLab snapshot replacement. |
| `build_workflows.py` / `*.workflow.json` | Reviewable WDL definitions with serialized daily recurrence, success evidence, and failed-run handling. These are not native import packages. |
| `sync-config.json` / `sync_config.py` | Retained historical settings and strict generation-time validation. All readiness flags remain false. Regenerate after editing configuration; do not bypass validation by editing generated expressions. |
| `test_sync.py` / `test_failures.py` / `test_runtime.py` | Static contract checks and synthetic execution of the generated expression/action subset. No network calls. |

## Build and verify

Run from this directory:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 build-catalog-sync.py
PYTHONDONTWRITEBYTECODE=1 python3 build-approved-sync.py
PYTHONDONTWRITEBYTECODE=1 python3 build_workflows.py
PYTHONDONTWRITEBYTECODE=1 python3 test_sync.py
PYTHONDONTWRITEBYTECODE=1 python3 test_failures.py
```

Connection references are deliberately unbound in both generated scopes. The
GitLab connector operation names remain compatible with the retained
`project-catalog.swagger.yaml` / `gitlab-sync.swagger.json` contract. Binding a
connection is a separate deployment action; the canvas apps must never receive
the GitLab connection or token. The historical configuration file is context,
not approval to use its targets. Sanitized artifacts require the
[normal reuse checks](../../../../../privacy.md).

## Catalog guarantees

Before any write, the flow requires a bounded, nonempty catalog at a source
revision; a complete SharePoint collection; unique case-insensitive IDs in both
collections; valid nonempty labels; usable item ETags; and no removed project.
The projected destination count, including the reserved General row, must fit
1,000 items. Continuation links and malformed/truncated inputs stop the run.

The flow also reads the ProjectId field definition and requires indexed,
writable, unique single-line text. It does not alter the list schema. This
constraint prevents duplicate creates across retries or competing instances.
The General row is preserved. The current GitLab connector returns IDs as
fallback labels; these must not overwrite existing human-readable titles.
Explicit non-fallback labels can update titles.

Writes run sequentially. Existing rows use REST MERGE with the exact ETag;
there is no wildcard overwrite or automatic write retry. Each create/update,
including a failed or timed-out response, is followed by a lookup and exact
ProjectId/Title/SourceRevision comparison. A matching no-op also gets read back.
Uncertain writes only succeed when that comparison proves the intended state.
A failed run can leave verified earlier rows updated: this is a convergent
reconciliation, not a multi-row transaction. Preserve diagnostics and rerun only
after investigating the failure; never restore a whole list over newer edits.

## Approved snapshot guarantees

The newest receipt must come from a bounded, complete file listing. Its required
fields are typed, its timestamp must be neither future nor stale, and its file
identity, ETag, byte count, line count, and candidate fingerprint must agree with
the source. `maxReceiptAgeHours` defaults to 48 and can be configured explicitly;
that is a freshness policy, not a guarantee of upstream schedule delivery.
The source must be LF-terminated JSONL without blank lines or CR bytes, at most
10 MiB, with positive batch and line counts.

Metadata is read before and after content. ETag and UniqueId must stay stable
and match the receipt, closing the content/metadata race. The inherited
`b64-edge-v1` fingerprint is not a cryptographic digest; trust also depends on
restricted source/receipt writers and authenticated revision checks. The sync
does not independently re-perform review or validate every RAD field.

The GitLab destination **must already exist** with readable base64 content and
`last_commit_id`. A 404 is not permission to create: it can mean a missing file
or ref. An empty dedicated file can bootstrap publication after explicit target
approval. Otherwise its exact complete-line content must be a prefix of the
approved snapshot. Older or divergent snapshots cannot erase newer GitLab
history. Equal bytes produce no commit; changed bytes use conditional update
with `last_commit_id`, with automatic write retry disabled.

The flow rereads GitLab after success, failure, or timeout and verifies the exact
base64 bytes, path, encoding, and returned revision. A lost write response can
therefore reconcile successfully; mismatch or unreadable readback fails without
claiming publication. Do not blindly replay a failed write or roll back over a
later commit. No branch, repository, file, or bridge to `.acorn/intake/` is created.

Read-only connector calls use at most three exponential retries, starting at five
seconds with a 30-second maximum interval. Writes never retry automatically.

## Scheduling and operations

The new templates serialize each flow with trigger concurrency one. Proposed
times are 03:00 and 03:30 Eastern, avoiding the missing 02:00 hour during the
spring DST transition. These are template defaults; the historical live schedules
were 02:00 and 02:30 and remain untouched. The flows are independent; a 30-minute
offset does not establish approval/export completion. Publication checks the
receipt itself on every run.

Concurrency control applies to one flow, not clones. Deployment must designate
one active writer for each target, disable superseded schedules, and verify
there is no run still in flight. The platform owns connector timeouts; this
source adds no claim of a bounded whole-run deadline. Operational acceptance must
measure duration, configure owner-visible failure and missed-run monitoring,
verify retention/access to run history, and test connection-owner continuity.
No notification is sent and no alert destination is selected by these artifacts.
`Sync_verified_BETA` records run ID, time, verified status, source revision and
counts after the scope succeeds; approved mirroring also records the source
receipt/batch and GitLab revision. It includes no payload text or credentials.
Failure paths retain failure status and require reconciliation.

Before enabling a production copy, supply and approve:

- exact GitLab project/ref/file and SharePoint site/list/library, scoped connection references, and non-personal ownership/rotation arrangements;
- the catalog uniqueness/index settings, single-writer policy, and a dedicated existing GitLab file with an accepted empty or exact-prefix baseline;
- an accepted approval-gated exporter and its receipt/library contract; PA-41A gates Sponsor Highlights operational publication;
- isolated allow/deny, limits, replay, ETag conflict, changed-source, stale receipt, real connector timeout/lost-response, readback, and recovery results;
- fresh native backups, an inspected import/rebind in isolation, monitoring owners, support instructions, and an approved activation window.

Set flags true only in the explicitly approved isolated copy during acceptance.
Keep production copies Off/false until their release gates pass. Rollback is to
disable scheduling and preserve verified data, then reconcile with its owner.
Do not reactivate an unfiltered exporter or delete receipts/commits as rollback.

## Evidence and review basis

The offline matrix executes the generated graph against synthetic connector
responses for disabled flags, changed/no-op replay, uncertain committed and
uncommitted writes, denied/missing reads, stale/conflicting source revisions,
newer/divergent/case-changed destinations, oversized snapshots, pagination, catalog duplicates/removals/capacity,
unique-field requirements, ETag conflicts, title preservation, and readback
failure. This is not proof of hosted WDL semantics, connector field shapes,
permission enforcement, importability, licensing, or real service fault injection.

The conditional-write and error-routing contracts were checked against
[WDL expression semantics](https://learn.microsoft.com/en-us/azure/logic-apps/expression-functions-reference),
[Microsoft run-after handling](https://learn.microsoft.com/en-us/azure/logic-apps/error-exception-handling),
[SharePoint ETag updates](https://learn.microsoft.com/en-us/sharepoint/dev/sp-add-ins/working-with-lists-and-list-items-with-rest),
[Power Automate concurrency limits](https://learn.microsoft.com/en-us/power-automate/limits-and-config),
and [GitLab conditional commit actions](https://docs.gitlab.com/api/commits/).
