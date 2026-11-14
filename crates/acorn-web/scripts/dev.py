"""Coordinate Dioxus and route-local Vite development; stop both process groups together."""
import os
from pathlib import Path
import signal
import subprocess
import sys
import termios
import time
import urllib.request
from rust import environment as rust_environment

root = Path(__file__).resolve().parents[3]
processes = []
terminal = termios.tcgetattr(sys.stdin) if sys.stdin.isatty() else None


def stop(_signum=None, _frame=None):
    for process in processes:
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGTERM)
    for process in processes:
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()


signal.signal(signal.SIGINT, lambda *_: sys.exit(130))
signal.signal(signal.SIGTERM, lambda *_: sys.exit(143))
try:
    frontend_port = int(sys.argv[sys.argv.index("--port") + 1])
    environment = {**rust_environment(), "ACORN_WEB_DEV_ORIGIN": f"http://127.0.0.1:{frontend_port}", "ACORN_MAP_DEV_URL": "http://127.0.0.1:8083"}
    processes.append(subprocess.Popen(["npm", "run", "dev"], cwd=root / "crates/acorn-web/geospatial", env=environment, start_new_session=True))
    for attempt in range(100):
        if processes[0].poll() is not None:
            raise RuntimeError("Vite stopped before becoming ready")
        try:
            urllib.request.urlopen("http://127.0.0.1:8083/@vite/client", timeout=1).close()
            break
        except OSError:
            time.sleep(0.2)
    else:
        raise RuntimeError("Vite did not become ready")
    processes.append(subprocess.Popen(sys.argv[1:], cwd=root, env=environment, start_new_session=True))
    while all(process.poll() is None for process in processes):
        time.sleep(0.2)
    sys.exit(next(process.returncode for process in processes if process.returncode is not None))
finally:
    try:
        stop()
    finally:
        if terminal is not None:
            termios.tcsetattr(sys.stdin, termios.TCSADRAIN, terminal)
