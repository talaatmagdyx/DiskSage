# Architecture

## Phase boundary

Phase 7 adds first-run guidance, adaptive appearance, keyboard-first interaction, virtualized large-list presentation, error recovery, and a redacted diagnostics export. Diagnostics are assembled in Rust from typed aggregate fields and written to an app-owned cache directory; configured paths, finding paths, hashes, contents, and cleanup item records are never serialized. The Phase 6 cleanup safety boundary remains unchanged.

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
        +-- project detector (configured roots + manifest context)
        +-- findings repository (flat local NDJSON pages)
        +-- duplicate coordinator (staged hashing + cancellation)
        +-- duplicate repository (flat local group pages)
        +-- cleanup coordinator (single-use plans and cancellation)
        +-- permanent executor (feature-gated, symlink-rejecting, no retry)
        +-- history repository (local per-item audit records)
        +-- diagnostics exporter (redacted aggregates -> app cache)
        |
        v
validated finding/group IDs -> immutable plan -> canonical/content revalidation -> OS Trash
```

The frontend is not trusted to authorize a cleanup path. Rule cleanup contains backend-issued finding IDs. Duplicate cleanup contains a scan ID plus backend-issued group/copy IDs and an explicit keep ID. Execution contains only an opaque plan ID plus confirmation token. The backend recovers paths from local persistence, freezes canonical targets, applies protected-path policy, re-hashes the keep and Trash copies, and revalidates again immediately before each Trash operation.

## Backend modules

- `commands`: narrow typed IPC adapters.
- `domain`: serialization contracts independent of Tauri.
- `platform`: OS-facing read-only services.
- `persistence`: app-owned local repositories.
- `safety`: protected-path policy and exact known-rule allowlisting.
- `cleanup`: plan coordination, revalidation, and OS Trash adapter.
- `duplicates`: root validation, staged hashing, deterministic keep selection, and keep-one cleanup coordination.
- `scanner::analysis`: bounded custom-root traversal for review-only large files and old installers.
- `observability`: structured logs with conservative production filters.

Heavy filesystem work runs outside Tauri's main thread. Scanners use bounded depth-first directory stacks, do not follow symlinks or cross devices, and check cancellation between entries and hashing chunks. Duplicate analysis retains only file metadata up to a hard candidate ceiling, eliminates unique sizes before hashing, and persists final groups as NDJSON.

Permanent execution shares the same plan consumption, protected-path policy, canonical/type/size/mtime revalidation, cancellation, per-item results, disk refresh, and audit history as Trash. It adds a final symlink/type check inside the executor and never retries a destructive failure.

## Frontend state

Disk metadata, settings, scanning, findings, cleanup, duplicates, and transient notifications have independent Zustand stores. The duplicate store owns keep selection and never allows the current keep copy to enter the Trash selection. IPC errors are normalized before reaching presentation components so raw Rust details are not shown in normal UI. Findings and duplicate groups switch to dynamically measured virtual rows above 100 entries.

## Persistence

Settings use schema-versioned atomic JSON. Scan findings, duplicate groups, and cleanup history use bounded, flat NDJSON records under the application-data directory. SQLite migrations remain appropriate when retention, indexed history queries, or substantially larger scan profiles are introduced.
