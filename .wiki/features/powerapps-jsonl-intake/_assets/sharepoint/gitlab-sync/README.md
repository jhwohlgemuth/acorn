# [BETA] Scheduled GitLab and SharePoint sync

The two flow definitions are independent of the Power Apps form. Both run under
the flow owner's connections and start with a single `Sync_configuration_BETA`
action. Copy [sync-config.json](sync-config.json), change its values, and run
the builders to produce reusable scope definitions. All three readiness flags
start `false`; the live flows also retain false readiness values until a
controlled source and destination test is complete.

| Flow title | Direction | Schedule | Live state |
| --- | --- | --- | --- |
| `[BETA] GitLab Project Catalog Sync` | GitLab project choices → SharePoint project list | Daily, 2:00 AM Eastern | Saved and Off. SharePoint site/list and GitLab project/branch come from `Sync configuration BETA`. `catalogSyncReady` is `false`, so the flow terminates before reading or writing if turned On. Flow checker: zero errors and warnings. |
| `[BETA] Approved JSONL GitLab Sync` | Finalized, approval-receipted SharePoint JSONL → dedicated GitLab path | Daily, 2:30 AM Eastern | Saved and Off. SharePoint site/library and GitLab project/branch/path come from `Sync configuration BETA`. It reads the newest `export-receipt-*.json`, validates file identity, byte count, fingerprint, and file revision, updates an existing GitLab file only when bytes differ, and reads back the result. Both readiness flags remain `false`. Flow checker: zero errors and warnings. |

Live flow details: [catalog sync](https://make.gov.powerautomate.us/environments/221f44a6-1468-ea4e-b93d-cd52f1723079/solutions/~preferred/flows/031af465-9b60-4e7b-7960-028505194a42/details) and [approved JSONL sync](https://make.gov.powerautomate.us/environments/221f44a6-1468-ea4e-b93d-cd52f1723079/solutions/~preferred/flows/044451dc-7cdf-f111-84ea-001dd8062052/details).

## Reuse settings

`sharePointSiteUrl` and `projectListId` select the catalog list. That list must
have `Title`, `ProjectId`, and `SourceRevision` columns. `approvedExportLibrary`
selects the library containing `logbook.jsonl` and the exporter’s
`export-receipt-<run-id>.json` files. `gitLabProjectId` and `gitLabBranch`
select the repository/ref; `approvedGitLabPath` selects a distinct
`.acorn/approved/*.jsonl` output. Set each readiness flag to `true` only after
testing the corresponding source, destination, and connector binding.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 build-catalog-sync.py
PYTHONDONTWRITEBYTECODE=1 python3 build-approved-sync.py
PYTHONDONTWRITEBYTECODE=1 python3 test_sync.py
```

The generated `*.clipboard.json` files are designer action sources, not native
flow packages. The ACORN GitLab Logbook connector has the revised
`project-catalog.cs` script, optional `projectId` and `branch` inputs on its
existing catalog action, and the new `GetProjectCatalogForRepository`,
`ReadApprovedJsonl`, and `CommitApprovedJsonl` operations. The catalog flow
uses those optional inputs so its downstream action references remain valid.
`gitlab-sync.swagger.json` is the JSON form of the same connector definition.
Both live flows use the existing protected GitLab connection. The canvas app
must not receive that connection.

## Publication safeguards

The catalog flow checks a bounded nonempty source and duplicate IDs before
sequential list writes. The generated catalog scope also has a removal guard;
that guard is not in the live flow yet. The approved flow requires the newest
exporter receipt and compares its file identity, byte count, candidate
fingerprint, and SharePoint revision with the current JSONL. Its GitLab write
has no automatic retry. It reads GitLab first, skips matching content, and
reads back exact content after an update. The live flow currently requires the
configured GitLab destination file to exist; a missing file stops the flow.
The generated approved scope also handles initial creation, but that branch is
not deployed. Do not turn either flow On until a controlled test verifies the
connector’s path encoding, the SharePoint receipt response, the receipt's
`ETag` field, and the intended list/repository permissions.

The current source library is **TEST ONLY** and the approval-gated exporter is
Off. Production publication also requires the approved exporter cutover, a
production receipt/library setting, and an existing dedicated GitLab file.
Keep both live flows Off and their readiness flags false until those
prerequisites and a controlled readback test are complete.
