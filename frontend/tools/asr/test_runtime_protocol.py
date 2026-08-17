"""Small, dependency-free contract tests for the ASR sidecar boundary."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from types import SimpleNamespace
from pathlib import Path

import verilecture_asr_runtime as runtime


def test_language_and_timestamp_normalization() -> None:
    assert runtime._qwen_language("zh") == "Chinese"
    assert runtime._qwen_language("auto") is None
    values = runtime._normalise_time_stamps(
        [{"text": "TCP", "start_time": 0.25, "end_time": 0.75}], "zh"
    )
    assert values == [{"startMs": 250, "endMs": 750, "text": "TCP", "language": "zh"}]
    wrapped = runtime._normalise_time_stamps(
        SimpleNamespace(
            items=[SimpleNamespace(text="课", start_time=1.0, end_time=1.25)]
        ),
        "zh",
    )
    assert wrapped == [{"startMs": 1000, "endMs": 1250, "text": "课", "language": "zh"}]


def test_json_lines_heartbeat_has_no_protocol_noise() -> None:
    source = Path(__file__).with_name("verilecture_asr_runtime.py")
    with tempfile.TemporaryDirectory() as directory:
        process = subprocess.Popen(
            [sys.executable, str(source), "--protocol", "json-lines"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        assert process.stdin is not None
        assert process.stdout is not None
        process.stdin.write(json.dumps({"operation": "heartbeat", "requestId": "heartbeat-1"}) + "\n")
        process.stdin.close()
        response = json.loads(process.stdout.readline())
        process.wait(timeout=10)
        assert response["ok"] is True
        assert response["requestId"] == "heartbeat-1"
        assert process.stdout.read() == ""


if __name__ == "__main__":
    test_language_and_timestamp_normalization()
    test_json_lines_heartbeat_has_no_protocol_noise()
    print("ASR runtime protocol tests passed")
