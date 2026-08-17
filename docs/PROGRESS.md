# Progress

## Current status — 2026-08-17

V3 的公开 `main` 已整理为由项目所有者 `Cecilia-Elaina` 署名的当前代码快照，
并将 `v0.3.0-alpha.1` 标签移动到该快照。旧 Meetily 参考历史不再属于公开默认分支；
README、NOTICE、第三方许可证和 About 页面仍保留完整的参考与许可声明。重写前的
整合历史仅保留在本地恢复引用中，用于必要时追溯，不作为公开贡献者历史发布。

正式工作目录已经迁移到：

```text
D:\Download\verilecture\verilecture-v3-workspace
```

原始目录 `D:\Download\verilecture-v3` 保持只读且未修改。本工作区从原始 V3
复制源码、资源、脚本、文档和构建配置，排除了 `node_modules`、`target`、
`dist`、`output`、虚拟环境、缓存和临时下载文件。之后的修改、测试、构建和
安装包都必须从本工作区产生。没有执行 Git/GitHub 命令或远程发布。

## 已完成

- Vite/React + Tauri 2 桌面壳、简体中文默认语言、英文切换、首次启动隐私门、
  硬件扫描、三模型固定目录和本地优先处理链已经从 V3 源码迁移。
- 三个可用模型保持为 `Qwen3-ASR-1.7B`、`Qwen3-ASR-0.6B` 和
  `Fun-ASR-Nano-2512`；Qwen 使用 ForcedAligner 时间戳，Fun 使用 CPU/VAD
  段级时间范围。
- `src-tauri/resources/runtime_registry.json` 已接入，模型 Profile 通过
  `runtimeBundleId` 指向 `cuda-qwen-fun`。Rust 不再保存 CUDA Runtime 的
  固定 URL、固定文件名、固定大小或固定 SHA-256 常量。
- Runtime Resolver 会校验 Registry、平台/架构、模型绑定、镜像状态和优先级；
  只允许 `published` Source，`pending-publication` 明确返回
  `MODEL_RUNTIME_SOURCE_UNAVAILABLE`。
- CUDA Runtime 使用版本化缓存目录、`.part` Range 续传、大小/SHA-256 校验、
  Zip64 安全解压、manifest/`--probe-cuda` 门禁、staging 和原子替换；失败时
  不会创建 READY。
- 新增 `scripts/mock_runtime_server.py`、`scripts/test_runtime_download_chain.py`
  和 `scripts/accept_local_runtime.py`。前两个覆盖本地 HTTP、Range、恢复、
  404、大小和 SHA 错误；后者使用真实 4.6 GB 资产进行完整本地验收。
- 真实 CUDA 资产通过 localhost 完成：首次响应截断、第二次
  `Range: bytes=8388608-` 恢复、精确大小/SHA-256 校验、Zip64 解压 6252 个
  文件、原子安装和真实 `ASR_CUDA_USABLE=1` 探针。
- Fun-ASR-Nano CPU 资源、Qwen 1.7B/0.6B CUDA 直连验证、音频导入、转写、
  版本化词典、教材处理、隐私/Provider 门禁、教程、双语 UI 和 NSIS 配置保留。
- 新工作区已经完成 Debug 和 Release Tauri/NSIS 构建。Release 安装包为：
  `src-tauri/target/release/bundle/nsis/课溯 · VeriLecture_0.3.0-alpha.1_x64-setup.exe`，
  最新修复包大小 `71427646` bytes，SHA-256 为
  `CCDFB6DE464353FEB748E6AE9E9CFBB00C3A139E021122003395F79F18A65350`。
- 已在隔离目录完成 Release 安装器验收：安装退出码为 0，Registry、runtime
  manifest、CPU sidecar、Fun CLI、FFmpeg 均存在；使用安装后的 sidecar 对只读
  Fun-ASR-Nano 测试模型完成 `load -> transcribe -> unload`，CPU 转写返回中文
  时间戳；随后卸载退出码为 0，隔离目录已清理。
- 2026-08-02 已连接外置 NVIDIA GPU 完成最终代表模型实机验收：
  `NVIDIA GeForce RTX 3060`、12 GiB、驱动 `596.36`、Compute Capability `8.6`；
  CUDA Runtime 探针返回 `ASR_CUDA_USABLE=1`、CUDA `12.6`。Qwen3-ASR-1.7B
  完成真实 `load -> transcribe -> unload`，返回 `executionDevice=CUDA`、
  `detectedLanguage=Chinese` 和 40 多个毫秒级中文片段，耗时约 60 秒。按照本轮
  验收范围，0.6B 未重复执行独立 smoke；它共用同一 CUDA Runtime、ForcedAligner
  和协议链路。结果记录在 `local-acceptance/gpu-smoke-report.json`。

