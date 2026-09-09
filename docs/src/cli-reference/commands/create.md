# 🏗️ Create

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Create an ACORN-backed runner or bot with <code>acorn create &lt;RESOURCE&gt;</code>.</p>
</blockquote>

Runner, bot, and MCP Bundle creation are available:

```sh
# Register and start a GitLab runner from ACORN configuration
acorn create runner --config ./.acorn.json

# Create a runner directly for a GitLab group
acorn create runner --group 12345 --name research-runner

# Create a detached bot container for a GitLab project
acorn create bot 12345 --detach

# Package the running executable
acorn create mcp

# Package a release binary and update an authoring Kitfile
acorn create mcp --binary ./acorn --platform linux --kitfile ./Kitfile
```

Runner creation supports Docker, Podman, and Apptainer selection with `--runtime`. A remote Docker daemon can be selected with `--remote`; volumes and published ports then belong to the remote host.

Run `acorn create <resource> --help` for the exact authentication, runtime, and network options.

`create mcp` emits a deterministic MCPB archive. Use `--dry-run` to inspect the manifest and inventory, `--clobber` to replace an existing regular file, and `--output` to select its destination. External binaries require an explicit `--platform darwin|linux|win32`; Windows inputs must end in `.exe`. See [MCP Bundles](../../packages/mcpb.md) for the manifest, trust, and release contract.

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Run the bot you created with <a href="serve.md">Serve</a>.</p>
