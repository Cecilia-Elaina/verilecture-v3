# Local development

Use PowerShell from `D:\Download\verilecture-v3`. If Cargo or model downloads
need the local proxy, use process-scoped environment variables rather than
committing proxy settings:

```powershell
$env:CARGO_NET_GIT_FETCH_WITH_CLI = 'true'
$env:CARGO_HTTP_PROXY = 'http://127.0.0.1:7890'
$env:HTTPS_PROXY = 'http://127.0.0.1:7890'
$env:HTTP_PROXY = 'http://127.0.0.1:7890'
```

The user previously configured Git HTTPS globally, but this project does not
run Git operations. Do not put API keys, model weights or user audio in the
source tree.

