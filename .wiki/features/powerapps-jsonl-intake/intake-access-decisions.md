# Intake access and license evidence

Updated October 25, 2026 (America/New_York) in Jason Wohlgemuth's existing signed-in session. The owner reports no administrator ticket and requests using existing access. An administrator ticket is not a prerequisite for isolated testing. No network policy or operational component, credential or connection binding was changed. The owner subsequently created a separate test connection using the existing authorized credential.

## Licenses observed

Power Apps Settings → Plan(s) lists:

- Power Apps for Office365 for Government
- Power Apps Per User Plan for Government
- Power Apps Community, expiring April 22, 2027

Sponsor Highlights app details designate the app Premium and require an end user to have a Per App Plan or Per User Plan. This provides direct evidence of Jason's assigned per-user entitlement; the earlier claim that all pilot license evidence was missing is superseded. It does not establish licenses for other users.

![Assigned plans](_assets/access-acceptance/jason-assigned-plans.png)

![App license designation](_assets/access-acceptance/app-premium-designation.png)

Microsoft's [licensing guidance](https://www.microsoft.com/licensing/guidance/Power-Platform) includes Power Automate use rights for flows associated with a Power Apps application. Its [Power Automate FAQ](https://learn.microsoft.com/en-us/power-platform/admin/power-automate-licensing/faqs) distinguishes app-context flows from standalone automation. The exported intake and catalog definitions use Power Apps V2 triggers and are dependencies of Sponsor Highlights. Together, these facts support the current Jason-only, app-triggered pilot. This is an assessment of the observed license and configuration, not a tenant-wide licensing determination. A standalone callback or unattended bot must be assessed separately; no additional licenses were purchased or assumed.

## Access remains an empirical acceptance gate

The retained pilot evidence includes both successful app-triggered append/replay and explicit network-range 403 responses. On October 25 the new isolated Jason connector returned HTTP 200 for VerifyMainBranch at 16:35:49 UTC and GetProjectCatalog at 16:36:09 UTC (13 projects). ReadLogbook returned HTTP 404 at 16:36:42 UTC for the isolated file that did not exist at that checkpoint. All three response schemas validated. See [request IDs and screenshots](restore-target-readiness.md#isolated-connector-revision). Those connector tests establish access at those times; no write or exact replay was performed during those connector-only tests. A subsequent native Power Apps-triggered catalog flow passed a manual designer test at 16:55 UTC, run `08584086595609022768772030053CU30`; it returned all 13 projects through the isolated connection. After native solution conversion, resubmission `08584086594173685533974706185CU16` also succeeded overall: the catalog action succeeded in 0.4 seconds, while the response was skipped. This confirms a repeat connector call through the new test reference, not response delivery to an app. After explicit native reference repair, Jason designer test `08584086587813997307821396476CU02` passed all three steps at 17:08 UTC. Native export 1.0.0.4 then restored into ORNL with a separate protected test connection. ORNL designer test `08584086581182120845753124938CU14` passed all three steps at 17:19 UTC; GetProjectCatalog returned HTTP 200, 13 projects, source `config` and the same revision. See [restoration and output evidence](restore-target-readiness.md#ornl-native-restore-acceptance). Later v6 acceptance verified native test-app catalog invocation in Jason and ORNL; ORNL app-driven run `08584086507609818994513589261CU31` passed October 25 at 3:22 PM America/New_York. App-driven intake remains unverified. These read successes do not establish repeatable egress or prove that moving to ORNL Personal Productivity resolves the intermittent denial.

Continue using existing authorized routes and protected connections where compatible. Test each environment's isolated connector against isolated files, recording timestamps, run IDs, HTTP outcomes, receipt identity and immutable readback. Do not rotate credentials to work around network policy. Do not write the operational inbox to demonstrate test access.

PA-02 remains open for repeatable access and final flow-context verification. PA-15 and PA-16 may proceed once their isolated route can be exercised safely; they need no new administrative ticket. Their acceptance cannot be inferred from the older single-user demo or from simulated responses.

## October 25 native intake write, replay and renewed denial

Jason's isolated native intake completed manual append `08584086552539263000712925851CU18` at approximately 18:07 UTC and exact replay `08584086551895520151909856703CU06` at approximately 18:08 UTC. Commit `bd5888fa7a1f4b555414384819173bc6409ab70d` added one TEST ONLY line to `.acorn/intake/acceptance/pa17-logbook.jsonl`; immutable readback and exact browser byte comparison passed. Replay skipped the commit. Both skipped Teams. These are manual tests of a Power Apps V2 flow, not app-context licensing or restored-app acceptance.

The next attempt, `08584086551396206729261127404CU24`, returned HTTP 403 from VerifyMainBranch at `2026-10-25T18:09:05Z`. The body explicitly identifies an IP network-range security restriction; action tracking ID `f7c38202-f69f-4e5c-b625-1eb9c38260fd`. Persistence was skipped, the outer unresolved response succeeded, and the isolated file's exact bytes remained unchanged. The flow's overall Succeeded badge reflects handled failure, not persistence. No source IP was included in the captured response, and none is inferred. No retry or network/credential workaround followed the explicit denial.

[Detailed outcomes, run evidence and screenshots](restore-target-readiness.md#jason-native-intake-write-and-replay) establish that intermittent denial persists even after native reference repair and successful writes. PA-02 remains open; no administrator ticket was created or made a prerequisite. Current access still cannot be represented as reliable.

## ORNL restored intake: reads pass, commit denied

Native intake/catalog version 1.0.0.5 imported successfully in ORNL Personal Productivity on October 25. Manual replay `08584086532554165170447843195CU19` and changed-summary conflict `08584086532144607697385592776CU20` both reached their intended responses without writes or delivery. Fresh submission `08584086531530438165830898752CU14` then passed branch, history, catalog and current-file reads but received HTTP 403 at Commit append. The response dated `2026-10-25T18:42:13Z` explicitly identifies a network-range security restriction; action tracking ID `d38bf387-a801-4cf6-823d-fdc4763103f1`. No source IP is inferred.

The unresolved response succeeded, confirmation/queued-new and Teams were skipped, and the isolated file stayed byte-identical at 457 bytes. No retry or network/credential workaround followed. The test intake was returned Off. This proves that successful reads in ORNL do not establish write access or resolve the intermittent restriction. PA-02 remains open. [Restoration and run evidence](restore-target-readiness.md#ornl-version-1005-import-and-restored-intake-acceptance).
