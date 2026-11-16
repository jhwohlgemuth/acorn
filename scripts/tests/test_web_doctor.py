"""Regression checks for browser prerequisite diagnostics."""
from contextlib import redirect_stdout
import importlib.util
from io import StringIO
from pathlib import Path
import unittest

SCRIPT = Path(__file__).resolve().parents[2] / "crates/acorn-web/scripts/doctor.py"
SPEC = importlib.util.spec_from_file_location("web_doctor", SCRIPT)
DOCTOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(DOCTOR)


def outputs(overrides=None):
    values = {
        ("dx", "--version"): "dioxus 0.7.3",
        ("node", "--version"): "v22.23.5",
        ("npm", "--version"): "10.9.9",
        ("rustup", "target", "list", "--installed"): "wasm32-unknown-unknown",
        ("rustup", "which", "rustc"): "/toolchain/bin/rustc",
    }
    return {**values, **(overrides or {})}.get


class WebDoctorTests(unittest.TestCase):
    def test_doctor_accepts_ready_environment(self):
        report = StringIO()
        with redirect_stdout(report):
            result = DOCTOR.diagnose(Path("dx"), 8080, output=outputs(), available=lambda _port: True)
        self.assertEqual(result, 0)
        self.assertIn("Web prerequisites are ready.", report.getvalue())
    def test_doctor_reports_custom_dioxus_failure(self):
        report = StringIO()
        with redirect_stdout(report):
            result = DOCTOR.diagnose(Path("custom-dx"), 8080, output=outputs(), available=lambda _port: True)
        self.assertEqual(result, 1)
        self.assertIn("Custom Dioxus CLI is unavailable", report.getvalue())
    def test_doctor_reports_managed_setup_without_failing(self):
        report = StringIO()
        values = outputs({
            (str(DOCTOR.DEFAULT_DIOXUS), "--version"): None,
            ("rustup", "target", "list", "--installed"): "x86_64-pc-windows-msvc",
        })
        with redirect_stdout(report):
            result = DOCTOR.diagnose(DOCTOR.DEFAULT_DIOXUS, 8080, output=values, available=lambda _port: True)
        self.assertEqual(result, 0)
        self.assertIn("[SETUP] Rust target", report.getvalue())
        self.assertIn("[SETUP] Dioxus CLI", report.getvalue())
    def test_doctor_reports_node_and_port_failures(self):
        report = StringIO()
        values = outputs({("node", "--version"): "v21.0.0", ("npm", "--version"): None})
        with redirect_stdout(report):
            result = DOCTOR.diagnose(Path("dx"), 8080, output=values, available=lambda port: port != DOCTOR.VITE_PORT)
        self.assertEqual(result, 1)
        self.assertIn(f"nvm install {DOCTOR.PINNED_NODE_VERSION}", report.getvalue())
        self.assertIn("Vite port 8083 is already in use", report.getvalue())
    def test_parse_version(self):
        self.assertEqual(DOCTOR.parse_version("v22.23.5"), (22, 23, 5))
        self.assertIsNone(DOCTOR.parse_version("not-a-version"))


if __name__ == "__main__":
    unittest.main()
