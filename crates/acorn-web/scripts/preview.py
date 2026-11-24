"""Serve local browser previews, with SPA routes, on loopback."""
import argparse
import webbrowser
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
    parser.add_argument("--open", action="store_true", help="Open the preview in the default browser")
    parser.add_argument("--path", default="/", help="Preview page to open")
    args = parser.parse_args()
    preview_path = args.path.strip("/")
    entry = args.directory / preview_path / "index.html"
    if not entry.is_file():
        parser.error(f"Missing preview entry: {entry}" if preview_path else "Build the browser first with make web-build")
    url_path = f"/{preview_path}/" if preview_path else "/"
    url = f"http://127.0.0.1:{args.port}{url_path}"
    with ThreadingHTTPServer(("127.0.0.1", args.port), partial(Handler, directory=str(args.directory))) as server:
        print(f"ACORN Research Intelligence preview: {url}", flush=True)
        if args.open:
            webbrowser.open(url)
        server.serve_forever()
