# Command Line Reference

- [Configuration](configuration.md)
- [Commands](commands/index.md)

```plaintext

   █████████     █████████     ███████    ███████████   ██████   █████
  ███▒▒▒▒▒███   ███▒▒▒▒▒███  ███▒▒▒▒▒███ ▒▒███▒▒▒▒▒███ ▒▒██████ ▒▒███
 ▒███    ▒███  ███     ▒▒▒  ███     ▒▒███ ▒███    ▒███  ▒███▒███ ▒███
 ▒███████████ ▒███         ▒███      ▒███ ▒██████████   ▒███▒▒███▒███
 ▒███▒▒▒▒▒███ ▒███         ▒███      ▒███ ▒███▒▒▒▒▒███  ▒███ ▒▒██████⠀
 ▒███    ▒███ ▒▒███     ███▒▒███     ███  ▒███    ▒███  ▒███  ▒▒█████
 █████   █████ ▒▒█████████  ▒▒▒███████▒   █████   █████ █████  ▒▒█████⠀
▒▒▒▒▒   ▒▒▒▒▒   ▒▒▒▒▒▒▒▒▒     ▒▒▒▒▒▒▒    ▒▒▒▒▒   ▒▒▒▒▒ ▒▒▒▒▒    ▒▒▒▒▒
⠀⠀⠀⠀⠀⠀⠀⠀~ Accessible Content Optimization for Research Needs ~⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀

Usage: acorn [FLAGS] [COMMAND]

COMMANDS:
  check     Perform static analysis on research activity data and apply standardized best practices
  doctor    Diagnose and correct system requirements for using acorn
  download  Download research activity data from buckets
  export    Export research activity data to a specific target
  format    Format research activity data in place (inherently includes some elements of `acorn check`)
  link      Add linked data context to research activity data
  schema    Print research activity data or research activity identifier metadata JSON schema to stdout
  help      Print this message or the help of the given subcommand(s)

FLAGS:
  -X, --offline
          Prevent communication with the internet - intended for disconnected, local environments

          Note: Use of --offline may require extra configuration options for certain commands

  -t, --threads <N>
          Limit number of threads used by Rayon for parallel processing

          See Rayon documentation for more information

          [default: 0]

  -v, --verbose...
          Increase logging verbosity

  -q, --quiet...
          Decrease logging verbosity

  -V, --version
          Print version information

  -h, --help
          Print help (see a summary with '-h')

```