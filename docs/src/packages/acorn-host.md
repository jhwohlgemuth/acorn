# Native host adapters

`acorn-host` exposes concrete native capabilities for small downstream tools. It has no default features, and each capability is additive:

| Feature | Capability |
|---|---|
| `fs` | UTF-8 reads, checksums, executable permissions, file URIs, safe relative paths, platform project-cache namespaces, and temporary directories |
| `config` | Caller-owned JSON, strict JSONC, YAML, and YML deserialization plus ordered path discovery |
| `http` | Async clients, exact authority policy, redirect revalidation, bounded probes, and diagnostic redaction |
| `download` | Bounded streaming, optional checksums, partial cleanup, and no-clobber installation |
| `source` | Local, file-URI, and HTTP source reads plus bounded noninteractive Git probes |
| `terminal` | ANSI styling for labels, which remain available as plain text without this feature |

Select only the features a tool uses:

```sh
cargo add acorn-core --no-default-features --features alloc
cargo add acorn-cmd --no-default-features
cargo add acorn-host --no-default-features --features fs,config,http,download,source,terminal
```

HTTP allow-list entries are exact `HOST` or `HOST:PORT` authorities. Empty lists are unrestricted. Original requests and every redirect are checked; credentials, queries, and fragments are removed from diagnostics. `HEAD` probes use a bounded `GET` fallback for servers that reject `HEAD`.

JSONC accepts line and block comments plus trailing commas while retaining JSON's quoted property names and double-quoted strings. `resolve_config_path` returns an explicit path unchanged or searches caller-supplied default names in order. `project_cache_directory` creates and returns a stable `SafePath` namespace beneath the application's platform cache root; callers that need unique children add them themselves.

Downloads write a sibling partial file, enforce optional size and checksum expectations, clean the partial on failure, and atomically create the destination without replacing an existing file. Workflow policy such as offline mode, required checksums, retry strategy, and executable installation remains with the caller.

The runnable fixture under `tests/downstream-host` demonstrates a manifest that depends only on `acorn-core`, `acorn-cmd`, and `acorn-host`, including a renamed `acorn-cmd` dependency.
