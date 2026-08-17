"""VeriLecture local ASR sidecar runtime.

The Tauri application talks to this process over a newline-delimited JSON
protocol.  The process deliberately keeps the loaded Qwen model alive between
requests; a 1.7B model must not be reloaded for every VAD chunk.  Fun-ASR-Nano
uses the official CPU-only llama-funasr-cli binary and therefore never uses
CUDA, even when an NVIDIA GPU is present in the host computer.

This file is the source used to build the Windows sidecar executable.  Model
weights and third-party binaries are downloaded separately and are never
committed to the repository.
"""

from __future__ import annotations

import argparse
import base64
import contextlib
import gc
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import wave


RUNTIME_VERSION = "verilecture-asr-runtime/0.3.0-alpha.1"
QWEN_ASR_17B = "qwen3-asr-1.7b"
QWEN_ASR_06B = "qwen3-asr-0.6b"
FUN_ASR_NANO = "fun-asr-nano-2512"
PRODUCT_MODELS = {QWEN_ASR_17B, QWEN_ASR_06B, FUN_ASR_NANO}

# The application uses stable ISO-639 codes while the official Qwen Python
# API expects canonical English language names.  Keeping this conversion at
# the sidecar boundary prevents a Chinese selection from being interpreted as
# an unknown language and silently falling back to auto detection.
QWEN_LANGUAGE_NAMES = {
    "zh": "Chinese",
    "en": "English",
    "yue": "Cantonese",
    "ar": "Arabic",
    "de": "German",
    "fr": "French",
    "es": "Spanish",
    "pt": "Portuguese",
    "id": "Indonesian",
    "it": "Italian",
    "ko": "Korean",
    "ru": "Russian",
    "th": "Thai",
    "vi": "Vietnamese",
    "ja": "Japanese",
    "tr": "Turkish",
    "hi": "Hindi",
    "ms": "Malay",
    "nl": "Dutch",
    "sv": "Swedish",
    "da": "Danish",
    "fi": "Finnish",
    "pl": "Polish",
    "cs": "Czech",
    "fil": "Filipino",
    "fa": "Persian",
    "el": "Greek",
    "hu": "Hungarian",
    "mk": "Macedonian",
    "ro": "Romanian",
}


class RuntimeFailure(Exception):
    def __init__(self, error_code: str, message: str | None = None) -> None:
        super().__init__(message or error_code)
        self.error_code = error_code


def _write_response(response: dict) -> None:
    # The Rust side treats stdout as a machine-readable channel.  Keep all
    # diagnostics on stderr and emit exactly one compact JSON line per request.
    payload = (json.dumps(response, ensure_ascii=False, separators=(",", ":")) + "\n").encode("utf-8")
    stdout_buffer = getattr(sys.stdout, "buffer", None)
    if stdout_buffer is not None:
        stdout_buffer.write(payload)
        stdout_buffer.flush()
    else:  # pragma: no cover - useful for embedded test streams
        sys.stdout.write(payload.decode("utf-8"))
        sys.stdout.flush()


def _error_response(error_code: str, message: str | None = None) -> dict:
    response = {
        "ok": False,
        "runtimeVersion": RUNTIME_VERSION,
        "errorCode": error_code,
    }
    if message:
        response["message"] = message[:500]
    return response


def _model_dir(request: dict) -> Path:
    value = request.get("modelDir")
    if not isinstance(value, str) or not value:
        raise RuntimeFailure("ASR_MODEL_DIRECTORY_INVALID")
    path = Path(value).resolve()
    if not path.is_dir():
        raise RuntimeFailure("ASR_MODEL_NOT_DOWNLOADED")
    return path


def _model_id(request: dict) -> str:
    model_id = request.get("modelId")
    if model_id not in PRODUCT_MODELS:
        raise RuntimeFailure("ASR_MODEL_NOT_IN_PRODUCT_REGISTRY")
    return model_id


