"""Regression checks for browser toolchain selection and terminal cleanup."""
from contextlib import nullcontext
import importlib.util
import os
from pathlib import Path
import runpy
import signal
import subprocess
import sys
import tempfile
import unittest
import urllib.request
from unittest.mock import MagicMock, patch

SCRIPTS = Path(__file__).resolve().parents[2] / "crates/acorn-web/scripts"
SPEC = importlib.util.spec_from_file_location("rust", SCRIPTS / "rust.py")
RUST = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUST)
CONTROL = MagicMock()
CONTROL.registration.side_effect = lambda _: nullcontext(CONTROL.request)
CONTROL.request.exists.return_value = False


class WebDevTests(unittest.TestCase):
    def test_cleanup_restores_terminal_even_when_child_cleanup_fails(self):
        terminal = ["original terminal settings"]
        termios = MagicMock(TCSADRAIN=1)
        vite = MagicMock(returncode=None)
        vite.poll.return_value = None
        vite.wait.side_effect = RuntimeError("cleanup failed")
        dioxus = MagicMock(returncode=7)
        dioxus.poll.return_value = 7
        with patch.dict(sys.modules, {"rust": RUST, "termios": termios, "process_control": CONTROL}), patch.object(sys, "argv", ["dev.py", "dx", "serve", "--port", "8084"]), \
                patch.object(sys.stdin, "isatty", return_value=True), patch("termios.tcgetattr", return_value=terminal), \
                patch("termios.tcsetattr") as restore, patch("signal.signal"), patch("os.killpg", create=True), \
                patch.object(RUST, "environment", return_value={"PATH": "/selected/bin"}), \
                patch("urllib.request.urlopen"), patch("subprocess.Popen", side_effect=[vite, dioxus]) as launch:
            with self.assertRaisesRegex(RuntimeError, "cleanup failed"):
                runpy.run_path(str(SCRIPTS / "dev.py"), run_name="__main__")
            restore.assert_called_once()
            self.assertEqual(restore.call_args.args[2], terminal)
            self.assertEqual(launch.call_args.kwargs["env"]["PATH"], "/selected/bin")
            if os.name == "nt":
                self.assertEqual(launch.call_args.kwargs["creationflags"], subprocess.CREATE_NEW_PROCESS_GROUP)
            else:
                self.assertTrue(launch.call_args.kwargs["start_new_session"])

    def test_cli_setup_runs_for_missing_or_stale_local_executable(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            executable = root / "target/tools/dioxus/bin/dx"
            marker = root / "setup-ran"
            overlay = root / "Makefile"
            overlay.write_text(f"include {SCRIPTS.parents[2] / 'Makefile'}\n" + "web-setup:\n\t@mkdir -p target/tools/dioxus/bin\n\t@printf '#!/bin/sh\\necho dioxus 0.7.10\\n' > target/tools/dioxus/bin/dx\n\t@chmod +x target/tools/dioxus/bin/dx\n\t@touch setup-ran\n")
            command = ["make", "--no-print-directory", "-s", "web-cli"]
            first = subprocess.run(command, cwd=root, text=True, capture_output=True)
            self.assertEqual(first.returncode, 0, first.stderr)
            self.assertTrue(executable.is_file())
            self.assertTrue(marker.is_file())
            marker.unlink()
            repeated = subprocess.run(command, cwd=root, text=True, capture_output=True)
            self.assertEqual(repeated.returncode, 0, repeated.stderr)
            self.assertFalse(marker.exists())
            executable.write_text("#!/bin/sh\necho dioxus 0.7.3\n")
            stale = subprocess.run(command, cwd=root, text=True, capture_output=True)
            self.assertEqual(stale.returncode, 0, stale.stderr)
            self.assertTrue(marker.exists())
            marker.unlink()
            custom = root / "custom-dx"
            executable.rename(custom)
            overridden = subprocess.run(command + [f"DIOXUS={custom}"], cwd=root, text=True, capture_output=True)
            self.assertEqual(overridden.returncode, 0, overridden.stderr)
            self.assertFalse(marker.exists())
            missing = subprocess.run(command + [f"DIOXUS={root / 'missing-dx'}"], cwd=root, text=True, capture_output=True)
            self.assertNotEqual(missing.returncode, 0)
            self.assertIn("Dioxus CLI not executable", missing.stderr)
            self.assertFalse(marker.exists())
            with overlay.open("a") as makefile:
                makefile.write("web-deps web-editor-bundle web-lint web-map web-prepare:\n\t@:\nweb-dev web-build:\n\t@test -x target/tools/dioxus/bin/dx\n")
            for target in ("web-dev", "web-build"):
                executable.unlink(missing_ok=True)
                marker.unlink(missing_ok=True)
                result = subprocess.run(command[:-1] + [target], cwd=root, text=True, capture_output=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertTrue(marker.exists(), f"{target} must prepare the missing CLI")

    def test_dioxus_launch_failure_stops_vite_and_restores_terminal(self):
        termios = MagicMock(TCSADRAIN=1)
        vite = MagicMock(returncode=None, pid=12345)
        vite.poll.return_value = None
        with patch.dict(sys.modules, {"rust": RUST, "termios": termios, "process_control": CONTROL}), patch.object(sys, "argv", ["dev.py", "missing-dx", "serve", "--port", "8084"]), \
                patch.object(sys.stdin, "isatty", return_value=True), patch("termios.tcgetattr", return_value=[]), \
                patch("termios.tcsetattr") as restore, patch("signal.signal"), patch("os.killpg", create=True) as terminate, \
                patch.object(RUST, "environment", return_value={"PATH": "/selected/bin"}), \
                patch("urllib.request.urlopen"), patch("subprocess.Popen", side_effect=[vite, FileNotFoundError("missing dx")]):
            with self.assertRaises(FileNotFoundError):
                runpy.run_path(str(SCRIPTS / "dev.py"), run_name="__main__")
            if os.name == "nt":
                vite.send_signal.assert_called_once_with(signal.CTRL_BREAK_EVENT)
            else:
                terminate.assert_called_once()
                self.assertEqual(terminate.call_args.args[0], vite.pid)
            vite.wait.assert_called_once_with(timeout=10)
            restore.assert_called_once()

    def test_rustup_selection_precedes_system_rust_and_preserves_overrides(self):
        original = {"PATH": "/opt/homebrew/bin:/custom/cargo/bin", "RUSTUP_TOOLCHAIN": "custom", "CARGO_HOME": "/custom/cargo"}
        with patch.dict(os.environ, original, clear=True), patch("subprocess.check_output", return_value="/selected/toolchain/bin/rustc\n") as which:
            selected = RUST.environment()
            which.assert_called_once_with(["rustup", "which", "rustc"], text=True)
            self.assertEqual(selected["PATH"], str(Path("/selected/toolchain/bin")) + os.pathsep + original["PATH"])
            self.assertEqual(selected["RUSTUP_TOOLCHAIN"], "custom")
            self.assertEqual(selected["CARGO_HOME"], "/custom/cargo")
            self.assertEqual(dict(os.environ), original)

    def test_rustup_selection_uses_subprocess_on_windows(self):
        command = ["cargo", "install", "dioxus-cli"]
        with patch.object(RUST, "environment", return_value={"PATH": "C:\\selected\\bin"}), \
                patch("subprocess.call", return_value=23) as call:
            self.assertEqual(RUST.execute(command, platform="nt"), 23)
            call.assert_called_once_with(command, env={"PATH": "C:\\selected\\bin"})

    def test_stop_request_stops_both_child_groups(self):
        termios = MagicMock(TCSADRAIN=1)
        vite = MagicMock(returncode=None, pid=12345)
        vite.poll.return_value = None
        dioxus = MagicMock(returncode=None, pid=23456)
        dioxus.poll.return_value = None
        with patch.dict(sys.modules, {"rust": RUST, "termios": termios, "process_control": CONTROL}), patch.object(sys, "argv", ["dev.py", "dx", "serve", "--port", "8084"]), \
                patch.object(sys.stdin, "isatty", return_value=False), patch("signal.signal"), \
                patch("os.killpg", create=True) as terminate, patch.object(CONTROL.request, "exists", side_effect=[False, False, True, True]), \
                patch.object(RUST, "environment", return_value={"PATH": "/selected/bin"}), \
                patch("urllib.request.urlopen"), patch("subprocess.Popen", side_effect=[vite, dioxus]):
            runpy.run_path(str(SCRIPTS / "dev.py"), run_name="__main__")
            vite.wait.assert_called_once_with(timeout=10)
            dioxus.wait.assert_called_once_with(timeout=10)
            if os.name == "nt":
                vite.send_signal.assert_called_once_with(signal.CTRL_BREAK_EVENT)
                dioxus.send_signal.assert_called_once_with(signal.CTRL_BREAK_EVENT)
            else:
                self.assertEqual([call.args[0] for call in terminate.call_args_list], [vite.pid, dioxus.pid])

    def test_toolchain_failure_starts_no_servers_and_restores_terminal(self):
        termios = MagicMock(TCSADRAIN=1)
        with patch.dict(sys.modules, {"rust": RUST, "termios": termios, "process_control": CONTROL}), patch.object(sys, "argv", ["dev.py", "dx", "serve", "--port", "8084"]), \
                patch.object(sys.stdin, "isatty", return_value=True), patch("termios.tcgetattr", return_value=[]), \
                patch("termios.tcsetattr") as restore, patch("signal.signal"), patch("subprocess.Popen") as launch, \
                patch.object(RUST, "environment", side_effect=subprocess.CalledProcessError(1, "rustup")):
            with self.assertRaises(subprocess.CalledProcessError):
                runpy.run_path(str(SCRIPTS / "dev.py"), run_name="__main__")
            launch.assert_not_called()
            restore.assert_called_once()


if __name__ == "__main__":
    unittest.main()
