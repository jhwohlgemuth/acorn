# 📡 Serve

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a Nutshell</p>
<p>Run an ACORN bot service with <code>acorn serve bot &lt;PROJECT_ID&gt;</code>.</p>
</blockquote>

Start a bot for a GitLab project:

```sh
acorn serve bot 12345
acorn serve bot 12345 --event-source webhook \
  --public-url https://bot.example.org --register-webhook
```

Bot event delivery can use polling, authenticated webhooks, or a hybrid of both. Use `acorn serve bot --help` to review bind, port, timestamp, polling, and webhook options before exposing the service.

The CLI currently lists `acorn serve mcp`, but the server path returns an explicit "not yet implemented" error. The bot server is the supported service path today.

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Diagnose the service host with <a href="doctor.md">Doctor</a>.</p>
