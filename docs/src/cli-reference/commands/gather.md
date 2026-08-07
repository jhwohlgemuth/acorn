# 🌾 Gather

> Discover persistent identifiers in files, documents, URLs, standard input, and literal text.

## Directory traversal

Directory inputs are searched recursively by default, including files in every descendant directory. When no input, `--text`, or standard input is supplied, `gather` recursively searches the current directory.

Use `--max-depth` to limit traversal. Direct children of an input directory have depth 1, grandchildren have depth 2, and so on. A depth of 0 processes no directory descendants but does not exclude explicitly supplied files.

```sh
# Search the complete directory tree
acorn gather path/to/project

# Search only files directly inside the directory
acorn gather path/to/project --max-depth 1

# Include one nested directory level
acorn gather path/to/project --max-depth 2
```

`--filter` and `--ignore` are applied to the paths found within the selected depth.

## Other inputs

```sh
acorn gather report.docx
acorn gather --text "doi:10.1234/example"
printf 'https://ror.org/01qz5mb56' | acorn gather
```

Non-empty standard input is gathered automatically and can be combined with positional inputs. Remote URLs require network access. Global `--offline` mode rejects remote input and cannot be combined with `--resolve`.
