# VeriLecture V0.3 Runtime Registry 设计与接入说明

## 结论

CUDA Runtime 不再写死在 Rust 代码中的单个下载地址。应用首次启动时读取随应用发布的 `runtime_registry.json`，按照平台、架构、硬件能力和模型档位选择运行时，再按照 Registry 中的镜像顺序下载、校验和安装。

本目录是当前可写工作区中的独立交付包，用于把方案、注册表和校验工具同步到 V3 工程。它不替代 V3 源码，也没有执行 Git/GitHub 发布操作。

## 当前已验证的资产

本 Registry 记录的是本机已经构建并验证过的 CUDA 运行时包：

- 文件名：`verilecture-asr-runtime-cuda-qwen-fun-windows-x64.zip`
- 平台：Windows x64
- CUDA：12.6
- 压缩包大小：`4,617,121,514` bytes
- 解压后大小：`4,615,546,066` bytes
- SHA-256：`4eafd198228821c9f5ca36ebd62a4ded53df6083ff1c3f8283127a8f9bc9a665`
- 支持的 GPU ASR：`qwen3-asr-1.7b`、`qwen3-asr-0.6b`

本地包的大小和 SHA-256 已通过 `validate_runtime_registry.py --asset ...` 校验。GitHub 镜像目前标记为 `pending-publication`，因此即使 JSON 中存在 URL，应用也不能把它当成已上线资源。旧代码中写死的 `verilecture-v3/releases/download/v0.3.0-alpha.1/...` 地址实测返回 404，这正是本次改造要消除的故障模式。

## 为什么需要 Registry

固定 URL 把“应用版本”和“运行时资产托管位置”绑定在了一起。只要 Release tag、仓库名或资产名发生变化，旧客户端就会在首次启动时直接失败，而且没有办法在不重新发布应用的情况下修复地址。

Registry 把以下信息从程序逻辑中移出：

1. 运行时版本和 CUDA 版本。
2. 资产文件名、压缩包大小、解压后预计占用空间和 SHA-256。
3. 适用的平台、架构、模型档位和最低硬件要求。
4. 一个或多个带优先级的下载镜像及其发布状态。

应用仍然固定校验 Registry 的结构和安全约束，但可以通过更新 Registry 资源来更换发布地址、增加镜像或停用失效资产。

## 文件布局

```text
verilecture-v3-runtime-registry/
├── runtime_registry.json          # 应用随包携带的运行时索引
├── runtime_registry.schema.json   # 编辑器/CI 可使用的 JSON Schema
├── validate_runtime_registry.py   # 离线结构与资产校验器
└── RUNTIME_REGISTRY_DESIGN.md     # 本说明
```

合并到 V3 后，生产文件应放到：

```text
frontend/src-tauri/resources/runtime_registry.json
```

Registry 本身只有几 KB，不把 4.6 GB 的 CUDA Runtime 或模型权重塞进 NSIS 安装包。应用安装包负责提供 CPU/Fun-ASR-Nano 的可运行路径；CUDA Runtime 和 Qwen 权重在首次使用对应 GPU 档位时按需安装。

## Registry 字段约定

| 字段 | 作用 |
| --- | --- |
| `schemaVersion` | 注册表结构版本；当前为 `1`，结构变化时递增。 |
| `registryVersion` | 与应用/资产发布批次对应的版本。 |
| `defaultChannel` | 当前默认 `alpha` 或 `stable` 通道。 |
| `id` | 运行时逻辑 ID，不能因托管地址变化而变化。 |
| `version` | 运行时自身版本，用于缓存隔离和升级回滚。 |
| `platform` / `architecture` | 当前只接受 `windows` / `x86_64`。 |
| `artifactName` | 下载后文件名，禁止路径分隔符。 |
| `compressedBytes` | 下载包精确字节数。 |
| `installedBytes` | 解压后预计占用空间，用于安装前磁盘检查。 |
| `sha256` | 下载包完整 SHA-256，必须是 64 位小写十六进制。 |
| `models` | 可使用该运行时的固定模型 ID；不能加入未测试模型。 |
| `requirements` | NVIDIA、显存和内存最低要求。 |
| `mirrors` | 带优先级的 HTTPS/HTTP 地址列表。 |
| `status` | `published`、`pending-publication` 或 `disabled`。 |

`published` 的运行时必须至少有一个 `published` 镜像；`pending-publication` 不能让应用进入 READY，只能让诊断页明确显示“运行时待发布”。

## 首次启动与安装流程

```text
首次启动
   │
   ├─ 扫描 Windows / NVIDIA / VRAM / RAM / CUDA 可用性
   │
   ├─ 选择模型档位
   │     ├─ 无可用 NVIDIA/CUDA → Fun-ASR-Nano-2512（CPU）
   │     ├─ NVIDIA + VRAM ≥ 6 GiB → Qwen3-ASR-0.6B
   │     └─ NVIDIA + VRAM ≥ 8 GiB、RAM ≥ 16 GiB → Qwen3-ASR-1.7B
   │
   ├─ GPU 档位读取 Registry，按 platform/architecture/models 匹配
   ├─ 安装前检查磁盘空间（压缩包 + 解压空间 + 安全余量）
   ├─ 下载到 `.part`，支持 HTTP Range 续传
   ├─ 校验精确大小和 SHA-256
   ├─ Zip64 安全解压到版本化临时目录
   ├─ 校验 sidecar manifest、文件清单和 CUDA probe
   ├─ 原子替换运行时目录，失败时恢复上一版本
   ├─ 下载并校验对应 Qwen ASR/ForcedAligner 权重
   ├─ 执行 load → transcribe → timestamp → unload 冒烟测试
   └─ 全部成功后才显示 READY
```

