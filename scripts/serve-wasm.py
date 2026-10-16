#!/usr/bin/env python3
"""Offline example/test server; loopback only, with shared-cancellation headers."""
import http.server
from pathlib import Path


class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(Path(__file__).resolve().parent.parent), **kwargs)

    def end_headers(self):
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cache-Control", "no-store")
        super().end_headers()


if __name__ == "__main__":
    print("http://127.0.0.1:8000/crates/engine-wasm/js/test-browser.html", flush=True)
    http.server.ThreadingHTTPServer(("127.0.0.1", 8000), Handler).serve_forever()
