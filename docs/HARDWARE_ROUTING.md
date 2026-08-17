# Hardware routing

Startup probes Windows version, OS/architecture, CPU, logical cores, AVX2, RAM, free disk,
NVIDIA name/VRAM/driver, `nvidia-smi`, CUDA runtime usability, network reach,
proxy environment presence, and model-directory write access. Proxy values are
never persisted; only the configured/not-configured state and environment
variable name are shown.

| Tier | Static gate | Device |
|---|---|---|
| Qwen 1.7B | CUDA smoke, VRAM ≥ 8 GiB, RAM ≥ 16 GiB | CUDA |
| Qwen 0.6B | CUDA smoke, VRAM ≥ 6 GiB, RAM ≥ 16 GiB | CUDA |
| Fun-ASR-Nano | no usable CUDA, RAM ≥ 8 GiB | CPU |

Unknown or failed probes never enable a GPU model. The selection is persisted
only after a verified install; there is no silent fallback during a running
job. A machine below 8 GiB RAM is shown as unsupported rather than being
silently routed to a model that cannot be expected to run reliably.
