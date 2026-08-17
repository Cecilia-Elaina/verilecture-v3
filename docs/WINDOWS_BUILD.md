# Windows build

## Local checks

```powershell
cd D:\Download\verilecture\verilecture-v3-workspace
pnpm typecheck
pnpm test -- --run
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri build --debug
pnpm tauri build
```

The default Tauri bundle targets NSIS with current-user installation. MSI is not
the default target because Windows Installer requires a numeric-only
pre-release version while the product contract intentionally uses
`0.3.0-alpha.1`; create a separate numeric packaging version only if an MSI is
specifically needed. Model weights are intentionally not bundled. The NSIS
installer includes the CPU/Fun runtime, official Fun-ASR CLI and FFmpeg under
the resource manifest; release code does not fall back to PATH Python or PATH
FFmpeg. The runtime manifest records the staged files, sizes and SHA-256
values.

The CUDA PyTorch runtime is intentionally a separate release asset. The
verified local asset is:

```text
verilecture-asr-runtime-cuda-qwen-fun-windows-x64.zip
bytes: 4617121514
sha256: 4eafd198228821c9f5ca36ebd62a4ded53df6083ff1c3f8283127a8f9bc9a665
```

The application resolves this asset through
`src-tauri/resources/runtime_registry.json`, accepts only published mirrors,
verifies the archive, extracts it into the versioned per-user runtime
directory, validates its manifest and runs the CUDA probe. This split is required because NSIS fails
when asked to map the approximately 4.6 GB CUDA resource tree into one
installer executable.

Use `tools/asr/build_windows_runtime.ps1` from the controlled runtime build
environment. The verified v3 CPU build used:

```powershell
powershell -ExecutionPolicy Bypass -File tools/asr/build_windows_runtime.ps1 `
  -PythonPath output\asr-build-venv\Scripts\python.exe `
  -FunCliPath <path-to-official>\llama-funasr-cli.exe `
  -CpuOnly
```

Set `VERILECTURE_PYTHON` and `VERILECTURE_FUN_ASR_CLI` when using the script's
environment-variable form, and stage the output into
`src-tauri/resources/asr-runtime`. Stage a verified `ffmpeg.exe` at
`src-tauri/resources/ffmpeg/ffmpeg.exe` with its notice before building an
installer.

For a complete three-tier build, use a separate Python 3.12 environment and
install `tools/asr/requirements-windows-cuda.txt`. It pins the CUDA 12.6
PyTorch wheel (`torch==2.7.1+cu126`); the ordinary PyPI `torch==2.7.1` wheel is
CPU-only on Windows and is rejected by the build script when `-CpuOnly` is not
provided. Then run the same script without `-CpuOnly` into a staging directory,
and create the external Zip64 asset from that directory. The local build used
`output/asr-runtime-staged-cuda-20260731` and was smoke-tested on an RTX 3060
12 GiB machine. The archive is not copied into `src-tauri/resources`, because
doing so makes NSIS fail at the 2 GB installer boundary.

Before calling a build release-ready, test install, first launch, hardware
scan, runtime download, model download, audio import, playback, export,
uninstall and data preservation in a fresh Windows user directory. The current
Release installer has passed isolated install, resource/Registry inspection,
the installed Fun CPU sidecar (`load -> transcribe -> unload`) with the staged
Chinese test model, and uninstall. The Qwen 1.7B and 0.6B sidecar/model paths
passed direct CUDA smoke outside the installer. The real runtime archive also
passed the local Mock HTTP interrupted download, Range resume, Zip64 extraction,
atomic install and probe chain. A build that lacks the runtime or decoder must
fail closed and must not display READY.

The current Release installer is:

```text
src-tauri/target/release/bundle/nsis/课溯 · VeriLecture_0.3.0-alpha.1_x64-setup.exe
bytes: 71416311
sha256: f7ed3d891729217877bd631ef317b759409fe4ead868de74a13f367c4c7cbd26
```

The local-chain commands are:

```powershell
python -B scripts\validate_runtime_registry.py --asset D:\Download\verilecture-v3\output\release-assets\verilecture-asr-runtime-cuda-qwen-fun-windows-x64.zip
python -B scripts\test_runtime_download_chain.py
python -B scripts\accept_local_runtime.py
```

This task builds locally only. Do not publish a release or perform Git/GitHub
operations.
