#!/usr/bin/env python3
"""Exercise the real CUDA runtime archive through the local Range server.

The public Registry remains ``pending-publication``. This script is an explicit
test-only local source override: it reads the Registry metadata, serves the
already-built archive from localhost, resumes an interrupted download, verifies
size/SHA-256, performs an atomic staging rename, and runs the real sidecar
probe. It never changes the production Registry and never contacts GitHub.
"""

from __future__ import annotations

import argparse
import hashlib
import http.client
import json
import os
import shutil
import subprocess
import urllib.error
import urllib.request
import zipfile
from pathlib import Path

from mock_runtime_server import MockRuntimeHTTPServer


class InterruptedDownload(Exception):
    pass


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def download_to_part(url: str, part: Path) -> tuple[bool, int]:
    existing = part.stat().st_size if part.exists() else 0
    request = urllib.request.Request(url)
    if existing:
        request.add_header("Range", f"bytes={existing}-")
    response = urllib.request.urlopen(request, timeout=30)
    append = existing > 0 and response.status == 206
    if existing and not append:
        existing = 0
    mode = "ab" if append else "wb"
    downloaded = existing
    with part.open(mode) as stream:
        try:
            while True:
                chunk = response.read(1024 * 1024)
                if not chunk:
                    break
                stream.write(chunk)
                downloaded += len(chunk)
        except http.client.IncompleteRead as exc:  # type: ignore[name-defined]
            stream.write(exc.partial)
            downloaded += len(exc.partial)
            raise InterruptedDownload from exc
    return append, downloaded


def safe_extract(archive: Path, destination: Path) -> int:
    destination.mkdir(parents=True, exist_ok=True)
    root = destination.resolve()
    count = 0
    with zipfile.ZipFile(archive) as zipped:
        for info in zipped.infolist():
            target = (destination / info.filename).resolve()
            if target != root and root not in target.parents:
                raise RuntimeError(f"unsafe archive path: {info.filename}")
            if info.is_dir():
                target.mkdir(parents=True, exist_ok=True)
                continue
            target.parent.mkdir(parents=True, exist_ok=True)
            with zipped.open(info) as source, target.open("wb") as output:
                shutil.copyfileobj(source, output, length=1024 * 1024)
            count += 1
    return count


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--asset",
        type=Path,
        default=Path(r"D:\Download\verilecture-v3\output\release-assets\verilecture-asr-runtime-cuda-qwen-fun-windows-x64.zip"),
    )
    parser.add_argument(
        "--keep",
        action="store_true",
        help="keep the downloaded archive and extracted runtime after the acceptance run",
    )
    args = parser.parse_args()

    registry_path = Path(__file__).parents[1] / "src-tauri" / "resources" / "runtime_registry.json"
    registry = json.loads(registry_path.read_text(encoding="utf-8"))
    runtime = next(item for item in registry["runtimes"] if item["id"] == "cuda-qwen-fun")
    if runtime["status"] != "pending-publication":
        raise RuntimeError("the local acceptance must not silently replace a published source")
    if not args.asset.is_file():
        raise FileNotFoundError(args.asset)

    root = Path(__file__).parents[1] / "local-acceptance" / "runtime-chain"
    if root.exists():
        shutil.rmtree(root)
    part = root / f"{runtime['artifactName']}.part"
    staging = root / "runtimes" / runtime["id"] / f".{runtime['version']}.installing"
    destination = root / "runtimes" / runtime["id"] / runtime["version"]
    root.mkdir(parents=True, exist_ok=True)
    interrupted = False
    range_used = False
    extracted_files = 0
    probe_output = ""
    try:
        with MockRuntimeHTTPServer(args.asset, drop_first_bytes=8 * 1024 * 1024) as server:
            try:
                _first_append, first_bytes = download_to_part(server.url, part)
                interrupted = first_bytes < runtime["compressedBytes"]
            except InterruptedDownload:
                interrupted = True
            append, downloaded_bytes = download_to_part(server.url, part)
            range_used = append
            if downloaded_bytes != runtime["compressedBytes"]:
                raise RuntimeError(f"size mismatch after resume: {downloaded_bytes}")
            if sha256_file(part) != runtime["sha256"]:
                raise RuntimeError("SHA-256 mismatch after resume")
            request_ranges = [item["rangeStart"] for item in server.requests]

        staging.mkdir(parents=True, exist_ok=True)
        extracted_files = safe_extract(part, staging)
        sidecar = staging / "verilecture-asr-runtime.exe"
        if not sidecar.is_file():
            raise RuntimeError("runtime sidecar missing after extraction")
        probe = subprocess.run(
            [str(sidecar), "--probe-cuda"],
            cwd=staging,
            capture_output=True,
            text=True,
            timeout=120,
            check=False,
        )
        probe_output = (probe.stdout + probe.stderr).strip()
        if probe.returncode != 0 or "ASR_CUDA_USABLE=1" not in probe.stdout:
            raise RuntimeError(f"CUDA probe failed: {probe_output}")

        destination.parent.mkdir(parents=True, exist_ok=True)
        if destination.exists():
            raise RuntimeError("acceptance destination unexpectedly exists")
        os.replace(staging, destination)
        if not destination.joinpath("verilecture-asr-runtime.exe").is_file():
            raise RuntimeError("atomic install destination missing sidecar")

        report = {
            "registryStatus": runtime["status"],
            "registrySourceOverriddenForTest": True,
            "asset": str(args.asset),
            "compressedBytes": downloaded_bytes,
            "sha256": runtime["sha256"],
            "interrupted": interrupted,
            "rangeResume": range_used,
            "requestRanges": request_ranges,
            "extractedFiles": extracted_files,
            "atomicInstall": True,
            "cudaProbe": "ASR_CUDA_USABLE=1",
        }
        (root.parent / "runtime-chain-report.json").write_text(
            json.dumps(report, indent=2), encoding="utf-8"
        )
        print(json.dumps(report, ensure_ascii=False))
        return 0
    finally:
        if not args.keep and root.exists():
            shutil.rmtree(root)


if __name__ == "__main__":
    raise SystemExit(main())
