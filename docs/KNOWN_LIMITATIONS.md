# Known limitations

- `v0.3.0-alpha.2` is an Alpha release with native Windows, Linux, and macOS
  desktop packages.
- The bundled Fun-ASR and CUDA sidecars are currently Windows x64 only. Linux and
  macOS packages can build and launch the desktop shell, but local ASR remains
  unavailable until native sidecars are published and tested.
- Qwen3-ASR-0.6B and Qwen3-ASR-1.7B require a compatible NVIDIA CUDA path and a
  separately hosted CUDA Runtime. The runtime registry remains gated until its
  public HTTPS artifact is verified.
- Model weights are not bundled. First use requires a network connection, free
  disk space, and a complete integrity check.
- Long audio is processed as a bounded local job; durable in-job resume is not
  exposed in the current interface.
- Text-model features are optional and require a user-configured provider plus
  explicit consent. Broader provider and long-document coverage is still being
  expanded.
- The Windows installer is unsigned unless a signing configuration is supplied
  by the release environment.
