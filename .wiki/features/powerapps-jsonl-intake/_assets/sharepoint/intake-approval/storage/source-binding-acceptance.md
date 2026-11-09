# PA-37S TEST ONLY source binding

On November 9, 2026, the owner approved a trusted `SourceListId` on each TEST ONLY review batch so finalization can reread the correct complete source interval. The column is a single-line text field on `Sponsor Highlights Review Batches TEST ONLY` (`2f405310-4681-40db-99ff-1bafd4b90676`), with no default. The seven preexisting batches were backfilled from their documented materialization histories and verified after reloading the list:

| Batch period start UTC | Source list ID | Basis |
| --- | --- | --- |
| 2026-09-01 00:00 | `a904a6c6-5dd0-427b-9575-8167c4c0c412` | Pilot materialization |
| 2026-10-26 00:00 | `a904a6c6-5dd0-427b-9575-8167c4c0c412` | Pilot materialization |
| 2026-11-08 22:30 | `746cf260-a4d3-4c8f-ab98-d78fe4a9afa1` | Fixture materializer run `08584074290834244800397373633CU25` |
| 2026-11-08 22:48 | `746cf260-a4d3-4c8f-ab98-d78fe4a9afa1` | Fixture materializer failed before Ready |
| 2026-11-08 23:55 | `746cf260-a4d3-4c8f-ab98-d78fe4a9afa1` | Fixture materializer run `08584074244989093229734115024CU20` |
| 2026-11-09 00:06 | `746cf260-a4d3-4c8f-ab98-d78fe4a9afa1` | Fixture materializer run `08584074235619154980944169987CU28` |
| 2026-11-09 00:27 | `746cf260-a4d3-4c8f-ab98-d78fe4a9afa1` | Fixture materializer failed on missing predecessor |

`build-source-binding.py` produces native update packages from current exports of the two TEST ONLY materializers. Static inspection verifies each package retains its original flow ID, leaves every package entry except `definition.json` untouched, adds the expected source-list ID to the batch-state guard, and writes that same ID on both Materializing and Ready updates. The fixture and operational flows remain Off pending package import and live validation. The backfill does not claim a Ready state for the two rejected fixture batches.

The owner separately approved uploading both specific native update ZIPs to the existing TEST ONLY flows in the Wohlgemuth, Jason Power Automate environment. The packages were staged in Downloads as `materialize-fixture-source-binding.zip` and `materialize-operational-source-binding.zip`. Chrome's native picker showed `Open` disabled for these files and for Power Automate's own exported ZIPs, even though the extension's `Allow access to file URLs` setting was enabled. The owner manually uploaded each package. Both import pages resolved the intended existing flow as the Update target, selected the existing `o9w@ornl.gov` SharePoint connection, and reported `All package resources were successfully imported`:

| Installed flow | Flow ID | Verified result |
| --- | --- | --- |
| `Materialize Approval Batch TEST ONLY` | `44c87c4e-6693-49a2-868d-8b1c78c383b5` | Off; designer Flow checker: zero errors, zero warnings. Installed export `PA37Soperationalinstalledverification_20261109233817.zip` matches the approved ZIP's flow ID, complete trigger, and complete action tree exactly. |
| `Materialize Approval Batch Fixture TEST ONLY` | `ec17c0f4-f486-489b-b9b2-908e16e9b882` | Off; designer Flow checker: zero errors, zero warnings. Installed export `PA37Sfixtureinstalledverification_20261109234220.zip` matches the approved ZIP's flow ID, complete trigger, and complete action tree exactly. |

The flow-details warning in each case is only `This flow is off`. A live Ready-batch retry has not run. Automatic approval review rejected temporarily turning on the fixture flow because of the prior instruction to leave both materializers Off. The owner has been asked for narrow authorization to run one existing Ready-batch retry per flow and then turn each Off. No mismatch rejection run has been performed.
