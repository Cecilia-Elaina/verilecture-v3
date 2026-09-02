# Result-screen screenshot brief

Status: **real capture not captured yet; concept mockup available**.

The public README and product site use a clearly labelled result-screen concept mockup alongside real import, settings, and course-lexicon screenshots. The concept image explains the intended information hierarchy; it must never be described as a real application capture or used as runtime evidence.

## Current concept asset

- `site/assets/product-result-concept.png` — generated concept mockup; the image itself is marked `概念示意 · 非实机截图`.

Keep this label in the image and in the surrounding README/site copy until a real Windows x64 result view passes the acceptance path below.

## Capture to add

Capture a real Windows x64 application run after the following path has been accepted with a sanitized, non-private sample recording:

1. Import a lecture recording.
2. Complete a verified local transcription route.
3. Inspect the raw and calibrated transcript views.
4. Generate review points through the configured, consented text service when that step is enabled.
5. Click a review point and confirm that the audio position follows its stored timestamp.

The capture should show the actual application detail view with:

- at least two review points;
- their source timestamps or segment counts;
- the transcript tab with raw/calibrated distinction;
- the audio control or visible playback position;
- the notice that automated organization can be wrong and should be checked.

Do not include student names, school identifiers, private paths, API keys, or unredacted classroom content.

## Recommended assets

After the capture passes the real runtime check, add only the real views that explain the value chain:

- `docs/screenshots/result-points-trace.png` — review points with source timestamps;
- `docs/screenshots/result-transcript-source.png` — transcript and source-return interaction;
- `site/assets/product-trace-result.webp` — a cropped website showcase image derived from the same accepted run.

Then replace the concept panel and its labels in `README.md` and `README.en.md`, and replace the concept as the first Product Tour tab on the website. Record the release, operating system, model tier, and validation scope next to the screenshot in the change description.

## Acceptance note

A screenshot is evidence of the captured UI state, not evidence that every platform or runtime supports the same path. Keep the platform table and [Known Limitations](../KNOWN_LIMITATIONS.md) synchronized with the actual acceptance result.
