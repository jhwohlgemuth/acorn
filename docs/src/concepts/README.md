# Concepts

These pages describe the data model and design ideas behind ACORN. Start with [Why ACORN?](../motivation.md) for the problem the project addresses, then use this section as a reference for its schemas, identifiers, and technology classifications.

## Research enablement
ACORN is managed by the Research Enablement Initiative (REI) at Oak Ridge National Laboratory (ORNL). REI aims to improve the way scientific research is conducted, shared, and evaluated. It is a team of developers, communication experts, and information scientists passionate about open science, transparency, and improving the researcher experience.

REI provides:

- A cross-domain model of research activity data (<span title="Research Activity Data">RAD</span>)
    - [ACORN](acorn.md): schemas, controlled vocabularies, and ontologies for describing research activities
    - [ASPECT](aspect/): classifications for technology associated with a research activity
- A Rust command-line application, [`acorn-cli`](https://crates.io/crates/acorn-cli)
- Rust libraries for schemas and RAD workflows, [`acorn-schema`](https://crates.io/crates/acorn-schema) and [`acorn-lib`](https://docs.rs/acorn-lib/latest/acorn/)
- An independently maintained Python package, [`acorn-py`](https://code.ornl.gov/research-enablement/acorn-py)
- The ORNL research activity catalog, [research.ornl.gov](https://research.ornl.gov) (currently in preview; contact REI for access)

### Buckets
RAD lives in versioned folder collections called "buckets". Each bucket contains a set of files and media assets that describe research activities. For multiple examples, see [ORNL's buckets](https://code.ornl.gov/research-enablement/buckets).

The ACORN CLI combines buckets through a flat-file[^flat] [configuration file](/cli-reference/configuration.html). This lets users aggregate multiple sources and generate reports from the combined data.

Buckets can be stored in a version-controlled GitLab or GitHub repository or on a local file system. They do not require a cloud provider.

Organizations can combine buckets across teams while retaining control over each repository's access and permissions.

> [!TIP]
> See the [research enablement wiki](https://code.ornl.gov/groups/research-enablement/-/wikis/Buckets) for more information on buckets.
[^flat]: A flat-file is a simple text file that contains data in a structured format, such as [JSON](https://www.json.org/json-en.html) or [YAML](https://yaml.org/). Flat-files are easy to read and write, and can be used to store configuration data for applications.
