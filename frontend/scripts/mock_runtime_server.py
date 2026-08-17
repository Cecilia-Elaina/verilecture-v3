#!/usr/bin/env python3
"""Local HTTP/Range fixture server for the VeriLecture runtime downloader.

It deliberately has no internet access and serves only bytes supplied by the
caller. The Rust downloader tests use an equivalent in-process fixture; this
module is useful for manual protocol checks and for black-box Tauri debugging.
"""

from __future__ import annotations

import argparse
import json
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Iterator, Optional


class _Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, _format: str, *_args: object) -> None:
        return

    def do_HEAD(self) -> None:  # noqa: N802 - stdlib hook
        self._serve(send_body=False)

    def do_GET(self) -> None:  # noqa: N802 - stdlib hook
        self._serve(send_body=True)

    def _serve(self, *, send_body: bool) -> None:
        server: "MockRuntimeHTTPServer" = self.server  # type: ignore[assignment]
        with server.lock:
            request_index = len(server.requests)
            range_start = _parse_range(self.headers.get("Range"))
            server.requests.append(
                {
                    "index": request_index,
                    "rangeStart": range_start,
                    "method": self.command,
                }
            )

        if server.status != 200:
            self.send_response(server.status)
            self.send_header("Content-Length", "0")
            self.send_header("Connection", "close")
            self.end_headers()
            return

        total_bytes = server.payload_size
        start = range_start if range_start is not None and not server.ignore_range else 0
        if start > total_bytes:
            self.send_response(416)
            self.send_header("Content-Length", "0")
            self.send_header("Connection", "close")
            self.end_headers()
            return
        body_bytes = total_bytes - start
        status = 206 if range_start is not None and not server.ignore_range else 200
        self.send_response(status)
        self.send_header("Content-Length", str(body_bytes))
        if status == 206:
            end = start + body_bytes - 1
            self.send_header("Content-Range", f"bytes {start}-{end}/{total_bytes}")
        self.send_header("Accept-Ranges", "bytes")
        self.send_header("Connection", "close")
        self.end_headers()
        if not send_body:
            return

        limit = body_bytes
        if server.drop_first_bytes is not None and request_index == 0:
            limit = min(limit, server.drop_first_bytes)
        sent = 0
        for chunk in server.iter_body(start, limit):
            self.wfile.write(chunk)
            self.wfile.flush()
            sent += len(chunk)
            if server.chunk_delay_seconds:
                time.sleep(server.chunk_delay_seconds)
        if sent < body_bytes:
            self.close_connection = True


def _parse_range(value: Optional[str]) -> Optional[int]:
    if not value or not value.lower().startswith("bytes="):
        return None
    start = value.split("=", 1)[1].split("-", 1)[0].strip()
    try:
        return int(start)
    except ValueError:
        return None


class MockRuntimeHTTPServer(ThreadingHTTPServer):
    daemon_threads = True
    allow_reuse_address = True

    def __init__(
        self,
        payload: bytes | Path,
        *,
        status: int = 200,
        corrupt: bool = False,
        drop_first_bytes: Optional[int] = None,
        ignore_range: bool = False,
        chunk_size: int = 64 * 1024,
        chunk_delay_seconds: float = 0.0,
    ) -> None:
        super().__init__(("127.0.0.1", 0), _Handler)
        self.payload = payload if isinstance(payload, bytes) else b""
        self.payload_path = payload if isinstance(payload, Path) else None
        self.status = status
        self.corrupt = corrupt
        self.drop_first_bytes = drop_first_bytes
        self.ignore_range = ignore_range
        self.chunk_size = max(1, chunk_size)
        self.chunk_delay_seconds = max(0.0, chunk_delay_seconds)
        self.lock = threading.Lock()
        self.requests: list[dict[str, object]] = []
        self.thread: Optional[threading.Thread] = None

    @property
    def url(self) -> str:
        return f"http://127.0.0.1:{self.server_port}/runtime.zip"

    @property
    def payload_size(self) -> int:
        if self.payload_path is not None:
            return self.payload_path.stat().st_size
        return len(self.payload)

    def iter_body(self, start: int, limit: int) -> Iterator[bytes]:
        remaining = limit
        offset = start
        if self.payload_path is not None:
            with self.payload_path.open("rb") as stream:
                stream.seek(start)
                while remaining:
                    chunk = stream.read(min(self.chunk_size, remaining))
                    if not chunk:
                        break
                    if self.corrupt and offset == 0:
                        chunk = bytes([chunk[0] ^ 0xFF]) + chunk[1:]
                    yield chunk
                    offset += len(chunk)
                    remaining -= len(chunk)
            return
        while remaining:
            chunk = self.payload[offset : min(offset + self.chunk_size, start + limit)]
            if not chunk:
                break
            if self.corrupt and offset == 0:
                chunk = bytes([chunk[0] ^ 0xFF]) + chunk[1:]
            yield chunk
            offset += len(chunk)
            remaining -= len(chunk)

    def __enter__(self) -> "MockRuntimeHTTPServer":
        self.thread = threading.Thread(target=self.serve_forever, daemon=True)
        self.thread.start()
        return self

    def __exit__(self, *_args: object) -> None:
        self.shutdown()
        self.server_close()
        if self.thread is not None:
            self.thread.join(timeout=2)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--file", type=Path, required=True)
    parser.add_argument("--status", type=int, default=200)
    parser.add_argument("--corrupt", action="store_true")
    parser.add_argument("--drop-first-bytes", type=int)
    parser.add_argument("--ignore-range", action="store_true")
    parser.add_argument("--chunk-size", type=int, default=64 * 1024)
    parser.add_argument("--chunk-delay-seconds", type=float, default=0.0)
    args = parser.parse_args()
    server = MockRuntimeHTTPServer(
        args.file,
        status=args.status,
        corrupt=args.corrupt,
        drop_first_bytes=args.drop_first_bytes,
        ignore_range=args.ignore_range,
        chunk_size=args.chunk_size,
        chunk_delay_seconds=args.chunk_delay_seconds,
    )
    with server:
        print(json.dumps({"url": server.url, "bytes": server.payload_size}), flush=True)
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            return 0


if __name__ == "__main__":
    raise SystemExit(main())
