"""Run a real Qwen ASR + ForcedAligner CUDA smoke test against a WAV sample."""

from __future__ import annotations

import argparse
import base64
import json
import struct
import subprocess
import sys
import time
import wave


def wav_to_f32le(path: str) -> bytes:
    with wave.open(path, "rb") as wav:
        channels = wav.getnchannels()
        sample_width = wav.getsampwidth()
        sample_rate = wav.getframerate()
        if channels != 1 or sample_width != 2 or sample_rate != 16000:
            raise SystemExit("Smoke sample must be 16 kHz, mono, 16-bit PCM WAV")
        samples = struct.unpack(
            "<%dh" % (wav.getnframes() * channels),
            wav.readframes(wav.getnframes()),
        )
    return b"".join(struct.pack("<f", sample / 32768.0) for sample in samples)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--runtime", required=True)
    parser.add_argument("--model-dir", required=True)
    parser.add_argument("--audio", required=True)
    parser.add_argument("--model-id", default="qwen3-asr-1.7b")
    parser.add_argument("--language", default="zh")
    args = parser.parse_args()

    audio = base64.b64encode(wav_to_f32le(args.audio)).decode("ascii")
    process = subprocess.Popen(
        ([args.runtime] if args.runtime.lower().endswith(".exe") else [sys.executable, args.runtime])
        + [
            "--protocol",
            "json-lines",
            "--model-id",
            args.model_id,
            "--model-dir",
            args.model_dir,
            "--device",
            "CUDA",
        ],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
    )

    requests = [
        {
            "operation": "load",
            "requestId": "qwen-load-1",
            "modelId": args.model_id,
            "modelDir": args.model_dir,
            "device": "CUDA",
        },
        {
            "operation": "transcribe",
            "requestId": "qwen-transcribe-1",
            "modelId": args.model_id,
            "modelDir": args.model_dir,
            "device": "CUDA",
            "language": args.language,
            "audioPcmF32LeBase64": audio,
        },
        {"operation": "unload", "requestId": "qwen-unload-1"},
    ]

    started = time.perf_counter()
    try:
        for request in requests:
            assert process.stdin is not None
            assert process.stdout is not None
            process.stdin.write(json.dumps(request, ensure_ascii=False) + "\n")
            process.stdin.flush()
            response_line = process.stdout.readline()
            if not response_line:
                raise RuntimeError("sidecar exited before responding")
            response = json.loads(response_line)
            # Keep the acceptance log ASCII-stable on Windows consoles while
            # retaining the actual Unicode response for validation.
            print(json.dumps(response, ensure_ascii=True, indent=2))
            if response.get("ok") is not True:
                raise RuntimeError(response.get("errorCode", "ASR_RUNTIME_FAILED"))
            if request["operation"] == "load" and response.get("executionDevice") != "CUDA":
                raise RuntimeError("Qwen smoke did not load on CUDA")
            if request["operation"] == "transcribe" and not response.get("segments"):
                raise RuntimeError("Qwen smoke returned no timestamped segments")
    finally:
        if process.stdin:
            process.stdin.close()
        stderr = process.stderr.read() if process.stderr else ""
        process.wait(timeout=60)
        if stderr.strip():
            print(stderr.strip(), file=sys.stderr)

    print(json.dumps({"elapsedSeconds": round(time.perf_counter() - started, 2)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
