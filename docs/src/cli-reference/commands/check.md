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
- `--disable-website-checks` : Disable all website-related checks (link integrity, etc.)

> [!TIP]
> `--disable-website-checks` is redundant when `acorn -X` is used for commands that need to be run in offline environments.

### Configure Readability
Readability can be configured via a `.env` file or by passing options directly to the command line. Command line options override `.env` settings.
- `--readability-metric <METRIC>` : Specify which readability metric to use

[^metrics]: See the [readability module documentation](https://docs.rs/acorn-lib/latest/acorn_lib/analyzer/readability/enum.ReadabilityType.html) for a full list of available readability metrics.

[^warn]: ⚠️ Skipping checksum verification may expose you to security risks. Use this option with caution.