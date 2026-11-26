# Source-built local macOS service MVP acceptance

Status: incomplete. The host owner authenticated system registration and lifecycle commands in their own terminal; no PIN was entered or recorded here. Logout, reboot, and uninstall rows remain open. The test service remains installed with both jobs paused while the owner is unavailable to sign out.

## Host and build

- Date: 2026-11-25.
- Host: `wohlgemuthjh`, macOS 26.7.3 (build 25G342), arm64; selected service user: `o9w`.
- Source baseline: `041577d0` plus the current working tree. Paired `target/debug/acorn` and `target/debug/acornd` both report version 0.6.3.
- Test config: `target/acd-mac-mvp/config.jsonc` (owner `o9w`, mode 0600, parent mode 0700), with local RPC port 18021 and `check-one`/`gather-one` jobs over fixture inputs. The fixture stays on the host through reboot; it is ignored build-tree data, not a release installation.
- Token directory: `~/Library/Application Support/org.ornl.acorn/daemon` (owner `o9w`, mode 0700); token is an owner-only regular file (mode 0600). Its value was never printed or included in command arguments.
- Reboot baseline: `kern.boottime` was 2026-11-16 08:52:15 local (Unix second 1794837135) before native acceptance transitions.

## Observed results

| Gate | Result |
| --- | --- |
| macOS plist syntax and preflight | Four focused macOS installer tests passed, including `plutil -lint` on a rendered plist. The full daemon library suite passed 58 tests before the additional status test; package Clippy passed with warnings denied. Daemon rustdoc built successfully. |
| Paired binaries | Both native binaries built and report version 0.6.3. `acornd --help` lists install/uninstall/start/stop/restart/status. `--linger` fails as Linux-only. |
| Privileged install and identity | The host owner installed from the durable `target/acd-mac-mvp` config. `/Library/LaunchDaemons/org.ornl.acornd.plist` is root:wheel mode 0644 and passes `plutil -lint`. `launchctl print system/org.ornl.acornd` reported a running LaunchDaemon with `UserName=o9w`, `HOME=/Users/o9w`, umask 077, the absolute paired `acornd` and config paths, and PID 53435. `ps` confirmed the process owner is `o9w` and parent PID 1. |
| Readiness and local protection | `/healthz` returned HTTP 204. The daemon directory is owner `o9w` mode 0700; token, socket `rpc-18021.sock`, history DB, and check/gather JSON reports are mode 0600. The CLI listed both configured jobs through the socket. Its first sandboxed call failed at transport; the same call outside the sandbox succeeded. |
| Authentication | Unauthenticated and deliberately wrong-token JSON-RPC requests each returned HTTP 401. No bearer value was printed. |
| Jobs and reports | Startup runs 1/2 were recorded for check/gather. Gather succeeded with exit 0; check produced a parseable JSON report with three findings and exit 70 from the chosen fixture, including an intentionally invalid link. Manual runs 9/10 were accepted and recorded with the same expected outcomes. Interval runs appeared for both jobs. Reports were parsed with `jq`. |
| Pause, file change, reload | Pausing `gather-one` kept its latest run at ID 10 through a file edit. After resume and another edit, run 11 succeeded with `trigger=file-change`. Authenticated reload returned generation 1. |
| Automatic restart and recovery | Sending TERM to PID 53435 caused launchd to start PID 60122 (`runs=2`, last signal 15); health returned 204 and run history remained. After pausing gather and sending TERM to PID 60122, launchd started PID 60928 (`runs=3`); gather stayed paused with prior history. Gather was then resumed. |
| Explicit stop/start/restart and idempotent reinstall | The host owner reported that all four `acornd` commands completed. The managed plist modification time advanced to 11:03:10 local and remained root:wheel mode 0644; launchd reported a running service at PID 74708 with parent 1 and user `o9w` after the sequence. Health returned 204, the owner-only socket and history DB remained, and local control listed both jobs. New startup check run 31 appeared in retained history. Per-command intermediate stop/start states were not independently captured. |
| Logout, reboot, uninstall | Not run. Gather interval was briefly shortened to 15 seconds and reload returned generation 2; before the requested logout, gather startup run 35 succeeded at Unix second 1795622781, with baseline 1795622788 (2026-11-25 16:06:28 UTC). The host owner could not sign out in this session. The fixture was restored and then set to hourly intervals (generation 4); both jobs were paused to stop background acceptance work. The system service remains installed. |

## Resume

When the host owner can sign out, change the disposable gather schedule to 15 seconds, reload, and resume `gather-one` before recording a fresh baseline. Remain signed out for at least 45 seconds, then inspect interval run timestamps after login. Current pre-reboot `kern.boottime` is Unix second 1794837135 (2026-11-16 08:52:15 local); record a new baseline before reboot and verify a changed boot time, launchd start, retained history, and new jobs after login. After both persistence checks, finish with `target/debug/acornd uninstall` in the host owner's terminal, confirming the service is gone and user data remains protected. Do not mark ACD-27 complete until every MVP row passes.
