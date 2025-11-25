# 🗸 Check
> Perform various checks on associated research activity data

## Example Usage
```shell
acorn check path/to/project/
```

## Categories of Checks
- 🏛️ **Schema Validation**: Ensure that all data files conform to the expected schema
- ✨ **Prose Quality**: Analyze written content for grammar, spelling, and more
- 👓 **Readability**: Evaluate the readability of textual content using established metrics[^metrics]
- 🔗 **Link Integrity**: Verify that all hyperlinks within the content are valid and reachable
- 📊 **Data Consistency**: Check for consistency and completeness in datasets
- 🚦 **Convention Adherence**: Ensure that naming conventions and organizational standards are followed

> [!NOTE]
> The [`check`](./check.md) command is run as part of the [`format`](./format.md) command to ensure that content meets quality standards before formatting.

## Customization Options
The `check` command supports several flags and options to customize its behavior (e.g., skipping certain checks, disabling certain behaviors, etc.)

Include `--exit-on-first-error` to stop execution upon encountering the first error.

Bypass verifying the checksum of downloaded artifacts with `--skip-verify-checksum`[^warn]

### Skip checks
- `--skip schema` : Skip schema validation checks
- `--skip prose` : Skip prose quality checks
- `--skip readability` : Skip readability checks
- `--skip schema,prose` : Skip both schema validation and prose quality checks (works for any combination of categories)
- `--disable-website-checks` : Disable all website-related checks (link integrity, etc.)

> [!NOTE]
> `--disable-website-checks` is redundant when `acorn --offline` is used for commands that need to be run in offline environments.

### Configure Readability
Readability can be configured by passing options directly to the command line or via a `.env` file. Command line options override `.env` settings.
- `--readability-metric <METRIC>` : Specify which readability metric to use
- Set `READABILITY_METRIC` in your `.env` file to choose the readability metric. Default metric is [`fkgl`](https://acorn.ornl.gov/acorn_lib/analyzer/readability/enum.ReadabilityType.html#variant.FKGL) (Fleshch-Kincaid Grade Level).
- Set `MAX_ALLOWED_FKGL` in your `.env` file to define the maximum acceptable FKGL score. Each metric has its own corresponding maximum score variable (e.g., `MAX_ALLOWED_ARI` for Automated Readability Index).

#### Example `.env` file
Configure ACORN to use the Coleman-Liau Index (CLI) readability metric with a maximum allowed score of 14.0 ([default value](https://acorn.ornl.gov/acorn_lib/util/constants/constant.MAX_ALLOWED_CLI.html) is `12.0`):
```env
READABILITY_METRIC=cli
MAX_ALLOWED_CLI=14.0
```

[^metrics]: See the [readability module documentation](https://docs.rs/acorn-lib/latest/acorn_lib/analyzer/readability/enum.ReadabilityType.html) for a full list of available readability metrics.

[^warn]: ⚠️ Skipping checksum verification may expose you to security risks. Use this option with caution.