# Decisions

## 2026-07-31 - independent V3 project

V3 is an independent product. The old Meetily-derived project is reference-only
and is not used as the formal build workspace.

## 2026-07-31 - fixed ASR tiers

Only Qwen3-ASR-1.7B, Qwen3-ASR-0.6B and Fun-ASR-Nano-2512 are product models.
Qwen tiers require the official ForcedAligner. No Whisper, Parakeet, cloud ASR,
Ollama or local summary model is a fallback.

## 2026-07-31 - CPU release path

Fun-ASR-Nano is the verified no-NVIDIA path. The V3 build uses a CPU-only
PyInstaller sidecar plus the official `llama-funasr-cli.exe`; a real Chinese
speech sample returned a CPU transcript after installation. The CPU runtime
remains the installer fallback even on a machine that will later download the
CUDA runtime.

The CPU route deliberately uses FunAudioLLM's official GGUF/llama.cpp edge
bundle rather than pretending that the CUDA-oriented PyTorch path is a CPU
fallback. The product does not label its VAD ranges as Qwen ForcedAligner
precision.

## 2026-07-31 - CUDA runtime split

The CUDA-enabled PyTorch sidecar was built and tested locally with CUDA 12.6,
Qwen3-ASR-1.7B, the official ForcedAligner and an RTX 3060 12 GiB GPU. The
runtime is approximately 4.6 GB, so it is not embedded in NSIS. The installer
ships CPU/Fun; a Qwen install downloads the CUDA runtime, verifies its SHA-256,
extracts it atomically and requires a real CUDA probe.

## 2026-07-31 - Windows packaging

The alpha installer target is NSIS. MSI remains disabled because the product
version `0.3.0-alpha.1` cannot be used as MSI's numeric-only pre-release
identifier. The package includes FFmpeg for non-WAV decoding, with its GPLv3
notice and hash; public redistribution requires a separate compliance review.

## 2026-07-31 - external GPU validation

The code probes CUDA and fails closed when no usable device is present. The
connected RTX 3060 machine passed Qwen 1.7B and Qwen 0.6B with ForcedAligner;
a physical no-NVIDIA machine remains a separate acceptance case.

## 2026-07-31 - model mirror and integrity policy

ModelScope is attempted first for artifacts with a registered mainland mirror;
Hugging Face remains the pinned fallback. Both paths must satisfy the same
expected byte count and SHA-256 before installation, and no model is marked
READY before the matching runtime smoke test.

## 2026-07-31 - READY startup gate and cancellation cleanup

`READY.json` stores the expected size and filesystem modification timestamp for
every non-archive artifact. Startup may use this quick gate; a full SHA-256
verification remains the repair path. Cancelling an import removes the managed
in-progress audio directory so a partial copy cannot be mistaken for a usable
recording.

## 2026-07-31 - provider failure auditability

Every lexicon-provider attempt records the selected textbook excerpt payloads
and run status, including transport failures and malformed structured output.
Accepted structured results receive a separate structured-data audit row;
failed responses are never merged into a lexicon profile.

## 2026-08-02 - official writable workspace

Because `D:\Download\verilecture-v3` is read-only, the only formal development
directory is `D:\Download\verilecture\verilecture-v3-workspace`. It was copied
from the V3 source without Git metadata, dependency/build caches, virtual
environments, output downloads or partial artifacts. The read-only source is
preserved and must not be modified.

## 2026-08-02 - Runtime Registry replaces fixed CUDA URL

`ModelDefinition` stores only `runtimeBundleId`. The URL, mirror priority,
runtime version, exact compressed/installed sizes and SHA-256 come from the
embedded `runtime_registry.json`. A pending or disabled Runtime Source is
rejected with `MODEL_RUNTIME_SOURCE_UNAVAILABLE`; a malformed Registry is
rejected with `MODEL_RUNTIME_REGISTRY_INVALID`. Release builds do not honor the
debug-only `VERILECTURE_RUNTIME_REGISTRY_OVERRIDE`.

## 2026-08-02 - local download-chain acceptance

The local Mock HTTP server is the permitted substitute for unavailable public
hosting. It passed full response, HTTP Range, interrupted response, resume,
404, size and SHA-256 fixture checks. The real 4.6 GB CUDA archive passed the
same localhost interrupted/Range flow, exact hash, Zip64 extraction of 6252
files, atomic staging promotion and `ASR_CUDA_USABLE=1`. This evidence is not
described as public Release acceptance; GitHub publication and clean-user
download remain deferred.

