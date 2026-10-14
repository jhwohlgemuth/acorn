# Proposed ACRE configuration examples

These complete configuration documents demonstrate the planned ACRE update in [the owning plan](../features/powerapps-gitlab-logbook-reporting.md). They are design fixtures, not configurations accepted by today's validator. `2.0.0` is the proposed next major schema version, subject to the versioning decision in PA-11. All three documents describe the same configuration. The built-in variable catalog must cover all ACORN/Xylem variables; a deployment only lists the references it needs, as shown here. Group names and intake option names below are the proposed schema contract to verify during PA-11.

## JSON — `.acorn.json`

```json
{
  "schemaVersion": "2.0.0",
  "members": ["administrator@example.org"],
  "variables": {
    "members": "ACORN_LOGBOOK_MEMBERS",
    "custom": {
      "power_automate.callback_url": "POWER_AUTOMATE_CALLBACK_URL",
      "power_automate.inbound_secret": "POWER_AUTOMATE_INBOUND_SECRET",
      "notification_channel": "ORG_NOTIFICATION_CHANNEL"
    }
  },
  "buckets": [
    {
      "id": "pilot-data",
      "source": "https://code.ornl.gov/research-enablement/experiments/power-automate"
    }
  ],
  "folders": [
    { "id": "acorn", "bucket": "pilot-data", "aliases": ["acorn-tooling"] },
    {
      "id": "research-data", "bucket": "pilot-data",
      "aliases": ["rad"],
      "members": ["researcher@example.org"],
      "variables": { "members": "RESEARCH_DATA_MEMBERS" }
    },
    {
      "id": "instrumentation", "bucket": "pilot-data",
      "aliases": ["instruments"],
      "path": "programs/instrumentation/index.json",
      "variables": {
        "members": "INSTRUMENTATION_MEMBERS",
        "custom": { "notification_channel": "INSTRUMENTATION_CHANNEL" }
      }
    }
  ],
  "options": {
    "powerAutomate": {
      "callback_destination": "powerautomate",
      "gitlab_project": "22464",
      "package": "acorn-automated-artifacts",
      "timezone": "UTC",
      "intake": {
        "branch": "main",
        "path": ".acorn/intake/submissions.jsonl",
        "pollIntervalSeconds": 30,
        "maxFileBytes": 10485760,
        "maxRecordBytes": 65536
      }
    }
  }
}
```

## JSONC — `.acorn.jsonc`

```jsonc
{
  // Proposed schema version; not supported by the current implementation.
  "schemaVersion": "2.0.0",
  // Global inline members combine with the environment-supplied list.
  "members": ["administrator@example.org"],
  "variables": {
    "members": "ACORN_LOGBOOK_MEMBERS",
    "custom": {
      "power_automate.callback_url": "POWER_AUTOMATE_CALLBACK_URL",
      "power_automate.inbound_secret": "POWER_AUTOMATE_INBOUND_SECRET",
      "notification_channel": "ORG_NOTIFICATION_CHANNEL"
    }
  },
  "buckets": [
    {
      "id": "pilot-data",
      "source": "https://code.ornl.gov/research-enablement/experiments/power-automate"
    }
  ],
  "folders": [
    { "id": "acorn", "bucket": "pilot-data", "aliases": ["acorn-tooling"] },
    {
      "id": "research-data", "bucket": "pilot-data",
      "aliases": ["rad"],
      "members": ["researcher@example.org"],
      "variables": { "members": "RESEARCH_DATA_MEMBERS" }
    },
    {
      "id": "instrumentation", "bucket": "pilot-data",
      "aliases": ["instruments"],
      // Explicit paths are repository-relative overrides.
      "path": "programs/instrumentation/index.json",
      "variables": {
        "members": "INSTRUMENTATION_MEMBERS",
        "custom": { "notification_channel": "INSTRUMENTATION_CHANNEL" }
      }
    }
  ],
  "options": {
    "powerAutomate": {
      "callback_destination": "powerautomate",
      "gitlab_project": "22464",
      "package": "acorn-automated-artifacts",
      "timezone": "UTC",
      // One inbox multiplexes records across the root folder registry.
      "intake": {
        "branch": "main",
        "path": ".acorn/intake/submissions.jsonl",
        "pollIntervalSeconds": 30,
        "maxFileBytes": 10485760,
        "maxRecordBytes": 65536
      }
    }
  }
}
```

## YAML — `.acorn.yaml`

