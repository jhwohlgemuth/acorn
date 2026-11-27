# acornd: persistent jobs and APIs

`acornd` runs configured `check` and `gather` jobs when inputs change or on a schedule. It also serves ACORN's authenticated JSON-RPC and gRPC APIs. Use `acorn daemon` to install the service and control its jobs; use `acornd run CONFIG` when you need to run it in the foreground.

The signed bundle workflow below is available locally and over SSH on Linux. Local macOS service registration uses paired source-built `acorn` and `acornd` binaries. Signed macOS bundles, macOS SSH management, and Windows service registration are separate work.

## Prepare a configuration

Save an [ACRE configuration](../configuration.md) on the machine that will run the service. This example enables local JSON-RPC and one file-change job:

```jsonc
{
  "schemaVersion": "2.0.0",
  "daemon": {
    "bind": { "grpc": null, "rpc": "127.0.0.1:8001" },
    "jobs": [
      {
        "id": "project-check",
        "root": "/home/alice/project",
        "input": "activity.json",
        "kind": "check",
        "triggers": { "onChange": true }
      }
    ]
  }
}
```

Replace `root` with an existing absolute project directory and `input` with an existing path inside it. For macOS, use a path such as `/Users/alice/project`. Keep the configuration private to the service user (`chmod 600 PATH`). The JSON-RPC listener is needed for `acorn daemon` job commands; this example disables the optional gRPC listener. See [Serve](serve.md) for the API transports and authentication.

## Install locally on Linux

Get the signed archive for the host architecture and keep its detached `.minisig` file beside it. The archive contains a matching `acorn` and `acornd` pair. From the account that will own the service, run:

```sh
acorn daemon verify-bundle /path/to/acornd-vVERSION-TARGET.tar.gz
acorn daemon install --config /absolute/path/to/.acorn.jsonc \
  --bundle /path/to/acornd-vVERSION-TARGET.tar.gz
acorn daemon service-status
acorn daemon list
```

Replace `VERSION` and `TARGET` with the archive's published version and Linux target, such as `x86_64-unknown-linux-musl`. The signed installer stages the paired binaries, activates the service, checks readiness, and rolls back if readiness fails. To keep the user's systemd service manager running after logout, add `--linger` to `install` if lingering is not already enabled and your account is authorized to enable it. Check with `loginctl show-user "$USER" -p Linger`. An administrator can enable lingering when required.

Source-built Linux installation is also supported: put matching `acorn` and `acornd` binaries in the same persistent directory and run `acorn daemon install --config /absolute/path/to/.acorn.jsonc`. The source-built path requires an existing private daemon token. Uninstall a source-built service before its first signed installation; later signed bundles can update an existing signed service.

## Install locally on macOS

Build matching `acorn` and `acornd` binaries and place both in a persistent, owner-only directory. Keep the configuration there or at another private absolute path. Do not register binaries or configuration files under a temporary build directory: launchd needs those paths after reboot.

Before the first install, create an owner-only, nonempty token at `~/Library/Application Support/org.ornl.acorn/daemon/token`. One way to create it is:

```sh
DATA="$HOME/Library/Application Support/org.ornl.acorn/daemon"
umask 077
mkdir -p "$DATA"
chmod 700 "$DATA"
[ -e "$DATA/token" ] || openssl rand -hex 32 > "$DATA/token"
chmod 600 "$DATA/token"
```

Run the paired CLI as the selected unprivileged user, using the persistent configuration path:

```sh
/absolute/path/to/acorn daemon install --config /absolute/path/to/.acorn.jsonc
/absolute/path/to/acorn daemon service-status
/absolute/path/to/acorn daemon list --config /absolute/path/to/.acorn.jsonc
```

Installation may ask for administrator authentication to register a system LaunchDaemon. `acornd` itself runs as the selected user. `--linger` applies only to Linux. The macOS source-built service and its job control passed local reboot and logout acceptance; signed bundle activation and SSH management are not available there yet.

## Install and control a Linux host over SSH

Put the private configuration and its project files on the Linux host first. The `--config` path below is **on that host**; the `--bundle` path is **on the computer running `acorn`**. SSH must already work with noninteractive authentication.

```sh
acorn daemon verify-bundle /local/path/acornd-vVERSION-TARGET.tar.gz
acorn daemon --host lab-linux install \
  --config /home/alice/.config/acorn/.acorn.jsonc \
  --bundle /local/path/acornd-vVERSION-TARGET.tar.gz
acorn daemon --host lab-linux service-status
acorn daemon --host lab-linux list
acorn daemon --host lab-linux history project-check --limit 10
```

`lab-linux` can be an SSH alias or an `ssh://user@host` URI. The CLI detects the Linux architecture, verifies the archive and signature locally, checks the transferred bytes, and invokes the paired host CLI to install and check readiness. The control token stays on the host. To keep the service running after logout, arrange `Linger=yes` for its user or add `--linger` to the install command when the host permits it.

If you omit `--bundle`, a release CLI can use its embedded package registry. A CLI without an embedded release URL needs `--release-url HTTPS_BASE_URL` for a public registry. Access-controlled registries require a locally downloaded archive and adjacent `.minisig` passed with `--bundle`; global `--offline` mode also requires `--bundle`.

## Manage jobs and update the service

The same subcommands work locally or, on Linux, with `--host HOST` after `daemon`:

```sh
acorn daemon status project-check
acorn daemon run project-check
acorn daemon history project-check --limit 10
acorn daemon pause project-check
acorn daemon resume project-check
acorn daemon reload
acorn daemon restart
acorn daemon stop
acorn daemon start
```

`reload` applies a complete change to the configured jobs without restarting the process. Changing a listener address or TLS settings requires `restart`. For local job commands using a non-default configuration path, pass `--config PATH` to the command, as in the macOS example. `service-status` shows the operating system's service state; `status` shows one job and its latest run.

Install a newer signed Linux bundle with the same `install --config ... --bundle ...` command to update the service. A failed readiness check restores the prior release. `acorn daemon uninstall` (or `acorn daemon --host HOST uninstall`) removes the service registration and socket but keeps the configuration, token, history, and reports. Uninstall does not purge that data.

Run `acorn daemon --help` and `acorn daemon <command> --help` for the exact options in your installed CLI.
