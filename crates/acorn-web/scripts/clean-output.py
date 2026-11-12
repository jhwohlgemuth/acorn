"""Remove only stale, generated release web assets before Dioxus packaging."""
from pathlib import Path
import shutil

root = Path(__file__).resolve().parents[3]
output = root / "target/dx/acorn-web/release/web/public"
if output.is_symlink():
    raise SystemExit("Refusing to clean a symlinked browser output directory")
if output.exists():
    shutil.rmtree(output)
