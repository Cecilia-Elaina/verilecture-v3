# Contributing to 课溯 · VeriLecture

Thank you for taking the time to improve the project. Contributions should make the product easier to understand, safer to use, or easier to verify. The current scope is a local-first desktop app for lecture recordings; please keep unrelated refactors out of a documentation or website change.

## Before you start

Read the documents that match your change:

- [README](./README.md) for product scope and current Alpha status.
- [Known Limitations](./docs/KNOWN_LIMITATIONS.md) before making a platform or capability claim.
- [Privacy and Security](./docs/PRIVACY_AND_SECURITY.md) before handling audio, transcripts, textbooks, lexicons, providers, or keys.
- [Writing Style Guide](./writing/STYLE_GUIDE.md) before changing any human-facing sentence.
- [Platform Build Notes](./docs/PLATFORM_BUILD.md) before changing packaging or release behavior.

Do not commit classroom recordings, raw transcripts, API keys, model weights, runtime archives, generated builds, or machine-specific paths. Use a short, sanitized sample when a test genuinely needs an input file.

## Local development

The supported application source is under `frontend/`.

```powershell
Set-Location .\frontend
pnpm install --frozen-lockfile
pnpm typecheck
pnpm test
pnpm build
```

For local Tauri development, provide an explicit sidecar Python path when needed:

```powershell
$env:VERILECTURE_DEV_PYTHON = "D:\path\to\python.exe"
pnpm tauri:dev
```

Do not make a release bundle depend silently on a user-installed Python. Release bundles must provide their own embedded runtime.

## Website and documentation changes

The public product site lives under `site/` and is deployed by `.github/workflows/pages.yml`. For a copy or presentation-only change:

1. Keep the product name as **课溯 · VeriLecture**.
2. Keep the Chinese slogan as **把课堂录音变成可核对的复习重点。**
3. Keep the English slogan as **Trace every key point back to the lecture.**
4. State whether a claim is current, representative, pending, or not yet verified.
5. Use real screenshots when making evidence claims. If a result screen is not accepted, a clearly labelled concept mockup may explain the intended hierarchy, but it must never be called a capture; keep the release, platform, and validation scope with the evidence.
6. Check both Chinese and English pages at desktop and mobile widths.

For changed prose, use the repository's writing sequence:

```text
Humanizer-style review → Vale
```

## Validation expectations

Match validation to impact:

- README, release notes, or static site copy: inspect the rendered diff, run the applicable prose check, and run focused site checks.
- Website structure or script changes: run `node --check site/script.js` and inspect navigation, language switching, downloads, keyboard focus, reduced motion, and mobile overflow.
- Application or shared contract changes: run `pnpm typecheck`, `pnpm test`, and `pnpm build` from `frontend/`, plus the relevant targeted checks.
- Packaging, runtime, security, or release changes: use the affected acceptance workflow and state clearly what still needs real hardware, a clean machine, or a hosted artifact.

Passing a static check does not prove real Windows hardware, provider, CUDA Runtime, Linux, or macOS acceptance.

## Pull requests

Keep each pull request narrow enough to review. The description should state:

- what changed;
- which user-facing claim or behavior it affects;
- what was checked;
- what remains unverified;
- whether screenshots or release notes need an update.

Do not include AI conversation logs, handoff notes, temporary captures, caches, or private data in a pull request.

## Release changes

For a new tagged release:

1. Update `frontend/package.json` and the corresponding application metadata when the version changes.
2. Add a user-facing entry to [CHANGELOG.md](./CHANGELOG.md).
3. Copy [the release template](./docs/releases/RELEASE_TEMPLATE.md) to `docs/releases/<tag>.md`.
4. Describe package availability separately from native local-ASR acceptance.
5. Publish and verify checksums only after the public asset names are final.
6. Keep large CUDA Runtime assets outside GitHub Release until their hosting, integrity, and runtime checks are complete.

The release workflow builds platform packages from a tag and uses the matching file under `docs/releases/`. A release note must not claim a platform or runtime is ready merely because a package was produced.

## Reporting a problem

Use a GitHub Issue for a reproducible, non-sensitive bug. Include the operating system, app version, relevant model tier, and a minimal reproduction without attaching private recordings or credentials. Use GitHub's private vulnerability reporting flow for security-sensitive issues; see [SECURITY.md](./SECURITY.md).

## License

By contributing, you agree that your contribution is provided under the repository's [MIT License](./LICENSE).
