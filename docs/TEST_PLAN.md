# Test plan

- Frontend: `pnpm typecheck`, `pnpm test`, `pnpm build`.
- Rust: `cargo fmt --check`, `cargo check`, `cargo test`.
- Python: `python -m py_compile tools/asr/verilecture_asr_runtime.py` and
  JSON-lines contract fixtures without model weights; use
  `tools/asr/smoke_qwen_cuda.py` only on a CUDA-capable acceptance machine
  with the pinned Qwen weights. The current RTX 3060 run passed the Qwen 1.7B
  + ForcedAligner path with CUDA 12.6.
- Provider/runtime fixtures: `python tools/test_provider_mock.py` and
  `python tools/asr/test_runtime_protocol.py`.
- Hardware policy fixtures: 8 GiB GPU, 6 GiB GPU, no CUDA, unknown probe.
- Model manager: mock HTTP Range resume, checksum mismatch, cancellation,
  staging cleanup, registry/static-artifact parity, filesystem-signature
  invalidation and no premature READY.
- Audio: WAV/non-WAV, Chinese paths, silent input, timestamp monotonicity and
  raw/calibrated immutability.
- Provider: four protocols, auth/timeout/rate-limit/invalid JSON and keyring
  redaction with fake keys only; every failed lexicon attempt keeps its run and
  selected-excerpt audit rows.
- Lexicon: text PDF/DOCX/PPTX/TXT/Markdown, scanned-PDF fail closed and 10% /
  120000-character cap.
- Packaging: Tauri debug, release, NSIS/MSI where the Windows toolchain is
  available, plus isolated install/uninstall smoke. The current release has a
  direct installed Fun CPU sidecar `load -> transcribe -> unload` check. The
  CUDA runtime is tested as a separate Zip64 asset; first-run runtime/model
  download and non-WAV acceptance remain open.
- UI: About view exposes all bundled third-party license/attribution surfaces
  in Simplified Chinese and English.
