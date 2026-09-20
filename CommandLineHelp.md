# Command-Line Help for `acorn-cli`

This document contains the help content for the `acorn-cli` command-line program.

**Command Overview:**

* [`acorn-cli`↴](#acorn-cli)
* [`acorn-cli check`↴](#acorn-cli-check)
* [`acorn-cli create`↴](#acorn-cli-create)
* [`acorn-cli create mcp`↴](#acorn-cli-create-mcp)
* [`acorn-cli create runner`↴](#acorn-cli-create-runner)
* [`acorn-cli create bot`↴](#acorn-cli-create-bot)
* [`acorn-cli create secret`↴](#acorn-cli-create-secret)
* [`acorn-cli doctor`↴](#acorn-cli-doctor)
* [`acorn-cli download`↴](#acorn-cli-download)
* [`acorn-cli download model`↴](#acorn-cli-download-model)
* [`acorn-cli download needle`↴](#acorn-cli-download-needle)
* [`acorn-cli export`↴](#acorn-cli-export)
* [`acorn-cli infer`↴](#acorn-cli-infer)
* [`acorn-cli format`↴](#acorn-cli-format)
* [`acorn-cli gather`↴](#acorn-cli-gather)
* [`acorn-cli import`↴](#acorn-cli-import)
* [`acorn-cli import model`↴](#acorn-cli-import-model)
* [`acorn-cli import spec`↴](#acorn-cli-import-spec)
* [`acorn-cli link`↴](#acorn-cli-link)
* [`acorn-cli report`↴](#acorn-cli-report)
* [`acorn-cli swhid`↴](#acorn-cli-swhid)
* [`acorn-cli skill`↴](#acorn-cli-skill)
* [`acorn-cli serve`↴](#acorn-cli-serve)
* [`acorn-cli serve bot`↴](#acorn-cli-serve-bot)
* [`acorn-cli serve grpc`↴](#acorn-cli-serve-grpc)
* [`acorn-cli serve mcp`↴](#acorn-cli-serve-mcp)
* [`acorn-cli serve rpc`↴](#acorn-cli-serve-rpc)
* [`acorn-cli sync`↴](#acorn-cli-sync)
* [`acorn-cli tui`↴](#acorn-cli-tui)
* [`acorn-cli exec`↴](#acorn-cli-exec)
* [`acorn-cli exec check`↴](#acorn-cli-exec-check)
* [`acorn-cli exec check-safe`↴](#acorn-cli-exec-check-safe)
* [`acorn-cli exec hook`↴](#acorn-cli-exec-hook)
* [`acorn-cli exec hook check`↴](#acorn-cli-exec-hook-check)
* [`acorn-cli exec manifest`↴](#acorn-cli-exec-manifest)
* [`acorn-cli exec manifest check`↴](#acorn-cli-exec-manifest-check)

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
* `infer` — Run bounded local tool inference through the managed Needle sidecar
* `format` — Formats supported metadata standards in place (inherently includes some elements of `acorn check`)
* `gather` — Gather persistent identifiers from documents, URLs, or text
* `import` — Import external metadata into ACORN
* `link` — Add linked data context to research activity data
* `report` — Generate bounded artifacts and publish immutable GitLab packages
* `swhid` — Calculate or verify a Software Hash Identifier (SWHID)
* `skill` — Print the embedded ACORN skill path and copy agent instructions to clipboard
* `serve` — Start a server (bot, MCP, etc.)
* `sync` — Synchronize local inference configuration from ACORN model entries
* `tui` — Launch the interactive terminal user interface (TUI)
* `exec` — Execute constrained Rhai extensions

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
* `-w`, `--watch` — Re-run check or gather when input content changes
* `--watch-mode <MODE>` — Select automatic, native, or polling watch behavior

  Default value: `auto`

  Possible values:
  - `auto`:
    Prefer native notifications and use polling when required
  - `native`:
    Require native filesystem notifications
  - `poll`:
    Poll local and network sources

* `-p`, `--poll-interval <DURATION>` — Set the interval used by watch polling and Vale sync retries
* `-t`, `--threads <N>` — Limit concurrent work used for parallel processing

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

    Check text provided through standard input
    $> echo 'Text to check' | acorn check

    Check text copied to the clipboard
    $> acorn check --paste

    Check YAML copied to the clipboard as RAD
    $> acorn check --paste --format yaml --standard rads

**Usage:** `acorn-cli check [OPTIONS] [LOCATION]`

**Command Alias:** `lint`

###### **Arguments:**

* `<LOCATION>` — Local path or HTTP(S) URI to check; piped standard input is used when omitted

###### **Options:**

* `--paste` — Check text copied to the system clipboard
* `-b`, `--branch <BRANCH>` — Check files that were changed in a given Git branch
* `-c`, `--commit <COMMIT>` — Check files that were changed in a given Git commit or commit range

   Check files that were changed in the last 10 commits &> acorn check --commit HEAD~10..HEAD

   See <https://git-scm.com/book/en/v2/Git-Tools-Revision-Selection> for more information of commit ranges
* `--config <PATH>` — Path to application configuration containing Rhai quality gates
* `-d`, `--disable-website-checks` — Disable website checks that require internet connection
* `--all` — Include all check results, including suggestions and info messages
* `-e`, `--exit-on-first-error` — Exit on first error
* `--ignore-sync-failure` — Continue watch mode when the prose analyzer cannot be synchronized

   Use `--skip prose` to skip prose analyzer synchronization and analysis entirely
* `-p`, `--poll-interval <DURATION>` — Set the interval used by watch polling and analyzer sync retries
* `--filter <LIST>` — Regular expression pattern(s) applied to absolute paths of files to include during checks

   Only files matching at least one pattern will be processed
* `--format <FORMAT>` — Force the file format used for standard input or clipboard text

   Supported formats: cff, json, jsonc, markdown, text, yaml
* `-i`, `--ignore <LIST>` — Regular expression pattern(s) applied to absolute paths of files to exclude from checking

   Only applies to `--path` values that point to a directory

   Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals

   Example: --ignore "[/]valid.json$,[/]draft.json$"
* `-m`, `--merge-request` — Format files that were changed in a given merge request (Gitlab) or pull request (GitHub)
* `--no-fail` — Prevent `acorn check` from exiting with a non-zero exit code when errors are found

   Useful for certain scenarios such as debugging

   > CAUTION: Use at your own risk
* `--raw` — Suppress output from external commands (e.g., Vale) while preserving ACORN output — useful for piping
* `-S`, `--skip <LIST>` — Skip one or more available check categories
* `--skip-verify-checksum` — Skip checksum verification when downloading files (e.g., downloading Vale to perform static analysis of prose) > CAUTION: Use at your own risk
* `-s`, `--standard <STANDARD>` — Metadata standard to validate against

   Supported standards: rads, cff, datacite, dcat, dcmi, document, invenio, huwise, kitfile, modelkit, raid, text (`docx` and `pdf` alias `document`)

  Default value: `rads`
* `-r`, `--readability-test <METRIC>` — Readability test to use

  Default value: `fkgl`
* `-t`, `--terse` — Use compact output format instead of detailed output
* `--vale-directory <PATH>` — Existing Vale installation directory containing the executable and `.vale.ini`
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli create`

Create research resources like GitLab runners, MCP servers, etc.

    $> acorn create runner --config ./acorn.json

    $> acorn create runner --group 12345

    Generate a secret and copy it to the system clipboard
    $> acorn create secret --clipboard

**Usage:** `acorn-cli create [COMMAND]`

###### **Subcommands:**

* `mcp` — Model context provider (MCP) server
* `runner` — GitLab runner
* `bot` — Bot server container
* `secret` — Cryptographically secure 256-bit secret



## `acorn-cli create mcp`

Model context provider (MCP) server

**Usage:** `acorn-cli create mcp [OPTIONS]`

###### **Options:**

* `--output <PATH>` — MCPB destination

  Default value: `acorn.mcpb`
* `--binary <PATH>` — Executable to package; defaults to the running ACORN executable
* `--platform <PLATFORM>` — Target platform; required when packaging another executable
* `--kitfile <PATH>` — Authoring Kitfile to update after creating the bundle
* `--clobber` — Replace an existing regular bundle
* `--dry-run` [alias: `what-if`] — Validate and print the plan without writing files



## `acorn-cli create runner`

GitLab runner

**Usage:** `acorn-cli create runner [OPTIONS]`

###### **Options:**

* `-c`, `--config <PATH>` — Path to configuration file for creating the resource

  Default value: `./`
* `--description <DESCRIPTION>` — Runner description (alternative to providing a configuration file)
* `-g`, `--group <ID>` — GitLab group ID to create a runner for (alternative to providing a configuration file)
* `-r`, `--repo <REPO>` — GitLab repository domain override for runner creation (e.g., code.ornl.gov)
* `-p`, `--project <ID>` — GitLab project ID to create a runner for (alternative to providing a configuration file)
* `-n`, `--name <NAME>` — Runner name (used as Docker container name)
* `-s`, `--server <SERVER>` — Hosting server location (for use during registration)
* `-t`, `--tags <LIST>` — Tags to register the runner with (alternative to providing a configuration file)
* `-u`, `--untagged` — Whether this runner can pick up jobs without matching tags (alternative to providing a configuration file)
* `-R`, `--runtime <RUNTIME>` — Container runtime override (docker, podman, apptainer). Defaults to the executor's runtime
* `--host <URI|ALIAS>` — SSH URI or alias of the remote host (e.g. ssh://user@host, my-alias).

   The local client and SSH configuration are used. Published ports and volumes belong to the remote host.
* `--context <NAME>` — Docker context / Podman connection name (from `docker context ls` / `podman system connection ls`)
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
* `--config <PATH>` — Application configuration containing deployment-owned workflow policy
* `-n`, `--name <NAME>` — Container name (defaults to `acorn-bot-{identifier}`)
* `-i`, `--image <IMAGE>` — Acorn Docker image to use for the bot container

  Default value: `acorn:latest`
* `-D`, `--domain <DOMAIN>` — GitLab domain override (e.g., code.ornl.gov)
* `--volume <VOLUME>` — Persistent named volume for bot state
* `--host <URI|ALIAS>` — SSH URI or alias of the remote host (e.g. ssh://user@host, my-alias).

   The local client and SSH configuration are used. Published ports and volumes belong to the remote host.
* `--context <NAME>` — Docker context / Podman connection name (from `docker context ls` / `podman system connection ls`)
* `-d`, `--detach` — Run as detached container/service instead of foreground/in-process
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli create secret`

Cryptographically secure 256-bit secret

**Usage:** `acorn-cli create secret [OPTIONS]`

###### **Options:**

* `--clipboard` — Copy the generated value to the system clipboard instead of standard output



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

    Replace selected files that already exist in the output directory
    $> acorn download https://github.com/user/repo --output ./content --clobber

    Download using default configuration in current directory (first match):
    .acorn.json, .acorn.jsonc, .acorn.yaml, .acorn.yml
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
* `needle` — Download and verify the pinned standalone Needle engine for this platform

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
* `-f`, `--flatten` — Save downloaded files directly beneath the output directory
* `-C`, `--clobber` — Replace existing files or directories at selected output paths
* `--strict` — Reject deprecated research activity fields during database ingestion
* `--canonicalize` — Rewrite deprecated research activity fields before database ingestion
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli download model`

Download model weights for use with ACORN research harnesses and local inference

**Usage:** `acorn-cli download model [OPTIONS] [MODEL]...`

###### **Arguments:**

* `<MODEL>` — Model ID/name(s), local path(s), direct URL(s), or explicit oci:// artifact reference(s)

###### **Options:**

* `--model-file <URI_OR_PATH>` — URI or local path to a model list file
* `-c`, `--config <PATH>` — Path to an application configuration file listing models
* `--no-fallback` — Disable automatic GGUF fallback repository discovery
* `--interactive` — Open an interactive picker when fallback finds multiple repositories
* `--search-limit <LIMIT>` — Maximum number of Hugging Face repositories to search

  Default value: `20`
* `--minimum-popularity <COUNT>` — Minimum downloads required for a GGUF fallback repository

  Default value: `100`
* `--sync <TARGET>` — Synchronize resolved models with configured inference applications

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
* `--dry-run` [alias: `what-if`] — Preview resolution and synchronization without writes
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
* `--raw` — Output structured JSON (or YAML with --format yaml) of resolved models
* `--format <FORMAT>` — Output format for --raw results

  Default value: `json`

  Possible values: `json`, `yaml`

* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli download needle`

Download and verify the pinned standalone Needle engine for this platform

**Usage:** `acorn-cli download needle`



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

  Possible values: `pdf`, `bag`, `cff`, `docx`, `json`, `markdown`, `powerpoint`, `yaml`

* `--archive-format <FORMAT>` — BagIt archive container format

  Possible values:
  - `7z`:
    7-Zip archive
  - `tar`:
    Uncompressed TAR archive
  - `tar.gz`:
    Gzip-compressed TAR archive
  - `zip`:
    ZIP archive

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
* `--tools` — Export the canonical Needle-compatible ACORN tool catalog as tools.json
* `--raw` — Suppress output from external commands while preserving ACORN output — useful for piping
* `-S`, `--skip <LIST>` — Skip one or more available export check categories
* `-r`, `--reference <PATH>` — Path to reference file to be used when exporting to PowerPoint

   Can be relative or absolute. Relative paths will be resolved against the directory of the target file.
* `-s`, `--standard <STANDARD>` — Metadata standard to validate against

   Supported standards: rads, cff, datacite, dcat, dcmi, document, invenio, huwise, kitfile, modelkit, raid, text (`docx` and `pdf` alias `document`)

  Default value: `rads`
* `--strict` — Fail if conversion loses data (only for crosswalk exports)
* `--target <TARGET>` — Artifact target to export

  Default value: `fact-sheet`

  Possible values:
  - `fact-sheet`:
    US letter sized single page PDF document presenting a certain research activity data
  - `highlight`:
    Single slide PowerPoint presentation for a certain research activity data
  - `poster`:
    Poster sized presentation format intended for large printing and presentation
  - `updates`:
    Standalone, since-filtered research activity logbook updates

* `--since <TIMESTAMP>` — Inclusive RFC 3339 timestamp for updates exports
* `--from <STANDARD>` — Metadata standard to use for source data inference (default: auto-detect)

   Supported standards: rads, cff, datacite, dcat, dcmi, document, invenio, huwise, kitfile, modelkit, raid, text (`docx` and `pdf` alias `document`)
* `--to <STANDARD>` — Target metadata standard for crosswalk conversion

   Supported standards: rads, cff, datacite, dcat, dcmi, document, invenio, huwise, kitfile, modelkit, raid, text (`docx` and `pdf` alias `document`)
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli infer`

Run bounded local tool inference through the managed Needle sidecar

    Use the pinned official runner
    $> acorn infer "Validate this research activity"

    Use a local or private-registry ModelKit
    $> acorn infer "Validate this" --custom-location oci://harbor.example/acorn/needle@sha256:...

**Usage:** `acorn-cli infer [OPTIONS] <PROMPT>`

**Command Alias:** `ask`

###### **Arguments:**

* `<PROMPT>` — Prompt used to select and invoke ACORN tools

###### **Options:**

* `--custom-location <LOCATION>` — ModelKit directory, file:// URI, HTTP(S) manifest URI, or digest-pinned oci:// reference
* `--threshold <0..1>` — Minimum calibrated confidence required before executing a tool

  Default value: `0.8`
* `--max-rounds <N>` — Maximum number of Needle tool-selection rounds

  Default value: `8`
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli format`

Formats supported metadata standards in place (inherently includes some elements of `acorn check`)

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
* `--config <PATH>` — Path to application configuration containing Rhai quality gates
* `-d`, `--dry-run` [alias: `what-if`] — Run format without making changes to target file(s). Will print a diff of changes
* `--enrich` — Enrich metadata after formatting it
* `--no-color` — Disable color in dry-run output
* `--provider <PROVIDER>` — Enrichment provider(s); may be repeated or comma-delimited
* `--filter <REGEX>` — Regex pattern applied to absolute paths of files to include in formatting process

   Only files matching the pattern will be processed
* `-i`, `--ignore <REGEX>` — Regex pattern applied to absolute paths of files that determines whether they should be included in formatting process

   Only applies to path values that point to a directory

   Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals

   Example: --ignore "[/]valid.json$"
* `-m`, `--merge-request` — Format files that were changed in a given merge request (Gitlab) or pull request (GitHub)
* `-s`, `--standard <STANDARD>` — Metadata standard to validate against

   Supported standards: rads, cff, datacite, dcat, dcmi, document, invenio, huwise, kitfile, modelkit, raid, text (`docx` and `pdf` alias `document`)

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

* `-p`, `--poll-interval <DURATION>` — Set the interval used by watch polling
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
    [Ames National Laboratory](https://www.ameslab.gov/)
  - `anl`:
    [Argonne National Laboratory](https://www.anl.gov/)
  - `bnl`:
    [Brookhaven National Laboratory](https://www.bnl.gov/)
  - `fnal`:
    [Fermi National Accelerator Laboratory](https://www.fnal.gov/)
  - `inl`:
    [Idaho National Laboratory](https://inl.gov/)
  - `lbnl`:
    [Lawrence Berkeley National Laboratory](https://www.lbl.gov/)
  - `llnl`:
    [Lawrence Livermore National Laboratory](https://www.llnl.gov/)
  - `lanl`:
    [Los Alamos National Laboratory](https://www.lanl.gov/)
  - `netl`:
    [National Energy Technology Laboratory](https://www.netl.doe.gov/)
  - `nrel`:
    [National Renewable Energy Laboratory](https://www.nrel.gov/)
  - `ornl`:
    [Oak Ridge National Laboratory](https://www.ornl.gov/)
  - `pnnl`:
    [Pacific Northwest National Laboratory](https://www.pnnl.gov/)
  - `pppl`:
    [Princeton Plasma Physics Laboratory](https://www.pppl.gov/)
  - `snl`:
    [Sandia National Laboratories](https://www.sandia.gov/)
  - `srnl`:
    [Savannah River National Laboratory](https://www.srnl.gov/)
  - `slac`:
    [SLAC National Accelerator Laboratory](https://www.slac.stanford.edu/)
  - `tjnaf`:
    [Thomas Jefferson National Accelerator Facility](https://www.jlab.org/)

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

* `--raw` — Print identifiers as plain text, or enriched RAD candidates as JSON lines with `--enrich`
* `--enrich` — Enrich gathered research activity data with all applicable providers
* `-t`, `--terse` — Use compact check output and suppress the discovery table
* `--resolve` — Resolve supported identifiers through their metadata providers
* `--citation-format <FORMAT>` — Citation format used for resolved DOI lines
* `-s`, `--standard <STANDARD>` — Metadata standard to validate against
* `--text <TEXT>` — Literal text to inspect; may be repeated
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli import`

Import external metadata into ACORN

**Usage:** `acorn-cli import [OPTIONS] [URI]... [COMMAND]`

**Command Alias:** `load`

###### **Subcommands:**

* `model` — Import model metadata into the local database
* `spec` — Import endpoint resources from an API specification

###### **Arguments:**

* `<URI>` — Local directories, repository URIs, or BagIt archive URIs

###### **Options:**

* `-c`, `--config <PATH>` — Path to configuration file (JSON, JSONC, or YAML)
* `--filter <LIST>` — Regular expression patterns used to include files
* `-i`, `--ignore <LIST>` — Regular expression patterns used to exclude files
* `-o`, `--output <PATH>` — Directory where imported content is saved

  Default value: `./content`
* `-f`, `--flatten` — Save imported files directly beneath the output directory
* `-C`, `--clobber` — Replace existing files or directories at selected output paths
* `--archive-format <FORMAT>` — Force archive handling and bypass content inference

  Possible values:
  - `7z`:
    7-Zip archive
  - `tar`:
    Uncompressed TAR archive
  - `tar.gz`:
    Gzip-compressed TAR archive
  - `zip`:
    ZIP archive

* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli import model`

Import model metadata into the local database

**Usage:** `acorn-cli import model [OPTIONS] [MODEL]...`

###### **Arguments:**

* `<MODEL>` — Model ID/name(s), local path(s), direct URL(s), or explicit oci:// artifact reference(s)

###### **Options:**

* `--model-file <URI_OR_PATH>` — URI or local path to a model list file
* `-c`, `--config <PATH>` — Path to an application configuration file listing models
* `--no-fallback` — Disable automatic GGUF fallback repository discovery
* `--interactive` — Open an interactive picker when fallback finds multiple repositories
* `--search-limit <LIMIT>` — Maximum number of Hugging Face repositories to search

  Default value: `20`
* `--minimum-popularity <COUNT>` — Minimum downloads required for a GGUF fallback repository

  Default value: `100`
* `--sync <TARGET>` — Synchronize resolved models with configured inference applications

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
* `--dry-run` [alias: `what-if`] — Preview resolution and synchronization without writes
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
* `--config <PATH>` — Path to application configuration containing Rhai quality gates
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



## `acorn-cli report`

Generate bounded artifacts and publish immutable GitLab packages

    $> acorn report --through 2026-10-01T00:00:00Z --commit "$CI_COMMIT_SHA"

**Usage:** `acorn-cli report [OPTIONS] --through <TIMESTAMP>`

###### **Options:**

* `-c`, `--config <PATH>` — Application configuration containing PowerAutomate project mappings
* `--commit <SHA>` — Exact merged default-branch commit used as report input
* `--from <TIMESTAMP>` — Explicit inclusive lower bound for local or recovery runs
* `--through <TIMESTAMP>` — Fixed exclusive reporting upper bound shared by every project
* `--root <PATH>` — Repository checkout root containing configured project paths

  Default value: `.`
* `-o`, `--output <PATH>` — Directory for generated reports and failure diagnostics

  Default value: `.acorn/reports`
* `--project <ID>` — Restrict reporting to selected configured project identifiers
* `--pipeline-id <ID>` — GitLab pipeline identifier recorded in the manifest
* `--job-id <ID>` — GitLab job identifier recorded in the manifest
* `--chrome-path <PATH>` — Path to Chromium executable used for PDF rendering
* `--dry-run` [alias: `what-if`] — Generate local artifacts and manifests without GitLab publication or callbacks
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli swhid`

Calculate or verify a Software Hash Identifier (SWHID)

    Hash a local file or directory
    $> acorn swhid ./src

    Calculate the current Git revision
    $> acorn swhid . --kind revision --reference HEAD

    Verify a local object
    $> acorn swhid ./src --verify swh:1:dir:0123456789abcdef0123456789abcdef01234567

**Usage:** `acorn-cli swhid [OPTIONS] [PATH]`

###### **Arguments:**

* `<PATH>` — Local file, directory, or Git repository

  Default value: `.`

###### **Options:**

* `--kind <KIND>` — Object source to calculate

  Default value: `auto`

  Possible values:
  - `auto`:
    Infer a local file or directory
  - `content`:
    Local file content
  - `directory`:
    Local directory traversal
  - `git-blob`:
    Git blob selected by REFERENCE
  - `git-tree`:
    Git tree selected by REFERENCE
  - `revision`:
    Git commit selected by REFERENCE
  - `release`:
    Annotated Git tag named by REFERENCE
  - `snapshot`:
    Local Git branches and tags

* `-r`, `--reference <REFERENCE>` — Git revision expression or annotated tag name
* `--verify <SWHID>` — Verify the calculated core object against this SWHID
* `--raw` — Print only the calculated identifier



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
* `grpc` — Authenticated Protobuf/gRPC server
* `mcp` — Model context protocol (MCP) server
* `rpc` — Authenticated JSON-RPC HTTP server



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
* `--config <PATH>` — Application configuration containing deployment-owned workflow policy
* `-d`, `--detach` — Run as detached container/service instead of foreground/in-process
* `--host <URI|ALIAS>` — SSH URI or alias of the remote host (e.g. ssh://user@host, my-alias).

   The local client and SSH configuration are used. Published ports and volumes belong to the remote host.
* `--context <NAME>` — Docker context / Podman connection name (from `docker context ls` / `podman system connection ls`)



## `acorn-cli serve grpc`

Authenticated Protobuf/gRPC server

**Usage:** `acorn-cli serve grpc [OPTIONS]`

###### **Options:**

* `--allow-mutation` — Enable registered mutating operations
* `--bind <ADDRESS>` — Address for the authenticated gRPC endpoint

  Default value: `127.0.0.1:50051`
* `--host <URI|ALIAS>` — SSH URI or alias of the remote host (e.g. ssh://user@host, my-alias).

   The local client and SSH configuration are used. Published ports and volumes belong to the remote host.
* `--context <NAME>` — Docker context / Podman connection name (from `docker context ls` / `podman system connection ls`)
* `-d`, `--detach` — Run as detached container/service instead of foreground/in-process
* `-R`, `--runtime <RUNTIME>` — Container runtime for detached execution
* `--tls-cert <PATH>` — PEM-encoded TLS certificate chain
* `--tls-key <PATH>` — PEM-encoded TLS private key



## `acorn-cli serve mcp`

Model context protocol (MCP) server

**Usage:** `acorn-cli serve mcp [OPTIONS]`

###### **Options:**

* `--allow-mutation` — Enable persistent mutating tools
* `--host <URI|ALIAS>` — SSH URI or alias of the remote host (e.g. ssh://user@host, my-alias).

   The local client and SSH configuration are used. Published ports and volumes belong to the remote host.
* `--context <NAME>` — Docker context / Podman connection name (from `docker context ls` / `podman system connection ls`)
* `-d`, `--detach` — Run as detached container/service instead of foreground/in-process
* `-R`, `--runtime <RUNTIME>` — Container runtime for detached execution



## `acorn-cli serve rpc`

Authenticated JSON-RPC HTTP server

**Usage:** `acorn-cli serve rpc [OPTIONS]`

###### **Options:**

* `--allow-mutation` — Enable registered mutating operations
* `--bind <ADDRESS>` — Address for the authenticated JSON-RPC HTTP endpoint

  Default value: `127.0.0.1:8001`
* `--host <URI|ALIAS>` — SSH URI or alias of the remote host (e.g. ssh://user@host, my-alias).

   The local client and SSH configuration are used. Published ports and volumes belong to the remote host.
* `--context <NAME>` — Docker context / Podman connection name (from `docker context ls` / `podman system connection ls`)
* `-d`, `--detach` — Run as detached container/service instead of foreground/in-process
* `-R`, `--runtime <RUNTIME>` — Container runtime for detached execution



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
* `--mcp` — Configure the ACORN MCP server in selected compatible clients
* `--models-dir <PATH>` — Default directory where downloaded model weights live (overrides config)
* `--opencode-config <PATH>` — Path to OpenCode configuration file (overrides config)
* `--llama-swap-config <PATH>` — Path to llama-swap configuration file (overrides config)
* `--vscode-config <PATH>` — Path to VS Code `chatLanguageModels.json` (overrides config)
* `--vscode-mcp-config <PATH>` — Path to VS Code workspace `mcp.json` (overrides default)
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



## `acorn-cli exec`

Execute constrained Rhai extensions

    Evaluate a script against an input file
    $> acorn exec check ./script.rhai ./input.json

    Perform an evaluation-free static check
    $> acorn exec check-safe ./script.rhai

ABI version: `// @acorn-exec-v0.2.0`

The evaluation context contains `hook`, `path`, `index`, `input`, and `proposed`. Scripts return a map containing a `findings` array.
Source is limited to 512 KiB; evaluation is bounded and does not permit dynamic evaluation or imports.

**Usage:** `acorn-cli exec [COMMAND]`

###### **Subcommands:**

* `check` — Evaluate a script against supplied input and render findings
* `check-safe` — Check Rhai source without evaluating it
* `hook` — Validate or evaluate a manifest-authorized workflow hook
* `manifest` — Validate executable-hook deployment manifests



## `acorn-cli exec check`

Evaluate a script against supplied input and render findings

**Usage:** `acorn-cli exec check [OPTIONS] <SCRIPT> <INPUT>`

###### **Arguments:**

* `<SCRIPT>` — Rhai script path
* `<INPUT>` — Input file to evaluate against

###### **Options:**

* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli exec check-safe`

Check Rhai source without evaluating it

**Usage:** `acorn-cli exec check-safe [OPTIONS] <SCRIPT>`

###### **Arguments:**

* `<SCRIPT>` — Rhai script path

###### **Options:**

* `--format <FORMAT>` — Output format (json or markdown)
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli exec hook`

Validate or evaluate a manifest-authorized workflow hook

**Usage:** `acorn-cli exec hook [COMMAND]`

###### **Subcommands:**

* `check` — Evaluate one hook without applying its proposals



## `acorn-cli exec hook check`

Evaluate one hook without applying its proposals

**Usage:** `acorn-cli exec hook check [OPTIONS] --manifest <MANIFEST> --hook <HOOK> --input <FILE> <SCRIPT>`

###### **Arguments:**

* `<SCRIPT>` — Rhai script path

###### **Options:**

* `--manifest <MANIFEST>` — Deployment manifest name or explicit manifest path
* `--hook <HOOK>` — Workflow hook to evaluate
* `--input <FILE>` — Input file to evaluate against
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli exec manifest`

Validate executable-hook deployment manifests

**Usage:** `acorn-cli exec manifest [COMMAND]`

###### **Subcommands:**

* `check` — Parse and validate a script manifest



## `acorn-cli exec manifest check`

Parse and validate a script manifest

**Usage:** `acorn-cli exec manifest check [OPTIONS] <MANIFEST>`

###### **Arguments:**

* `<MANIFEST>` — Script manifest path

###### **Options:**

* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
