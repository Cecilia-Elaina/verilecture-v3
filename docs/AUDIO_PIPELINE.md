# Audio pipeline

1. Validate the user-selected path and copy the source into an application-
   managed, random-ID directory without deleting or modifying the source.
2. Decode WAV directly; use a release-approved FFmpeg fallback for other
   supported containers.
3. Convert to mono 16 kHz finite float samples and clamp/resample safely.
4. Detect speech ranges and split long ranges into bounded ASR chunks.
5. Run the selected local sidecar; Qwen uses ForcedAligner timestamps and Fun
   uses its VAD segment ranges.
6. Store `raw` segments first, then create a separate calibrated version.
7. Generate exam points only through a consented text provider and validate
   every referenced segment and audio range before saving.

The current synchronous implementation is safe for small/medium files. Large
file streaming, resumable job checkpoints and progress persistence remain
tracked in `docs/PROGRESS.md`.

