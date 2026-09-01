# 课溯 · VeriLecture

<div align="center">

**把每一段课堂录音，变成可回听、可校准、可追溯的学习材料。**

[![版本](https://img.shields.io/badge/版本-0.3.0--alpha.1-e56b35?style=flat-square)](https://github.com/Cecilia-Elaina/verilecture-v3/releases/tag/v0.3.0-alpha.1)
[![平台](https://img.shields.io/badge/平台-Windows%20%7C%20Linux%20%7C%20macOS-1f5f4f?style=flat-square)](https://github.com/Cecilia-Elaina/verilecture-v3/releases)
[![隐私](https://img.shields.io/badge/隐私-本地优先-2d8065?style=flat-square)](#隐私设计)
[![许可证](https://img.shields.io/badge/许可证-MIT-5a716b?style=flat-square)](./LICENSE)

<p>
  <strong>简体中文</strong>
  &nbsp;·&nbsp;
  <a href="./README.en.md">English</a>
</p>

<p>
  <a href="https://cecilia-elaina.github.io/verilecture-v3/">产品主页</a>
  &nbsp;·&nbsp;
  <a href="https://cecilia-elaina.github.io/verilecture-v3/">Product website</a>
</p>

<a href="./docs/screenshots/audio-import-selected.png"><img src="./docs/screenshots/readme/audio-import-selected.png" alt="课溯音频导入界面" width="900" /></a>

<sub>本地优先 · 原始音频、原始转写和证据链留在本机</sub>

</div>

## 为什么是课溯

课堂笔记不应该只剩下一段无法核对的摘要。课溯把学习流程整理成一条可回溯的证据链：

<p align="center">
  <a href="./docs/diagrams/evidence-chain-zh.svg"><img src="./docs/diagrams/evidence-chain-zh.svg" alt="课溯可回溯证据链流程图" width="900" /></a>
</p>

它面向学生、教师和研究者：获得 AI 辅助的效率，同时保留对原始材料、隐私和学习判断的控制权。

## 核心能力

- **本地优先**：音频、原始转写和课程文件默认保留在本机。
- **硬件感知引导**：首次启动扫描 CPU、内存、GPU、CUDA 和存储，并推荐合适的本地 ASR 档位。
- **三档本地 ASR**：Fun-ASR-Nano-2512（CPU 回退）、Qwen3-ASR-0.6B（轻量 CUDA）和 Qwen3-ASR-1.7B（高质量 CUDA）。
- **证据不丢失**：原始音频与原始转写不会被覆盖；修订、校准和用户编辑都会产生带来源的新版本。
- **教师语义安全**：明确陈述、重复强调、推断主题和教材主题保持区分。
- **文本模型可选**：重点整理是独立且由用户控制的步骤，不影响本地转写。

## 平台支持

桌面应用已接入 Windows、Linux 和 macOS 的原生构建矩阵。当前公开的本地 ASR 运行时仍是 Windows x64 版本，因此跨平台安装包先提供统一的桌面体验；Linux 和 macOS 的原生 ASR 运行时将在单独验证后发布。

| 平台 | 安装包 | 当前本地 ASR 状态 |
| --- | --- | --- |
| Windows x64 | NSIS | Fun-ASR 与 CUDA Runtime 路径可用 |
| Linux x64 | AppImage | 原生桌面包可构建；本地 ASR 运行时待发布 |
| macOS | DMG | 原生桌面包可构建；本地 ASR 运行时待发布 |

## 产品预览

<table>
<tr>
<td width="50%" valign="top">
<a href="./docs/screenshots/onboarding-model-selection.png"><img src="./docs/screenshots/readme/onboarding-model-selection.png" alt="选择本地 ASR 档位" width="100%" /></a>
<br /><sub><b>01 · 选择本地 ASR 档位</b><br />根据硬件选择易理解、可验证的模型层级。</sub>
</td>
<td width="50%" valign="top">
<a href="./docs/screenshots/onboarding-text-model.png"><img src="./docs/screenshots/readme/onboarding-text-model.png" alt="配置文本模型" width="100%" /></a>
<br /><sub><b>02 · 连接文本模型</b><br />只有在你选择时，才配置文本整理服务。</sub>
</td>
</tr>
<tr>
<td valign="top">
<a href="./docs/screenshots/onboarding-import-audio.png"><img src="./docs/screenshots/readme/onboarding-import-audio.png" alt="导入课堂音频" width="100%" /></a>
<br /><sub><b>03 · 导入课堂音频</b><br />WAV、MP3 和 M4A 都是直接入口。</sub>
</td>
<td valign="top">
<a href="./docs/screenshots/audio-records-empty.png"><img src="./docs/screenshots/readme/audio-records-empty.png" alt="音频记录空状态" width="100%" /></a>
<br /><sub><b>04 · 回到专注的记录空间</b><br />清晰的空状态让下一步一目了然。</sub>
</td>
</tr>
<tr>
<td valign="top">
<a href="./docs/screenshots/settings-models-and-text.png"><img src="./docs/screenshots/readme/settings-models-and-text.png" alt="模型与文本设置" width="100%" /></a>
<br /><sub><b>05 · 看见运行边界</b><br />硬件、本地 ASR 与文本模型状态集中展示。</sub>
</td>
<td valign="top">
<a href="./docs/screenshots/lexicon-empty-state.png"><img src="./docs/screenshots/readme/lexicon-empty-state.png" alt="专业词库空状态" width="100%" /></a>
<br /><sub><b>06 · 添加课程词汇</b><br />本地词库帮助识别专业术语。</sub>
</td>
</tr>
<tr>
<td valign="top">
<a href="./docs/screenshots/about-and-licenses.png"><img src="./docs/screenshots/readme/about-and-licenses.png" alt="关于与第三方许可" width="100%" /></a>
<br /><sub><b>07 · 透明的基础</b><br />第三方组件与许可始终可见。</sub>
</td>
<td valign="top">
<a href="./docs/screenshots/audio-import-selected.png"><img src="./docs/screenshots/readme/audio-import-selected.png" alt="已选择的中文课堂音频" width="100%" /></a>
<br /><sub><b>08 · 随时回到证据</b><br />原始录音与每个派生版本保持关联。</sub>
</td>
</tr>
</table>

## 本地 ASR 档位

| 档位 | 适用场景 | 加速方式 | `v0.3.0-alpha.1` 状态 |
| --- | --- | --- | --- |
| **Fun-ASR-Nano-2512** | 没有可用 NVIDIA CUDA 的机器 | CPU | CPU 安装和运行路径已完成代表性验证 |
| **Qwen3-ASR-0.6B** | 显存较小的 CUDA 机器 | NVIDIA CUDA | 注册表和安装路径已准备，独立 GPU 复测暂缓 |
| **Qwen3-ASR-1.7B** | 更高质量的本地转写 | NVIDIA CUDA | 已完成代表性 RTX 3060 GPU 冒烟测试 |

Qwen 档位共用 CUDA Runtime 与 ForcedAligner。约 4.6 GB（约 4.3 GiB）的 Runtime 不包含在本次 Release 中，也不会提交进 Git；在它被放置到稳定 HTTPS 地址并标记为 `published` 前，安装器会保持下载门禁，避免展示失效下载。发布方法见 [CUDA Runtime 说明](./docs/CUDA_RUNTIME.md)。

## 隐私设计

- **音频、源文件和原始转写保留在本机。**
- 本地 ASR 不上传音频。
- 文本模型请求需要明确授权和用户配置的服务商。
- 只有在对应权限开启后，源文本片段或结构化词库内容才会发送。
- 原始材料不可变；编辑会创建带来源的新版本。

详见 [隐私与安全](./docs/PRIVACY_AND_SECURITY.md)、[模型来源](./docs/MODEL_REGISTRY.md) 和 [第三方声明](./THIRD_PARTY_NOTICES.md)。

## 下载与运行

从 [V3 预发布版](https://github.com/Cecilia-Elaina/verilecture-v3/releases/tag/v0.3.0-alpha.1) 下载当前可用的 Windows x64 NSIS 安装包。这是 alpha 版本，请先备份重要录音。后续 `v*` 标签会由同一个发布工作流生成 Windows、Linux 和 macOS 安装包。

首次启动建议：

1. 阅读隐私边界并确认授权范围。
2. 让课溯扫描硬件、CUDA、存储和模型权限。
3. 安装推荐的本地 ASR 档位。
4. 先用短音频测试，再导入完整课堂录音。

没有可用 NVIDIA GPU 时请选择 **Fun-ASR-Nano-2512**。Qwen 安装请等待 Release 说明明确写出 CUDA Runtime 已公开发布。

## 从源码构建

受支持的应用源码位于 `frontend/`，构建生成物不会进入 Git。

```powershell
Set-Location .\frontend
pnpm install --frozen-lockfile
pnpm typecheck
pnpm test
pnpm build
pnpm exec tauri build --ci --bundles nsis      # Windows
# pnpm exec tauri build --ci --bundles appimage # Linux
# pnpm exec tauri build --ci --bundles dmg      # macOS
```

Linux 构建需要 Tauri 的 WebKitGTK 开发依赖，macOS 构建需要 Xcode Command Line Tools；具体依赖见 [Tauri prerequisites](https://tauri.app/start/prerequisites/) 和 [平台构建说明](./docs/PLATFORM_BUILD.md)。Windows 构建会在构建时获取并校验 FFmpeg；Linux/macOS 当前使用系统 `ffmpeg`，不会把大体积第三方二进制提交进 Git。

## 版本状态与许可

`v0.3.0-alpha.1` 是 V3 的第一个公开里程碑，不是最终稳定版。三平台原生构建已纳入 CI；Windows x64 的本地工作流、引导、硬件路由、CPU 路径和代表性 CUDA 路径已完成。Linux/macOS 的本地 ASR 运行时、CUDA Runtime 公共托管和干净环境安装仍需单独验收。

查看 [已知限制](./docs/KNOWN_LIMITATIONS.md)。

课溯采用 [MIT License](./LICENSE)。项目保留 Meetily 上游归属和第三方软件许可，请阅读 [NOTICE](./NOTICE) 与 [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md)。

**开发者：Cecilia-Elaina**

<div align="center">

<p>
  <strong>简体中文</strong>
  &nbsp;·&nbsp;
  <a href="./README.en.md">English</a>
</p>

</div>
