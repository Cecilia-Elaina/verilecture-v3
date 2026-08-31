# Third-party notices

VeriLecture V3 is distributed under the MIT license. The application does
not bundle multi-gigabyte model weights; the model manager downloads the
following pinned artifacts only after the user starts installation. The
license text for redistribution-sensitive components is kept in
`frontend/src-tauri/resources/licenses/`.

| Component | Use | License / source |
| --- | --- | --- |
| Qwen3-ASR 0.6B/1.7B | Local CUDA speech recognition | Apache-2.0 · `QwenLM/Qwen3-ASR` |
| Qwen3-ForcedAligner-0.6B | Qwen timestamp alignment | Apache-2.0 · `QwenLM/Qwen3-ASR` |
| Fun-ASR-Nano-2512 | CPU/edge speech recognition and VAD | Apache-2.0 · `FunAudioLLM/Fun-ASR-Nano-2512` |
| FFmpeg | Non-WAV decode and resampling support | LGPL/GPL build terms depend on the exact binary; see the bundled FFmpeg notices |
| PyTorch | CUDA runtime used by the Qwen sidecar | BSD-style/PyTorch distribution notices |
| Transformers / qwen-asr | Qwen sidecar Python dependencies | Their upstream licenses and notices |
| PyInstaller | Embedded Windows sidecar packaging | GPL with the applicable bootloader exception |
| Tauri 2 | Windows desktop shell and IPC | MIT / Apache-2.0 components |
| Meetily Community Edition | Historical architecture reference only | MIT attribution retained; not the current maintainer or product |

Upstream project pages and immutable model revisions are recorded in
`docs/MODEL_REGISTRY.md`. This file is a notice index, not a replacement for
the upstream license texts.
