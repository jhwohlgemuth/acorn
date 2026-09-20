# 📡 Serve

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Run an ACORN bot, local stdio MCP service, or authenticated JSON-RPC or gRPC endpoint with <code>acorn serve</code>.</p>
</blockquote>

Start a bot for a GitLab project:

```sh
acorn serve bot 12345
acorn serve bot 12345 --event-source webhook \
  --public-url https://bot.example.org --register-webhook
```

Bot event delivery can use polling, authenticated webhooks, or a hybrid of both. Use `acorn serve bot --help` to review bind, port, timestamp, polling, and webhook options before exposing the service.

Repository automation is constrained by the `workflow` section of application configuration passed with `--config`. The policy is validated before the bot starts; unknown workflow names, formats, actions, effects, and unsafe paths are rejected.

```yaml
workflow:
  allow-forks: false
  # destination: "12345" # Required, with allow-forks: true, for fork publication.
  workflows:
    repository-quality:
      paths:
        - "**/*.cff"
        - "**/*.{json,jsonc,yaml,yml,md}"
      formats: [cff, json, jsonc, markdown, yaml]
      actions: [validate, format, link]
      outputs:
        mode: source-and-artifacts
      effects:
        - branch
        - commit
        - commit-status
        - completion-comment
        - draft-merge-request
        - label
        - note
```

```sh
acorn serve bot 12345 --event-source webhook \
  --config /etc/acorn/.acorn.json
```

Paths are repository-relative globs. Repository content cannot add workflows or widen this policy. Fork writes are denied by default and require both `allow-forks: true` and an explicit `destination`; publication then targets only that configured project. Detached bot containers mount this file read-only.

Start the local MCP server with `acorn serve mcp`. Protocol frames use stdin/stdout and diagnostics use stderr. The default catalog is read-only; global `--offline` removes open-world tools. `--allow-mutation` enables persistent tools and requires a local database selected by the global database options:

```sh
acorn serve mcp
acorn --offline serve mcp
acorn --database-path ./acorn.db serve mcp --allow-mutation
```

Start the JSON-RPC 2.0 HTTP endpoint with a dedicated bearer token. The endpoint accepts `POST /rpc`, requires `application/json`, limits request bodies, and is read-only unless mutation is explicitly enabled:

```sh
export ACORN_RPC_TOKEN='replace-with-a-secret'
acorn serve rpc --bind 127.0.0.1:8001

curl --fail-with-body http://127.0.0.1:8001/rpc \
  --header 'Authorization: Bearer replace-with-a-secret' \
  --header 'Content-Type: application/json' \
  --data '{"jsonrpc":"2.0","id":1,"method":"acorn.version"}'
```

Use global `--offline` to deny open-world operations. `serve rpc --allow-mutation` enables registered mutations such as durable operation replay and requires local database persistence.

The optional Protobuf/gRPC transport exposes the same operation registry through the versioned `acorn.rpc.v1.AcornRpc` service. Build the CLI with the non-default capability and use a dedicated bearer token:

```sh
cargo build --release --package acorn-cli --features grpc
export ACORN_GRPC_TOKEN='replace-with-a-different-secret'
./target/release/acorn serve grpc --bind 127.0.0.1:50051
```

Call the version operation with `grpcurl` and the committed Protobuf contract:

```sh
grpcurl -plaintext \
  -H 'authorization: Bearer replace-with-a-different-secret' \
  -import-path crates/acorn-lib/proto \
  -proto acorn/rpc/v1/rpc.proto \
  -d '{"id":{"text":"version-1"},"method":"acorn.version","params":{}}' \
  127.0.0.1:50051 \
  acorn.rpc.v1.AcornRpc/Invoke
```

Loopback development may use plaintext HTTP/2. A non-loopback bind requires a PEM certificate chain and private key:

```sh
export ACORN_GRPC_TOKEN='replace-with-a-different-secret'
./target/release/acorn serve grpc \
  --bind 0.0.0.0:50051 \
  --tls-cert /etc/acorn/tls/server.pem \
  --tls-key /etc/acorn/tls/server.key
```

Clients send `authorization: Bearer …` metadata. Remote clients require TLS and are rejected under global offline policy; message limits and deadlines apply on both sides. Operation failures remain structured `RpcError` outcomes, while authentication, malformed transport requests, limits, deadlines, and connection failures use gRPC status codes. The committed contract is at `crates/acorn-lib/proto/acorn/rpc/v1/rpc.proto`; generated Rust remains in Cargo's build output and is not committed. See [Protobuf and gRPC](../../packages/grpc.md) for batch and notification examples, the Rust client API, and the relationship to ACP and MCP.

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Diagnose the service host with <a href="doctor.md">Doctor</a>.</p>
