# 🏗️ Create

<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Create an ACORN-backed runner or bot with <code>acorn create &lt;RESOURCE&gt;</code>.</p>
</blockquote>

Runner, bot, and MCP Bundle creation are available:

```sh
# Register and start a GitLab runner from ACORN configuration
acorn create runner

# Create a runner directly for a GitLab group
acorn create runner --group 12345 --description "Research runner" --name research-runner

# Register a local shell runner for a self-managed GitLab group
acorn create runner --no-config --group 12345 --description "Windows shell runner" --runtime shell --tags windows-x86_64 --repo code.ornl.gov

# Create a detached bot container for a GitLab project
acorn create bot 12345 --detach

# Package the running executable
acorn create mcp

# Package a release binary and update an authoring Kitfile
acorn create mcp --binary ./acorn --platform linux --kitfile ./Kitfile
```

Runner creation supports Docker, Podman, Apptainer, and local shell selection with `--runtime`. Shell registration runs `gitlab-runner` on the machine running ACORN; it does not create a container. Install GitLab Runner first and ensure its running service reads the `config.toml` written by registration. Set `GITLAB_TOKEN`, `PRIVATE_TOKEN`, or `CI_JOB_TOKEN` with runner-creation access. Use `--repo` for the GitLab domain. Shell jobs run on that host with the service account's permissions, so use a host dedicated to trusted jobs. See [GitLab Runner commands](https://docs.gitlab.com/runner/commands/) and the [shell executor](https://docs.gitlab.com/runner/executors/shell/).

Runner containers always start detached. A remote Docker daemon can be selected with `--host` or configured per runner with `target.host`; volumes then belong to the remote host. `--host`, `--context`, and `target.host` select container targets, not the GitLab domain or a remote shell host. See the [full runner configuration](../configuration.md#flat-file) for a config-only example.

Run `acorn create <resource> --help` for the exact authentication, runtime, and network options.

`create mcp` emits a deterministic MCPB archive. Use `--dry-run` to inspect the manifest and inventory, `--clobber` to replace an existing regular file, and `--output` to select its destination. External binaries require an explicit `--platform darwin|linux|win32`; Windows inputs must end in `.exe`. See [MCP Bundles](../../packages/mcpb.md) for the manifest, trust, and release contract.

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Run the bot you created with <a href="serve.md">Serve</a>.</p>
