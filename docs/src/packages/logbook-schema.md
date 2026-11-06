# Logbook schema

The logbook records time-coded developments alongside a research activity's canonical metadata. It supports two kinds of entry: an `ActivityEntry` describes a development at an instant, while a `StatusUpdateEntry` assesses project health over a reporting period. Both are represented by the Rust enum `LogbookEntry` and serialized inside `ResearchActivity.logbook.entries`.

The exact public type is `LogbookEntry`; “logbook entries” refers to a `Vec<LogbookEntry>` rather than a separate `LogBookEntries` type. Serialized field names use camel case even though the Rust fields use snake case.

## Information flow

```pikchr
{{#include logbook-information-flow.pikchr}}
```

The main reviewed workflow starts with a submission envelope in JSONL. `intake stage`, `intake inspect`, and `intake apply` admit the envelope and write its normalized entry to the selected research activity. A JSONL line is an immutable submission envelope, not a bare `LogbookEntry`; direct JSON, YAML, ZON, or Markdown authoring can place the schema value in a RAD without that intake envelope.

After decoding, ACORN validates the selected entry variant, its nested values, and the containing logbook. Reporting resolves inherited ranges, selects entries for the requested interval, and creates a `LogbookUpdates` projection ordered newest first. The report pipeline renders that projection without requiring metadata graduation. Graduation is an optional, separately reviewed path for supported activity entries: accepted values are copied into canonical metadata while the source entry remains in the logbook, marked `archived: true` and `state: completed`.

The editable diagram source is [`logbook-information-flow.pikchr`](logbook-information-flow.pikchr). See [intake](../cli-reference/commands/intake.md) for receipt review and correction behavior and [report](../cli-reference/commands/report.md) for artifact generation.

## Type structure

```pikchr
{{#include logbook-entry-structure.pikchr}}
```

`LogbookEntry` is an untagged enum. JSON, YAML, and ZON do not contain a separate entry-type discriminator; deserialization chooses `Activity` or `StatusUpdate` from the fields present in the object. Each variant rejects unknown fields. `ProjectHealth` is different: it is internally tagged by its `status` field.

The editable diagram source is [`logbook-entry-structure.pikchr`](logbook-entry-structure.pikchr).

## Container fields

`ResearchActivity.logbook` is optional. When present, the `Logbook` object has these fields:

| Serialized field | Rust type | Requirement |
| --- | --- | --- |
| `summary` | `Option<String>` | Optional editorial overview; 1–1,000 characters when present. |
| `range` | `Option<ReportingWindow>` | Optional default reporting window for status updates that omit their own range. |
| `entries` | `Vec<LogbookEntry>` | Required; 1–1,000 entries. Source order is preserved during serialization. |

Entry identifiers must be unique across both variants in one logbook. A status update must resolve to a bounded period with a lower bound. It can supply its own `range`, or inherit the top-level `logbook.range`; an entry-specific range replaces the default rather than merging with it.

`ReportingWindow` is half-open:

| Serialized field | Rust type | Meaning |
| --- | --- | --- |
| `left` | `Option<String>` | Inclusive RFC 3339 lower bound. `null` means full history where the operation permits it. |
| `right` | `String` | Exclusive RFC 3339 upper bound; it must be later than `left` when `left` is present. |

All logbook timestamps must be complete RFC 3339 instants with an explicit offset, such as `2026-10-31T16:45:00Z` or `2026-10-31T12:45:00-04:00`.

## Activity entries

An `ActivityEntry` captures one development and its provenance.

| Serialized field | Rust type | Requirement |
| --- | --- | --- |
| `identifier` | `String` | Required kebab-case identifier; 1–100 characters and unique within the logbook. |
| `projectId` | `String` | Required kebab-case project identifier; 1–100 characters. |
| `global` | `bool` | Defaults to `false`; when true, the entry can appear among general updates on the report title page. |
| `timestamp` | `String` | Required RFC 3339 instant with an explicit offset. |
| `category` | `LogbookCategory` | Required subject vocabulary value. |
| `classification` | `Option<ClassificationLevel>` | Optional classification; classified entries must use `category: administration`. |
| `eventType` | `Option<LogbookEventType>` | Optional event format; classified entries must use `eventType: other`. |
| `state` | `LogbookState` | Required temporal interpretation of the timestamp. |
| `origin` | `LogbookEntryOrigin` | Defaults to `manual`. |
| `discovery` | `Option<LogbookDiscovery>` | Required only when `origin` is `discovered`; forbidden for manual entries. |
| `summary` | `String` | Required description; 1–1,200 characters. |
| `items` | `Option<Vec<String>>` | Optional 1–20 supporting statements; each is 1–1,000 characters. |
| `archived` | `bool` | Defaults to `false`; graduation sets accepted source activities to `true`. |

`LogbookDiscovery` contains the enrichment `provider`, a non-empty provider-native `source` of at most 2,048 characters, and `discoveredAt`, an RFC 3339 instant. The provider and source also produce the deterministic identifier used for discovered candidates.

### Activity vocabularies

`category` accepts:

- `administration`, `collaboration`, `data`, `engagement`, `funding`, `infrastructure`
- `method`, `other`, `personnel`, `publication`, `research`, `software`