def _required_files(model_id: str) -> tuple[str, ...]:
    if model_id == QWEN_ASR_17B:
        return (
            "config.json",
            "generation_config.json",
            "model-00001-of-00002.safetensors",
            "model-00002-of-00002.safetensors",
            "model.safetensors.index.json",
            "preprocessor_config.json",
            "tokenizer_config.json",
            "vocab.json",
            "forced-aligner/config.json",
            "forced-aligner/generation_config.json",
            "forced-aligner/model.safetensors",
            "forced-aligner/preprocessor_config.json",
            "forced-aligner/tokenizer_config.json",
            "forced-aligner/vocab.json",
        )
    if model_id == QWEN_ASR_06B:
        return (
            "config.json",
            "generation_config.json",
            "model.safetensors",
            "preprocessor_config.json",
            "tokenizer_config.json",
            "vocab.json",
            "forced-aligner/config.json",
            "forced-aligner/generation_config.json",
            "forced-aligner/model.safetensors",
            "forced-aligner/preprocessor_config.json",
            "forced-aligner/tokenizer_config.json",
            "forced-aligner/vocab.json",
        )
    return ("funasr-encoder-f16.gguf", "qwen3-0.6b-q8_0.gguf", "fsmn-vad.gguf")


def _check_model_files(model_id: str, model_dir: Path) -> None:
    for filename in _required_files(model_id):
        if not (model_dir / filename).is_file():
            raise RuntimeFailure("ASR_MODEL_ARTIFACT_MISSING", filename)


def _qwen_language(language: object) -> str | None:
    if language is None or language == "" or language == "auto":
        return None
    if language == "translate_to_en":
        # Qwen3-ASR is used here as an ASR-only provider. Translation must be
        # an explicit, separately authorized text-processing operation; never
        # turn an invalid ASR language value into an implicit translation.
        raise RuntimeFailure("ASR_TRANSLATION_NOT_SUPPORTED_LOCAL")
    if not isinstance(language, str) or language not in QWEN_LANGUAGE_NAMES:
        raise RuntimeFailure("ASR_LANGUAGE_UNSUPPORTED")
    return QWEN_LANGUAGE_NAMES[language]


def _quiet_stdout():
    """Keep third-party Python logs off the JSON protocol stdout stream."""

    return contextlib.redirect_stdout(sys.stderr)


def _decode_cli_text(payload: bytes) -> str:
    """Decode the Windows FunASR CLI output without losing Chinese text.

    The official binary is distributed as a native Windows console program;
    depending on the host code page, its stdout may be UTF-8 or GB18030. Try
    UTF-8 first and use the Windows-compatible Chinese fallback only when the
    byte stream is not valid UTF-8.
    """

    for encoding in ("utf-8-sig", "gb18030"):
        try:
            return payload.decode(encoding)
        except UnicodeDecodeError:
            continue
    return payload.decode("utf-8", errors="replace")


def _timestamp_ms(value: object) -> int | None:
    if isinstance(value, bool):
        return None
    if isinstance(value, (int, float)):
        number = float(value)
    elif isinstance(value, str):
        try:
            number = float(value)
        except ValueError:
            return None
    else:
        return None
    # Qwen3 ForcedAligner exposes seconds. Accept millisecond values as a
    # defensive compatibility measure for a future runtime wrapper.
    return round(number if abs(number) >= 10_000 else number * 1000)


def _normalise_time_stamps(time_stamps: object, language: str) -> list[dict]:
    if isinstance(time_stamps, (list, tuple)):
        items = time_stamps
    else:
        # Qwen3 ForcedAligner returns a ForcedAlignResult wrapper whose
        # character/word items live on `.items`.
        items = getattr(time_stamps, "items", None)
    if not isinstance(items, (list, tuple)):
        return []
    segments: list[dict] = []
    for item in items:
        text: object = None
        start: object = None
        end: object = None
        if isinstance(item, dict):
            text = item.get("text") or item.get("word") or item.get("char") or item.get("token")
            start = next((item[key] for key in ("start", "start_time", "startTime") if key in item), None)
            end = next((item[key] for key in ("end", "end_time", "endTime") if key in item), None)
        elif isinstance(item, (list, tuple)) and len(item) >= 3:
            text, start, end = item[0], item[1], item[2]
        else:
            text = getattr(item, "text", None) or getattr(item, "word", None) or getattr(item, "char", None)
            start = getattr(item, "start_time", None)
            if start is None: start = getattr(item, "start", None)
            end = getattr(item, "end_time", None)
            if end is None: end = getattr(item, "end", None)
        if not isinstance(text, str) or not text.strip():
            continue
        start_ms = _timestamp_ms(start)
        end_ms = _timestamp_ms(end)
        if start_ms is None or end_ms is None or end_ms <= start_ms:
            continue
        segments.append({
            "startMs": max(0, start_ms),
            "endMs": max(start_ms + 1, end_ms),
            "text": text.strip(),
            "language": language,
        })
    return segments


