[CmdletBinding()]
param(
    [string]$SourcePath,
    [string]$CacheRoot
)

$ErrorActionPreference = 'Stop'

$expectedSha256 = '09948d4cdd0650da6ff5a87577469f2a218dc2615ae379f8f734d24c49de0f73'
$downloadUrl = 'https://github.com/GyanD/codexffmpeg/releases/download/8.1.1/ffmpeg-8.1.1-full_build.zip'
$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$destination = Join-Path $repoRoot 'frontend\src-tauri\resources\ffmpeg\ffmpeg.exe'

if ([string]::IsNullOrWhiteSpace($CacheRoot)) {
    if (-not [string]::IsNullOrWhiteSpace($env:RUNNER_TEMP)) {
        $CacheRoot = Join-Path $env:RUNNER_TEMP 'verilecture-ffmpeg'
    } else {
        $CacheRoot = 'D:\Dev\caches\verilecture\ffmpeg'
    }
}

function Get-VerifiedExecutable([string]$candidate) {
    if (-not (Test-Path -LiteralPath $candidate -PathType Leaf)) {
        throw "FFmpeg source file was not found: $candidate"
    }
    $hash = (Get-FileHash -LiteralPath $candidate -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($hash -ne $expectedSha256) {
        throw "FFmpeg SHA-256 mismatch. Expected $expectedSha256, got $hash."
    }
    return $candidate
}

$resolvedSource = $null
$extractRoot = $null
try {
    if (-not [string]::IsNullOrWhiteSpace($SourcePath)) {
        $resolvedSourcePath = (Resolve-Path -LiteralPath $SourcePath).Path
        if ((Get-Item -LiteralPath $resolvedSourcePath).PSIsContainer) {
            $resolvedSource = Get-VerifiedExecutable (Join-Path $resolvedSourcePath 'ffmpeg.exe')
        } elseif ([IO.Path]::GetExtension($resolvedSourcePath).ToLowerInvariant() -eq '.zip') {
            $extractRoot = Join-Path $CacheRoot 'source-extract'
            if (Test-Path -LiteralPath $extractRoot) { Remove-Item -LiteralPath $extractRoot -Recurse -Force }
            New-Item -ItemType Directory -Force -Path $extractRoot | Out-Null
            Expand-Archive -LiteralPath $resolvedSourcePath -DestinationPath $extractRoot -Force
            $resolvedSource = Get-VerifiedExecutable ((Get-ChildItem -LiteralPath $extractRoot -Filter 'ffmpeg.exe' -File -Recurse | Select-Object -First 1).FullName)
        } else {
            $resolvedSource = Get-VerifiedExecutable $resolvedSourcePath
        }
    } else {
        New-Item -ItemType Directory -Force -Path $CacheRoot | Out-Null
        $archive = Join-Path $CacheRoot 'ffmpeg-8.1.1-full_build.zip'
        if (-not (Test-Path -LiteralPath $archive -PathType Leaf)) {
            Write-Host "Downloading pinned FFmpeg archive..."
            Invoke-WebRequest -Uri $downloadUrl -OutFile $archive -UseBasicParsing
        }
        $extractRoot = Join-Path $CacheRoot 'download-extract'
        if (Test-Path -LiteralPath $extractRoot) { Remove-Item -LiteralPath $extractRoot -Recurse -Force }
        New-Item -ItemType Directory -Force -Path $extractRoot | Out-Null
        Expand-Archive -LiteralPath $archive -DestinationPath $extractRoot -Force
        $ffmpeg = Get-ChildItem -LiteralPath $extractRoot -Filter 'ffmpeg.exe' -File -Recurse | Select-Object -First 1
        if ($null -eq $ffmpeg) { throw 'The downloaded archive did not contain ffmpeg.exe.' }
        $resolvedSource = Get-VerifiedExecutable $ffmpeg.FullName
    }

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $destination) | Out-Null
    Copy-Item -LiteralPath $resolvedSource -Destination $destination -Force
    Write-Host "FFmpeg verified and staged at $destination"
} finally {
    if ($null -ne $extractRoot -and (Test-Path -LiteralPath $extractRoot)) {
        Remove-Item -LiteralPath $extractRoot -Recurse -Force
    }
}