- 2026-08-06 已定位并修复 Windows 首次硬件扫描闪退：实际中文安装目录的分阶段
  验收在 manifest 解析和文件大小检查阶段正常，在 SHA-256 阶段以
  `0xC00000FD / STATUS_STACK_OVERFLOW` 退出。原因是运行时校验和模型校验各自
  在 GUI 主线程栈上创建了 1 MiB 缓冲区；缓冲区已改为堆分配。修复后的 Release
  程序在同一中文目录完成 manifest、CUDA 探针和完整硬件扫描阶段，连续保持运行，
  没有再次闪退。此问题是应用实现缺陷，不是用户电脑的 NVIDIA 硬件或驱动故障。
- 最新 NSIS 包已安装到全新的中文目录 `D:\Software\课溯 · VeriLecture-release-candidate-20260806`；
  安装退出码为 0，文件齐全。通过实际界面点击首次启动的“继续”后，硬件扫描完成并
  进入“安装本地模型”第 2 步，识别到 RTX 3060 并推荐 Qwen3-ASR-1.7B。点击模型安装
  在运行时资产尚未公开时安全显示失败提示，应用仍保持运行，没有再次闪退。
- 实际用户验收进一步确认：`Fun-ASR-Nano-2512` 可以直接开始 CPU 模型下载；两个 Qwen
  档位会在 CUDA Runtime 下载前返回 `MODEL_RUNTIME_SOURCE_UNAVAILABLE`，因为嵌入的
  Runtime Registry 仍为 `pending-publication`。前端已增加中英文明确提示，说明需要先
  选择 Fun 或等待 CUDA Runtime 发布，而不是笼统显示“操作失败”。

## 本轮验证命令

```powershell
cd D:\Download\verilecture\verilecture-v3-workspace
python -B scripts\validate_runtime_registry.py
python -B scripts\validate_runtime_registry.py --asset D:\Download\verilecture-v3\output\release-assets\verilecture-asr-runtime-cuda-qwen-fun-windows-x64.zip
python -B scripts\test_runtime_download_chain.py
python -B scripts\accept_local_runtime.py
cargo fmt --manifest-path src-tauri\Cargo.toml --check
cargo test --manifest-path src-tauri\Cargo.toml
pnpm tauri build --no-bundle
pnpm typecheck
pnpm test -- --run
pnpm build
```

真实资产登记值：

```text
sizeBytes = 4617121514
sha256 = 4eafd198228821c9f5ca36ebd62a4ded53df6083ff1c3f8283127a8f9bc9a665
```

## 2026-08-07 云端重点生成故障定位与修复

- 读取当前安装实例 `PID 27200` 对应的正式数据库审计记录，确认录音
  `计算机网络` 已完成 Fun-ASR-Nano 本地转写，共 234 个转写片段；失败发生在
  `exam_points_map` 的第一次云端文本请求，不是 ASR、音频导入或时间戳问题。
- 旧版本只保存了 `PROVIDER_RESPONSE_INVALID`，没有保存 HTTP 状态、结束原因或空响应
  信息，因此无法从历史记录反推出具体的云端 HTTP 原文；没有读取或输出 API Key。
- 发现重点生成模板的 `output_schema` 要求顶层 `chapters`，而 Rust 解析器只接受
  `examPoints`，已统一为扁平 `examPoints` 协议并补充解析单元测试。
- DeepSeek JSON 重点任务已关闭默认 thinking 模式，并新增余额不足、鉴权失败、
  模型/接口不存在、限流、超时、空响应、截断和 JSON 无效等安全错误分类。
- 新 NSIS 安装包已构建：
  `src-tauri/target/release/bundle/nsis/课溯 · VeriLecture_0.3.0-alpha.1_x64-setup.exe`
  ，大小 `71,422,276` bytes，SHA-256 为
  `A0A96E5E2FFB3D7B20837EA5AE127D7A5286C0E7F164B2EA7A6203E6EFAC2B03`。
- 修复后的 Rust 28 项测试、前端 3 项测试、TypeScript 类型检查和格式检查均通过；
  未停止或修改当前运行中的 PID 27200。

## 2026-08-07 导入与硬件扫描 UI 修复

- 修复音频导入拖放区图标整体旋转的问题；上传箭头保持水平，并移除装饰性“音频 →
  证据”文字，导入区只保留上传图标。
