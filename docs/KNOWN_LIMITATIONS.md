# Known limitations

- VeriLecture V3 is an alpha Windows x64 release. Compatibility can vary with
  Windows scaling settings, GPU drivers, and available storage.
- Fun-ASR-Nano-2512 provides the CPU path. The Qwen tiers require a compatible
  NVIDIA GPU and a separately hosted CUDA Runtime; Qwen installation remains
  unavailable until that runtime is publicly available.
- Qwen3-ASR-1.7B has passed a representative CUDA smoke test on an RTX 3060
  12 GiB system. Qwen3-ASR-0.6B uses the same runtime path but has less
  independent hardware coverage.
- Model weights are not bundled in the installer. The first model installation
  needs a network connection, free disk space, and a complete integrity check.
- Long audio files are processed as bounded local jobs; durable in-job resume
  is not exposed in the current interface.
- Text-model features are optional and require a user-configured provider plus
  explicit consent. Broader provider and long-document coverage is still being
  expanded.
- The Windows installer is unsigned unless a signing configuration is supplied
  by the release environment.
