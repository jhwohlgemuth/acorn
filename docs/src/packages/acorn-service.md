# Application service

`acorn-service` exposes three transport-neutral operations over supplied content: Research Activity validation, readability checking, and deterministic repository processing. It also owns the typed JSON-RPC registry used by `acorn-server`.

```bash
cargo add acorn-service
```

Requests never name local files to open. Repository inputs carry lexical relative paths and content, and processing returns ordered reports and proposed changes without writing a working tree. Use `acorn-server` when these same operations need authenticated JSON-RPC or typed gRPC transport.
