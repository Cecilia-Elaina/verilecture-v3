# Known limitations

- Qwen3-ASR-1.7B plus ForcedAligner has passed the final direct CUDA smoke on the
  connected RTX 3060 12 GiB machine: CUDA probe, CUDA load, Chinese
  transcription with timestamped segments and unload all passed. Qwen3-ASR-0.6B
  uses the same CUDA Runtime, ForcedAligner and sidecar protocol but was not
  repeated in this final run by scope. The real CUDA archive has also passed a
  localhost interrupted-download/Range-resume, size/SHA-256, Zip64 extraction,
  atomic-install and CUDA-probe acceptance.
  A clean Windows user-profile run through the published external source is
  still not performed because the Registry source remains
  `pending-publication`.
- Fun-ASR-Nano's official GGUF route is currently represented by VAD/segment
  ranges; the product does not label those ranges as Qwen ForcedAligner word
  timestamps.
- The local release resource tree contains a PyInstaller CPU sidecar, the
  official Fun CLI, and a GPLv3 FFmpeg build with notices. The CUDA runtime is
  intentionally a separate approximately 4.6 GB release asset because NSIS
  cannot create a single installer containing that resource tree. Public
  distribution still requires a licensing/source-offer review.
- Audio import currently performs decoding and sidecar work as one bounded
  local job. Durable checkpoint/resume within a long transcription job is not
  yet exposed in the UI.
- Live provider calls, textbook excerpt authorization and long-document
  performance need acceptance testing with user-controlled data.
- Model weights are never bundled in the installer; first launch requires a
  verified download and available disk space.
- Runtime Registry integration is local and complete. Public Release hosting
  and clean-machine download acceptance are intentionally deferred; no
  GitHub/Git operation is performed by this workspace.
- The startup `READY.json` check is intentionally a quick size/mtime gate;
  full SHA-256 verification is performed during installation and explicit
  integrity repair, not on every launch.
- The final installer has been installed into an isolated workspace directory,
  its CPU resource tree and sidecar have been verified, and its uninstaller
  has removed that test directory. A fresh Windows user-profile acceptance run
  including published runtime/model download, non-WAV import and data
  preservation remains outstanding.
