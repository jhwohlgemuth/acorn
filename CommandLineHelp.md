# Command-Line Help for `acorn-cli`

This document contains the help content for the `acorn-cli` command-line program.

**Command Overview:**

* [`acorn-cli`↴](#acorn-cli)
* [`acorn-cli check`↴](#acorn-cli-check)
* [`acorn-cli create`↴](#acorn-cli-create)
* [`acorn-cli create mcp`↴](#acorn-cli-create-mcp)
* [`acorn-cli create runner`↴](#acorn-cli-create-runner)
* [`acorn-cli create bot`↴](#acorn-cli-create-bot)
* [`acorn-cli doctor`↴](#acorn-cli-doctor)
* [`acorn-cli download`↴](#acorn-cli-download)
* [`acorn-cli download model`↴](#acorn-cli-download-model)
* [`acorn-cli export`↴](#acorn-cli-export)
* [`acorn-cli format`↴](#acorn-cli-format)
* [`acorn-cli gather`↴](#acorn-cli-gather)
* [`acorn-cli import`↴](#acorn-cli-import)
* [`acorn-cli import model`↴](#acorn-cli-import-model)
* [`acorn-cli import spec`↴](#acorn-cli-import-spec)
* [`acorn-cli link`↴](#acorn-cli-link)
* [`acorn-cli skill`↴](#acorn-cli-skill)
* [`acorn-cli serve`↴](#acorn-cli-serve)
* [`acorn-cli serve bot`↴](#acorn-cli-serve-bot)
* [`acorn-cli serve mcp`↴](#acorn-cli-serve-mcp)
* [`acorn-cli sync`↴](#acorn-cli-sync)
* [`acorn-cli tui`↴](#acorn-cli-tui)

## `acorn-cli`

"Plant an ACORN and grow your science"

⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣤⣄⣀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⣀⠀⢴⣶⠀⢶⣦⠀⢄⣀⠀⠠⢾⣿⠿⠿⠿⠿⢦⠀
⠀⠀⠀⠀⠀⠀⠺⠿⠇⢸⣿⣇⠘⣿⣆⠘⣿⡆⠠⣄⡀⠀⠀⠀⠀⠀⠀    
⠀⠀⠀⠀⢀⣴⣶⣶⣤⣄⡉⠛⠀⢹⣿⡄⢹⣿⡀⢻⣧⠀⡀⠀⠀⠀⠀    
⠀⠀⠀⣰⣿⣿⣿⣿⣿⣿⣿⣿⣶⣤⡈⠓⠀⣿⣧⠈⢿⡆⠸⡄⠀⠀⠀   █████████     █████████     ███████    ███████████   ██████   █████
⠀⠀⣰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣦⣈⠙⢆⠘⣿⡀⢻⠀⠀   ███▒▒▒▒▒███   ███▒▒▒▒▒███  ███▒▒▒▒▒███ ▒▒███▒▒▒▒▒███ ▒▒██████ ▒▒███
⠀⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠀⠹⣧⠈⠀⠀  ▒███    ▒███  ███     ▒▒▒  ███     ▒▒███ ▒███    ▒███  ▒███▒███ ▒███
⠀⣸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠈⠃⠀⠀⠀ ▒███████████ ▒███         ▒███      ▒███ ▒██████████   ▒███▒▒███▒███
⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁⠀⠀⠀⠀ ▒███▒▒▒▒▒███ ▒███         ▒███      ▒███ ▒███▒▒▒▒▒███  ▒███ ▒▒██████⠀
⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀⠀⠀⠀ ▒███    ▒███ ▒▒███     ███▒▒███     ███  ▒███    ▒███  ▒███  ▒▒█████
⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠃⠀⠀⠀⠀⠀⠀ █████   █████ ▒▒█████████  ▒▒▒███████▒   █████   █████ █████  ▒▒█████⠀
⠀⢹⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠋⠀⠀⠀⠀⠀⠀⠀⠀▒▒▒▒▒   ▒▒▒▒▒   ▒▒▒▒▒▒▒▒▒     ▒▒▒▒▒▒▒    ▒▒▒▒▒   ▒▒▒▒▒ ▒▒▒▒▒    ▒▒▒▒▒
⠀⠈⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠟⠉⠀⠀⠀⠀⠀⠀⠀    
⠀ ⣿⣿⠿⠿⠿⠿⠿⠿⠟⠛⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀~ Accessible Content Optimization for Research Needs ~⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀

**Usage:** `acorn-cli [OPTIONS] [COMMAND]`

###### **Subcommands:**

* `check` — Perform static analysis on research activity data and apply standardized best practices
* `create` — Create research resources like GitLab runners, MCP servers, etc.
* `doctor` — Diagnose and correct system requirements for using acorn
* `download` — Download research activity data from buckets or model weights for local inference
* `export` — Export research activity data to a specific target
* `format` — Formats research activity data in place (inherently includes some elements of `acorn check`)
* `gather` — Gather persistent identifiers from documents, URLs, or text
* `import` — Import external metadata into ACORN
* `link` — Add linked data context to research activity data
* `skill` — Print the embedded ACORN skill path and copy agent instructions to clipboard
* `serve` — Start a server (bot, MCP, etc.)
* `sync` — Synchronize local inference configuration from ACORN model entries
* `tui` — Launch the interactive terminal user interface (TUI)

###### **Options:**

* `--database-backend <BACKEND>` — Select database backend (`sqlite` or `duckdb`) for local cache/history storage

  Possible values:
  - `sqlite`:
    SQLite backend
  - `duckdb`:
    DuckDB backend

* `-P`, `--database-path <PATH>` — Override local database path (defaults to cache directory when omitted)
* `-N`, `--no-local-database` — Disable local database initialization at startup
* `-i`, `--interactive` — Launch interactive terminal user interface (TUI)
* `-X`, `--offline` — Prevent communication with the internet - intended for disconnected local environments

   Note: Use of --offline may require extra configuration options for certain commands
* `-t`, `--threads <N>` — Limit number of threads used by rayon for parallel processing

   See Rayon documentation for more information

  Default value: `10`
* `--clear-cache` — Clear database cache data and downloaded Chromium artifacts
* `--reset-database` — Clear all data from database (all tables)
* `--no-clear-cache` — Disable automatic cache cleanup on startup (expires old cache entries)
* `--cache-ttl <SECONDS>` — Override default cache TTL (in seconds). Default is 2592000 (30 days)

   Set to 0 to disable caching
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli check`

Perform static analysis on research activity data and apply standardized best practices

    Check a file
    $> acorn check /path/to/file.json

    Check all supported files in a folder
    $> acorn check /path/to/folder

    Skip static analysis
    $> acorn check /path/to/folder --skip schema

    Skip schema validation
    $> acorn check /path/to/folder --skip prose,schema

    Check file(s) that were changed in a given Git commit
    $> acorn check --commit <commit hash>

    Check file(s) that were changed in latest Git commit
    $> acorn check --commit HEAD

**Usage:** `acorn-cli check [OPTIONS] [PATH]`

**Command Alias:** `lint`

###### **Arguments:**

* `<PATH>` — Path to check

  Default value: `./`

###### **Options:**

* `-b`, `--branch <BRANCH>` — Check files that were changed in a given Git branch
* `-c`, `--commit <COMMIT>` — Check files that were changed in a given Git commit or commit range

   Check files that were changed in the last 10 commits &> acorn check --commit HEAD~10..HEAD

   See <https://git-scm.com/book/en/v2/Git-Tools-Revision-Selection> for more information of commit ranges
* `-d`, `--disable-website-checks` — Disable website checks that require internet connection
* `--all` — Include all check results, including suggestions and info messages
* `-e`, `--exit-on-first-error` — Exit on first error
* `--filter <LIST>` — Regular expression pattern(s) applied to absolute paths of files to include during checks

   Only files matching at least one pattern will be processed
* `-i`, `--ignore <LIST>` — Regular expression pattern(s) applied to absolute paths of files to exclude from checking

   Only applies to `--path` values that point to a directory

   Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals

   Example: --ignore "[/]valid.json$,[/]draft.json$"
* `-m`, `--merge-request` — Format files that were changed in a given merge request (Gitlab) or pull request (GitHub)
* `--no-fail` — Prevent `acorn check` from exiting with a non-zero exit code when errors are found

   Useful for certain scenarios such as debugging

   > CAUTION: Use at your own risk
* `--raw` — Suppress output from external commands (e.g., Vale) while preserving ACORN output — useful for piping
* `-S`, `--skip <LIST>` — Skip one or more available quality check categories
* `--skip-verify-checksum` — Skip checksum verification when downloading files (e.g., downloading Vale to perform static analysis of prose) > CAUTION: Use at your own risk
* `-s`, `--standard <STANDARD>` — Metadata standard to validate against

   Supported standards: rads, cff, datacite, dcat, dcmi, docx, invenio, huwise, raid, text

  Default value: `rads`
* `-r`, `--readability-test <METRIC>` — Readability test to use

  Default value: `fkgl`
* `-t`, `--terse` — Use compact output format instead of detailed output
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli create`

Create research resources like GitLab runners, MCP servers, etc.

    $> acorn create runner --config ./acorn.json

    $> acorn create runner --group 12345

**Usage:** `acorn-cli create [COMMAND]`

###### **Subcommands:**

* `mcp` — Model context provider (MCP) server
* `runner` — GitLab runner
* `bot` — Bot server container



## `acorn-cli create mcp`

Model context provider (MCP) server

**Usage:** `acorn-cli create mcp`



## `acorn-cli create runner`

GitLab runner

**Usage:** `acorn-cli create runner [OPTIONS]`

###### **Options:**

* `-c`, `--config <PATH>` — Path to configuration file for creating the resource

  Default value: `./`
* `-d`, `--description <DESCRIPTION>` — Runner description (alternative to providing a configuration file)
* `-g`, `--group <ID>` — GitLab group ID to create a runner for (alternative to providing a configuration file)
* `-r`, `--repo <REPO>` — GitLab repository domain override for runner creation (e.g., code.ornl.gov)
* `-p`, `--project <ID>` — GitLab project ID to create a runner for (alternative to providing a configuration file)
* `-n`, `--name <NAME>` — Runner name (used as Docker container name)
* `-s`, `--server <SERVER>` — Hosting server location (for use during registration)
* `-t`, `--tags <LIST>` — Tags to register the runner with (alternative to providing a configuration file)
* `-u`, `--untagged` — Whether this runner can pick up jobs without matching tags (alternative to providing a configuration file)
* `-R`, `--runtime <RUNTIME>` — Container runtime override (docker, podman, apptainer). Defaults to the executor's runtime
* `--remote <URI>` — SSH endpoint of the remote Docker daemon

   The local Docker client and SSH configuration are used. Published ports and volumes belong to the remote host.
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli create bot`

Bot server container

**Usage:** `acorn-cli create bot [OPTIONS] <PROJECT_ID>`

###### **Arguments:**

* `<PROJECT_ID>` — GitLab project identifier to poll for note events

###### **Options:**

* `-b`, `--bind <ADDRESS>` — Address and port for the bot HTTP server to bind to
* `-p`, `--port <PORT>` — Port for the bot HTTP server to bind to
* `--after <TIMESTAMP>` — Initial ISO 8601 timestamp to start polling from
* `--poll-interval <SECONDS>` — Polling interval in seconds

  Default value: `10`
* `-R`, `--runtime <RUNTIME>` — Container runtime (docker, podman, apptainer)
* `--event-source <EVENT_SOURCE>` — Event delivery source

  Default value: `poll`

  Possible values:
  - `poll`:
    Poll GitLab's Events API
  - `webhook`:
    Receive authenticated GitLab webhooks
  - `hybrid`:
    Receive webhooks and retain polling as best-effort reconciliation

* `--public-url <URL>` — Externally reachable HTTPS base URL for webhook delivery
* `--register-webhook` — Register the project webhook before starting
* `-n`, `--name <NAME>` — Container name (defaults to `acorn-bot-{identifier}`)
* `-i`, `--image <IMAGE>` — Acorn Docker image to use for the bot container

  Default value: `acorn:latest`
* `-d`, `--domain <DOMAIN>` — GitLab domain override (e.g., code.ornl.gov)
* `--volume <VOLUME>` — Persistent named volume for bot state
* `--remote <URI>` — SSH endpoint of the remote Docker daemon

   The local Docker client and SSH configuration are used. Published ports and volumes belong to the remote host.
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli doctor`

Diagnose and correct system requirements for using acorn

    Print system diagnostics and identify issues
    $> acorn doctor

    Auto-correct systems issues
    $> acorn doctor --fix

    Start interactive TUI to selectively apply fixes
    $> acorn doctor --fix --interactive

    Generate JSON report for use when filing bug reports
    $> acorn doctor --report

**Usage:** `acorn-cli doctor [OPTIONS]`

###### **Options:**

* `-f`, `--fix` — Auto-correct systems issues where possible
* `-i`, `--interactive` — Start interactive TUI to selectively apply fixes
* `-r`, `--report` — Choose to output a JSON issue report to stdout
* `-c`, `--check <LIST>` — Select one or more available diagnostic checks

  Default value: `all`

  Possible values: `all`, `system`, `memory`, `network`, `gpu`, `software`

* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli download`

Download research activity data from buckets or model weights for local inference

    Download research activity data from a bucket repository URL
    $> acorn download https://github.com/user/repo

    Download research activity data from a list of buckets
    $> acorn download --config /path/to/.acorn.json

    Download research activity data to a specific output directory
    $> acorn download --config /path/to/.acorn.yml --output /path/to/output
    
    Download using default configuration in current directory (first match):
    .acorn.json, .acorn.jsonc, .acorn.yaml, .acorn.yml, .acorn
    $> acorn download

    Download default GGUF (Q4_K_M) model from a Hugging Face repository
    $> acorn download model meta-llama/Llama-3.1-8B

    Download a specific quantization
    $> acorn download model meta-llama/Llama-3.1-8B --filter "Q8_0.*\\.gguf$"

    Exclude low-quality quantizations
    $> acorn download model meta-llama/Llama-3.1-8B --ignore "Q2_|Q3_|imatrix"

    Download models from a config file
    $> acorn download model --config .acorn.json

**Usage:** `acorn-cli download [OPTIONS] [URL]... [COMMAND]`

**Command Alias:** `fetch`

###### **Subcommands:**

* `model` — Download model weights for use with ACORN research harnesses and local inference

###### **Arguments:**

* `<URL>` — Bucket repository URL(s)

###### **Options:**

* `-c`, `--config <PATH>` — Path to configuration file (JSON, JSONC, or YAML)
* `--filter <LIST>` — Regular expression pattern(s) used to include files while downloading

   Only files matching at least one pattern will be processed
* `-i`, `--ignore <LIST>` — Regular expression pattern(s) to ignore while downloading

   Values augment built-in ignores and can be provided as a comma-separated list
* `-o`, `--output <PATH>` — Path to output directory where the downloaded content should be saved

  Default value: `./content`
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli download model`

Download model weights for use with ACORN research harnesses and local inference

**Usage:** `acorn-cli download model [OPTIONS] [MODEL]...`

###### **Arguments:**

* `<MODEL>` — Model ID/name(s), local path(s), or direct URL(s) to download

###### **Options:**

* `--model-file <URI_OR_PATH>` — URI or local path to a model list file

   Accepts a plain-text list of repository IDs or a JSON/YAML list of IDs or model details
* `--sync <TARGET>` — Add downloaded models to ACORN and synchronize OpenCode and/or llama-swap configuration

  Possible values:
  - `all`:
    Synchronize all eligible targets
  - `opencode`:
    Synchronize OpenCode only
  - `llama-swap`:
    Synchronize llama-swap only
  - `vscode`:
    Synchronize VS Code only
  - `goose`:
    Synchronize Goose CLI only

* `--force` — Assume synchronized models exist under the models directory
* `-c`, `--config <PATH>` — Path to configuration file listing models to download
* `--filter <LIST>` — Regular expression pattern(s) used to include model weight sources while downloading

   Only sources matching at least one pattern will be downloaded
* `-i`, `--ignore <LIST>` — Regular expression pattern(s) to ignore model weight sources while downloading

   Sources matching any ignore pattern are excluded even if they match a filter pattern
* `--quantization <LIST>` — Ordered, exact GGUF quantizations to consider
* `--gpu-memory <MEMORY>` — Maximum GPU memory available for model weights
* `-o`, `--local-dir <PATH>` [alias: `output`] — Local directory where downloaded model weights should be saved

   Defaults to $HOME/.models when omitted
* `--copy` — Copy local model files into the model directory instead of referencing in place
* `--symlink` — Create symlinks to local model files in the model directory instead of referencing in place
* `--whitelist <NAME>` — Only download models with these user-facing names
* `--whitelist-file <URI_OR_PATH>` — URI or local path to a model whitelist file

   Accepts a plain-text list of names or a JSON/YAML list of names or model details
* `--skip-verify-checksum` — Skip SHA-256 checksum verification when checksum sidecar metadata is available
* `--search-limit <LIMIT>` — Maximum number of Hugging Face repositories to search when discovering GGUF quantizations

   Applies when the requested repository contains no GGUF files and automatic fallback is enabled

  Default value: `20`
* `--minimum-popularity <COUNT>` — Minimum number of downloads required for a GGUF fallback repository

  Default value: `100`
* `--no-fallback` — Disable automatic GGUF quantization repository discovery when the target has no GGUF files
* `--interactive` — Open an interactive picker when GGUF fallback finds multiple repositories
* `--dry-run` [alias: `what-if`] — Preview resolved models and fallbacks without downloading
* `--raw` — Output structured JSON (or YAML with --format yaml) of resolved models
* `--format <FORMAT>` — Output format for --raw results

  Default value: `json`

  Possible values: `json`, `yaml`

* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli export`

Export research activity data to a specific target

    $> acorn export /path/to/data

    $> acorn export /path/to/data/data.json -vv

    $> acorn export /path/to/data --target highlight --format powerpoint

    $> acorn export /path/to/data -t poster -o /path/to/output

    Export folder as a BagIt archive to /path/to/bag.zip
    $> acorn export /path/to/data --format bag --output /path/to/bag

    Export file(s) that were changed in a given Git commit
    $> acorn export --commit <commit hash>

    Export file(s) that were changed in latest Git commit
    $> acorn export --commit HEAD

**Usage:** `acorn-cli export [OPTIONS] [PATH]`

###### **Arguments:**

* `<PATH>` — Path of input files to be exported

  Default value: `./`

###### **Options:**

* `-b`, `--branch <BRANCH>` — Export files that were changed in a given Git branch
* `--chrome-path <PATH>` — Chrome or Chromium executable to use for PDF export
* `--combine` — Combine all export artifacts into a single artifact
* `-c`, `--commit <COMMIT>` — Export files that were changed in a given Git commit
* `--dry-run` [alias: `what-if`] — Run export without making changes. Will print what would happen
* `-f`, `--format <FORMAT>` — Export target file format

   When used with --schema, specifies the output format (json or yaml)

  Default value: `pdf`

  Possible values: `pdf`, `bag`, `cff`, `json`, `markdown`, `powerpoint`, `yaml`

* `--filter <LIST>` — Regular expression pattern(s) applied to absolute paths of files to include during export

   Only files matching at least one pattern will be processed
* `-i`, `--ignore <LIST>` — Regular expression pattern(s) applied to absolute paths of files to exclude from export process

   Only applies to `--path` values that point to a directory

   Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals

   Example: --ignore "[/]valid.json$,[/]draft.json$"
* `--show-aspect-labels` — Show ASPECT attribute labels in exported charts
* `--show-aspect-scores` — Show ASPECT numeric scores in exported charts
* `-m`, `--merge-request` — Export files that were changed in a given merge request (Gitlab) or pull request (GitHub)
* `-o`, `--output <DIRECTORY>` — Path to output directory where the export artifacts should be saved

  Default value: `./export`
* `--schema` — Print JSON or YAML schema for the selected standard to stdout

   Use --format to specify output format (default: json)
* `--raw` — Suppress output from external commands while preserving ACORN output — useful for piping
* `-S`, `--skip <LIST>` — Skip one or more available export check categories
* `-r`, `--reference <PATH>` — Path to reference file to be used when exporting to PowerPoint

   Can be relative or absolute. Relative paths will be resolved against the directory of the target file.
* `-s`, `--standard <STANDARD>` — Metadata standard to validate against

   Supported standards: rads, cff, datacite, dcat, dcmi, docx, invenio, huwise, raid, text

  Default value: `rads`
* `--strict` — Fail if conversion loses data (only for crosswalk exports)
* `--from <STANDARD>` — Metadata standard to use for source data inference (default: auto-detect)

   Supported standards: rads, cff, datacite, dcat, dcmi, docx, invenio, huwise, raid, text
* `--to <STANDARD>` — Target metadata standard for crosswalk conversion

   Supported standards: rads, cff, datacite, dcat, dcmi, docx, invenio, huwise, raid, text
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli format`

Formats research activity data in place (inherently includes some elements of `acorn check`)

    $> acorn format /path/to/file.json

    $> acorn format /path/to/folder --dry-run

    Format file(s) that were changed in a given Git commit
    $> acorn format --commit <commit hash>

    Format file(s) that were changed in latest Git commit
    $> acorn format --commit HEAD

**Usage:** `acorn-cli format [OPTIONS] [PATH]`

**Command Alias:** `fmt`

###### **Arguments:**

* `<PATH>` — Path to look for files to format

  Default value: `./`

###### **Options:**

* `-b`, `--branch <BRANCH>` — Format files that were changed in a given Git branch
* `-c`, `--commit <COMMIT>` — Format files that were changed in a given Git commit
* `-d`, `--dry-run` [alias: `what-if`] — Run format without making changes to target file(s). Will print a diff of changes
* `--no-color` — Disable color in dry-run output
* `--filter <REGEX>` — Regex pattern applied to absolute paths of files to include in formatting process

   Only files matching the pattern will be processed
* `-i`, `--ignore <REGEX>` — Regex pattern applied to absolute paths of files that determines whether they should be included in formatting process

   Only applies to path values that point to a directory

   Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals

   Example: --ignore "[/]valid.json$"
* `-m`, `--merge-request` — Format files that were changed in a given merge request (Gitlab) or pull request (GitHub)
* `-s`, `--standard <STANDARD>` — Metadata standard to validate against

   Supported standards: rads, cff, datacite, dcat, dcmi, docx, invenio, huwise, raid, text

  Default value: `rads`
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli gather`

Gather persistent identifiers from documents, URLs, or text

    $> acorn gather report.docx --text "doi:10.1234/example"

Directory inputs are searched recursively. Use `--max-depth 1` to search only direct children.

**Usage:** `acorn-cli gather [OPTIONS] [INPUT]...`

**Command Alias:** `harvest`

###### **Arguments:**

* `<INPUT>` — Files, directories, URLs, or PID values; piped standard input is included automatically

###### **Options:**

* `--filter <REGEX>` — Regex pattern applied to absolute paths of files to include in processing

   Only files matching the pattern will be processed
* `-i`, `--ignore <REGEX>` — Regex pattern applied to absolute paths of files that determines whether they should be included in processing

   Only applies to path values that point to a directory

   Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals

   Example: --ignore "[/]valid.json$"
* `--max-depth <DEPTH>` — Maximum directory descendant depth to search; direct children have depth 1
* `-m`, `--merge-request` — Process files that were changed in a given merge request (GitLab) or pull request (GitHub)
* `-o`, `--output <PATH>` — Write the report to a file instead of stdout
* `--osti <OSTI>` — Search DOE CODE and return projects, people, or organizations

  Possible values:
  - `projects`:
    Software projects
  - `people`:
    Developers and contributors
  - `organizations`:
    Credited organizations

* `--organization <TERM>` — Restrict remote results to a credited organization
* `--organization-role <ORGANIZATION_ROLE>` — Organization relationship to match

  Possible values:
  - `any`:
    Any credited organization relationship
  - `site-owner`:
    Submitting site
  - `research`:
    Research organization
  - `sponsor`:
    Sponsoring organization
  - `contributor`:
    Contributing organization
  - `developer`:
    Developing organization

* `--lab <LAB>` — Restrict results to a DOE laboratory and add its known ROR to persisted project candidates

  Possible values:
  - `ames`:
    Ames National Laboratory
  - `anl`:
    Argonne National Laboratory
  - `bnl`:
    Brookhaven National Laboratory
  - `fnal`:
    Fermi National Accelerator Laboratory
  - `inl`:
    Idaho National Laboratory
  - `lbnl`:
    Lawrence Berkeley National Laboratory
  - `llnl`:
    Lawrence Livermore National Laboratory
  - `lanl`:
    Los Alamos National Laboratory
  - `netl`:
    National Energy Technology Laboratory
  - `nrel`:
    National Renewable Energy Laboratory
  - `ornl`:
    Oak Ridge National Laboratory
  - `pnnl`:
    Pacific Northwest National Laboratory
  - `pppl`:
    Princeton Plasma Physics Laboratory
  - `snl`:
    Sandia National Laboratories
  - `srnl`:
    Savannah River National Laboratory
  - `slac`:
    SLAC National Accelerator Laboratory
  - `tjnaf`:
    Thomas Jefferson National Accelerator Facility

* `--limit <COUNT>` — Number of upstream projects requested per page

  Default value: `20`
* `--offset <COUNT>` — Zero-based upstream project offset

  Default value: `0`
* `--all` — Retrieve every matching upstream project page
* `--format <FORMAT>` — Structured report format; defaults to console for terminals and JSON for pipes or files

  Possible values:
  - `console`:
    Human-readable console table
  - `json`:
    JavaScript Object Notation
  - `markdown`:
    GitHub-Flavored Markdown
  - `yaml`:
    YAML

* `--raw` — Print discovered identifiers as plain text, one per line
* `--resolve` — Resolve supported identifiers through their metadata providers
* `--citation-format <FORMAT>` — Citation format used for resolved DOI lines
* `-s`, `--standard <STANDARD>` — Metadata standard to validate against
* `--text <TEXT>` — Literal text to inspect; may be repeated
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli import`

Import external metadata into ACORN

**Usage:** `acorn-cli import [COMMAND]`

**Command Alias:** `load`

###### **Subcommands:**

* `model` — Import model metadata into the local database
* `spec` — Import endpoint resources from an API specification



## `acorn-cli import model`

Import model metadata into the local database

**Usage:** `acorn-cli import model [OPTIONS] [MODEL]...`

###### **Arguments:**

* `<MODEL>` — Hugging Face model repository ID(s) to inspect

###### **Options:**

* `--model-file <URI_OR_PATH>` — URI or local path to a model list file

   Accepts a plain-text list of repository IDs or a JSON/YAML list of IDs or model details
* `--sync <TARGET>` — Add imported models to ACORN and synchronize OpenCode and/or llama-swap configuration

  Possible values:
  - `all`:
    Synchronize all eligible targets
  - `opencode`:
    Synchronize OpenCode only
  - `llama-swap`:
    Synchronize llama-swap only
  - `vscode`:
    Synchronize VS Code only
  - `goose`:
    Synchronize Goose CLI only

* `--force` — Assume synchronized models exist under the models directory
* `--dry-run` [alias: `what-if`] — Resolve model metadata without database or configuration writes
* `-c`, `--config <PATH>` — Path to configuration file listing models to inspect
* `--no-fallback` — Disable automatic GGUF quantization repository discovery
* `--interactive` — Open an interactive picker when GGUF fallback finds multiple repositories
* `--search-limit <LIMIT>` — Maximum number of Hugging Face repositories to search during GGUF fallback discovery

  Default value: `20`
* `--minimum-popularity <COUNT>` — Minimum number of downloads required for a GGUF fallback repository

  Default value: `100`
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli import spec`

Import endpoint resources from an API specification

**Usage:** `acorn-cli import spec [OPTIONS] [SOURCE]`

**Command Alias:** `openapi`

###### **Arguments:**

* `<SOURCE>` — API specification source path or HTTP(S) URL

  Default value: `openapi.yaml`

###### **Options:**

* `--name <NAME>` — Endpoint template name to place on the generated object
* `--domain <DOMAIN>` — API domain for the generated endpoint object
* `--root <ROOT>` — Optional API root path for the generated endpoint object
* `--auth-token <TOKEN>` — Optional bearer token value for the generated endpoint object
* `-o`, `--output <PATH>` — Write generated endpoint JSON to this file
* `--dry-run` [alias: `what-if`] — Print generated endpoint JSON without writing a file
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli link`

Add linked data context to research activity data

    $> acorn link /path/to/data/

**Usage:** `acorn-cli link [OPTIONS] [PATH]`

###### **Arguments:**

* `<PATH>` — Path to look for files to process

  Default value: `./`

###### **Options:**

* `-b`, `--branch <BRANCH>` — Process files that were changed in a given Git branch
* `-c`, `--commit <COMMIT>` — Process files that were changed in a given Git commit
* `-d`, `--dry-run` [alias: `what-if`] — Run link without making changes to target file(s). Will print a diff of changes
* `-i`, `--ignore <REGEX>` — Regex pattern applied to absolute paths of files that determines whether they should be included in processing

   Only applies to path values that point to a directory

   Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals

   Example: --ignore "[/]valid.json$"
* `-m`, `--merge-request` — Processes files that were changed in a given merge request (Gitlab) or pull request (GitHub)
* `-s`, `--standard <STANDARD>` — Metadata standard to validate against

  Default value: `rads`
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli skill`

Print the embedded ACORN skill path and copy agent instructions to clipboard

    $> acorn skill

**Usage:** `acorn-cli skill`



## `acorn-cli serve`

Start a server (bot, MCP, etc.)

    Start a GitLab bot server
    $> acorn serve bot <project-id>

    Start an MCP server
    $> acorn serve mcp

**Usage:** `acorn-cli serve [COMMAND]`

###### **Subcommands:**

* `bot` — Webhook/API bot server that polls GitLab for merge request note events
* `mcp` — Model context protocol (MCP) server



## `acorn-cli serve bot`

Webhook/API bot server that polls GitLab for merge request note events

**Usage:** `acorn-cli serve bot [OPTIONS] <PROJECT_ID>`

###### **Arguments:**

* `<PROJECT_ID>` — GitLab project identifier to poll for note events

###### **Options:**

* `-b`, `--bind <ADDRESS>` — Address and port for the bot HTTP server to bind to
* `-p`, `--port <PORT>` — Port for the bot HTTP server to bind to
* `--after <TIMESTAMP>` — Initial ISO 8601 timestamp to start polling from
* `--poll-interval <SECONDS>` — Polling interval in seconds

  Default value: `10`
* `-R`, `--runtime <RUNTIME>` — Container runtime (docker, podman, apptainer)
* `--event-source <EVENT_SOURCE>` — Event delivery source

  Default value: `poll`

  Possible values:
  - `poll`:
    Poll GitLab's Events API
  - `webhook`:
    Receive authenticated GitLab webhooks
  - `hybrid`:
    Receive webhooks and retain polling as best-effort reconciliation

* `--public-url <URL>` — Externally reachable HTTPS base URL for webhook delivery
* `--register-webhook` — Register the project webhook before starting
* `-d`, `--detach` — Create and run the bot in a detached Docker/Podman container instead of in-process



## `acorn-cli serve mcp`

Model context protocol (MCP) server

**Usage:** `acorn-cli serve mcp`



## `acorn-cli sync`

Synchronize local inference configuration from ACORN model entries

    Synchronize configurations for installed applications
    $> acorn sync

    Include OpenCode configuration
    $> acorn sync --opencode

    Include llama-swap configuration
    $> acorn sync --llama-swap

    Include VS Code configuration
    $> acorn sync --vscode

    Include Goose CLI configuration
    $> acorn sync --goose

    Preview changes without writing files
    $> acorn sync --dry-run

**Usage:** `acorn-cli sync [OPTIONS]`

###### **Options:**

* `-c`, `--config <PATH>` — Path to ACORN configuration file (JSON, JSONC, or YAML)
* `--model-file <URI_OR_PATH>` — URI or local path to a model list file

   Accepts a plain-text list of repository IDs or a JSON/YAML list of IDs or model details
* `--force` — Bypass command detection for explicitly included targets
* `--assume-models` — Assume configured models exist without checking directories or GGUF files
* `--no-fallback` — Disable resolution through previously downloaded GGUF fallback repositories
* `--opencode` — Include OpenCode configuration synchronization
* `--llama-swap` — Include llama-swap configuration synchronization
* `--vscode` — Include VS Code custom endpoint configuration synchronization
* `--goose` — Include Goose CLI configuration synchronization
* `--models-dir <PATH>` — Default directory where downloaded model weights live (overrides config)
* `--opencode-config <PATH>` — Path to OpenCode configuration file (overrides config)
* `--llama-swap-config <PATH>` — Path to llama-swap configuration file (overrides config)
* `--vscode-config <PATH>` — Path to VS Code `chatLanguageModels.json` (overrides config)
* `--goose-config <PATH>` — Path to Goose CLI `config.yaml` (overrides config)
* `--dry-run` [alias: `what-if`] — Preview changes without writing any files
* `--no-color` — Disable color in dry-run output
* `--prune` — Remove stale managed models from target configurations
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli tui`

Launch the interactive terminal user interface (TUI)

    $> acorn tui

**Usage:** `acorn-cli tui`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
