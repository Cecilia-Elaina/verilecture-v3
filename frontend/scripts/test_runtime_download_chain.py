#!/usr/bin/env python3
"""Black-box checks for the local Mock HTTP Range server.

These checks use a small deterministic fixture to verify protocol behavior;
they are not a substitute for the separate 4.6 GB runtime size/SHA-256 check.
The Rust unit tests exercise the production downloader with the same Range,
resume, interruption, 404, size and checksum gates.
"""

from __future__ import annotations

import hashlib
import http.client
import json
import urllib.error
import urllib.request
from pathlib import Path

from mock_runtime_server import MockRuntimeHTTPServer


def get(url: str, headers: dict[str, str] | None = None) -> tuple[int, dict[str, str], bytes]:
    request = urllib.request.Request(url, headers=headers or {})
    try:
        with urllib.request.urlopen(request, timeout=5) as response:
            response_headers = dict(response.headers)
            try:
                body = response.read()
            except http.client.IncompleteRead as exc:
                body = exc.partial
            return response.status, response_headers, body
    except urllib.error.HTTPError as exc:
        return exc.code, dict(exc.headers), exc.read()


def main() -> int:
    payload = bytes((index % 251 for index in range(128 * 1024)))
    digest = hashlib.sha256(payload).hexdigest()

    with MockRuntimeHTTPServer(payload) as server:
        status, headers, body = get(server.url)
        assert status == 200 and body == payload
        assert headers.get("Accept-Ranges") == "bytes"

        status, headers, body = get(server.url, {"Range": "bytes=8192-"})
        assert status == 206 and body == payload[8192:]
        assert headers.get("Content-Range", "").startswith("bytes 8192-")

    with MockRuntimeHTTPServer(payload, drop_first_bytes=8192) as server:
        status, _headers, partial = get(server.url)
        assert status == 200 and partial == payload[:8192]
        status, _headers, resumed = get(server.url, {"Range": "bytes=8192-"})
        assert status == 206 and partial + resumed == payload
        assert server.requests[1]["rangeStart"] == 8192

    with MockRuntimeHTTPServer(payload, corrupt=True) as server:
        _status, _headers, body = get(server.url)
        assert hashlib.sha256(body).hexdigest() != digest

    with MockRuntimeHTTPServer(payload, status=404) as server:
        status, _headers, _body = get(server.url)
        assert status == 404

    registry_path = Path(__file__).parents[1] / "src-tauri" / "resources" / "runtime_registry.json"
    registry = json.loads(registry_path.read_text(encoding="utf-8"))
    runtime = next(item for item in registry["runtimes"] if item["id"] == "cuda-qwen-fun")
    assert runtime["status"] == "published"
    published_mirrors = [mirror for mirror in runtime["mirrors"] if mirror["status"] == "published"]
    assert len(published_mirrors) == 1
    assert published_mirrors[0]["id"] == "huggingface"
    assert published_mirrors[0]["url"].startswith("https://huggingface.co/")
    print("MOCK_RUNTIME_CHAIN_OK")
    print("FULL_DOWNLOAD_OK RANGE_OK RESUME_OK SHA_ERROR_OK HTTP_404_OK HF_SOURCE_OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