```yaml
schema_version: "2.0.0"
members:
  - administrator@example.org
variables:
  members: ACORN_LOGBOOK_MEMBERS
  custom:
    power_automate.callback_url: POWER_AUTOMATE_CALLBACK_URL
    power_automate.inbound_secret: POWER_AUTOMATE_INBOUND_SECRET
    notification_channel: ORG_NOTIFICATION_CHANNEL
buckets:
  - id: pilot-data
    source: https://code.ornl.gov/research-enablement/experiments/power-automate
folders:
  - id: acorn
    bucket: pilot-data
    aliases: [acorn-tooling]
  - id: research-data
    bucket: pilot-data
    aliases: [rad]
    members: [researcher@example.org]
    variables:
      members: RESEARCH_DATA_MEMBERS
  - id: instrumentation
    bucket: pilot-data
    aliases: [instruments]
    path: programs/instrumentation/index.json
    variables:
      members: INSTRUMENTATION_MEMBERS
      custom:
        notification_channel: INSTRUMENTATION_CHANNEL
options:
  powerAutomate:
    callback_destination: powerautomate
    gitlab_project: "22464"
    package: acorn-automated-artifacts
    timezone: UTC
    # Planned JSONL intake settings; not accepted by the current schema.
    intake:
      branch: main
      path: .acorn/intake/submissions.jsonl
      poll_interval_seconds: 30
      max_file_bytes: 10485760
      max_record_bytes: 65536
```

Unused option counterparts are intentionally omitted from serialized output. Their typed slots remain documented in the schema: `schemaVersion`, `config`, and `options` have no counterparts, while `options.scripts` covers both root `scripts` and `scriptManifest`. Reserved handlers retain `unimplemented!()` placeholders, but configuration parsing and serialization never invoke them. Empty optional attributes and recursively empty parents are omitted in every format; meaningful `false` and `0` values remain. With `options.folders` omitted here, relative paths use the GitLab fallback.

## Companion deployment `.env`

```dotenv
ACORN_LOGBOOK_MEMBERS=maintainer@example.org,operator@example.org
RESEARCH_DATA_MEMBERS=analyst@example.org
INSTRUMENTATION_MEMBERS=engineer@example.org
ACORN_DATABASE_PATH=.acorn/acorn.db
ACORN_LOG_LEVEL=info
XYLEM_LOG_LEVEL=info
GITLAB_TOKEN=replace-with-deployment-token
POWER_AUTOMATE_INBOUND_SECRET=replace-with-generated-secret
POWER_AUTOMATE_CALLBACK_URL=https://example.org/replace-with-callback-url
ORG_NOTIFICATION_CHANNEL=research-highlights
INSTRUMENTATION_CHANNEL=instrumentation-highlights
```

This is placeholder deployment data. Process environment values override the corresponding `.env` value, including empty values. Inline and environment members combine; global and project membership grant access independently. Explicit missing variable references fail configuration validation. Changing environment values requires service restart.

## Routing and access outcomes

| Selector | Canonical project | Destination | Additional project access |
| --- | --- | --- | --- |
| `acorn` or alias `acorn-tooling` | `acorn` | `acorn/index.json` | None required; global members apply |
| `research-data` or alias `rad` | `research-data` | `research-data/index.json` | `researcher@example.org` and `analyst@example.org` |
| `instrumentation` or alias `instruments` | `instrumentation` | `programs/instrumentation/index.json` | `engineer@example.org` |

Global access includes administrator, maintainer, and operator for all three projects. Instrumentation overrides only the custom notification channel for its consumers; it does not remove global membership. Unknown/ambiguous selectors and conflicting ID/alias pairs are rejected. The JSONL adapter resolves aliases to canonical IDs before constructing the existing submission envelope. The Power Automate producer writes canonical `projectId`; ACRE's YAML casing does not change the JSONL wire contract. Every entry still follows durable admission, duplicate detection, review, and human merge.

## Folder roots and absolute URI destinations

The full examples above omit `options.folders.root`, so their relative destinations use `options.powerAutomate.gitlab_project` as the repository fallback. Power Automate is not required for folder configuration.

A local root without Power Automate, in JSON:

```json
{
  "schemaVersion": "2.0.0",
  "members": ["administrator@example.org"],
  "folders": [{ "id": "acorn" }],
  "options": { "folders": { "root": "file:///C:/research/" } }
}
```

Equivalent JSONC:

```jsonc
{
  "schemaVersion": "2.0.0",
  "members": ["administrator@example.org"],
  // Resolves to file:///C:/research/acorn/index.json.
  "folders": [{ "id": "acorn" }],
  "options": { "folders": { "root": "file:///C:/research/" } }
}
```

Equivalent YAML:

```yaml
schema_version: "2.0.0"
members: [administrator@example.org]
folders:
  - id: acorn
options:
  folders:
    root: file:///C:/research/
```

Use `file:///srv/research/` for a POSIX local root or `https://data.example.org/research/` for a remote root. The latter resolves the omitted folder path to `https://data.example.org/research/acorn/index.json`; supported operations depend on the remote adapter, and an HTTPS URI does not by itself enable writes.

With neither a root nor a GitLab repository, every folder must have an explicit absolute URI:

```json
{
  "schemaVersion": "2.0.0",
  "members": ["administrator@example.org"],
  "folders": [
    { "id": "acorn", "path": "file:///C:/research/acorn/index.json" },
    { "id": "research-data", "path": "https://data.example.org/research-data/index.json" }
  ]
}
```

The JSONC form may use the same JSON document unchanged. In YAML:

