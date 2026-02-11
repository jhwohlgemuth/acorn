# Command-Line Help for `acorn-cli`

This document contains the help content for the `acorn-cli` command-line program.

**Command Overview:**

* [`acorn-cli`↴](#acorn-cli)
* [`acorn-cli check`↴](#acorn-cli-check)
* [`acorn-cli doctor`↴](#acorn-cli-doctor)
* [`acorn-cli download`↴](#acorn-cli-download)
* [`acorn-cli export`↴](#acorn-cli-export)
* [`acorn-cli format`↴](#acorn-cli-format)
* [`acorn-cli gather`↴](#acorn-cli-gather)
* [`acorn-cli link`↴](#acorn-cli-link)
* [`acorn-cli schema`↴](#acorn-cli-schema)
* [`acorn-cli schema rad`↴](#acorn-cli-schema-rad)
* [`acorn-cli schema raid`↴](#acorn-cli-schema-raid)

## `acorn-cli`

"Plant an ACORN and grow your science"

<code><pre>
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
⠀ ⣿⣿⠿⠿⠿⠿⠿⠿⠟⠛⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀~ Accessible Content Optimization for Research Needs ~⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
</pre></code>

**Usage:** `acorn [FLAGS] [COMMAND]`

###### **Subcommands:**

* `check` — Perform static analysis on research activity data and apply standardized best practices
* `doctor` — Diagnose and correct system requirements for using acorn
* `download` — Download research activity data from buckets
* `export` — Export research activity data to a specific target
* `format` — Formats research activity data in place (inherently includes some elements of `acorn check`)
* `gather` — Gather (extract) research activity data
* `link` — Add linked data context to research activity data
* `schema` — Print research activity data (RAD) or research activity identifier (RAiD) metadata JSON schema to stdout

###### **Options:**

* `-X`, `--offline` — Prevent communication with the internet - intended for disconnected local environments

   Note: Use of --offline may require extra configuration options for certain commands
* `-t`, `--threads <N>` — Limit number of threads used by rayon for parallel processing

   See Rayon documentation for more information

  Default value: `0`
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity
* `-V`, `--version` — Prints version information
* `-h`, `--help` — Print help (see a summary with '-h')

  Possible values: `true`, `false`




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

**Usage:** `acorn-cli acorn check [OPTIONS] [FLAGS] <PATH>`

###### **Arguments:**

* `<PATH>` — Path to check

  Default value: `./`

###### **Options:**

* `-b`, `--branch <BRANCH>` — Check files that were changed in a given Git branch
* `-c`, `--commit <COMMIT>` — Check files that were changed in a given Git commit or commit range

   Check files that were changed in the last 10 commits &> acorn check --commit HEAD~10..HEAD

   See <https://git-scm.com/book/en/v2/Git-Tools-Revision-Selection> for more information of commit ranges
* `-d`, `--disable-website-checks` — Disable website checks that require internet connection
* `-e`, `--exit-on-first-error` — Exit on first error
* `-i`, `--ignore <REGEX>` — Regex pattern applied to absolute paths of files that determines whether they should be included in checking process

   Only applies to `--path` values that point to a directory

   Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals

   Example: --ignore "[/]valid.json$"
* `-m`, `--merge-request` — Format files that were changed in a given merge request (Gitlab) or pull request (GitHub)
* `-r`, `--readability-test <METRIC>` — Readability test to use

  Default value: `fkgl`

  Possible values:
  - `fkgl`:
    Flesch-Kincaid Grade Level (FKGL)
  - `ari`:
    Automated Readability Index (ARI)
  - `cli`:
    Coleman-Liau Index (CLI)
  - `fres`:
    Flesch Reading Ease (FRES)
  - `gfi`:
    Gunning Fog Index (GFI)
  - `lix`:
    Lix (abbreviation of Swedish läsbarhetsindex)
  - `smog`:
    SMOG Index (SMOG)

* `-s`, `--skip <LIST>` — Skip one or more available quality check categories

  Possible values: `conventions`, `prose`, `readability`, `schema`

* `--skip-verify-checksum` — Skip checksum verification when downloading files (e.g., downloading Vale to perform static analysis of prose) > CAUTION: Use at your own risk
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

Download research activity data from buckets

    Download research activity data from a bucket repository URL
    $> acorn download https://github.com/user/repo

    Download research activity data from a list of buckets
    $> acorn download --config /path/to/.acorn.json

    Download research activity data to a specific output directory
    $> acorn download --config /path/to/.acorn.yml --output /path/to/output

**Usage:** `acorn-cli download [OPTIONS] [URL]`

###### **Arguments:**

* `<URL>` — Bucket repository URL

###### **Options:**

* `-c`, `--config <PATH>` — Path to configuration file (alternative to URL argument)
* `-o`, `--output <PATH>` — Path to output directory where the downloaded content should be saved

  Default value: `./content`
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli export`

Export research activity data to a specific target

    $> acorn export /path/to/data

    $> acorn export /path/to/data/data.json -vv

    $> acorn export /path/to/data --target highlight --format powerpoint

    $> acorn export /path/to/data -t poster -o /path/to/output

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
* `--combine` — Combine all export artifacts into a single artifact
* `-c`, `--commit <COMMIT>` — Export files that were changed in a given Git commit
* `-f`, `--format <FORMAT>` — Export target file format

  Default value: `PDF`

  Possible values: `pdf`, `bag`, `json`, `markdown`, `powerpoint`, `yaml`

* `-i`, `--ignore <REGEX>` — Regex pattern applied to absolute paths of files that determines whether they should be included in export process

   Only applies to `--path` values that point to a directory

   Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals

   Example: --ignore "[/]valid.json$"
* `-m`, `--merge-request` — Export files that were changed in a given merge request (Gitlab) or pull request (GitHub)
* `-o`, `--output <DIRECTORY>` — Path to output directory where the export artifacts should be saved

  Default value: `./export`
* `-r`, `--reference <PATH>` — Path to reference file to be used when exporting to PowerPoint

   Can be relative or absolute. Relative paths will be resolved against the directory of the target file.
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

###### **Arguments:**

* `<PATH>` — Path to look for files to format

  Default value: `./`

###### **Options:**

* `-b`, `--branch <BRANCH>` — Format files that were changed in a given Git branch
* `-c`, `--commit <COMMIT>` — Format files that were changed in a given Git commit
* `-d`, `--dry-run` — Run format without making changes to target file(s). Will print a diff of changes
* `-i`, `--ignore <REGEX>` — Regex pattern applied to absolute paths of files that determines whether they should be included in formatting process

   Only applies to path values that point to a directory

   Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals

   Example: --ignore "[/]valid.json$"
* `-m`, `--merge-request` — Format files that were changed in a given merge request (Gitlab) or pull request (GitHub)
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli gather`

Gather (extract) research activity data

    $> acorn gather /path/to/data/

**Usage:** `acorn-cli gather [OPTIONS] [PATH]`

###### **Arguments:**

* `<PATH>` — Path to look for files to process

  Default value: `./`

###### **Options:**

* `-i`, `--ignore <REGEX>` — Regex pattern applied to absolute paths of files that determines whether they should be included in processing

   Only applies to path values that point to a directory

   Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals

   Example: --ignore "[/]valid.json$"
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
* `-d`, `--dry-run` — Run link without making changes to target file(s). Will print a diff of changes
* `-i`, `--ignore <REGEX>` — Regex pattern applied to absolute paths of files that determines whether they should be included in processing

   Only applies to path values that point to a directory

   Patterns that contain whitespace or special characters should be enclosed in quotes for most terminals

   Example: --ignore "[/]valid.json$"
* `-m`, `--merge-request` — Processes files that were changed in a given merge request (Gitlab) or pull request (GitHub)
* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `acorn-cli schema`

Print research activity data (RAD) or research activity identifier (RAiD) metadata JSON schema to stdout

**Usage:** `acorn-cli schema [COMMAND]`

###### **Subcommands:**

* `rad` — Print research activity data (RAD) JSON schema to stdout
* `raid` — Print research activity identifier (RAiD) metadata JSON schema to stdout



## `acorn-cli schema rad`

Print research activity data (RAD) JSON schema to stdout

**Usage:** `acorn-cli schema rad`



## `acorn-cli schema raid`

Print research activity identifier (RAiD) metadata JSON schema to stdout

**Usage:** `acorn-cli schema raid`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

