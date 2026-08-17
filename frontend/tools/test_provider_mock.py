"""Protocol matrix for the local Provider mock (no external network)."""

from __future__ import annotations

import json
import threading
import urllib.error
import urllib.request

from provider_mock import make_server


def request(url: str) -> tuple[int, dict | str]:
    try:
        with urllib.request.urlopen(urllib.request.Request(url, method="POST", data=b"{}"), timeout=1) as response:
            body = response.read().decode("utf-8")
            try:
                payload: dict | str = json.loads(body)
            except json.JSONDecodeError:
                payload = body
            return response.status, payload
    except urllib.error.HTTPError as error:
        return error.code, json.loads(error.read().decode("utf-8"))


def main() -> int:
    server = make_server(0, "ok")
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base = f"http://127.0.0.1:{server.server_address[1]}"
    try:
        for path in ("/v1/responses", "/v1/chat/completions", "/v1/messages", "/v1beta/models/test:generateContent"):
            status, payload = request(base + path)
            assert status == 200
            assert isinstance(payload, dict)
        for mode, expected in (("auth", 401), ("rate", 429), ("invalid-json", 200), ("schema", 200)):
            server.mock_mode = mode
            status, _ = request(base + "/v1/responses")
            assert status == expected
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=2)
    print("Provider mock matrix passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
