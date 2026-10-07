# Protobuf and gRPC

`acorn-server` exposes the canonical `acorn-service` operations through an explicit Protobuf contract. The unversioned `acorn.service.AcornService` contains only three typed RPCs:

- `ValidateResearchActivity`
- `CheckReadability`
- `ProcessRepository`

There is no generic JSON-value invocation envelope, batch RPC, notification RPC, or version RPC. `acorn.version` remains informational package metadata in JSON-RPC and MCP only.

The committed contract is [`crates/acorn-server/proto/acorn/service/service.proto`](https://code.ornl.gov/research-enablement/acorn/-/blob/main/crates/acorn-server/proto/acorn/service/service.proto). Cargo generates Rust client and server bindings during the build. `acorn_server::grpc::GrpcClient` wraps those bindings with canonical `acorn-service` request and response types, bearer metadata, deadlines, endpoint policy, and message bounds. `acorn_server::client::JsonRpcClient` provides the equivalent typed calls plus generic calls, notifications, and ordered mixed batches.

## Run the server

Set a dedicated bearer token and start both loopback listeners:

```bash
export ACORN_SERVICE_TOKEN='replace-with-a-secret'
cargo run -p acorn-server
```

JSON-RPC listens on `127.0.0.1:8001`; gRPC listens on `127.0.0.1:50051`. Either listener can be disabled explicitly. Bind, message-size, timeout, shutdown, mutation, and TLS settings use CLI-over-environment-over-default precedence; see `acorn-server --help`.

Application calls require `authorization: Bearer …`. Non-loopback listeners require a certificate and private key. The default request/message and output limit is 1 MiB, and the default application timeout and shutdown grace are 30 seconds.

## Protobuf checks

```bash
make protolint
cargo test -p acorn-server
```

Buf keeps normal field, enum, RPC message, and breaking checks. The configuration narrowly permits the intentionally unversioned package and `AcornService` name because this pre-release contract does not introduce protocol negotiation or a version namespace.
