---
type: Plan
title: "Add Protobuf and gRPC operation transport"
description: "Expose the shared operation registry through a versioned Protobuf service using tonic."
difficulty: high
impact: medium
package: [acorn-lib, acorn-cli]
tags: [feature, api, rpc, protobuf, grpc, tonic]
status: planned
---

# Project code boundaries

- **Update:** workspace dependencies, `crates/acorn-lib/Cargo.toml`, `src/io/api/mod.rs`, CLI serve parsing/dispatch/tests, generated help, and deployment documentation.
- **Create:** `crates/acorn-lib/proto/acorn/rpc/v1/rpc.proto`, `crates/acorn-lib/build.rs`, `crates/acorn-lib/src/io/api/grpc/{mod,client,server,tests}.rs`, and `crates/acorn-cli/src/commands/serve/grpc/mod.rs`.
- **Delete:** none. Generated Rust belongs in `OUT_DIR` and is not committed.

# Dependencies

Requires the implemented typed operation registry in `io::api::json_rpc`. gRPC is a sibling transport over the operation registry, not a binary JSON-RPC encoding. It is optional for the webhook, GitLab, PowerAutomate, ACP, and MCP plans. Schedule its shared module, manifest, and CLI integration only after the [PowerApps reporting](../../features/powerapps-gitlab-logbook-reporting.md) MVP releases those locks.

# Goal

Provide a generic Protobuf/gRPC server and client for the same allowlisted ACORN operations exposed through JSON-RPC. Use `tonic` and `prost` with a committed, versioned `.proto` contract and build-time generation from a pinned vendored `protoc`.

# Decisions

- Use compatible pinned releases of `tonic`, `tonic-prost`, `prost`, and `prost-types`; use `tonic-prost-build` and `protoc-bin-vendored` as build dependencies. Reverify versions immediately before implementation.
- Add a non-default `grpc` capability. Default, minimal, alloc, and WASM graphs must not resolve gRPC, HTTP/2, or code-generation dependencies.
- Support both server and configured remote client through a generic `Invoke`/`Notify`/`Batch` service.
- Transport/authentication failures use `tonic::Status`; operation failures use the shared RPC error shape.
- Require TLS and a dedicated credential for non-loopback endpoints. Reject remote clients in offline mode.
- Preserve field numbers, reserve removed fields, and version the package as `acorn.rpc.v1`.

# Protobuf contract

```proto
syntax = "proto3";
package acorn.rpc.v1;

import "google/protobuf/empty.proto";
import "google/protobuf/struct.proto";

message RequestId {
  oneof value { int64 number = 1; string text = 2; }
}

message RpcRequest {
  RequestId id = 1;
  string method = 2;
  google.protobuf.Value params = 3;
}

message RpcNotification {
  string method = 1;
  google.protobuf.Value params = 2;
}

message RpcError {
  int32 code = 1;
  string message = 2;
  google.protobuf.Value data = 3;
}

message RpcResponse {
  RequestId id = 1;
  oneof outcome { google.protobuf.Value result = 2; RpcError error = 3; }
}

message BatchRequest { repeated RpcRequest requests = 1; }
message BatchResponse { repeated RpcResponse responses = 1; }

service AcornRpc {
  rpc Invoke(RpcRequest) returns (RpcResponse);
  rpc Notify(RpcNotification) returns (google.protobuf.Empty);
  rpc Batch(BatchRequest) returns (BatchResponse);
}
```

# Plan conflicts

| Plan or lane | Conflict | Resolution |
|---|---|---|
| PowerApps reporting | `io/api/mod.rs`, webhook operation registration, bot/CLI composition, manifests, and generated help | PowerApps lands first; rebase gRPC after its MVP without changing the isolated `io::api::powerautomate` wire contract. |
| Workspace graph | Manifests, lockfile, and new `build.rs` | One dependency owner; add and audit one feature slice. |
| Agentic server | Tonic may select Axum, Tower, Tokio, and TLS versions also used by MCP/ACP | Disable defaults, select only required features, and inspect duplicate trees. |
| CLI composition | `serve grpc` and generated help | Land through the CLI owner after the library server/client pass. |
| Domain schemas | Protobuf transport DTOs could leak into inference/workflow models | Map at the adapter; domain types remain transport-neutral. |

# Validation

- Prove JSON-RPC and gRPC invocations of the same fixture method produce equivalent normalized results and policy decisions.
- Cover Invoke, Notify, Batch, invalid parameters, result validation, auth, TLS policy, limits, deadlines, cancellation, and transport failures.
- Verify reproducible code generation with the pinned vendored compiler.
- Run independent `grpc`, `rpc-http`, and combined feature checks plus lint, tests, audit, and portable graphs.

# Acceptance criteria

- A configured gRPC server and client invoke the same operation registry as JSON-RPC.
- Protobuf/gRPC remains optional and cannot change JSON-RPC, webhook, ACP, or MCP wire contracts.
- Generated code is reproducible and no generated Rust is checked in.

# Next plan

After this plan's acceptance criteria pass, continue with [ACP inference and verified fixes](../../features/opencode-acp-inference.md). ACP should register inference services against the established operation contract while its official SDK retains wire ownership.

# Citations

- [Protocol Buffers language guide](https://protobuf.dev/programming-guides/proto3/)
- [tonic documentation](https://docs.rs/tonic/)
