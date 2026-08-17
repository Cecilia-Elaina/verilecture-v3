# Implementation plan

## Completed in the current implementation pass

1. Created the independent V3 project, product contract, attribution and
   migration-backed settings.
2. Implemented the fixed three-model catalog and hardware routing policy.
3. Implemented resumable model downloads, SHA-256 verification, safe ZIP
   extraction, atomic installation and sidecar smoke-test gating.
4. Implemented local audio import, WAV decoding, bundled-runtime lookup,
   resampling, VAD ranges, chunk offsets and raw/calibrated transcript storage.
5. Implemented textbook parsing, lexicon versioning, task binding and
   terminology calibration without rewriting raw data.
6. Implemented provider protocols, consent, keyring storage, structured
   map/reduce exam-point analysis, validation, audit metadata and export.
7. Completed the bilingual shell, record player/detail view, settings/about
   screens and browser-preview guardrails.
8. Completed the versioned lexicon editor, independent textbook/structured-data
   consent gates, hardware proxy diagnostics and ModelScope-first download
   fallback with integrity repair.

## Completed release gates in this pass

- Built the CPU-only sidecar with the v3-controlled Python 3.12/PyInstaller
  environment and staged the official Fun CLI plus FFmpeg notices.
- Ran real Fun-ASR-Nano CPU inference on a Chinese speech sample, both from the
  staged sidecar and from the installed resource tree.
- Built the NSIS Debug/Release packages and installed the final Release package
  into an isolated workspace directory; verified the runtime, model registry,
  FFmpeg and license assets are present.
- Completed the local bilingual browser pass at 1024x720, including the fixed
  hardware-scan-to-model-selection transition, provider preview, lexicon
  editor save, consent gate and first-run tutorial next/skip flow.

## Remaining release gates

1. Run first-launch, model download/resume/cancel, import, playback, export and
   data-preservation tests in a fresh Windows user profile. The isolated
   installer/uninstaller smoke is already complete.
2. On a separate NVIDIA machine, verify the 1.7B and 0.6B static routing,
   actual model load, ForcedAligner timestamps, memory behavior and the
   no-fallback failure states.
3. Run live user-supplied text-provider calls and long textbook/consent cases.
4. Complete the GPLv3 FFmpeg public-distribution/source-offer review before
   publishing any installer outside the local test scope.

Cloud text-provider integration remains implemented but is intentionally not a
release acceptance dependency for the current local model verification pass.
Do not publish or perform Git/GitHub operations in this task.
