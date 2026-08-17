[CmdletBinding()]
param(
    [string]$PythonPath = $env:VERILECTURE_PYTHON,
    [string]$FunCliPath = $env:VERILECTURE_FUN_ASR_CLI,
    [string]$OutputDirectory = '',
    [switch]$CpuOnly
)

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$runtimeSource = Join-Path $repoRoot 'tools\asr\verilecture_asr_runtime.py'
if ([string]::IsNullOrWhiteSpace($PythonPath)) {
    throw 'Set VERILECTURE_PYTHON to the controlled Python 3.12 environment used for the Qwen runtime.'
}
if ([string]::IsNullOrWhiteSpace($FunCliPath)) {
    throw 'Set VERILECTURE_FUN_ASR_CLI to the official Windows llama-funasr-cli.exe.'
}
if (-not (Test-Path -LiteralPath $PythonPath -PathType Leaf)) {
    throw "Python runtime was not found: $PythonPath"
}
if (-not (Test-Path -LiteralPath $FunCliPath -PathType Leaf)) {
    throw "Fun-ASR-Nano runtime was not found: $FunCliPath"
}
if (-not (Test-Path -LiteralPath $runtimeSource -PathType Leaf)) {
    throw "ASR sidecar source was not found: $runtimeSource"
}
if (-not $CpuOnly) {
    $cudaBuildProbe = & $PythonPath -c "import torch, qwen_asr; print('CUDA_BUILD=' + ('1' if torch.version.cuda else '0'))"
    if ($LASTEXITCODE -ne 0 -or (($cudaBuildProbe -join "`n") -notmatch 'CUDA_BUILD=1')) {
        throw 'The full runtime build requires a CUDA-enabled torch wheel and qwen-asr in the controlled Python environment. Use requirements-windows-cuda.txt.'
    }
    Write-Host "CUDA build environment detected: $($cudaBuildProbe -join ' ')"
}
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $OutputDirectory = Join-Path $repoRoot 'src-tauri\resources\asr-runtime'
}

$buildRoot = Join-Path $repoRoot 'output\asr-runtime-build'
$distRoot = Join-Path $buildRoot 'dist'
$workRoot = Join-Path $buildRoot 'work'
$specRoot = Join-Path $buildRoot 'spec'
New-Item -ItemType Directory -Force -Path $buildRoot, $distRoot, $workRoot, $specRoot | Out-Null

$pyInstallerArgs = @(
    '-m', 'PyInstaller',
    '--noconfirm',
    '--clean',
    '--onedir',
    '--name', 'verilecture-asr-runtime',
    '--distpath', $distRoot,
    '--workpath', $workRoot,
    '--specpath', $specRoot
)

if ($CpuOnly) {
    # Fun-ASR-Nano is the supported no-CUDA route.  Keep this installer-side
    # runtime independent of the multi-gigabyte CUDA PyTorch distribution;
    # Qwen CUDA validation uses the separately staged full runtime.
    $pyInstallerArgs += @(
        '--exclude-module', 'torch',
        '--exclude-module', 'numpy',
        '--exclude-module', 'qwen_asr',
        '--exclude-module', 'qwen_omni_utils',
        '--exclude-module', 'transformers',
        '--exclude-module', 'accelerate',
        '--exclude-module', 'nagisa',
        '--exclude-module', 'soynlp',
        '--exclude-module', 'dynet',
        '--exclude-module', 'av',
        '--exclude-module', 'librosa',
        '--exclude-module', 'scipy',
        '--exclude-module', 'sklearn',
        '--exclude-module', 'numba',
        '--exclude-module', 'llvmlite'
    )
} else {
    $pyInstallerArgs += @(
        '--collect-all', 'qwen_asr',
        '--collect-all', 'qwen_omni_utils',
        '--collect-all', 'nagisa',
        '--collect-all', 'soynlp',
        '--hidden-import', 'qwen_asr',
        '--hidden-import', 'six',
        '--hidden-import', 'dynet',
        '--hidden-import', 'nagisa',
        '--hidden-import', 'soynlp',
        '--hidden-import', 'av',
        '--hidden-import', 'transformers',
        '--hidden-import', 'accelerate'
    )
}

$pyInstallerArgs += $runtimeSource

if ($CpuOnly) {
    Write-Host 'Building the CPU/Fun-ASR-Nano VeriLecture ASR sidecar...'
} else {
    Write-Host 'Building the CUDA-capable Qwen/Fun VeriLecture ASR sidecar...'
}
& $PythonPath @pyInstallerArgs
if ($LASTEXITCODE -ne 0) {
    throw "PyInstaller failed with exit code $LASTEXITCODE"
}

$builtDirectory = Join-Path $distRoot 'verilecture-asr-runtime'
$builtExecutable = Join-Path $builtDirectory 'verilecture-asr-runtime.exe'
if (-not (Test-Path -LiteralPath $builtExecutable -PathType Leaf)) {
    throw "PyInstaller did not produce $builtExecutable"
}

New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
$OutputDirectory = (Resolve-Path -LiteralPath $OutputDirectory).Path
Copy-Item -Path (Join-Path $builtDirectory '*') -Destination $OutputDirectory -Recurse -Force
Copy-Item -LiteralPath $FunCliPath -Destination (Join-Path $OutputDirectory 'llama-funasr-cli.exe') -Force

$runtimeFiles = @(
    Get-ChildItem -LiteralPath $OutputDirectory -Recurse -File |
        Where-Object { $_.Name -ne 'runtime-manifest.json' } |
        ForEach-Object {
            $relativePath = $_.FullName.Substring($OutputDirectory.Length + 1).Replace('\', '/')
            [ordered]@{
                path = $relativePath
                bytes = $_.Length
                sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash.ToLowerInvariant()
            }
        }
)
$runtimeManifest = [ordered]@{
    schemaVersion = 1
    runtimeVersion = 'verilecture-asr-runtime/0.3.0-alpha.1'
    platform = 'windows-x86_64'
    runtimeFlavor = if ($CpuOnly) { 'cpu-fun' } else { 'cuda-qwen-fun' }
    cudaBuild = (-not $CpuOnly)
    entrypoint = 'verilecture-asr-runtime.exe'
    sourceEntrypoint = 'verilecture_asr_runtime.py'
    pythonExecutable = 'embedded-in-executable'
    files = $runtimeFiles
    policy = 'release builds must ship the exact embedded runtime; do not fall back to PATH Python'
}
$manifestPath = Join-Path $OutputDirectory 'runtime-manifest.json'
$manifestJson = $runtimeManifest | ConvertTo-Json -Depth 6
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText($manifestPath, $manifestJson, $utf8NoBom)

Write-Host "ASR runtime staged at $OutputDirectory"
Write-Host 'Model weights are intentionally not copied into the application bundle; onboarding downloads only the verified registry artifacts.'
