# TEST ONLY review storage permissions

Site: `https://ornl.sharepoint.com/sites/ResearchEnablement`. The owner approved this site for the three PA-36 TEST ONLY objects on November 8, 2026. The site has `Research Enablement Owners` (Full Control), `Research Enablement Members` (Edit), and `Research Enablement Visitors` (Read). All three review objects have unique permission scopes: their permission pages show only Owners with Full Control, with no inherited Members or Visitors grant. The `PA36 Restrict Approval Storage TEST ONLY` flow run `08584074390771993067233319842CU12` succeeded on November 8, 2026; each object's permission page was checked afterward.

| Principal | Batches list | Items list | Approved projection library | Flow operations |
|---|---|---|---|---|
| Research Enablement Owners | Full Control | Full Control | Full Control | Recovery and administration |
| Sponsor Highlights Reviewers TEST ONLY | None | None | None | List/read; later save drafts, after caller group check |
| Sponsor Highlights Finalizers TEST ONLY | None | None | None | Later finalize, after caller group check |
| Sponsor Highlights Exporter TEST ONLY | None | None | Read | Later read Approved manifest and frozen files only |
| Research Enablement Members / Visitors | None | None | None | None unless separately named in a review group |
| Contributor | None | None | None | None |

All review flows use the owner's existing SharePoint connection. A Power Apps caller identity is checked on every invocation against the named group; a shared flow link or a hidden app button is not authorization. The app receives bounded response data, never a SharePoint list connection. The exporter may read the projection library only after a batch is Approved. The library contains no mutable drafts or source payloads. No item-level unique scopes are used.

The live `PeriodKey` and `MembershipKey` column settings each show Required and Enforce unique values set to Yes. Both review lists have item version history enabled with a 50-version limit. `BatchId`, `PeriodKey`, `PeriodStartUtc`, and `State` are indexed on batches; `BatchId`, `Decision`, `MembershipKey`, `ProjectId`, and `ReportSortKey` are indexed on items. The operational intake list now has an indexed `Created` column for bounded source enumeration.

`Sponsor Highlights Reviewers TEST ONLY` (SharePoint group ID `42`) and `Sponsor Highlights Finalizers TEST ONLY` (ID `43`) each contain only `o9w@ornl.gov`. `Sponsor Highlights Exporter TEST ONLY` (ID `44`) has no members. Their group creation forms assigned no site-wide permission; group membership alone does not grant direct access to the review objects. The owner approved these groups and storage permission changes on November 8, 2026. A denied-access check with an unrelated site member remains an acceptance gate.
