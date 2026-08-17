# Architecture

```text
React + Vite UI
        ↕ Tauri IPC
Rust orchestration, SQLite, filesystem, keyring, HTTP text providers
        ↕ JSON Lines over stdin/stdout
Python ASR sidecar / official Fun-ASR executable
```

The sidecar never reads SQLite and never receives API keys. Rust owns job
state, persistence, consent, model installation and cloud requests. No fixed
localhost port is used for process communication.

The current source is intentionally compact while the public boundaries are
kept in `src/lib/contracts.ts`, `src/lib/tauri.ts`, `src-tauri/src/audio.rs`,
`models.rs` and `providers.rs`.

