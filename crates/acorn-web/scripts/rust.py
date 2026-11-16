"""Run browser Rust commands with the toolchain selected by rustup."""
import os
from pathlib import Path
import subprocess
import sys


def environment():
    selected = subprocess.check_output(["rustup", "which", "rustc"], text=True).strip()
    return {**os.environ, "PATH": str(Path(selected).parent) + os.pathsep + os.environ.get("PATH", "")}


def execute(command, platform=os.name):
    if platform == "nt":
        return subprocess.call(command, env=environment())
    os.execvpe(command[0], command, environment())


if __name__ == "__main__":
    sys.exit(execute(sys.argv[1:]))
