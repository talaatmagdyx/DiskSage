# Architecture

## Phase boundary

Phase 1 is a read-only application foundation. The only filesystem-facing IPC currently registered is mounted-disk metadata lookup. Settings may be read and atomically written to DiskSage's own application-data directory. Cleanup types are not commands and cannot be invoked by the frontend.

## Trust boundaries

```text
React UI (untrusted request data)
        |
        | typed Tauri IPC
        v
Rust command adapters
        |
        +-- disk metadata service (read-only, bounded worker)
        +-- settings repository (app-owned file only)
        |
        v [future phases only]
validated finding IDs -> backend cleanup plan -> revalidation -> executor
```

The frontend is not trusted to authorize a path. A future cleanup request will contain backend-issued finding IDs, and execution will contain only an opaque plan ID plus confirmation token. The backend will recover paths from local state, apply the protected-path policy, and revalidate metadata immediately before any operation.

## Backend modules

- `commands`: narrow typed IPC adapters.
- `domain`: serialization contracts independent of Tauri.
- `platform`: OS-facing read-only services.
- `persistence`: app-owned local repositories.
- `safety`: protected-path policy; future validators compose here.
- `observability`: structured logs with conservative production filters.

Heavy filesystem work must run outside Tauri's main thread. `list_disks` already uses a blocking worker to preserve this invariant.

## Frontend state

Disk metadata and settings have independent Zustand stores. Findings will have one canonical store when added; large finding lists must never be copied between page stores. IPC errors are normalized before reaching presentation components so raw Rust details are not shown in normal UI.

## Persistence

Phase 1 uses schema-versioned JSON settings. Writes use a sibling temporary file, `sync_all`, and atomic rename. Invalid or corrupt settings are reported and left unchanged. SQLite migrations are deferred until scan findings and cleanup history require indexed, bounded persistence.

