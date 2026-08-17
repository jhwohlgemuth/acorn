# 🏗️ Create

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a Nutshell</p>
<p>Create an ACORN-backed runner or bot with <code>acorn create &lt;RESOURCE&gt;</code>.</p>
</blockquote>

Runner and bot creation are available:

```sh
# Register and start a GitLab runner from ACORN configuration
acorn create runner --config ./.acorn.json

# Create a runner directly for a GitLab group
acorn create runner --group 12345 --name research-runner

# Create a detached bot container for a GitLab project
acorn create bot 12345 --detach
```

Runner creation supports Docker, Podman, and Apptainer selection with `--runtime`. A remote Docker daemon can be selected with `--remote`; volumes and published ports then belong to the remote host.

Run `acorn create <resource> --help` for the exact authentication, runtime, and network options.

The CLI currently lists `acorn create mcp`, but that path returns an explicit "not yet implemented" error. Use [`serve bot`](serve.md) or the supported runner and bot creation paths above.

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Run the bot you created with <a href="serve.md">Serve</a>.</p>
