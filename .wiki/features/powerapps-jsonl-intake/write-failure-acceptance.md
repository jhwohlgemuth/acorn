# Intake write-failure acceptance

Verified October 24, 2026 in the existing pilot environment. The saved response repair passed an isolated cloud runtime test. The live intake flow and its app binding were unchanged.

## Controlled test

An owner-only copy of the live flow was created as **ACORN Intake Failure Acceptance TEST ONLY 2026-10-24**, solution flow ID `74365385-02d0-f111-a05c-001dd8062052`. It reused the existing protected `acorn-powerautomate-intake` connection. No permissions or credentials were changed.

Only **Commit append** received a Power Automate Testing static result: status Failed, error InternalServerError, message `TEST ONLY: simulated failed commit; connector write is not executed.` This substitutes a failed action result instead of invoking the connector write. Other prerequisite reads used the approved connector normally. Flow checker showed zero errors and warnings after temporary activation. The flow is not associated with the PowerApp.

Test envelope UUID: `d365c30d-0d0d-42d4-a12e-88846d0de8bd`. This is a separate synthetic failure fixture, not the captured personnel demo receipt.

Runtime: [run 08584087221220248940052484856CU15](https://make.gov.powerautomate.us/environments/221f44a6-1468-ea4e-b93d-cd52f1723079/solutions/~preferred/flows/74365385-02d0-f111-a05c-001dd8062052/runs/08584087221220248940052484856CU15), October 24 at 23:32:45 UTC.

| Action | Observed outcome |
| --- | --- |
| Verify access to main | Succeeded |
| Write action | Succeeded |
| Commit append | Failed with the configured simulated error |
| Verify written snapshot | Skipped |
| Confirm written bytes | Skipped |
| Queued new | Skipped |
| Unresolved readback | Skipped |
| Persist submission v3 | Failed |
| Unresolved failure | Succeeded once |

The overall run is Succeeded because the outer response handles the failure. That run-level status is not evidence of a successful GitLab append. No commit was attempted by the mocked action and no success receipt was produced. The response body remains protected in run history; the preserved source definition specifies `disposition: unresolved` and an empty commit SHA.

## Dependency checks

The preserved live source definitions also passed five checks in [failure-dependency-checks.json](_assets/failure-tests/failure-dependency-checks.json): no nested `Unresolved_write`, the outer fallback accepts Failed/TimedOut/Skipped scope states, immutable readback requires a successful commit, confirmation requires successful readback, and automatic commit retries remain disabled. Source SHA-256 hashes are recorded with those checks.

This accepts the repaired response graph using a real cloud runtime with a simulated failed commit. It does not claim a real GitLab timeout after a possibly committed write, reliable network egress, or the concurrent-writer matrix. Those remain separate intake acceptance work. The disabled test copy was deleted during owner-requested cleanup on October 25, 2026, after its configuration and run evidence were preserved here. The operational flow was not changed.

Evidence: [mock configuration](_assets/failure-tests/failure-test-configuration.png), [failed commit and skipped confirmation](_assets/failure-tests/failure-commit-and-skips.png), [single outer response](_assets/failure-tests/failure-response.png), and matching text snapshots beside each image.
