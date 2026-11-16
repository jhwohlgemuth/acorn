"""Diagnose browser development prerequisites without changing the machine."""
import argparse
from pathlib import Path
import re
import socket
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
DEFAULT_DIOXUS = ROOT / "target/tools/dioxus/bin/dx"
MINIMUM_NODE_VERSION = (22, 17, 0)
PINNED_NODE_VERSION = (ROOT / ".nvmrc").read_text(encoding="utf-8").strip()
VITE_PORT = 8083
WASM_TARGET = "wasm32-unknown-unknown"


def command_output(command):
    try:
        completed = subprocess.run(command, capture_output=True, check=False, text=True)
    except OSError:
        return None
    return completed.stdout.strip() if completed.returncode == 0 else None


def diagnose(dioxus, frontend_port, output=command_output, available=None):
    port_available = available or is_port_available
    diagnostics = [("PASS", f"Python {sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}")]
    rustc = output(("rustup", "which", "rustc"))
    diagnostics.append(("PASS", f"Rustup toolchain: {rustc}") if rustc else ("FAIL", "Rustup-managed Rust is unavailable"))
    targets = output(("rustup", "target", "list", "--installed")) if rustc else None
    if targets and WASM_TARGET in targets.splitlines():
        diagnostics.append(("PASS", f"Rust target: {WASM_TARGET}"))
    elif rustc:
        diagnostics.append(("SETUP", f"Rust target {WASM_TARGET} is missing; make web-setup will install it"))
    node = output(("node", "--version"))
    node_version = parse_version(node) if node else None
    if node_version and node_version >= MINIMUM_NODE_VERSION:
        diagnostics.append(("PASS", f"Node.js {node.lstrip('v')}"))
    else:
        diagnostics.append(("FAIL", f"Node.js 22.17+ is required. With nvm: nvm install {PINNED_NODE_VERSION} && nvm use {PINNED_NODE_VERSION}"))
    npm = output(("npm", "--version"))
    diagnostics.append(("PASS", f"npm {npm}") if npm else ("FAIL", "npm is unavailable; activate the pinned Node.js version"))
    dioxus_version = output((str(dioxus), "--version"))
    if dioxus_version:
        diagnostics.append(("PASS", dioxus_version))
    elif is_managed_dioxus(dioxus):
        diagnostics.append(("SETUP", "Dioxus CLI is missing; make web-setup or make web-dev will install it"))
    else:
        diagnostics.append(("FAIL", f"Custom Dioxus CLI is unavailable: {dioxus}"))
    for name, port in (("Dioxus", frontend_port), ("Vite", VITE_PORT)):
        diagnostics.append(("PASS", f"{name} port {port} is available") if port_available(port) else ("FAIL", f"{name} port {port} is already in use"))
    for level, message in diagnostics:
        print(f"[{level}] {message}")
    failures = sum(level == "FAIL" for level, _ in diagnostics)
    setup = sum(level == "SETUP" for level, _ in diagnostics)
    if failures:
        print(f"Web prerequisites have {failures} blocking issue(s).")
        return 1
    if setup:
        print(f"Web prerequisites are usable after {setup} non-blocking setup item(s).")
    else:
        print("Web prerequisites are ready.")
    return 0


def is_managed_dioxus(dioxus):
    candidate = Path(dioxus)
    executable = candidate.with_suffix(".exe") if sys.platform == "win32" else candidate
    managed = DEFAULT_DIOXUS.with_suffix(".exe") if sys.platform == "win32" else DEFAULT_DIOXUS
    return executable.resolve(strict=False) == managed.resolve(strict=False)


def is_port_available(port):
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        try:
            listener.bind(("127.0.0.1", port))
        except OSError:
            return False
    return True


def parse_version(value):
    match = re.fullmatch(r"v?(\d+)\.(\d+)\.(\d+)(?:[-+].*)?", value.strip())
    return tuple(map(int, match.groups())) if match else None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dioxus", type=Path)
    parser.add_argument("--frontend-port", default=8080, type=int)
    arguments = parser.parse_args()
    return diagnose(arguments.dioxus, arguments.frontend_port)


if __name__ == "__main__":
    sys.exit(main())
