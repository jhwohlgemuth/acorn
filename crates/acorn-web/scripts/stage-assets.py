"""Stage generated browser assets without platform-specific shell commands."""
from pathlib import Path
import shutil

root = Path(__file__).resolve().parents[3]
generated = root / "crates/acorn-web/assets/generated"
generated.mkdir(parents=True, exist_ok=True)
shutil.copy2(root / "docs/src/_assets/acorn.svg", generated / "acorn.svg")
shutil.copy2(root / "crates/acorn-wasm/web/webmcp.js", generated / "webmcp.js")