- 修复 Windows 硬件扫描系统版本乱码：不再读取 `cmd /C ver` 的本地代码页字节，
  改用 `sysinfo` 的 Windows 注册表版本读取；旧缓存中含替换字符时，前端显示稳定的
  `Windows` 回退值，重新扫描后显示完整版本名。
- UI 修复版 Rust 28 项测试、前端 3 项测试、类型检查和格式检查均通过；新的 NSIS
  安装包大小为 `71,422,961` bytes，SHA-256 为
  `E18793E1DED81FBAE94B21A4539833C9C8F217968326B42FF6E83385795D554B`。

## 2026-08-08 重点生成、硬件扫描与导入 UI 修复

- 读取当前数据库确认：最新一次重点生成失败发生在 `exam_points_map`，输入
  `25,131` 字符，Provider 返回 `PROVIDER_OUTPUT_TRUNCATED`；ASR 记录仍是已完成的
  234 个中文转写片段，未被改写。
- 将重点映射分块上限从 `28,000` 调整为 `14,000` 字符，每块最多输出 20 个候选；
  同时限制汇总候选和字段长度，并将汇总输出上限提高到 8,192 tokens，降低 JSON
  结果撞上 Provider 输出上限的概率。
- Windows 子进程统一使用 `CREATE_NO_WINDOW`，硬件扫描后端增加串行锁，设置页扫描期间
  禁用按钮并显示进行中状态，避免重复扫描同时启动 `nvidia-smi`/ASR Runtime。
- 新 NSIS 安装包已构建并完成本地校验：大小 `71,427,646` bytes，SHA-256 为
  `CCDFB6DE464353FEB748E6AE9E9CFBB00C3A139E021122003395F79F18A65350`。
- 本轮 Rust 28 项测试、前端 3 项测试、TypeScript 类型检查、格式检查和 Registry
  校验均通过。

## 尚未完成或不在本地权限范围内

- CUDA Runtime 尚未上传到稳定的 HTTPS 对象存储，Registry 镜像保持
  `pending-publication`；当前 4.6 GiB 单文件不适合 GitHub Release，外部发布仍是
  V0.3 的最后一个发布前置条件。
- 未执行公网下载和干净 Windows 用户配置文件的首次启动验收；发布前需要重新
  下载并核对 HTTP 状态、Content-Length、SHA-256、manifest、probe 和模型 Smoke。
- 最新修复版尚未在当前 DeepSeek 账户上重新点击“生成考试重点”完成真实验收；需要安装
  新 NSIS 包并重试一次，以记录新的精确 Provider 错误码或确认重点生成成功。
- 仍未完成稳定 HTTPS Runtime 发布，Registry 镜像仍为 `pending-publication`；因此
  公网 CUDA 下载、干净 Windows 用户配置文件的首次启动和 Qwen 模型的公网下载
  仍需发布前完成。外置 GPU 的代表性实机 Smoke 已通过，当前没有把缺少公网资产
   误报为 READY。

## 2026-08-15 - V3 仓库整合与公开发布准备

- [重写前历史记录] 已将旧 alpha2 变更先提交为 `694f50a`，再将
  `codex/verilecture-v0.1` 与 `codex/verilecture-alpha2` 无冲突合并到本地
  `main`（合并提交分别为 `c7c2e15` 与 `96b1568`）；这些提交现在只作为本地
  恢复依据，不属于公开 V3 默认分支。
- V3 正式源码现位于仓库约定的 `frontend/src` 与 `frontend/src-tauri/src`；
  `backend/` 继续作为归档参考，不再作为受支持应用入口。
- 已加入双语根 README、英文命名的产品截图、V3 发行说明、Windows CI 和标签触发的
  NSIS 发布工作流。FFmpeg 大二进制改为构建时下载/本地供给并做 SHA-256 校验，
  不进入 Git；模型、缓存、构建输出和 4.6 GiB CUDA Runtime 同样不进入 Git。
- 当前本地目标版本为 `v0.3.0-alpha.1`。CUDA Runtime 仍为
  `pending-publication`，所以这次发布准备不会宣称 Qwen 在干净机器上可安装。
- 本地 Windows NSIS 打包已成功：安装器大小 `110,421,695` bytes，SHA-256 为
  `B59118576BCF87855088305AF32B8A7D6CC8477BDAF25191F8192A40D969F1CF`；该安装器仍需
  在干净用户配置文件中完成一次从安装到首次启动的最终验收。
