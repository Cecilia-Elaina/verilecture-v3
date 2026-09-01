# VeriLecture

<div align="center">

**A local-first lecture evidence chain for Windows, Linux, and macOS.**

[![Version](https://img.shields.io/badge/version-0.3.0--alpha.1-e56b35?style=flat-square)](https://github.com/Cecilia-Elaina/verilecture-v3/releases/tag/v0.3.0-alpha.1)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-1f5f4f?style=flat-square)](https://github.com/Cecilia-Elaina/verilecture-v3/releases)
[![Privacy](https://img.shields.io/badge/privacy-local--first-2d8065?style=flat-square)](#privacy-by-design)
[![License](https://img.shields.io/badge/license-MIT-5a716b?style=flat-square)](./LICENSE)

<p>
  <a href="./README.md">简体中文</a>
  &nbsp;·&nbsp;
  <strong>English</strong>
</p>

<p>
  <a href="https://cecilia-elaina.github.io/verilecture-v3/">Product website</a>
  &nbsp;·&nbsp;
  <a href="https://cecilia-elaina.github.io/verilecture-v3/">产品主页</a>
</p>

<a href="./docs/screenshots/audio-import-selected.png"><img src="./docs/screenshots/readme/audio-import-selected.png" alt="VeriLecture audio import" width="900" /></a>

<sub>Local-first by design · original audio, raw transcripts, and the evidence chain stay on your device</sub>

</div>

## Why VeriLecture

Lecture notes should not end as an opaque summary that nobody can verify. VeriLecture turns the study workflow into an evidence chain:

<p align="center">
  <a href="./docs/diagrams/evidence-chain-en.svg"><img src="./docs/diagrams/evidence-chain-en.svg" alt="VeriLecture evidence chain flow" width="900" /></a>
</p>

It is built for students, teachers, and researchers who want AI-assisted speed without giving up control of source material, privacy, or academic judgment.

## Core capabilities

- **Local-first by default**: audio, raw transcripts, and course files stay on the device.
- **Hardware-aware onboarding**: the first launch scans CPU, memory, GPU, CUDA, and storage, then recommends a suitable local ASR tier.
- **Three local ASR tiers**: Fun-ASR-Nano-2512 for CPU fallback, Qwen3-ASR-0.6B for lighter CUDA machines, and Qwen3-ASR-1.7B for higher-quality CUDA transcription.
- **Evidence-preserving workflow**: original audio and raw transcripts are never overwritten; corrections, calibration, and user edits create new versioned records with provenance.
- **Teacher-safe structure**: explicit teacher statements remain distinct from repeated emphasis, inferred topics, and textbook topics.
- **Optional text-model step**: review-point generation is separate and user-controlled; it is not required for local transcription.

## Platform support

The desktop application is wired into a native Windows, Linux, and macOS build matrix. The currently published local ASR runtime is still a Windows x64 build, so the cross-platform packages first provide the shared desktop experience; native ASR runtimes for Linux and macOS will be published after separate validation.

| Platform | Package | Current local ASR status |
| --- | --- | --- |
| Windows x64 | NSIS | Fun-ASR and CUDA Runtime paths available |
| Linux x64 | AppImage | Native desktop package builds; local ASR runtime pending |
| macOS | DMG | Native desktop package builds; local ASR runtime pending |

## Product preview

<table>
<tr>
<td width="50%" valign="top">
<a href="./docs/screenshots/onboarding-model-selection.png"><img src="./docs/screenshots/readme/onboarding-model-selection.png" alt="Local ASR tier selection" width="100%" /></a>
<br /><sub><b>01 · Choose a local ASR tier</b><br />Select an understandable, verifiable model tier for the available hardware.</sub>
</td>
<td width="50%" valign="top">
<a href="./docs/screenshots/onboarding-text-model.png"><img src="./docs/screenshots/readme/onboarding-text-model.png" alt="Text model configuration" width="100%" /></a>
<br /><sub><b>02 · Connect a text model</b><br />Configure a review service only when you choose to use one.</sub>
</td>
</tr>
<tr>
<td valign="top">
<a href="./docs/screenshots/onboarding-import-audio.png"><img src="./docs/screenshots/readme/onboarding-import-audio.png" alt="Import lecture audio" width="100%" /></a>
<br /><sub><b>03 · Import lecture audio</b><br />WAV, MP3, and M4A are all first-class entry points.</sub>
</td>
<td valign="top">
<a href="./docs/screenshots/audio-records-empty.png"><img src="./docs/screenshots/readme/audio-records-empty.png" alt="Empty audio records state" width="100%" /></a>
<br /><sub><b>04 · Return to a focused records space</b><br />A clear empty state makes the next step obvious.</sub>
</td>
</tr>
<tr>
<td valign="top">
<a href="./docs/screenshots/settings-models-and-text.png"><img src="./docs/screenshots/readme/settings-models-and-text.png" alt="Models and text settings" width="100%" /></a>
<br /><sub><b>05 · See the runtime boundary</b><br />Hardware, local ASR, and text-model status stay visible in one place.</sub>
</td>
<td valign="top">
<a href="./docs/screenshots/lexicon-empty-state.png"><img src="./docs/screenshots/readme/lexicon-empty-state.png" alt="Empty course lexicon state" width="100%" /></a>
<br /><sub><b>06 · Add course vocabulary</b><br />A local lexicon helps protect domain-specific terms.</sub>
</td>
</tr>
<tr>
<td valign="top">
<a href="./docs/screenshots/about-and-licenses.png"><img src="./docs/screenshots/readme/about-and-licenses.png" alt="About and third-party licenses" width="100%" /></a>
<br /><sub><b>07 · Keep the foundation transparent</b><br />Third-party components and licenses remain easy to inspect.</sub>
</td>
<td valign="top">
<a href="./docs/screenshots/audio-import-selected.png"><img src="./docs/screenshots/readme/audio-import-selected.png" alt="Selected lecture audio" width="100%" /></a>
<br /><sub><b>08 · Return to the evidence</b><br />The source recording stays associated with every derived version.</sub>
</td>
</tr>
</table>

## Local ASR tiers

| Tier | Best for | Acceleration | `v0.3.0-alpha.1` status |
| --- | --- | --- | --- |
| **Fun-ASR-Nano-2512** | Machines without a usable NVIDIA CUDA path | CPU | Representative CPU installation and runtime path verified |
| **Qwen3-ASR-0.6B** | Lower-VRAM CUDA machines | NVIDIA CUDA | Registry and installer path prepared; independent GPU retest deferred |
| **Qwen3-ASR-1.7B** | Higher-quality local transcription | NVIDIA CUDA | Representative RTX 3060 GPU smoke test passed |

The Qwen tiers share a CUDA Runtime and ForcedAligner. The approximately 4.6 GB (about 4.3 GiB) Runtime is intentionally excluded from this Release and from Git. Until it is hosted at a stable HTTPS address and marked `published`, the installer keeps the download gated instead of presenting a broken artifact. See [CUDA Runtime publishing](./docs/CUDA_RUNTIME.md) for the release procedure.

## Privacy by design

- **Audio, source files, and raw transcripts remain local.**
- Local ASR does not upload audio.
- Text-model requests require explicit consent and a user-configured provider.
- Source excerpts and structured lexicon content are sent only when the matching permission is enabled.
- Raw material is immutable; edits create new versions with provenance.

See [Privacy and Security](./docs/PRIVACY_AND_SECURITY.md), [Model Sources](./docs/MODEL_REGISTRY.md), and [Third-party notices](./THIRD_PARTY_NOTICES.md).

## Download and run

Download the currently available Windows x64 NSIS installer from the [V3 pre-release](https://github.com/Cecilia-Elaina/verilecture-v3/releases/tag/v0.3.0-alpha.1). This is an alpha build; back up important recordings before installing. Future `v*` tags use the same release workflow to produce Windows, Linux, and macOS packages.

On first launch:

1. Review the privacy boundary and consent scope.
2. Let VeriLecture scan hardware, CUDA, storage, and model permissions.
3. Install the recommended local ASR tier.
4. Run a short audio test before importing a full lecture.

Choose **Fun-ASR-Nano-2512** when no usable NVIDIA GPU is available. For Qwen, wait for a Release whose notes explicitly state that the CUDA Runtime is publicly hosted.

## Build from source

The supported application source lives under `frontend/`; generated build output stays outside Git.

```powershell
Set-Location .\frontend
pnpm install --frozen-lockfile
pnpm typecheck
pnpm test
pnpm build
pnpm exec tauri build --ci --bundles nsis      # Windows
# pnpm exec tauri build --ci --config src-tauri/tauri.linux.conf.json --bundles appimage # Linux
# pnpm exec tauri build --ci --config src-tauri/tauri.macos.conf.json --bundles dmg      # macOS
```

Linux builds need Tauri's WebKitGTK development dependencies, and macOS builds need Xcode Command Line Tools; see the [Tauri prerequisites](https://tauri.app/start/prerequisites/) and [platform build notes](./docs/PLATFORM_BUILD.md). Windows fetches and verifies FFmpeg at build time; Linux/macOS currently use a system `ffmpeg` and do not commit a large third-party binary.

## Status and license

`v0.3.0-alpha.1` is the first public V3 milestone, not a final stable release. Native builds for all three platforms now run in CI; the local workflow, onboarding, hardware routing, CPU path, and representative CUDA path are validated on Windows x64. Native local ASR runtimes for Linux/macOS, public CUDA Runtime hosting, and clean-machine installation still need separate acceptance.

Read [Known Limitations](./docs/KNOWN_LIMITATIONS.md).

VeriLecture is released under the [MIT License](./LICENSE). Meetily upstream attribution and third-party licenses remain visible in [NOTICE](./NOTICE) and [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md).

**Developed by Cecilia-Elaina**

<div align="center">

<p>
  <a href="./README.md">简体中文</a>
  &nbsp;·&nbsp;
  <strong>English</strong>
</p>

</div>
