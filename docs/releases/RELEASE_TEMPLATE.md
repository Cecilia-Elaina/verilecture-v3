# 课溯 · VeriLecture `vX.Y.Z`

> 把课堂录音变成可核对的复习重点。<br />
> *Trace every key point back to the lecture.*

Write the release as a user-facing handoff. Explain what a person can download and try before listing implementation details. Copy this file to `docs/releases/<tag>.md`, replace every placeholder, and remove this instruction paragraph before tagging.

## What this release makes possible

State the smallest concrete outcome a user can verify in this release.

- [ ] A user-visible workflow or result.
- [ ] A source-preserving or privacy boundary that changed.
- [ ] A documentation or packaging change that affects installation.

## What to try

Give two to four short actions. Use real entry points, not future intentions.

1.
2.
3.

## Package and runtime status

Package availability and native local-ASR acceptance are separate facts.

| Platform | Package | Desktop package | Native local ASR | Notes |
| --- | --- | --- | --- | --- |
| Windows x64 | NSIS | Published / pending | Verified / pending |  |
| Linux x64 | AppImage | Published / pending | Verified / pending |  |
| macOS | DMG | Published / pending | Verified / pending |  |

## Trust and data boundary

State where audio, raw transcripts, course files, lexicons, and generated results stay. If a text provider is involved, state exactly what is sent and which consent is required.

## Documentation visuals

If a real result capture is not accepted yet, a generated concept mockup may explain the intended information hierarchy only when it is visibly labelled as a concept and described as non-evidence. Keep the real capture requirement in the release note.

## Important limitations

- This is an Alpha release unless the project explicitly says otherwise.
- Do not infer runtime support from the existence of a desktop package.
- List pending hardware, provider, CUDA Runtime, clean-machine, signing, or long-audio checks.
- Tell users to back up important recordings when the release is pre-release software.

## Checksums and downloads

Link each public installer and its matching SHA256 file. Use final public asset names before calculating checksums.

- [Windows x64 installer]()
- [Linux x64 AppImage]()
- [macOS DMG]()
- [SHA256 files]()

## Technical notes

Keep build, dependency, migration, or CI details here after the user-facing sections. Link to the relevant document instead of repeating a long procedure.

## License and attribution

Link [LICENSE](../../LICENSE), [NOTICE](../../NOTICE), and [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md).