Fun-ASR-Nano 的 CPU 路径不应等待 CUDA Registry，也不应因没有独显而尝试下载 CUDA Runtime。普通用户只看到“本地处理”和可用性提示；模型名称、Provider 和镜像只在“AI 诊断”中显示。

## 镜像选择与 404 处理

1. 过滤当前平台、架构、模型和硬件要求不匹配的运行时。
2. 在 `status=published` 的镜像中按 `priority` 从高到低尝试。
3. 404、连接中断、超时等传输错误可以切换下一个镜像。
4. 下载成功但大小或 SHA-256 不匹配时，删除 `.part`，记录安全错误；可以尝试另一个镜像，但绝不能把不匹配的内容安装为 READY。
5. 所有镜像都不可用时，保留可重试状态，给用户显示具体错误；不得在首次启动中静默失败或假装已安装。
6. Registry 中没有 `published` 镜像时，直接显示“CUDA 运行时尚未发布”，不要发起请求到 `pending-publication` 地址。

当前 CUDA Runtime 压缩包约 4.6 GiB，超过 GitHub 单个 Release 资产 2 GiB 的限制，因此 V0.3 首选使用阿里云 OSS 或同类对象存储。对象必须提供固定 HTTPS GET、准确的 Content-Length 和 HTTP Range；Registry 不应写入会过期的预签名 URL。建议使用不可变的版本化对象路径，例如 `verilecture/runtime/0.3.0-alpha.1/<artifactName>`，并只给该对象设置 `public-read`。

若未来要使用 GitHub Release，必须把 Runtime 拆成多个小于 2 GiB 的分片，并同步扩展下载器和 Registry 的分片组装协议；当前单 URL、单 SHA-256、Range 续传实现不支持这种方案，因此不作为 V0.3 首次发布路径。

## 缓存、升级和回滚

推荐的缓存布局为：

```text
<app-data>/runtimes/
└── cuda-qwen-fun/
    └── 0.3.0-alpha.1/
        ├── runtime_manifest.json
        ├── verilecture-asr-runtime.exe
        └── ...
```

下载临时文件使用同一文件系统下的 `.part`；解压目录使用 `.<runtime-id>.<version>.installing`；验证完成后使用原子 rename/swap。旧版本暂时保留为 `.previous`，新版本冒烟测试失败时恢复。成功切换后再清理旧版本，避免中途断电造成不可启动状态。

禁止：

- 直接解压覆盖当前运行时。
- 只校验 URL 或文件名，不校验大小和 SHA-256。
- 仅因 sidecar 文件存在就显示 READY。
- 用 GPU 不可用时的 CPU fallback 掩盖 Qwen CUDA 运行时安装失败。

## V3 源码合并清单

当前 V3 目录不可写，因此本目录先提供可直接合并的设计和注册表。恢复 V3 写权限后按以下顺序接入：

1. 将 `runtime_registry.json` 复制到 `frontend/src-tauri/resources/`，并加入 Tauri resource bundle。
2. 在 `frontend/src-tauri/src/models.rs` 增加 Registry 结构体和 UTF-8 JSON 加载逻辑，删除 `CUDA_RUNTIME_ARCHIVE_URL`、固定 bytes 和固定 SHA-256 常量。
3. 将当前 `install_cuda_runtime` 改为接收 Registry 选择出的运行时条目，而不是读取单个常量。
4. 下载循环改为遍历 published mirrors；pending/disabled mirror 不发起请求。
5. 保留现有 Range 续传、Zip64 解压、manifest 校验、CUDA probe、原子 swap 和 READY 门禁。
6. 将 Runtime Registry 状态、命中的镜像、失败原因和 SHA-256 显示在 AI 诊断页；普通用户界面不显示模型名。
7. 增加 Rust 测试：重复 ID、空 URL、SHA 格式、published/pending 状态不一致、镜像 fallback、校验失败不 READY。
8. 更新 `docs/PROGRESS.md` 和 `docs/DECISIONS.md`，记录 Registry 替代硬编码 URL 的决策。

## 验收标准

- Registry 可以离线解析，且通过本目录校验器。
- Registry 指向的本地 CUDA 包大小和 SHA-256 与构建资产一致。
- 未发布镜像不会被下载，404 不会造成首次启动崩溃或假 READY。
- 断点续传、校验失败、磁盘不足、CUDA probe 失败和中断恢复都有明确状态。
- 无独显设备直接走 Fun-ASR-Nano CPU 路径，不下载 CUDA Runtime。
- 有合适 NVIDIA 显卡时，1.7B/0.6B 只使用已经通过 CUDA load、中文转写、单调时间戳和 unload 验证的组合。
- 云端功能仍不在本次验收范围内。
