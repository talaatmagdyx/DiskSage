# Architecture

## Phase boundary

Phase 3 adds trash-first cleanup for exact safe cache targets. Mounted-disk metadata, targeted scans, settings, findings, and cleanup history remain local. Permanent deletion, expert cleanup, and arbitrary path cleanup are unavailable.

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
        +-- cleanup coordinator (single-use plans and cancellation)
        +-- history repository (local per-item audit records)
        |
        v
validated finding IDs -> immutable plan -> canonical revalidation -> OS Trash
```

The frontend is not trusted to authorize a path. Plan creation contains backend-issued finding IDs, and execution contains only an opaque plan ID plus confirmation token. The backend recovers paths from local scan persistence, freezes the canonical target, applies exact rule and protected-path policies, and revalidates immediately before each Trash operation.

## Backend modules

- `commands`: narrow typed IPC adapters.
- `domain`: serialization contracts independent of Tauri.
- `platform`: OS-facing read-only services.
- `persistence`: app-owned local repositories.
- `safety`: protected-path policy and exact known-rule allowlisting.
- `cleanup`: plan coordination, revalidation, and OS Trash adapter.
- `observability`: structured logs with conservative production filters.

Heavy filesystem work runs outside Tauri's main thread. The scanner uses a bounded depth-first directory stack, keeps one directory handle active at a time, does not follow symlinks, does not cross devices, checks cancellation between entries, and throttles progress to at most roughly seven events per second.

## Frontend state

Disk metadata, settings, scanning, findings, and cleanup have independent Zustand stores. Findings remain canonical in one store and successful cleanup events remove only completed IDs. IPC errors are normalized before reaching presentation components so raw Rust details are not shown in normal UI.

## Persistence

Settings use schema-versioned atomic JSON. Scan findings and cleanup history use bounded, flat NDJSON records under the application-data directory. SQLite migrations remain appropriate when retention, indexed history queries, or substantially larger scan profiles are introduced.
