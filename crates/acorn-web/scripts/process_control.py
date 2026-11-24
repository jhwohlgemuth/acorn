"""Register local web processes and accept file-based stop requests."""
from contextlib import contextmanager
import os
from pathlib import Path
import uuid

if os.name == "nt":
    import msvcrt
else:
    import fcntl

REGISTRY = Path(__file__).resolve().parents[3] / "target/web-processes"


def active(record):
    owned = False
    try:
        with record.open("r+b") as handle:
            try:
                lock(handle, blocking=False)
            except OSError:
                owned = True
            else:
                unlock(handle)
    except FileNotFoundError:
        pass
    return owned


def lock(handle, blocking):
    handle.seek(0)
    if os.name == "nt":
        mode = msvcrt.LK_LOCK if blocking else msvcrt.LK_NBLCK
        msvcrt.locking(handle.fileno(), mode, 1)
    else:
        mode = fcntl.LOCK_EX | (0 if blocking else fcntl.LOCK_NB)
        fcntl.flock(handle.fileno(), mode)


@contextmanager
def registration(name):
    REGISTRY.mkdir(parents=True, exist_ok=True)
    record = REGISTRY / f"{name}-{uuid.uuid4().hex}.lock"
    ready = record.with_suffix(".ready")
    request = record.with_suffix(".stop")
    try:
        with record.open("x+b") as handle:
            handle.write(b"1")
            handle.flush()
            lock(handle, blocking=True)
            ready.touch()
            try:
                yield request
            finally:
                request.unlink(missing_ok=True)
                ready.unlink(missing_ok=True)
                unlock(handle)
    finally:
        record.unlink(missing_ok=True)


def unlock(handle):
    handle.seek(0)
    if os.name == "nt":
        msvcrt.locking(handle.fileno(), msvcrt.LK_UNLCK, 1)
    else:
        fcntl.flock(handle.fileno(), fcntl.LOCK_UN)
