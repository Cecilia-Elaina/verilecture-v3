# CUDA Runtime 托管发布清单

## 推荐托管方式

当前 Runtime 压缩包为 `4,617,121,514` bytes（约 4.3 GiB），不能作为单个 GitHub Release 资产发布；GitHub 的单个 Release 资产上限为 2 GiB。因此 V0.3 的首选托管方式是支持大对象、HTTPS 和 HTTP Range 的对象存储，建议使用阿里云 OSS。

建议使用版本化且不可变的对象路径，例如：

```text
Bucket：由发布者创建的 OSS Bucket
对象：verilecture/runtime/0.3.0-alpha.1/verilecture-asr-runtime-cuda-qwen-fun-windows-x64.zip
```

对应的固定 HTTPS 地址应为：

```text
https://<bucket>.<endpoint>/verilecture/runtime/0.3.0-alpha.1/verilecture-asr-runtime-cuda-qwen-fun-windows-x64.zip
```

对象需要允许客户端匿名 HTTPS GET；建议只给这个对象设置 `public-read`，不要设置 `public-read-write`。如果使用 CDN 或自定义域名，必须保留 HTTPS、准确的 Content-Length 和 Range 响应。

该地址目前尚未确定，Registry 仍应保持 `pending-publication`；发布者完成上传并把最终 URL 提供给开发环境后，才能改成 `published`。

## 发布前检查

在将镜像状态改为 `published` 之前，需要在可联网环境完成：

1. 创建 OSS Bucket 和最小权限的发布账号；上传账号只在发布环境使用，访问密钥不能写入应用或 Registry。
2. 使用 OSS 分片上传完整 Zip64 资产，确认上传后的对象路径和文件名没有被改变。
3. 把对象设置为 `public-read`，生成固定 HTTPS URL；不要使用会过期的预签名 URL 作为内置 Registry 地址。
4. 从一台干净 Windows x64 机器下载，确认 HTTP 状态为 200，不能只看网页显示。
5. 单独验证 `Range: bytes=0-0` 返回 206 和正确的总大小，确认断点续传可用。
6. 对下载文件计算 SHA-256，必须等于：

   ```text
   4eafd198228821c9f5ca36ebd62a4ded53df6083ff1c3f8283127a8f9bc9a665
   ```

7. 确认 Content-Length 或最终文件大小为 `4,617,121,514` bytes。
8. 用 7-Zip 或应用自己的 Zip64 解压路径检查文件数和运行时 manifest。
9. 在带 NVIDIA GPU 的机器上执行 CUDA probe 和 Qwen 1.7B 冒烟测试；0.6B 与 1.7B 共用该 Runtime，本轮不再单独重复实机测试。
10. 只有上述步骤通过后，才把 Registry 中的镜像和运行时状态改为 `published`。

## 版本策略

推荐为每个应用版本建立不可变的 OSS 对象路径，并将运行时 URL 指向该版本路径。这样旧版本不会因为未来覆盖同名对象而下载到不兼容的运行时。若必须替换同一对象路径，应同步更新 SHA-256/bytes 并重新构建应用，不能只改远端文件。

运行时和模型权重应作为 Release 独立资产，不要打入 Windows 安装包。安装包只负责应用、资源、CPU/Fun-ASR-Nano 路径和 Registry；GPU 用户首次选择 Qwen 档位时再按 Registry 下载 CUDA Runtime 和模型权重。

## 404 后的处理

如果对象被删除、路径改名或资产 URL 返回 404：

- 不修改应用二进制中的 URL 常量。
- 在 Registry 增加新的 published mirror，或者将原 mirror 标记为 disabled。
- 保持旧资产的 SHA-256 记录不变，除非确实构建了新的运行时。
- 让客户端通过 Registry 更新获取新地址；下载器仍需进行大小、SHA-256、manifest、probe 和冒烟测试。
