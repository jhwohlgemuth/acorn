# PA-17 native backup and restore handoff

Verified October 25, 2026. This archive is a collection of native packages and evidence, not itself a Power Platform import file. Extract it first. Inner ZIPs are unchanged native exports. Complete recovery acceptance is BLOCKED, not passed.

## Import inputs

| Package | Purpose | Current result |
| --- | --- | --- |
| ACORNPA17RestoreReadinessTest_1_0_0_6.zip | Isolated canvas app, intake/catalog flows, connector and references | ORNL import succeeded; app-driven catalog lists 13 projects in both environments |
| ACORNPA17CallbackRestoreTest_1_0_0_1.zip | Isolated callback, receipt table, key and references | Jason source receipt/replay/conflict passed; ORNL import denied by missing prvCreateEntity |
| SponsorHighlightsNativeBackup20261025_20261025185256.zip | Read-only operational canvas and dependent flow backup | Native archive verified; retains operational routes and delivery behavior; never use for isolated tests |
| ACORNPA17CallbackBackup_1_0_0_1.zip | Read-only operational callback/table schema backup | Native archive verified; retains operational identities; never import over operations for testing |

The operational app package references its existing custom connector; do not treat the differently named test connector as an automatic replacement. Protected credentials and table rows are not in these exports. The operational files and isolated test files serve different purposes; never import the operational backups into the test solution.

## Accepted evidence

- PA14 recovery and result panels accepted in both environments, including owner-reported full browser restart and storage denial.
- V6 native import into ORNL: activity f8525c54-972a-46a7-a164-d751d8994483, October 25, 3:18–3:19 PM America/New_York.
- Restored ORNL app abd4b93f-3abc-43f1-82ae-1cc33e1f3527 invokes catalog run 08584086507609818994513589261CU31; all three actions succeeded and all 13 projects appeared.
- Jason isolated manual intake write/replay passed. ORNL restored manual replay/conflict passed. Those are not app-driven write acceptance.
- ORNL test intake is Off; checker has zero errors and only the intentional Off warning. No operational component or inbox was changed by restoration work.

## Remaining acceptance, in order

1. Establish permitted GitLab writes through the existing authorized route. The latest fresh ORNL commit was explicitly denied by network-range policy (2026-10-25T18:42:13Z, tracking d38bf387-a801-4cf6-823d-fdc4763103f1). Successful catalog reads do not resolve it. No credential or network workaround is authorized.
2. Complete app-driven fresh write, frozen-ID replay and changed-payload conflict in the isolated environments. Use only project 22464/main/.acorn/intake/acceptance/pa17-logbook.jsonl; preserve exact bytes, commit proof, disabled automatic write retries and false notification guards. Turn test intake Off afterward.
3. Obtain existing authorized capability to create the isolated callback table in ORNL, or an explicit change to the required restore target. Current ORNL import failed on prvCreateEntity. Do not retry unchanged privileges or substitute an operational table. No administrator ticket is required by this plan, but the missing capability is real.
4. Import the unchanged callback test ZIP, bind existing protected test references, verify the unique webhook key, and test first receipt/replay/conflict with one unchanged test row and no Teams/email delivery. Source-only Jason tests do not prove restoration into ORNL.
5. Preserve evidence and prune PA-17 only after the remaining acceptance passes. Shared release remains PA-12; reliable access, boundaries and concurrency remain PA-02/15/16.

See pa17-native-restore-instructions.md for dependency and binding checks, pa17-native-backup-manifest.json for package hashes, and pa17-ornl-v6-restore-evidence.json for the accepted target checkpoint.
