# Architecture

## Phase boundary

Phase 2 remains read-only. Mounted-disk metadata and backend-owned targeted cache scans are available. Settings and scan findings are persisted only inside DiskSage's application-data directory. Cleanup types are not commands and cannot be invoked by the frontend.

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
        +-- scan coordinator (one heavy scan, cancellable worker)
        +-- rules registry (versioned backend-owned targets)
        +-- findings repository (flat local NDJSON pages)
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

Heavy filesystem work runs outside Tauri's main thread. The scanner uses a bounded depth-first directory stack, keeps one directory handle active at a time, does not follow symlinks, does not cross devices, checks cancellation between entries, and throttles progress to at most roughly seven events per second.

## Frontend state

Disk metadata and settings have independent Zustand stores. Findings will have one canonical store when added; large finding lists must never be copied between page stores. IPC errors are normalized before reaching presentation components so raw Rust details are not shown in normal UI.

## Persistence

Phase 1 uses schema-versioned JSON settings. Writes use a sibling temporary file, `sync_all`, and atomic rename. Invalid or corrupt settings are reported and left unchanged. SQLite migrations are deferred until scan findings and cleanup history require indexed, bounded persistence.
