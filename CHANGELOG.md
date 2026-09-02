# Changelog

All notable user-facing changes to 课溯 · VeriLecture are recorded here. This project is still in Alpha, so entries distinguish published packages from platform or runtime validation.

## [Unreleased]

### Documentation

- Keep the README and product site centered on traceable review points, local-first storage, source preservation, and hardware-aware setup.
- Add a clearly labelled result-screen concept mockup and capture brief so public visuals do not imply an unverified UI state.

## [0.3.0-alpha.4] - 2026-09-02

### Added

- Windows x64 NSIS, Linux x64 AppImage, and macOS DMG desktop packages.
- Platform-specific SHA256 checksum files with matching public asset names.
- Native square application icons for the published bundles.
- A public product site with direct platform download links and a language switcher.

### Changed

- Keep raw transcripts, corrections, calibration, and generated review points as separate records.
- Make local audio handling, consent-controlled text-model requests, and hardware routing visible in the first-run and settings surfaces.

### Validation boundary

- The Windows x64 Fun-ASR path and a representative CUDA path have corresponding validation.
- Linux and macOS packages currently provide the desktop shell; native local ASR sidecars still require separate publication and validation.
- The Qwen CUDA Runtime remains gated until a stable HTTPS artifact is publicly hosted and accepted.

### Known limitations

- This is an Alpha release. Back up important recordings before installing.
- Model weights download on first use.
- Long audio does not expose durable in-job resume in the current interface.
- The Windows installer is unsigned unless signing is configured in the release environment.

## Earlier releases

- [v0.3.0-alpha.3](./docs/releases/v0.3.0-alpha.3.md)
- [v0.3.0-alpha.2](./docs/releases/v0.3.0-alpha.2.md)
- [v0.3.0-alpha.1](./docs/releases/v0.3.0-alpha.1.md)

[Unreleased]: https://github.com/xiajiadi/verilecture-v3/compare/v0.3.0-alpha.4...HEAD
[0.3.0-alpha.4]: https://github.com/xiajiadi/verilecture-v3/releases/tag/v0.3.0-alpha.4
