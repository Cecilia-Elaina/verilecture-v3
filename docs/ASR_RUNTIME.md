# ASR runtime

`tools/asr/verilecture_asr_runtime.py` is a JSON-lines development sidecar.
It accepts `load`, `transcribe`, `unload`, and `heartbeat` operations. Qwen is
loaded only on CUDA and receives the local ForcedAligner directory. Fun-ASR is
forced to CPU and invokes the official `llama-funasr-cli` runtime.

Qwen chunks are limited to 20 seconds by the Rust audio pipeline, well below
the official five-minute alignment window. Returned relative timestamps are
translated to global millisecond timestamps before persistence.

Production packaging requires a locked embedded Python bundle described by
`src-tauri/resources/asr-runtime/runtime-manifest.json`. The build script
records the runtime flavor and SHA-256/byte size for every staged runtime file;
the CUDA flavor must be built from the CUDA-specific requirements file. The
CPU/Fun flavor is embedded in the NSIS installer. The CUDA flavor is a pinned
Zip64 asset resolved from `src-tauri/resources/runtime_registry.json` and
downloaded into the versioned per-user
`runtimes/cuda-qwen-fun/<runtime-version>` directory before a Qwen model is
installed. The production downloader accepts only `published` Registry
sources, supports Range resume, checks exact size and SHA-256, extracts into a
staging directory, and requires the extracted manifest plus `--probe-cuda` to
pass before an atomic swap. A `pending-publication` source returns
`MODEL_RUNTIME_SOURCE_UNAVAILABLE`; it is never treated as READY.
Development can use `VERILECTURE_DEV_PYTHON`; a release build fails closed when
the bundled runtime is absent rather than silently launching a system Python.
The Fun-ASR official Windows runtime remains a model-managed executable and is
also hash-verified.

The Registry has no fixed CUDA URL in Rust. `ModelDefinition.runtime_bundle_id`
resolves to a Registry entry and then to its prioritized published mirrors.
`VERILECTURE_RUNTIME_REGISTRY_OVERRIDE` is accepted only by debug/test builds
for the local Mock HTTP acceptance scripts; release builds always use the
embedded Registry resource.

Qwen's timestamp response is a `ForcedAlignResult` wrapper with an `items`
collection rather than a bare list. The Python adapter normalizes both forms
into the product's monotonic millisecond segment schema.
