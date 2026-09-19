"""Loopback fixture for immutable pack GETs and HTTP byte ranges."""
from contextlib import contextmanager
from http.server import BaseHTTPRequestHandler, HTTPServer
import re
from threading import Thread


@contextmanager
def serve_packs(packs):
    metadata = (packs / "fra_for_eng" / "language_data.hash").read_text().strip().splitlines()
    if len(metadata) != 2:
        raise ValueError("Expected core and sentences metadata")
    objects = {}
    for part, line in zip(("core", "sentences"), metadata):
        hash_value, size = map(int, line.split(";"))
        path = packs / "fra_for_eng" / f"language_data_{part}.rkyv"
        if size <= 0 or path.stat().st_size != size:
            raise ValueError(f"Invalid fixture size for {part}")
        objects[f"/fra_for_eng/language_data_{part}_{hash_value}.rkyv"] = (part, path, size)

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_args):
            pass

        def do_POST(self):
            if self.path != "/__offline":
                self.send_error(404)
                return
            self.server.offline = True
            self.send_response(204)
            self.end_headers()

        def do_GET(self):
            obj = objects.get(self.path)
            if obj is None:  # Includes stale hashes and path traversal.
                self.send_error(404)
                return
            if self.server.offline:
                self.server.offline_downloads += 1
                self.send_error(503, "Fixture downloads disabled")
                return
            part, path, size = obj
            match = re.fullmatch(r"bytes=(\d+)-(\d+)", self.headers.get("Range", ""))
            if match is None:
                self.send_error(400, "Expected an explicit byte range")
                return
            start, end = map(int, match.groups())
            if start >= size or end < start:
                self.send_response(416)
                self.send_header("Content-Range", f"bytes */{size}")
                self.end_headers()
                return
            end = min(end, size - 1)
            remaining = end - start + 1
            with path.open("rb") as source:
                source.seek(start)
                self.send_response(206)
                self.send_header("Content-Type", "application/octet-stream")
                self.send_header("Accept-Ranges", "bytes")
                self.send_header("Content-Range", f"bytes {start}-{end}/{size}")
                self.send_header("Content-Length", str(remaining))
                self.end_headers()
                while remaining:
                    chunk = source.read(min(64 * 1024, remaining))
                    if not chunk:
                        raise EOFError("Fixture pack changed during download")
                    self.wfile.write(chunk)
                    remaining -= len(chunk)
            self.server.downloads.append((part, start))

    with HTTPServer(("127.0.0.1", 0), Handler) as server:
        server.url = f"http://127.0.0.1:{server.server_port}"
        server.offline = False
        server.offline_downloads = 0
        server.downloads = []
        thread = Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            yield server
        finally:
            server.shutdown()
            thread.join()
