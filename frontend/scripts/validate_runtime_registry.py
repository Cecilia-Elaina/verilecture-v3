#!/usr/bin/env python3
"""Validate the runtime registry and optionally a local runtime archive.

This validator intentionally performs no network requests. A registry entry may
point at a not-yet-published mirror, but an installed runtime is never trusted
until its local size, SHA-256, manifest, and CUDA probe have all passed in the
application.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path
from urllib.parse import urlparse


SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
ID_RE = re.compile(r"^[a-z0-9][a-z0-9._-]+$")
ALLOWED_CHANNELS = {"alpha", "stable"}
ALLOWED_STATUSES = {"published", "pending-publication", "disabled"}
ALLOWED_PLATFORMS = {"windows", "linux", "macos"}
ALLOWED_ARCHITECTURES = {"x86_64", "aarch64"}


class ValidationError(Exception):
    """A user-actionable registry validation error."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValidationError(message)


def load_registry(path: Path) -> dict:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ValidationError(f"registry not found: {path}") from exc
    except json.JSONDecodeError as exc:
        raise ValidationError(f"invalid JSON at {path}: {exc}") from exc

    require(isinstance(value, dict), "registry root must be an object")
    return value


def validate_registry(registry: dict) -> list[dict]:
    require(registry.get("schemaVersion") == 1, "schemaVersion must be 1")
    require(isinstance(registry.get("registryVersion"), str), "registryVersion must be a string")
    require(bool(registry["registryVersion"].strip()), "registryVersion must not be empty")
    require(isinstance(registry.get("generatedAt"), str), "generatedAt must be a string")
    require(re.fullmatch(r"\d{4}-\d{2}-\d{2}", registry["generatedAt"]) is not None, "generatedAt must be YYYY-MM-DD")
    require(registry.get("defaultChannel") in ALLOWED_CHANNELS, "defaultChannel must be alpha or stable")

    runtimes = registry.get("runtimes")
    require(isinstance(runtimes, list) and runtimes, "runtimes must be a non-empty array")

    seen_runtime_keys: set[tuple[str, str, str]] = set()
    for runtime in runtimes:
        require(isinstance(runtime, dict), "each runtime must be an object")
        runtime_id = runtime.get("id")
        require(isinstance(runtime_id, str) and ID_RE.fullmatch(runtime_id) is not None, f"invalid runtime id: {runtime_id!r}")
        for field in ("version", "artifactName", "cudaVersion"):
            require(isinstance(runtime.get(field), str) and runtime[field].strip(), f"{runtime_id}: {field} must be a non-empty string")
        require(runtime.get("channel") in ALLOWED_CHANNELS, f"{runtime_id}: invalid channel")
        platform = runtime.get("platform")
        architecture = runtime.get("architecture")
        require(platform in ALLOWED_PLATFORMS, f"{runtime_id}: unsupported platform")
        require(architecture in ALLOWED_ARCHITECTURES, f"{runtime_id}: unsupported architecture")
        runtime_key = (runtime_id, platform, architecture)
        require(runtime_key not in seen_runtime_keys, f"duplicate runtime target: {runtime_id}/{platform}/{architecture}")
        seen_runtime_keys.add(runtime_key)
        require(runtime["artifactName"].lower().endswith(".zip"), f"{runtime_id}: artifactName must end in .zip")

        for field in ("compressedBytes", "installedBytes"):
            require(isinstance(runtime.get(field), int) and not isinstance(runtime[field], bool), f"{runtime_id}: {field} must be an integer")
            require(runtime[field] > 0 if field == "compressedBytes" else runtime[field] >= 0, f"{runtime_id}: invalid {field}")
        require(isinstance(runtime.get("sha256"), str) and SHA256_RE.fullmatch(runtime["sha256"]) is not None, f"{runtime_id}: sha256 must be 64 lowercase hex characters")
        require(re.fullmatch(r"\d+\.\d+", runtime["cudaVersion"]) is not None, f"{runtime_id}: invalid cudaVersion")

        models = runtime.get("models")
        require(isinstance(models, list) and models and all(isinstance(model, str) and model.strip() for model in models), f"{runtime_id}: models must be a non-empty string array")
        require(len(models) == len(set(models)), f"{runtime_id}: duplicate model id")

        requirements = runtime.get("requirements")
        require(isinstance(requirements, dict), f"{runtime_id}: requirements must be an object")
        require(requirements.get("nvidia") is True, f"{runtime_id}: CUDA runtime must require NVIDIA")
        for field in ("minVramBytes", "minRamBytes"):
            value = requirements.get(field)
            require(isinstance(value, int) and not isinstance(value, bool) and value > 0, f"{runtime_id}: {field} must be a positive integer")

        status = runtime.get("status")
        require(status in ALLOWED_STATUSES, f"{runtime_id}: invalid runtime status")

        mirrors = runtime.get("mirrors")
        require(isinstance(mirrors, list) and mirrors, f"{runtime_id}: mirrors must be a non-empty array")
        seen_mirror_ids: set[str] = set()
        published_mirrors = 0
        for mirror in mirrors:
            require(isinstance(mirror, dict), f"{runtime_id}: each mirror must be an object")
            mirror_id = mirror.get("id")
            require(isinstance(mirror_id, str) and ID_RE.fullmatch(mirror_id) is not None, f"{runtime_id}: invalid mirror id")
            require(mirror_id not in seen_mirror_ids, f"{runtime_id}: duplicate mirror id {mirror_id}")
            seen_mirror_ids.add(mirror_id)
            priority = mirror.get("priority")
            require(isinstance(priority, int) and not isinstance(priority, bool) and priority >= 0, f"{runtime_id}/{mirror_id}: priority must be a non-negative integer")
            url = mirror.get("url")
            require(isinstance(url, str) and url.strip(), f"{runtime_id}/{mirror_id}: url must not be empty")
            parsed_url = urlparse(url)
            require(parsed_url.scheme in {"http", "https"} and bool(parsed_url.netloc), f"{runtime_id}/{mirror_id}: url must be http(s)")
            mirror_status = mirror.get("status")
            require(mirror_status in ALLOWED_STATUSES, f"{runtime_id}/{mirror_id}: invalid mirror status")
            if mirror_status == "published":
                published_mirrors += 1

        if status == "published":
            require(published_mirrors > 0, f"{runtime_id}: published runtime needs a published mirror")
        if status == "pending-publication":
            require(published_mirrors == 0, f"{runtime_id}: pending-publication runtime cannot have a published mirror")

    return runtimes


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def validate_asset(path: Path, runtime: dict) -> None:
    require(path.is_file(), f"asset not found: {path}")
    actual_bytes = path.stat().st_size
    require(actual_bytes == runtime["compressedBytes"], f"asset size mismatch: expected {runtime['compressedBytes']}, got {actual_bytes}")
    actual_sha = sha256_file(path)
    require(actual_sha == runtime["sha256"], f"asset SHA-256 mismatch: expected {runtime['sha256']}, got {actual_sha}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--registry",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "src-tauri" / "resources" / "runtime_registry.json",
    )
    parser.add_argument("--asset", type=Path, help="optional local runtime archive to verify")
    parser.add_argument("--runtime-id", help="runtime id to use with --asset; defaults to the only entry")
    args = parser.parse_args()

    try:
        registry = load_registry(args.registry)
        runtimes = validate_registry(registry)
        runtime = None
        if args.asset:
            if args.runtime_id:
                runtime = next((item for item in runtimes if item["id"] == args.runtime_id), None)
                require(runtime is not None, f"runtime id not found: {args.runtime_id}")
            else:
                require(len(runtimes) == 1, "--runtime-id is required when the registry has multiple runtimes")
                runtime = runtimes[0]
            validate_asset(args.asset, runtime)
    except ValidationError as exc:
        print(f"INVALID: {exc}", file=sys.stderr)
        return 1

    pending = [item["id"] for item in runtimes if item["status"] == "pending-publication"]
    print(f"VALID: {len(runtimes)} runtime entr{'y' if len(runtimes) == 1 else 'ies'}")
    if pending:
        print(f"PENDING_PUBLICATION: {', '.join(pending)}")
    if args.asset:
        print(f"ASSET_VALID: {runtime['id']} ({runtime['compressedBytes']} bytes, sha256={runtime['sha256']})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
