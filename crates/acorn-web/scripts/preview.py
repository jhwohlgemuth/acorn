"""Serve only built browser assets, with SPA routes, on loopback."""
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit

class Handler(SimpleHTTPRequestHandler):
    def do_GET(self):
        requested = Path(self.translate_path(self.path))
        if not requested.exists() and not Path(urlsplit(self.path).path).suffix:
            self.path = "/index.html"
        super().do_GET()

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--port", type=int, default=8080)
    args = parser.parse_args()
    if not (args.directory / "index.html").is_file():
        parser.error("Build the browser first with make web-build")
    print(f"ACORN preview: http://127.0.0.1:{args.port}", flush=True)
    ThreadingHTTPServer(("127.0.0.1", args.port), partial(Handler, directory=str(args.directory))).serve_forever()
