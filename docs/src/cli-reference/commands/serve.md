# 📡 Serve

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Run an ACORN bot, local stdio MCP service, or authenticated JSON-RPC endpoint with <code>acorn serve</code>.</p>
</blockquote>

Start a bot for a GitLab project:

```sh
acorn serve bot 12345
acorn serve bot 12345 --event-source webhook \
  --public-url https://bot.example.org --register-webhook
```

Bot event delivery can use polling, authenticated webhooks, or a hybrid of both. Use `acorn serve bot --help` to review bind, port, timestamp, polling, and webhook options before exposing the service.

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

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Diagnose the service host with <a href="doctor.md">Doctor</a>.</p>
