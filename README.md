# 课溯 · VeriLecture

<div align="center">

**把课堂录音变成可核对的复习重点。**

`Trace every key point back to the lecture.`

[![版本](https://img.shields.io/badge/版本-0.3.0--alpha.4-e56b35?style=flat-square)](https://github.com/xiajiadi/verilecture-v3/releases/tag/v0.3.0-alpha.4)
[![平台](https://img.shields.io/badge/平台-Windows%20%7C%20Linux%20%7C%20macOS-1f5f4f?style=flat-square)](https://github.com/xiajiadi/verilecture-v3/releases)
[![隐私](https://img.shields.io/badge/隐私-本地优先-2d8065?style=flat-square)](#隐私设计)
[![许可证](https://img.shields.io/badge/许可证-MIT-5a716b?style=flat-square)](./LICENSE)

<p>
  <strong>简体中文</strong>
  &nbsp;·&nbsp;
  <a href="./README.en.md">English</a>
</p>

<p>
  <a href="https://xiajiadi.github.io/verilecture-v3/">产品主页</a>
  &nbsp;·&nbsp;
  <a href="https://xiajiadi.github.io/verilecture-v3/">Product website</a>
</p>

<a href="./docs/screenshots/audio-import-selected.png"><img src="./docs/screenshots/readme/audio-import-selected.png" alt="课溯音频导入界面" width="900" /></a>

<sub>本地优先 · 原始音频、原始转写和派生版本留在本机</sub>

</div>

## 课溯解决什么问题

课后复习时，找到关键原话往往比重新听一遍更难。课溯把课堂录音整理成复习重点，并保留每条重点的来源。需要确认时，可以直接回到对应的课堂原音。

<p align="center">
  <a href="./docs/diagrams/evidence-chain-zh.svg"><img src="./docs/diagrams/evidence-chain-zh.svg" alt="课溯可回溯证据链流程图" width="900" /></a>
</p>

## 工作方式

1. **导入课堂录音**：支持 WAV、MP3、M4A、MP4、FLAC 和 OGG。
2. **保留原始转写**：本地优先处理，原始文字单独保存。
3. **用课程材料校准**：教材和专业词库帮助检查高风险术语。
4. **整理课堂重点**：老师明确说的内容与系统推测分开呈现。
5. **回到原音核对**：从重点跳到对应时间点，也可以导出结果。

## 主要能力

- **本地优先**：音频、原始转写和课程文件默认保留在本机。
- **按设备选择路线**：首次启动检查 CPU、内存、GPU、CUDA 和存储，再推荐可用的本地转写档位。
- **多档本地转写**：Fun-ASR-Nano-2512 适用于 CPU；Qwen3-ASR-0.6B 和 Qwen3-ASR-1.7B 适用于 CUDA 机器。
- **原始内容不覆盖**：修订、校准和用户编辑都会生成带来源的新版本。
- **老师的话与推测分开**：明确考点、不考内容、重复强调和系统推测分别保留。
- **文本模型可选**：重点整理是独立步骤；只有在授权后，必要文本才会发送到所选服务商。

## 平台状态

`v0.3.0-alpha.4` 已发布 Windows、Linux 和 macOS 桌面安装包。Linux 和 macOS 包先提供桌面应用；原生本地转写运行时仍待单独验证后发布。

| 平台 | 安装包 | 当前本地转写状态 |
| --- | --- | --- |
| Windows x64 | NSIS | Fun-ASR 与 CUDA Runtime 路径可用 |
| Linux x64 | AppImage | 桌面包已发布；本地转写运行时待发布 |
| macOS | DMG | 桌面包已发布；本地转写运行时待发布 |

## 产品预览

<table>
<tr>
<td width="50%" valign="top">
<a href="./docs/screenshots/onboarding-model-selection.png"><img src="./docs/screenshots/readme/onboarding-model-selection.png" alt="选择本地转写档位" width="100%" /></a>
<br /><sub><b>01 · 选择本地转写档位</b><br />根据硬件选择可用的本地路线。</sub>
</td>
<td width="50%" valign="top">
<a href="./docs/screenshots/onboarding-text-model.png"><img src="./docs/screenshots/readme/onboarding-text-model.png" alt="配置文本模型" width="100%" /></a>
<br /><sub><b>02 · 连接文本模型</b><br />只在需要时配置文本整理服务。</sub>
</td>
</tr>
<tr>
<td valign="top">
<a href="./docs/screenshots/onboarding-import-audio.png"><img src="./docs/screenshots/readme/onboarding-import-audio.png" alt="导入课堂音频" width="100%" /></a>
<br /><sub><b>03 · 导入课堂音频</b><br />WAV、MP3 和 M4A 都可直接导入。</sub>
</td>
<td valign="top">
<a href="./docs/screenshots/audio-records-empty.png"><img src="./docs/screenshots/readme/audio-records-empty.png" alt="音频记录空状态" width="100%" /></a>
<br /><sub><b>04 · 查看音频记录</b><br />从记录进入原始转写和回听。</sub>
</td>
</tr>
<tr>
<td valign="top">
<a href="./docs/screenshots/settings-models-and-text.png"><img src="./docs/screenshots/readme/settings-models-and-text.png" alt="模型与文本设置" width="100%" /></a>
<br /><sub><b>05 · 查看运行状态</b><br />硬件、本地转写和文本模型状态集中显示。</sub>
</td>
<td valign="top">
<a href="./docs/screenshots/lexicon-empty-state.png"><img src="./docs/screenshots/readme/lexicon-empty-state.png" alt="专业词库空状态" width="100%" /></a>
<br /><sub><b>06 · 维护课程词库</b><br />用课程词汇校准专业术语。</sub>
</td>
</tr>
<tr>
<td valign="top">
<a href="./docs/screenshots/about-and-licenses.png"><img src="./docs/screenshots/readme/about-and-licenses.png" alt="关于与第三方许可" width="100%" /></a>
<br /><sub><b>07 · 查看许可</b><br />第三方组件和许可列在应用中。</sub>
</td>
<td valign="top">
<a href="./docs/screenshots/audio-import-selected.png"><img src="./docs/screenshots/readme/audio-import-selected.png" alt="已选择的中文课堂音频" width="100%" /></a>
<br /><sub><b>08 · 回到来源</b><br />每个派生版本保留原始录音关联。</sub>
</td>
</tr>
</table>

## 本地转写档位

| 档位 | 适用场景 | 加速方式 | `v0.3.0-alpha.4` 状态 |
| --- | --- | --- | --- |
| **Fun-ASR-Nano-2512** | 没有可用 NVIDIA CUDA 的机器 | CPU | CPU 安装和运行路径已完成代表性验证 |
| **Qwen3-ASR-0.6B** | 显存较小的 CUDA 机器 | NVIDIA CUDA | 注册表和安装路径已准备，独立 GPU 复测暂缓 |
| **Qwen3-ASR-1.7B** | 更高质量的本地转写 | NVIDIA CUDA | 已完成代表性 RTX 3060 GPU 冒烟测试 |

Qwen 档位共用 CUDA Runtime 与 ForcedAligner。约 4.6 GB（约 4.3 GiB）的 Runtime 不包含在本次 Release 中，也不会提交进 Git；在它被放置到稳定 HTTPS 地址并标记为 `published` 前，安装器会保持下载门禁。发布方法见 [CUDA Runtime 说明](./docs/CUDA_RUNTIME.md)。

## 隐私设计

- **音频、源文件和原始转写默认保留在本机。**
- 本地转写不上传音频。
- 文本模型请求需要明确授权和用户配置的服务商。
- 只有在对应权限开启后，源文本片段或结构化词库内容才会发送。
- 原始材料不可变；编辑会创建带来源的新版本。

详见 [隐私与安全](./docs/PRIVACY_AND_SECURITY.md)、[模型来源](./docs/MODEL_REGISTRY.md) 和 [第三方声明](./THIRD_PARTY_NOTICES.md)。

## 下载与运行

从 [V3 预发布版](https://github.com/xiajiadi/verilecture-v3/releases/tag/v0.3.0-alpha.4) 下载 Windows x64 NSIS、Linux x64 AppImage 或 macOS DMG。这是 Alpha 版本，请先备份重要录音。

首次启动：

1. 阅读隐私边界并确认授权范围。
2. 让课溯扫描硬件、CUDA、存储和模型目录。
3. 安装推荐的本地转写档位。
4. 先用短音频测试，再导入完整课堂录音。

没有可用 NVIDIA GPU 时请选择 **Fun-ASR-Nano-2512**。Qwen 安装请等待 Release 说明明确写出 CUDA Runtime 已公开发布。

## 从源码构建

受支持的应用源码位于 `frontend/`，构建生成物不会进入 Git。修改用户可见文字前，请先阅读 [writing/STYLE_GUIDE.md](./writing/STYLE_GUIDE.md)。

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

Linux 构建需要 Tauri 的 WebKitGTK 开发依赖，macOS 构建需要 Xcode Command Line Tools；具体依赖见 [Tauri prerequisites](https://tauri.app/start/prerequisites/) 和 [平台构建说明](./docs/PLATFORM_BUILD.md)。Windows 构建会在构建时获取并校验 FFmpeg；Linux/macOS 当前使用系统 `ffmpeg`，不会把大体积第三方二进制提交进 Git。

## 版本状态与许可

`v0.3.0-alpha.4` 是 V3 的第二个公开里程碑，不是最终稳定版。Windows、Linux 和 macOS 桌面安装包已由发布工作流生成。Windows x64 的本地工作流、引导、硬件路由、CPU 路径和代表性 CUDA 路径已完成；Linux/macOS 的本地转写运行时、CUDA Runtime 公共托管和干净环境安装仍需单独验收。

查看 [已知限制](./docs/KNOWN_LIMITATIONS.md)。

课溯采用 [MIT License](./LICENSE)。项目保留 Meetily 上游归属和第三方软件许可，请阅读 [NOTICE](./NOTICE) 与 [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md)。

**开发者：xiajiadi**

<div align="center">

<p>
  <strong>简体中文</strong>
  &nbsp;·&nbsp;
  <a href="./README.en.md">English</a>
</p>

</div>
