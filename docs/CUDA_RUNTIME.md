# CUDA Runtime 发布说明

当前 CUDA Runtime 是 Windows x64 专用的 Qwen 本地 ASR 运行包。它不是让用户
额外安装的完整 CUDA Toolkit，而是应用在首次使用 Qwen 时下载、校验并安装的
运行时 bundle；用户机器仍需要兼容的 NVIDIA 驱动和满足显存/内存要求。

## 当前发布位置

Windows x64 的 Qwen CUDA Runtime 已发布到 [Hugging Face Runtime 仓库](https://huggingface.co/moyuling/verilecture-asr-runtime)。应用使用固定的公开 HTTPS 直链下载归档，并在安装前核对文件大小和 SHA-256：

`https://huggingface.co/moyuling/verilecture-asr-runtime/resolve/main/verilecture-asr-runtime-cuda-qwen-fun-windows-x64.zip`

## 为什么不直接放进 GitHub Release

当前压缩包约 `4.6 GB`（约 `4.3 GiB`），超过 GitHub 单个 Release 资产的大小限制。不要把它
提交进 Git，也不要把它作为一个大文件直接上传到当前 Release。应使用支持
HTTPS、Range 请求和大文件下载的对象存储，例如 S3、R2 或 OSS。

## 发布步骤

1. 在受控的 Windows CUDA 环境生成归档，并保留文件名
   `verilecture-asr-runtime-cuda-qwen-fun-windows-x64.zip`。
2. 用仓库中的校验器检查本地归档：

   ```powershell
   python -B frontend/scripts/validate_runtime_registry.py `
     --asset D:\path\to\verilecture-asr-runtime-cuda-qwen-fun-windows-x64.zip
   ```

3. 用真实归档运行本地接受测试。它会测试中断下载、HTTP Range 续传、大小与
   SHA-256、Zip64 解压、原子安装和 CUDA probe：

   ```powershell
   python -B frontend/scripts/accept_local_runtime.py `
     --asset D:\path\to\verilecture-asr-runtime-cuda-qwen-fun-windows-x64.zip
   ```

4. 将同一个文件上传到支持大文件的稳定 HTTPS 地址。对象必须支持公开读取
   （或应用已实现的鉴权方式）、`HEAD`/`GET`、HTTP Range，并返回正确的
   `Content-Length`；不要上传登录页、HTML 错误页或会变化的临时链接。
5. 从公网检查地址后，再更新
   `frontend/src-tauri/resources/runtime_registry.json`：填写真实 `url`，将
   runtime 和 mirror 的 `status` 从 `pending-publication` 改为 `published`，
   并保持 `compressedBytes`、`installedBytes` 和 `sha256` 与归档完全一致。
6. 重新运行校验器、Rust 测试，并在干净 Windows 用户目录中完成一次从公开
   地址下载到 Qwen 冒烟测试的验收，然后再发布对应的应用 Release。

注册表当前指向 Hugging Face 的公开归档。若后续替换归档，先完成公网 HEAD、Range、大小和 SHA-256 校验，再更新注册表；验收完成前保持 `pending-publication`，这样应用不会下载未确认的 Qwen Runtime。

Linux 和 macOS 的本地 ASR Runtime 另行构建、授权、校验和验收；不能复用这个
Windows `.exe` bundle。
