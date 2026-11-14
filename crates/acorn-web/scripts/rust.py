"""Run browser Rust commands with the toolchain selected by rustup."""
import os
from pathlib import Path
import subprocess
import sys


def environment():
    selected = subprocess.check_output(["rustup", "which", "rustc"], text=True).strip()
    return {**os.environ, "PATH": str(Path(selected).parent) + os.pathsep + os.environ.get("PATH", "")}


if __name__ == "__main__":
    os.execvpe(sys.argv[1], sys.argv[1:], environment())
