"""Ask this checkout's registered web development processes to stop."""
import time

from process_control import REGISTRY, active


def stop(registry=REGISTRY, timeout=25):
    records = sorted(ready.with_suffix(".lock") for ready in registry.glob("*.ready"))
    running = [record for record in records if active(record)]
    for record in records:
        if record not in running:
            record.with_suffix(".stop").unlink(missing_ok=True)
            record.with_suffix(".ready").unlink(missing_ok=True)
            record.unlink(missing_ok=True)
    for record in running:
        print(f"Stopping {record.name.rsplit('-', 1)[0]} from this checkout.")
        record.with_suffix(".stop").touch()
    deadline = time.monotonic() + timeout
    pending = [record for record in running if active(record)]
    while pending and time.monotonic() < deadline:
        time.sleep(0.1)
        pending = [record for record in pending if active(record)]
    for record in running:
        if not active(record):
            record.with_suffix(".stop").unlink(missing_ok=True)
    if not running:
        print("No web-dev or browser preview processes found for this checkout.")
    for record in pending:
        print(f"Timed out waiting for {record.name.rsplit('-', 1)[0]} to stop.")
    return not pending


if __name__ == "__main__":
    raise SystemExit(0 if stop() else 1)
