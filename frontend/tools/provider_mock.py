"""Local-only Provider protocol mock used by VeriLecture tests.

It deliberately never prints request bodies or authorization headers.  The
server covers the four adapter shapes used by the desktop app and deterministic
failure modes needed by the acceptance matrix.
"""

from __future__ import annotations

import argparse
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


class MockHandler(BaseHTTPRequestHandler):
    server_version = "VeriLectureProviderMock/1"

    def log_message(self, format: str, *args: object) -> None:  # noqa: A002
        return

    def _mode(self) -> str:
        return str(getattr(self.server, "mock_mode", "ok"))

    def _send(self, status: int, payload: object, content_type: str = "application/json") -> None:
        body = json.dumps(payload, ensure_ascii=False).encode("utf-8") if not isinstance(payload, bytes) else payload
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:  # noqa: N802
        if self.path == "/health":
            self._send(200, {"ok": True, "service": "provider-mock"})
        else:
            self._send(404, {"error": "not_found"})

    def do_POST(self) -> None:  # noqa: N802
        length = int(self.headers.get("Content-Length", "0"))
        _ = self.rfile.read(length)
        mode = self._mode()
        if mode == "auth":
            self._send(401, {"error": {"type": "authentication_error"}})
            return
        if mode == "rate":
            self._send(429, {"error": {"type": "rate_limit"}})
            return
        if mode == "timeout":
            import time

            time.sleep(2.0)
            self._send(504, {"error": {"type": "timeout"}})
            return
        if mode == "invalid-json":
            self._send(200, b"not json", "text/plain")
            return
        if mode == "schema":
            self._send(200, {"unexpected": True})
            return
        if mode == "stream-interrupt":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", "100")
            self.end_headers()
            self.wfile.write(b'{"choices":[')
            self.wfile.flush()
            self.connection.close()
            return

        if self.path.endswith("/responses"):
            self._send(200, {"output_text": '{"ok":true,"message":"READY"}'})
        elif self.path.endswith("/messages"):
            self._send(200, {"content": [{"type": "text", "text": '{"ok":true,"message":"READY"}'}]})
        elif ":generateContent" in self.path:
            self._send(200, {"candidates": [{"content": {"parts": [{"text": '{"ok":true,"message":"READY"}'}]}}]})
        else:
            self._send(200, {"choices": [{"message": {"content": '{"ok":true,"message":"READY"}'}}]})


def make_server(port: int, mode: str) -> ThreadingHTTPServer:
    server = ThreadingHTTPServer(("127.0.0.1", port), MockHandler)
    server.mock_mode = mode
    return server


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=0)
    parser.add_argument("--mode", default="ok")
    args = parser.parse_args()
    server = make_server(args.port, args.mode)
    print(f"READY {server.server_address[1]}", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
