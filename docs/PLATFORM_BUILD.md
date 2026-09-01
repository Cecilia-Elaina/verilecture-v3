# Platform builds

The desktop application is built with Tauri from `frontend/`. GitHub Actions
uses a matrix with one native runner per platform:

| Runner | Package | Tauri target |
| --- | --- | --- |
| Windows | NSIS installer | `nsis` |
| Ubuntu | AppImage | `appimage` |
| macOS | DMG | `dmg` |

The shared Rust and frontend code is compiled on all three runners. Linux uses
the WebKitGTK development packages required by Tauri; macOS uses Xcode Command
Line Tools. The exact Linux package list is kept in
`.github/workflows/v3-ci.yml` and `.github/workflows/v3-release.yml`.

## Local package commands

From `frontend/`, after installing Node.js, pnpm, Rust, and the platform's Tauri
prerequisites:

```sh
pnpm install --frozen-lockfile
pnpm typecheck
pnpm test
pnpm build
pnpm exec tauri build --ci --bundles nsis      # Windows
pnpm exec tauri build --ci --config src-tauri/tauri.linux.conf.json --bundles appimage # Linux
pnpm exec tauri build --ci --config src-tauri/tauri.macos.conf.json --bundles dmg      # macOS
```

Windows packages fetch and verify the pinned FFmpeg binary during the release
build. Linux and macOS currently resolve `ffmpeg` from the host system at run
time. Local ASR sidecars and the CUDA Runtime remain Windows x64 assets until
native equivalents complete their own build, license, checksum, and smoke-test
acceptance.