class Runtime:
    def __init__(self, default_model_id: str, default_model_dir: Path, default_device: str) -> None:
        self.default_model_id = default_model_id
        self.default_model_dir = default_model_dir
        self.default_device = default_device.upper()
        self.loaded_model_id: str | None = None
        self.loaded_model_dir: Path | None = None
        self.loaded_device: str | None = None
        self.qwen_model = None

    def _load_qwen(self, model_id: str, model_dir: Path, device: str) -> None:
        if device != "CUDA":
            raise RuntimeFailure("ASR_QWEN_CPU_UNSUPPORTED")

        # Import lazily so Fun-ASR-Nano remains a small CPU path and does not
        # require the PyTorch/Qwen dependency bundle.
        try:
            with _quiet_stdout():
                import torch
                from qwen_asr import Qwen3ASRModel
        except Exception as exc:  # pragma: no cover - depends on packaged env
            raise RuntimeFailure("ASR_QWEN_RUNTIME_IMPORT_FAILED", str(exc)) from exc

        if not torch.cuda.is_available():
            raise RuntimeFailure("ASR_CUDA_UNAVAILABLE")

        dtype = torch.bfloat16
        try:
            with _quiet_stdout():
                self.qwen_model = Qwen3ASRModel.from_pretrained(
                    str(model_dir),
                    dtype=dtype,
                    device_map="cuda:0",
                    forced_aligner=str(model_dir / "forced-aligner"),
                    forced_aligner_kwargs={
                        "dtype": dtype,
                        "device_map": "cuda:0",
                    },
                )
        except Exception as exc:  # pragma: no cover - depends on model/runtime
            self.qwen_model = None
            raise RuntimeFailure("ASR_QWEN_MODEL_LOAD_FAILED", str(exc)) from exc

        self.loaded_model_id = model_id
        self.loaded_model_dir = model_dir
        self.loaded_device = "CUDA"

    def _load_fun(self, model_id: str, model_dir: Path) -> None:
        # The official Windows x64 binary is CPU-only.  This is intentional:
        # Fun-ASR-Nano is the conservative route for machines without a usable
        # NVIDIA/CUDA runtime.
        cli = self._fun_cli_path(model_dir)
        if not cli.is_file():
            raise RuntimeFailure("ASR_FUN_RUNTIME_NOT_INSTALLED")
        self.loaded_model_id = model_id
        self.loaded_model_dir = model_dir
        self.loaded_device = "CPU"

    def _fun_cli_path(self, model_dir: Path) -> Path:
        candidates: list[Path] = []
        configured = os.environ.get("VERILECTURE_FUN_ASR_CLI")
        if configured:
            candidates.append(Path(configured))
        executable_dir = Path(sys.executable).resolve().parent
        source_dir = Path(__file__).resolve().parent
        candidates.extend(
            [
                executable_dir / "llama-funasr-cli.exe",
                executable_dir / "binaries" / "llama-funasr-cli.exe",
                source_dir / "llama-funasr-cli.exe",
                model_dir / "runtime" / "llama-funasr-cli.exe",
                model_dir / "runtime" / "bin" / "llama-funasr-cli.exe",
                model_dir.parent / "llama-funasr-cli.exe",
                model_dir.parent / "binaries" / "llama-funasr-cli.exe",
            ]
        )
        for root in (model_dir, source_dir, executable_dir):
            if root.is_dir():
                candidates.extend(root.rglob("llama-funasr-cli.exe"))
        for candidate in candidates:
            if candidate.is_file():
                return candidate
        return candidates[0] if candidates else Path("llama-funasr-cli.exe")

    def load(self, request: dict) -> dict:
        model_id = _model_id(request)
        model_dir = _model_dir(request)
        _check_model_files(model_id, model_dir)
        device = str(request.get("device") or self.default_device).upper()
        if model_id == FUN_ASR_NANO:
            device = "CPU"

        if self.loaded_model_id == model_id and self.loaded_model_dir == model_dir:
            return {"ok": True, "runtimeVersion": RUNTIME_VERSION}
        self.unload()
        if model_id in (QWEN_ASR_17B, QWEN_ASR_06B):
            self._load_qwen(model_id, model_dir, device)
        else:
            self._load_fun(model_id, model_dir)
        return {
            "ok": True,
            "runtimeVersion": RUNTIME_VERSION,
            "executionDevice": self.loaded_device,
        }

    def unload(self) -> dict:
        self.qwen_model = None
        self.loaded_model_id = None
        self.loaded_model_dir = None
        self.loaded_device = None
        gc.collect()
        return {"ok": True, "runtimeVersion": RUNTIME_VERSION}

    def transcribe(self, request: dict) -> dict:
        model_id = _model_id(request)
        model_dir = _model_dir(request)
        _check_model_files(model_id, model_dir)
        if self.loaded_model_id != model_id or self.loaded_model_dir != model_dir:
            self.load(request)

        encoded = request.get("audioPcmF32LeBase64")
        if not isinstance(encoded, str) or not encoded:
            raise RuntimeFailure("ASR_AUDIO_PAYLOAD_MISSING")
        try:
            raw = base64.b64decode(encoded, validate=True)
        except Exception as exc:
            raise RuntimeFailure("ASR_AUDIO_PAYLOAD_INVALID") from exc
        if len(raw) % 4 != 0 or len(raw) < 640:
            raise RuntimeFailure("ASR_AUDIO_TOO_SHORT")

        if model_id == FUN_ASR_NANO:
            import array

            samples = array.array("f")
            samples.frombytes(raw)
            if sys.byteorder != "little":
                samples.byteswap()
        else:
            try:
                import numpy as np
            except Exception as exc:  # pragma: no cover - packaged dependency check
                raise RuntimeFailure("ASR_NUMPY_RUNTIME_IMPORT_FAILED", str(exc)) from exc
            samples = np.frombuffer(raw, dtype="<f4")
        sample_rate = 16000
        duration_ms = max(1, round(len(samples) * 1000 / sample_rate))
        language = request.get("language")
        if language == "translate_to_en":
            raise RuntimeFailure("ASR_TRANSLATION_NOT_SUPPORTED_LOCAL")
        language = language if isinstance(language, str) and language not in ("", "auto") else None

        if model_id == FUN_ASR_NANO:
            text = self._transcribe_fun(model_dir, samples, sample_rate)
            detected_language = language or "auto"
            timestamped_segments: list[dict] = []
        else:
            text, detected_language, timestamped_segments = self._transcribe_qwen(samples, language)

        if model_id != FUN_ASR_NANO and timestamped_segments:
            return {
                "ok": True,
                "runtimeVersion": RUNTIME_VERSION,
                "detectedLanguage": detected_language,
                "segments": timestamped_segments,
            }

        if not text.strip():
            return {
                "ok": True,
                "runtimeVersion": RUNTIME_VERSION,
                "detectedLanguage": detected_language,
                "segments": [],
            }
        return {
            "ok": True,
            "runtimeVersion": RUNTIME_VERSION,
            "detectedLanguage": detected_language,
            "segments": [
                {
                    "startMs": 0,
                    "endMs": duration_ms,
                    "text": text.strip(),
                    "language": detected_language,
                }
            ],
        }

    def _transcribe_qwen(self, samples, language: str | None) -> tuple[str, str, list[dict]]:
        if self.qwen_model is None:
            raise RuntimeFailure("ASR_MODEL_NOT_LOADED")
        try:
            with _quiet_stdout():
                outputs = self.qwen_model.transcribe(
                    audio=(samples, 16000),
                    language=_qwen_language(language),
                    return_time_stamps=True,
                )
        except Exception as exc:  # pragma: no cover - depends on CUDA/model
            raise RuntimeFailure("ASR_QWEN_TRANSCRIBE_FAILED", str(exc)) from exc

        if not outputs:
            return "", language or "auto", []
        result = outputs[0]
        if isinstance(result, (tuple, list)):
            detected = str(result[0] or language or "auto")
            text = str(result[1] if len(result) > 1 else "")
            return text, detected, []
        detected = str(getattr(result, "language", None) or language or "auto")
        text = str(getattr(result, "text", result))
        timestamped = _normalise_time_stamps(getattr(result, "time_stamps", None), detected)
        return text, detected, timestamped


    def _transcribe_fun(self, model_dir: Path, samples, sample_rate: int) -> str:
        cli = self._fun_cli_path(model_dir)
        audio_path: str | None = None
        try:
            with tempfile.NamedTemporaryFile(prefix="verilecture-asr-", suffix=".wav", delete=False) as handle:
                audio_path = handle.name
            with wave.open(audio_path, "wb") as wav:
                wav.setnchannels(1)
                wav.setsampwidth(2)
                wav.setframerate(sample_rate)
                pcm = bytearray()
                for sample in samples:
                    value = max(-1.0, min(1.0, float(sample)))
                    integer = int(value * 32767.0)
                    pcm.extend(integer.to_bytes(2, byteorder="little", signed=True))
                wav.writeframes(pcm)
            command = [
                str(cli),
                "--enc",
                str(model_dir / "funasr-encoder-f16.gguf"),
                "-m",
                str(model_dir / "qwen3-0.6b-q8_0.gguf"),
                "--vad",
                str(model_dir / "fsmn-vad.gguf"),
                "-a",
                audio_path,
            ]
            completed = subprocess.run(command, capture_output=True, check=False)
            if completed.returncode != 0:
                diagnostics = _decode_cli_text(completed.stderr)
                raise RuntimeFailure("ASR_FUN_TRANSCRIBE_FAILED", diagnostics)
            output = _decode_cli_text(completed.stdout)
            lines = [line.strip() for line in output.splitlines() if line.strip()]
            return " ".join(lines)
        finally:
            if audio_path:
                try:
                    Path(audio_path).unlink(missing_ok=True)
                except OSError:
                    pass


