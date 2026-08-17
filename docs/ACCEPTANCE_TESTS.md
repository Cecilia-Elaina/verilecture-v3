# Acceptance checklist

1. First launch cannot bypass hardware scan, model verification/smoke test and
   the configured provider gate in a real build.
2. No CUDA machine recommends and runs Fun-ASR-Nano CPU; no Qwen fallback is
   hidden.
3. GPU machine with verified 8 GiB+ VRAM can select Qwen 1.7B; the final RTX 3060
   external-GPU smoke has passed with `executionDevice=CUDA`.
4. Chinese audio with explicit `zh` remains Chinese and its timestamps are
   monotonic global milliseconds.
5. Import never deletes or modifies the source audio.
6. Raw transcript remains unchanged after lexicon calibration or editing.
7. Textbook full file/full text never appears in a cloud request; audit rows
   remain below the hard cap.
8. Exam points reference existing transcript segments, valid chapter IDs or
   `unmatched`, and valid audio ranges.
9. English and Simplified Chinese contain no untranslated functional UI copy.
10. Windows installer opens the first-run flow in a clean user directory.
11. Runtime Registry parses successfully; a pending-publication entry is
    rejected by the production downloader with
    `MODEL_RUNTIME_SOURCE_UNAVAILABLE`.
12. The local Mock HTTP server covers full response, HTTP Range, interrupted
    response, resume, 404, size mismatch and SHA-256 mismatch. The fixture
    tests are separate from the real-asset acceptance.
13. The real 4.6 GB CUDA archive has completed a localhost interrupted download
    and Range resume, exact size/SHA-256 verification, Zip64 extraction of 6252
    files, atomic staging promotion and `ASR_CUDA_USABLE=1` probe.
14. Production release builds do not honor
    `VERILECTURE_RUNTIME_REGISTRY_OVERRIDE` and do not contain a fixed CUDA
    download URL.