Compatibility aliases normalize `conference` and `tour` to `engagement`, `technical` to `infrastructure`, `general` to `other`, and `code` to `software`.

`state` accepts `completed`, `in-progress`, and `upcoming`. It describes whether work was complete, underway, or expected at the recorded instant; it is distinct from the research activity's lifecycle status and from an entry's `archived` flag.

`eventType` accepts `conference`, `exhibition`, `meeting`, `other`, `presentation`, `tour`, `training`, and `workshop`.

## Status update entries

A `StatusUpdateEntry` records the most important change for a reporting period, why it matters, and the project's overall health.

| Serialized field | Rust type | Requirement |
| --- | --- | --- |
| `identifier` | `String` | Required kebab-case identifier; 1–100 characters and unique within the logbook. |
| `projectId` | `String` | Required kebab-case project identifier; 1–100 characters. |
| `timestamp` | `String` | Required RFC 3339 submission instant with an explicit offset. |
| `range` | `Option<ReportingWindow>` | Optional entry-specific reporting period; otherwise the top-level logbook range is inherited. |
| `summary` | `String` | Required most important change or result; 1–1,200 characters. |
| `significance` | `String` | Required explanation for leadership; 1–1,000 characters. |
| `health` | `ProjectHealth` | Required tagged health assessment. |
| `evidence` | `Option<String>` | Optional concrete proof; 1–1,000 characters when present. |
| `risk` | `Option<ProjectRisk>` | Optional leadership-relevant risk or dependency. |

`health.status` accepts `on-track`, `ahead-of-plan`, `at-risk`, `delayed`, `paused`, and `complete`. The `at-risk`, `delayed`, and `paused` variants may also carry a non-empty `reason` of at most 1,000 characters. Awaiting decisions and dependencies belong in `risk`, not in a new health state.

`ProjectRisk` has a required `kind` and a required 1–1,000 character `explanation`. The kind vocabulary is `data`, `infrastructure`, `security-accreditation`, `software-environment`, `sponsor-prioritization`, `partner-coordination`, `funding-contracting`, `staffing`, `validation`, `classified-environment-access`, or `other`.

## Example serialized logbook

The following logbook contains one activity and one status update. The status update inherits the top-level reporting window.

```json
{
  "summary": "October research and delivery updates.",
  "range": {
    "left": "2026-10-01T00:00:00Z",
    "right": "2026-11-01T00:00:00Z"
  },
  "entries": [
    {
      "identifier": "release-validated",
      "projectId": "acorn",
      "global": true,
      "timestamp": "2026-10-21T17:30:00Z",
      "category": "software",
      "state": "completed",
      "origin": "manual",
      "summary": "Validated the release on Linux x86-64.",
      "items": [
        "Verified online and cached offline execution.",
        "Confirmed process cleanup after completion."
      ],
      "archived": false
    },
    {
      "identifier": "october-status",
      "projectId": "acorn",
      "timestamp": "2026-10-31T16:45:00Z",
      "summary": "Release validation expanded to another supported host.",
      "significance": "The additional evidence reduces deployment uncertainty.",
      "health": {
        "status": "on-track"
      },
      "evidence": "The host smoke suite completed without a policy escape.",
      "risk": {
        "kind": "staffing",
        "explanation": "A second reviewer is needed before the next publication window."
      }
    }
  ]
}
```

## Ordering and reporting semantics

Storage order and display order intentionally differ:

- Serialization preserves the order of `logbook.entries` in the source record.
- `entries_all` parses every timestamp, resolves a missing status-update range from `logbook.range`, and returns cloned entries newest first.
- `updates_since` includes entries whose entry timestamp is at or after the inclusive instant, regardless of variant.
- `updates_within` includes activities whose timestamp falls in `[left, right)`. It includes status updates whose resolved reporting range overlaps the requested half-open window.
- A `LogbookUpdates` projection combines the selected entries with the activity title, subtitle, keywords, principal investigators, sponsors, and the selection bounds used by report rendering.

An invalid timestamp fails selection instead of being silently skipped. A bounded report excludes an activity exactly at the exclusive `right` instant. Overlap selection lets a status assessment submitted after a period still appear in the report for the period it assesses.

## Validation summary

Validation operates at three levels:

1. Each variant validates field lengths, timestamps, identifiers, and nested values.
2. `ActivityEntry` enforces the origin/provenance pairing and the classification category and event constraints.
3. `Logbook` enforces unique identifiers and ensures every status update resolves to a reporting window with a lower bound.

The enum delegates `Validate` to the selected entry variant. The containing `ResearchActivity` validates the optional logbook as a nested value, so ordinary whole-record validation reports logbook errors with their entry paths.

## Markdown representation

The schema types support Markdown conversion. Decoding expects YAML front matter delimited by `---`; the typed fields come from that front matter. Rendering produces a human-readable `# Logbook` document with one heading per entry. Activity entries include state, category, origin, optional classification and event information, summary, and supporting items. Status entries include health, leadership significance, optional health context, evidence, and risk details.

For the complete Rust surface, see the [`research_activity::logbook` API documentation](https://docs.rs/acorn-schema/latest/acorn_schema/research_activity/logbook/).
