# VeriLecture agent instructions

`CODEX_MASTER_SPEC.md` is the product and execution contract for this repository.

- Build the supported application in `frontend/src-tauri/src` and `frontend/src`; `backend/` is an archived reference only.
- Preserve Meetily's stable audio capture, recording, database, player, editor, and Tauri IPC behavior. Prefer migrations and adapters over rewrites.
- Never overwrite original audio or raw transcripts. Corrections, calibration, and user edits are new versions with provenance.
- Never upload audio, transcripts, or source excerpts without the matching recorded consent.
- Keep explicit teacher statements separate from repeated emphasis, inferred topics, and textbook topics.
- Treat negation, exam scope, numbers, units, formulas, and abbreviations as high-risk changes.
- Ordinary users choose a course type and local/cloud processing, never a model name. Technical routing stays in diagnostics.
- All new user-facing copy is localized in Simplified Chinese and English; Simplified Chinese is the default.
- Run proportionate frontend, Rust, migration, and packaging checks for every completed phase. Do not claim unverified functionality.
- Keep `docs/PROGRESS.md` and `docs/DECISIONS.md` current, and preserve upstream MIT attribution.