def _probe_cuda() -> int:
    try:
        import torch

        runtime_detected = bool(torch.version.cuda)
        usable = bool(runtime_detected and torch.cuda.is_available())
        if usable:
            torch.cuda.get_device_name(0)
        print(f"CUDA_RUNTIME_DETECTED={'1' if runtime_detected else '0'}")
        print(f"ASR_CUDA_USABLE={'1' if usable else '0'}")
        if torch.version.cuda:
            print(f"CUDA_VERSION={torch.version.cuda}")
        return 0
    except Exception as exc:
        print("CUDA_RUNTIME_DETECTED=0")
        print("ASR_CUDA_USABLE=0")
        print(f"ASR_CUDA_PROBE_ERROR={type(exc).__name__}")
        return 0


def main() -> int:
    parser = argparse.ArgumentParser(add_help=True)
    parser.add_argument("--probe-cuda", action="store_true")
    parser.add_argument("--protocol", default="json-lines")
    parser.add_argument("--model-id", default=FUN_ASR_NANO)
    parser.add_argument("--model-dir", default=".")
    parser.add_argument("--device", default="CPU")
    args = parser.parse_args()
    if args.probe_cuda:
        return _probe_cuda()
    if args.protocol != "json-lines":
        print("unsupported protocol", file=sys.stderr)
        return 2

    runtime = Runtime(args.model_id, Path(args.model_dir), args.device)
    for raw_line in sys.stdin:
        if not raw_line.strip():
            continue
        request: object = None
        try:
            request = json.loads(raw_line)
            operation = request.get("operation")
            if operation == "load":
                response = runtime.load(request)
            elif operation == "transcribe":
                response = runtime.transcribe(request)
            elif operation == "unload":
                response = runtime.unload()
            elif operation == "heartbeat":
                response = {"ok": True, "runtimeVersion": RUNTIME_VERSION, "heartbeat": True}
            else:
                response = _error_response("ASR_OPERATION_UNSUPPORTED")
        except RuntimeFailure as exc:
            response = _error_response(exc.error_code, str(exc))
        except Exception as exc:  # Keep protocol alive for the next request.
            response = _error_response("ASR_RUNTIME_REQUEST_FAILED", str(exc))
        if isinstance(request, dict) and isinstance(request.get("requestId"), str):
            response["requestId"] = request["requestId"]
        _write_response(response)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
