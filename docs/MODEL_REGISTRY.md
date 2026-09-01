# Model sources

The application ships a fixed registry at
`frontend/src-tauri/resources/model-registry.json`. It defines the supported
ASR tiers and keeps downloads tied to immutable source revisions.

The two Qwen profiles use a shared Windows x64 CUDA Runtime; Fun-ASR-Nano stays
on the Windows x64 CPU path. Linux and macOS package builds do not advertise a
local ASR tier until native sidecars are published. Downloads are checked by byte
count and SHA-256 before installation.

Official sources:

- [Qwen3-ASR-1.7B](https://huggingface.co/Qwen/Qwen3-ASR-1.7B), revision
  `7278e1e70fe206f11671096ffdd38061171dd6e`.
- [Qwen3-ASR-0.6B](https://huggingface.co/Qwen/Qwen3-ASR-0.6B), revision
  `5eb144179a02acc5e5ba31e748d22b0cf3e303b0`.
- [Qwen3-ForcedAligner-0.6B](https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B),
  revision `c7cbfc2048c462b0d63a45797104fc9db3ad62b7`.
- [Fun-ASR-Nano GGUF](https://huggingface.co/FunAudioLLM/Fun-ASR-Nano-GGUF),
  revision `46e849502a867080d66d351b8dfb1018b607e509`.
- [Fun-ASR releases](https://github.com/QwenAudio/Fun-ASR/releases), runtime
  `runtime-llamacpp-v0.1.9`.

The registry records ModelScope mirrors for mainland-China network conditions
and Hugging Face as the pinned-source fallback. A mirror is accepted only when
it resolves to the expected byte count and SHA-256.

Large files support HTTP Range resume and are installed into a staging
directory. A model is marked ready only after its runtime checks pass.
