"""Coordinate Dioxus and route-local Vite development; stop both process groups together."""
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
import urllib.request
from rust import environment as rust_environment

try:
    import termios
except ImportError:
    termios = None

root = Path(__file__).resolve().parents[3]
processes = []
terminal = termios.tcgetattr(sys.stdin) if termios is not None and sys.stdin.isatty() else None


def launch(command, cwd, environment):
    options = {"creationflags": subprocess.CREATE_NEW_PROCESS_GROUP} if os.name == "nt" else {"start_new_session": True}
    return subprocess.Popen(command, cwd=cwd, env=environment, **options)


def terminate(process, force=False):
    if os.name == "nt":
        if force:
            process.kill()
        else:
            try:
                process.send_signal(signal.CTRL_BREAK_EVENT)
            except OSError:
                process.terminate()
    else:
        os.killpg(process.pid, signal.SIGKILL if force else signal.SIGTERM)


def stop(_signum=None, _frame=None):
    for process in processes:
        if process.poll() is None:
            terminate(process)
    for process in processes:
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            terminate(process, force=True)
            process.wait()


signal.signal(signal.SIGINT, lambda *_: sys.exit(130))
signal.signal(signal.SIGTERM, lambda *_: sys.exit(143))
try:
    frontend_port = int(sys.argv[sys.argv.index("--port") + 1])
    environment = {**rust_environment(), "ACORN_WEB_DEV_ORIGIN": f"http://127.0.0.1:{frontend_port}", "ACORN_MAP_DEV_URL": "http://127.0.0.1:8083"}
    processes.append(launch(["npm", "run", "dev"], root / "crates/acorn-web/geospatial", environment))
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
    processes.append(launch(sys.argv[1:], root, environment))
    while all(process.poll() is None for process in processes):
        time.sleep(0.2)
    sys.exit(next(process.returncode for process in processes if process.returncode is not None))
finally:
    try:
        stop()
    finally:
        if terminal is not None and termios is not None:
            termios.tcsetattr(sys.stdin, termios.TCSADRAIN, terminal)