## 2026-08-06 - Windows startup hashing must not use stack-sized buffers

The first-run hardware scan validates the packaged ASR runtime manifest and the model
installer verifies downloaded artifacts. Both code paths may run on the Windows GUI
entry thread, whose stack is not large enough for a 1 MiB local byte array. The old
implementation therefore terminated the installed app with `0xC00000FD` while hashing
the runtime entrypoint; this was reproduced in the real Chinese install path and was
not a GPU or driver failure. Streaming buffers in runtime and model SHA-256 helpers
are now heap allocated. The integrity checks remain enabled, and the packaged app must
still be reinstalled and walked through from a clean user profile before release.

## 2026-08-06 - Qwen installation is gated on published CUDA Runtime

Qwen3-ASR-1.7B and Qwen3-ASR-0.6B share the CUDA Runtime bundle. The installer must
fail closed while that bundle is marked `pending-publication`; it must not silently
fall back to CPU or download an unverified URL. Fun-ASR-Nano has an independent CPU
artifact and can therefore install while the Qwen runtime is unpublished. The UI now
reports this specific condition and suggests Fun-ASR-Nano or retrying after publication.

## 2026-08-06 - large Runtime uses immutable object storage

The CUDA Runtime archive is approximately 4.6 GiB, so it is not published as one GitHub
Release asset. The V0.3 publication path uses an object-storage URL with HTTPS GET,
accurate Content-Length, HTTP Range support and a stable versioned object path. The
Registry remains `pending-publication` until an external publisher uploads the exact
archive, verifies its size and SHA-256 from a clean Windows environment, and provides
the final URL. A public object may be readable, but its upload credentials never enter
the application or Registry.

## 2026-08-07 - structured exam-point generation contract and diagnostics

The exam-point prompt templates and Rust parser must use the same flat
`{"examPoints":[...]}` contract. The previous templates described a `chapters` envelope
while the parser expected `examPoints`, which could silently discard valid model output.
For DeepSeek structured JSON tasks, the request disables thinking mode and retains JSON
output mode. Provider failures are classified without persisting response bodies or
secrets, including request rejection, authentication, balance, missing endpoint/model,
rate limit, timeout, empty output, truncation and invalid JSON. The raw transcript remains
unchanged when analysis fails.

## 2026-08-07 - hardware UI must not display code-page decoded OS text

Windows `cmd /C ver` writes localized console output using a system code page, so treating
its bytes as UTF-8 can display replacement characters in the hardware summary. The
hardware probe now uses `sysinfo`'s Unicode Windows version lookup. The frontend also
rejects known replacement-character or `Microsoft Windows [` cache values and falls back
to a stable `Windows` label until the next scan refreshes the profile.

## 2026-08-08 - scan isolation and bounded exam-point requests

The Windows GUI must treat hardware and runtime probes as invisible, serialized
background work. All child commands use `CREATE_NO_WINDOW`, and the app state
serializes hardware scans so repeated clicks cannot launch overlapping
`nvidia-smi` or CUDA sidecar processes. The Settings action is disabled while a
scan is active.

The exam-point mapper now uses 14,000-character transcript chunks and a 20-point
per-chunk cap. Candidate fields are bounded before the reducer, and the reducer
is given up to 8,192 output tokens. This preserves evidence IDs while reducing
the chance that a valid JSON response is cut off by the provider.

## 2026-08-15 - V3 repository and release boundary

The V3 workspace is integrated into the repository layout required by the project
contract: supported frontend and Tauri code live under `frontend/`, while the
Meetily-derived `backend/` remains an archived reference. Before publicizing the
V3 snapshot, the pre-V3 integration history was intentionally compacted into a
single owner-authored commit so the public default branch represents the current
VeriLecture product rather than the full history of the reference project. The
reference project, its MIT license, and third-party notices remain credited in
the repository. The pre-rewrite integration history is retained only in a local
recovery reference and is not part of public `main`.

The public-facing repository now includes a bilingual README, curated screenshots,
release notes, Windows CI, and a tag-driven NSIS release workflow. Large generated
or third-party binaries are kept out of Git: FFmpeg is obtained at build time and
verified by SHA-256, and the CUDA Runtime remains excluded until its stable HTTPS
object and published Registry entry exist. This keeps the alpha release honest and
prevents a broken Qwen download from being presented as a supported clean-install
path.
