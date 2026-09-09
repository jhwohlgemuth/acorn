# 📡 Serve

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Run an ACORN bot or local stdio MCP service with <code>acorn serve</code>.</p>
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

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Diagnose the service host with <a href="doctor.md">Doctor</a>.</p>
