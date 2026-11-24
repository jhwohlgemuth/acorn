"""Exercise explicit web process registration and stop requests."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from unittest.mock import patch

SCRIPTS = Path(__file__).resolve().parents[2] / "crates/acorn-web/scripts"
CONTROL_SPEC = importlib.util.spec_from_file_location("process_control", SCRIPTS / "process_control.py")
CONTROL = importlib.util.module_from_spec(CONTROL_SPEC)
CONTROL_SPEC.loader.exec_module(CONTROL)
STOP_SPEC = importlib.util.spec_from_file_location("web_stop", SCRIPTS / "stop.py")
STOP = importlib.util.module_from_spec(STOP_SPEC)
with patch.dict(sys.modules, {"process_control": CONTROL}):
    STOP_SPEC.loader.exec_module(STOP)


class WebStopTests(unittest.TestCase):
    def test_active_registration_stops_after_request(self):
        with tempfile.TemporaryDirectory() as directory:
            registry = Path(directory)
            with patch.object(CONTROL, "REGISTRY", registry):
                with CONTROL.registration("web-dev") as request:
                    record = next(registry.glob("*.lock"))
                    self.assertTrue(CONTROL.active(record))
                    self.assertTrue(record.with_suffix(".ready").exists())
                    results = []
                    worker = threading.Thread(target=lambda: results.append(STOP.stop(registry, timeout=2)))
                    worker.start()
                    deadline = time.monotonic() + 2
                    while not request.exists() and time.monotonic() < deadline:
                        time.sleep(0.01)
                    self.assertTrue(request.exists())
                worker.join(timeout=2)
                self.assertFalse(worker.is_alive())
                self.assertEqual(results, [True])
                self.assertFalse(record.exists())
                self.assertFalse(record.with_suffix(".ready").exists())
                self.assertFalse(request.exists())

    def test_preview_exits_on_its_own_stop_request(self):
        with tempfile.TemporaryDirectory() as directory:
            page = Path(directory)
            (page / "index.html").write_text("preview")
            before = set(CONTROL.REGISTRY.glob("web-preview-*.lock"))
            process = subprocess.Popen(
                [sys.executable, str(SCRIPTS / "preview.py"), str(page), "--port", "0"],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            try:
                deadline = time.monotonic() + 5
                records = set()
                while not records and process.poll() is None and time.monotonic() < deadline:
                    records = set(CONTROL.REGISTRY.glob("web-preview-*.lock")) - before
                    time.sleep(0.01)
                self.assertEqual(len(records), 1, process.stderr.read() if process.poll() is not None else "preview did not register")
                record = next(iter(records))
                record.with_suffix(".stop").touch()
                output, error = process.communicate(timeout=5)
                self.assertEqual(process.returncode, 0, error)
                self.assertIn("ACORN Research Intelligence preview", output)
                self.assertFalse(record.exists())
            finally:
                if process.poll() is None:
                    process.terminate()
                process.communicate(timeout=5)

    def test_stale_registration_is_removed(self):
        with tempfile.TemporaryDirectory() as directory:
            registry = Path(directory)
            record = registry / "web-dev-stale.lock"
            ready = record.with_suffix(".ready")
            request = record.with_suffix(".stop")
            record.write_bytes(b"1")
            ready.touch()
            request.touch()
            self.assertTrue(STOP.stop(registry, timeout=0))
            self.assertFalse(record.exists())
            self.assertFalse(ready.exists())
            self.assertFalse(request.exists())

    def test_unresponsive_registration_reports_timeout(self):
        with tempfile.TemporaryDirectory() as directory:
            registry = Path(directory)
            with patch.object(CONTROL, "REGISTRY", registry):
                with CONTROL.registration("web-dev") as request:
                    record = next(registry.glob("*.lock"))
                    self.assertFalse(STOP.stop(registry, timeout=0))
                    self.assertTrue(request.exists())
                    self.assertTrue(record.exists())

if __name__ == "__main__":
    unittest.main()