```yaml
schema_version: "2.0.0"
members: [administrator@example.org]
folders:
  - id: acorn
    path: file:///C:/research/acorn/index.json
  - id: research-data
    path: https://data.example.org/research-data/index.json
```

| Configuration | Relative or omitted folder path |
| --- | --- |
| `options.folders.root` present | Resolve against that root; omission means `<id>/index.json` |
| Root absent, GitLab repository present | Resolve inside that repository; omission means `<id>/index.json` |
| Both absent | Reject relative or omitted paths; require an explicit absolute URI |

An explicit absolute URI designates its own destination. A root takes precedence over the GitLab fallback; neither the configuration directory nor working directory is an implicit base. Roots are directory URIs, with a missing trailing slash normalized before joining. Native absolute filesystem paths are supported with a local file root, but without either configuration source use explicit `file:` URIs, not bare Windows/POSIX/UNC paths. GitLab mutation still requires a destination inside its selected repository or mapped checkout; conflicting targets and unsupported operations fail explicitly.

## Bucket association and standalone local folders

This proposed configuration demonstrates a folder belonging to a local bucket and a standalone folder. Neither declaration triggers copying. Bucket IDs and folder bucket references are planned additions.

```json
{
  "schemaVersion": "2.0.0",
  "members": ["administrator@example.org"],
  "buckets": [
    { "id": "research", "name": "Research activities", "source": "file:///C:/research/" }
  ],
  "folders": [
    { "id": "acorn", "bucket": "research", "path": "file:///C:/research/acorn/index.json" },
    { "id": "independent-study", "path": "file:///C:/independent-study/index.json" }
  ]
}
```

This JSON is also valid JSONC. Equivalent YAML:

```yaml
schema_version: "2.0.0"
members: [administrator@example.org]
buckets:
  - id: research
    name: Research activities
    source: file:///C:/research/
folders:
  - id: acorn
    bucket: research
    path: file:///C:/research/acorn/index.json
  - id: independent-study
    path: file:///C:/independent-study/index.json
```

Both folders use absolute URIs because neither a folder root nor a GitLab fallback is configured; the bucket reference supplies no implicit root. The standalone activity is accessed in place, with its parent directory as the associated-file base. Importing the research bucket into a different destination is a separate explicit operation and does not redirect registered folders. Bucket membership and copying are separate concerns.

Discovery within a bucket grants no intake access. Global and folder members govern authorization. Standalone local JSONL intake requires a durable proposal and explicit human application before canonical writes; it does not infer GitLab review support from filesystem location. During implementation, add these distinctions and examples to the ACRE specification and configuration guides, alongside local bucket import/copy documentation.

### Index references for concise buckets

A folder may use a zero-based integer `bucket` reference instead of a string ID. This works for both concise and detailed bucket entries. String `"0"` is an ID, while integer `0` selects the first declared bucket. Resolve indexes before filtering or discovery. Reject negative, fractional, out-of-range, and boolean references. Index references and bucket array order must survive formatting and cross-format serialization unchanged; zero must not be omitted as empty.

JSON:

```json
{
  "schemaVersion": "2.0.0",
  "members": ["administrator@example.org"],
  "buckets": ["file:///C:/research/", "file:///C:/archive/"],
  "folders": [
    { "id": "acorn", "bucket": 0, "path": "file:///C:/research/acorn/index.json" },
    { "id": "history", "bucket": 1, "path": "file:///C:/archive/history/index.json" }
  ]
}
```

JSONC:

```jsonc
{
  "schemaVersion": "2.0.0",
  "members": ["administrator@example.org"],
  // Order matters for integer references.
  "buckets": ["file:///C:/research/", "file:///C:/archive/"],
  "folders": [
    { "id": "acorn", "bucket": 0, "path": "file:///C:/research/acorn/index.json" },
    { "id": "history", "bucket": 1, "path": "file:///C:/archive/history/index.json" }
  ]
}
```

YAML:

```yaml
schema_version: "2.0.0"
members: [administrator@example.org]
buckets:
  - file:///C:/research/
  - file:///C:/archive/
folders:
  - id: acorn
    bucket: 0
    path: file:///C:/research/acorn/index.json
  - id: history
    bucket: 1
    path: file:///C:/archive/history/index.json
```

Prefer IDs in long-lived configurations: reordering buckets changes index associations for new work. Persist the resolved bucket identity and destination at admission so retries retain their original routing. Index association does not supply a path root, trigger copying, or grant access. These examples are proposed extensions, not valid current ACRE 1.0.0 configurations.

## Specification and migration deliverables

Update `ACRE_SPEC.md`, the ACRE concept page, CLI configuration/validation/formatting/schema-export documentation, generated schemas, and Xylem's configuration documentation together with implementation. Publish all three examples as validated fixtures and document schema versioning, legacy nesting/casing migration, defaults, aliases, member parsing and precedence, variable catalog/custom consumers, intake limits, rejection behavior, and restart requirements. Replace proposed field/version labels only after the implementation validates them. Keep the older deployed configuration example explicitly labeled as legacy until migration is available.
