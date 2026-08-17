# Model registry

The source registry is `src-tauri/resources/model-registry.json`; the Rust
downloader contains the same immutable artifact records so a missing or
modified resource cannot silently broaden the product model set.

Each model profile also declares `runtimeBundleId`. The two Qwen profiles use
`cuda-qwen-fun`; Fun-ASR-Nano uses no CUDA bundle and stays on the CPU path.
The bundle's URL, size, SHA-256, version and mirror status are defined only in
`src-tauri/resources/runtime_registry.json`. See
[`RUNTIME_REGISTRY_DESIGN.md`](RUNTIME_REGISTRY_DESIGN.md) for the resolver and
atomic installation contract.

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

The JSON registry records ModelScope repository mirrors for mainland-China
network conditions and Hugging Face as the pinned-source fallback. For files
available on both services, the downloader tries the ModelScope `master`
artifact URL first and falls back to the pinned Hugging Face URL when the
mirror is unavailable. A mirror is only accepted when it resolves to the same
expected byte count and SHA-256. The current downloader also honors system
proxy environment variables without persisting proxy values.

Large files are checked by byte count and streaming SHA-256. Downloads support
HTTP Range and `.part` files, then install into a staging directory. `READY`
is written only after runtime load and a smoke test pass.
