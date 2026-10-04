# Protobuf and gRPC

ACORN's optional gRPC capability provides a versioned binary transport for the same operation registry used by the JSON-RPC endpoint. It is useful for service-to-service integrations that want generated clients, HTTP/2 transport, request deadlines, and Protobuf compatibility without duplicating ACORN's operation handlers or policy checks.

The committed contract is [`crates/acorn-lib/proto/acorn/rpc/v1/rpc.proto`](https://code.ornl.gov/research-enablement/acorn/-/blob/main/crates/acorn-lib/proto/acorn/rpc/v1/rpc.proto). Cargo generates the Rust client and server modules during the build; generated files remain in Cargo's build directory.

The `acorn.rpc.v1.AcornRpc` service has three RPCs:

| RPC | Purpose | Response |
| --- | --- | --- |
| `Invoke` | Run one registered operation with a string or integer correlation ID. | An `InvokeResponse` containing either a result or a structured `RpcError`. |
| `Notify` | Run one operation when the caller does not need its result. | `google.protobuf.Empty` after the request is accepted for dispatch. |
| `Batch` | Run an ordered list of invocations. | Responses in request order. |

The RPC envelope is generated from Protobuf. Operation parameters and results use `google.protobuf.Value`, so callers can send the same JSON-compatible shapes accepted by JSON-RPC. ACORN validates each payload against the registered operation's schema before calling its handler.

## Start a local server

The gRPC capability is not part of the default CLI build. Enable it, set a dedicated bearer token, and bind the development server to a loopback address:

```sh
cargo build --release --package acorn-cli --features grpc
export ACORN_GRPC_TOKEN='development-secret'
./target/release/acorn serve grpc --bind 127.0.0.1:50051
```

The examples below use `grpcurl`. Server reflection is not enabled, so each command supplies the repository's contract with `-import-path` and `-proto`.

## Invoke an operation

Call the read-only `acorn.version` operation:

```sh
grpcurl -plaintext \
  -H 'authorization: Bearer development-secret' \
  -import-path crates/acorn-lib/proto \
  -proto acorn/rpc/v1/rpc.proto \
  -d '{"id":{"text":"version-1"},"method":"acorn.version","params":{}}' \
  127.0.0.1:50051 \
  acorn.rpc.v1.AcornRpc/Invoke
```

The response repeats the request ID and returns either `result` or `error`. A successful response has this shape:

```json
{
  "id": { "text": "version-1" },
  "result": { "version": "0.0.0" }
}
```

The version value reflects the CLI build that is running.

## Invoke an ordered batch

`Batch` reduces round trips while retaining one response for each request:

```sh
grpcurl -plaintext \
  -H 'authorization: Bearer development-secret' \
  -import-path crates/acorn-lib/proto \
  -proto acorn/rpc/v1/rpc.proto \
  -d '{"requests":[
    {"id":{"text":"first"},"method":"acorn.version","params":{}},
    {"id":{"text":"second"},"method":"acorn.version","params":{}}
  ]}' \
  127.0.0.1:50051 \
  acorn.rpc.v1.AcornRpc/Batch
```

The server accepts at most 128 requests in a batch by default and preserves their order in `BatchResponse.responses`.

## Send a notification

`Notify` runs an operation without returning its operation result. The gRPC call still reports transport-level failures:

```sh
grpcurl -plaintext \
  -H 'authorization: Bearer development-secret' \
  -import-path crates/acorn-lib/proto \
  -proto acorn/rpc/v1/rpc.proto \
  -d '{"method":"acorn.version","params":{}}' \
  127.0.0.1:50051 \
  acorn.rpc.v1.AcornRpc/Notify
```

Use notifications only when losing the operation result is acceptable. Mutation policy applies to notifications in the same way that it applies to regular invocations.

## Use the Rust client

Enable the library capability in an application:

```sh
cargo add acorn-lib --features grpc
cargo add color-eyre serde_json
cargo add tokio --features macros,rt-multi-thread
```

The generated messages and configured client are available under `acorn::io::api::grpc`:

```rust
use acorn::io::api::grpc::client::{GrpcClient, GrpcClientConfig};
use acorn::io::api::grpc::proto::{invoke_response, request_id, InvokeRequest, RequestId};
use acorn::io::api::grpc::{json_value, protobuf_value};
use acorn::io::api::Secret;
use color_eyre::eyre::{eyre, Result};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let config = GrpcClientConfig::new(
        "http://127.0.0.1:50051",
        Secret::from("development-secret".to_string()),
    );
    let mut client = GrpcClient::connect(config).await?;
    let params = protobuf_value(json!({})).map_err(|error| eyre!("{error:?}"))?;
    let response = client
        .invoke(InvokeRequest {
            id: Some(RequestId {
                value: Some(request_id::Value::Text("version-1".to_string())),
            }),
            method: "acorn.version".to_string(),
            params: Some(params),
        })
        .await?;

    match response.outcome {
        Some(invoke_response::Outcome::Result(value)) => {
            let value = json_value(value).map_err(|error| eyre!("{error:?}"))?;
            println!("{value}");
        }
        Some(invoke_response::Outcome::Error(error)) => {
            eprintln!("operation failed ({}): {}", error.code, error.message);
        }
        None => return Err(eyre!("gRPC response did not contain an outcome")),
    }
    Ok(())
}
```

`GrpcClientConfig` also configures a PEM certificate authority, maximum encoded and decoded message size, offline policy, and connection/request timeout.

## Operations and policy

The baseline registry currently exposes these operations:

| Operation | Parameters | Behavior |
| --- | --- | --- |
| `acorn.version` | `{}` | Return the running ACORN version. |
| `acorn.workflows.process` | `workflow` and a repository `input` | Compute a repository workflow change set. |
| `acorn.operations.status` | `operation_key` | Read the state of a durable operation. |
| `acorn.operations.replay` | `operation_key` | Queue a durable operation for replay. This is a mutation. |

The server is read-only by default. Pass `--allow-mutation` to expose registered mutations such as `acorn.operations.replay`; mutation support also requires a local database selected by ACORN's global database options. The server includes global `--offline` policy in every invocation context. Client integrations that set `GrpcClientConfig::with_offline(true)` cannot connect to remote gRPC endpoints.

Authentication failures, invalid transport messages, message limits, deadlines, and connection failures use gRPC status codes. A valid invocation that reaches an operation handler returns an `InvokeResponse`; operation-level failures use its structured `RpcError` outcome. The defaults are a 1 MiB encoded and decoded message limit, a 30-second deadline, and 128 requests per batch.

Loopback endpoints may use plaintext HTTP/2 for local development. Non-loopback servers require a PEM certificate chain and private key, and non-loopback clients require HTTPS:

```sh
export ACORN_GRPC_TOKEN='replace-with-a-secret'
./target/release/acorn serve grpc \
  --bind 0.0.0.0:50051 \
  --tls-cert /etc/acorn/tls/server.pem \
  --tls-key /etc/acorn/tls/server.key
```

## Foundation for ACP and MCP

The gRPC transport established the transport-neutral service boundary now reused by the outbound MCP client. JSON-RPC, gRPC, and MCP share effect metadata and mutation and offline policy while preserving their own request and result types. The gRPC adapter also established transport-level deadlines and cancellation behavior.

Each protocol still keeps its own contract:

- The [MCP server](../cli-reference/commands/serve.md) exposes ACORN's canonical tool registry over stdio. The [MCP client](mcp-client.md) uses native MCP initialization, discovery, calls, cancellation, and shutdown for explicitly configured remote tools.
- The ACP client uses one-shot sessions, normalized streamed updates, and fixed-policy permission responses to supervise external agents.
- gRPC remains ACORN's typed binary operation transport. It does not wrap ACP or MCP frames, and gRPC compatibility does not imply ACP or MCP wire compatibility.

This separation lets ACORN reuse policy and deterministic capabilities while implementing discovery, sessions, streaming, and permissions in the protocol that defines them. See [Intelligence in Depth layers](../concepts/agentic-layers.md) for the broader agent architecture.

## Develop the contract

Edit `crates/acorn-lib/proto/acorn/rpc/v1/rpc.proto` when changing the public transport. Run Protobuf linting directly with `make protolint`, or run the full development lint workflow with `make lint`. Buf's standard rules enforce package versioning, RPC message naming, field naming, and related conventions before Cargo generates bindings. Linting does not replace a compatibility review of field numbers and wire types when the `v1` contract changes.
