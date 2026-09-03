# Result-screen screenshot brief

Status: **Windows x64 real result capture accepted; concept mockup retained as a design reference**.

The public README and product site now use the accepted Windows x64 result capture alongside real import, settings, and course-lexicon screenshots. The original concept image explains the intended information hierarchy and remains available only as a design reference; it must not be described as a real application capture or used as runtime evidence.

## Current concept asset

- `site/assets/product-result-concept.png` — generated concept mockup retained for design history; the image itself is marked `概念示意 · 非实机截图`.
- `site/assets/product-trace-result.webp` — cropped Windows x64 result capture used by the public product tour.

Keep this label in the concept image, but do not use the concept image as the primary README/site result view now that the Windows x64 capture has passed the acceptance path below.

## Accepted capture

The accepted Windows x64 application run used a sanitized, non-private sample recording and completed the following path:

1. Import a lecture recording.
2. Complete a verified local transcription route.
3. Inspect the raw and calibrated transcript views.
4. Generate review points through the configured, consented text service when that step is enabled.
5. Click a review point and confirm that the audio position follows its stored timestamp.

Evidence added:

- `docs/screenshots/result-points-trace.png` — review points, source timestamps, and playback entry points.
- `docs/screenshots/result-transcript-source.png` — transcript view, raw-transcript distinction, and source-audio control.
- `site/assets/product-trace-result.webp` — cropped result view used in the public product tour.

Scope recorded for this evidence: Windows x64, the current Alpha runtime, local Fun-ASR route, consented text organization, and timestamp-based playback return. This does not certify Linux/macOS local-ASR runtime support.

The capture should show the actual application detail view with:

- at least two review points;
- their source timestamps or segment counts;
- the transcript tab with raw/calibrated distinction;
- the audio control or visible playback position;
- the notice that automated organization can be wrong and should be checked.

Do not include student names, school identifiers, private paths, API keys, or unredacted classroom content.

## Recommended assets

Keep only the real views that explain the value chain:

- `docs/screenshots/result-points-trace.png` — review points with source timestamps;
- `docs/screenshots/result-transcript-source.png` — transcript and source-return interaction;
- `site/assets/product-trace-result.webp` — a cropped website showcase image derived from the same accepted run.

The concept panel and its labels in `README.md` and `README.en.md`, and the first Product Tour tab on the website, now point to the accepted result capture. Record the release, operating system, model tier, and validation scope next to the screenshot in the change description.

## Acceptance note

A screenshot is evidence of the captured UI state, not evidence that every platform or runtime supports the same path. Keep the platform table and [Known Limitations](../KNOWN_LIMITATIONS.md) synchronized with the actual acceptance result.
