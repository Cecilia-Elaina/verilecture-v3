# Security and privacy reporting

课溯 · VeriLecture is an Alpha desktop application. It is designed around local-first storage and explicit consent, but those design goals are not a guarantee that every environment or provider is safe. Report concrete security or privacy problems so they can be checked against the current code and release.

## Supported release

The actively supported public line is the latest Alpha release shown in the repository README and Releases page. Older pre-releases may not receive fixes.

## Report privately

Use GitHub's private vulnerability reporting or Security Advisory flow for:

- possible audio, transcript, textbook, lexicon, or generated-result disclosure;
- consent bypasses or unexpected provider requests;
- API-key exposure or credential-store problems;
- archive extraction, path traversal, installer, or update issues;
- vulnerabilities that could execute code or access files outside the intended workspace.

Do not open a public Issue until the maintainer confirms that the report contains no sensitive details.

Include the affected version, operating system, reproduction steps, expected boundary, and a minimal sanitized example. Do not attach classroom recordings, student information, raw transcripts, API keys, model weights, or runtime archives.

## Privacy boundaries to verify

The intended boundaries are documented in [Privacy and Security](./docs/PRIVACY_AND_SECURITY.md):

- audio and source files remain local by default;
- local ASR does not upload audio;
- transcript text, structured lexicon data, and limited textbook excerpts require separate consent before a configured text provider receives them;
- source files and raw transcripts are not deleted or overwritten by import, calibration, or editing;
- model downloads are verified before installation.

If observed behavior differs from these statements, treat it as a reportable issue even when the provider or operating system is still in Alpha validation.

## Public bug reports

Use a public GitHub Issue for a non-sensitive reproducible bug. Remove personal paths, recording names, transcript text, credentials, and machine identifiers before posting.
