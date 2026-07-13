# IPC contracts

All request structs use strict deserialization with unknown fields denied. Responses use camel-case JSON. Failures use `CommandError`; frontend presentation maps its code to a user-safe message and does not render `details`.

## Registered through Phase 7

| Command | Request | Response | Side effects |
| --- | --- | --- | --- |
| `get_app_info` | none | app name, version, platform, destructive capability flag | none |
| `export_diagnostics` | none | app-cache report path | writes a new redacted JSON report under the app cache and reveals it |
| `list_disks` | none | flat `DiskInfo[]` | reads mounted-disk metadata |
| `get_disk_info` | `{ mountPath }` | `DiskInfo` | reads mounted-disk metadata |
| `get_settings` | none | `AppSettings` | reads app-owned JSON |
| `update_settings` | `{ settings }` | validated `AppSettings` | atomically writes app-owned JSON |
| `get_scan_profiles` | none | `ScanProfile[]` | none |
| `start_scan` | `{ profile, excludedPaths, custom? }` | opaque scan ID | starts one backend-owned targeted or explicitly scoped analysis worker |
| `cancel_scan` | `{ scanId }` | none | signals the matching cancellation token |
| `get_scan_status` | `{ scanId }` | `ScanSummary` | reads app-owned status |
| `get_scan_findings` | `{ scanId, offset, limit }` | flat `Finding[]` | pages app-owned NDJSON |
| `reveal_item` | `{ scanId, findingId }` | none | resolves the path from backend persistence and opens its parent location |
| `create_cleanup_plan` | `{ scanId, findingIds, action }` | expiring immutable `CleanupPlan` | snapshots exact backend findings; no filesystem mutation |
| `execute_cleanup_plan` | `{ planId, confirmationToken }` | opaque operation ID | consumes the plan once and starts independent Trash operations |
| `cancel_cleanup` | `{ operationId }` | none | skips remaining items after the current bounded step |
| `get_cleanup_history` | `{ offset, limit }` | `CleanupSummary[]` | pages app-owned local history |
| `clear_cleanup_history` | none | none | removes app-owned local history only |
| `start_duplicate_scan` | `{ roots, minimumSizeBytes, byteForByteVerification }` | opaque duplicate scan ID | validates explicit roots and starts staged local hashing |
| `cancel_duplicate_scan` | `{ scanId }` | none | interrupts traversal or hashing between bounded chunks |
| `get_duplicate_scan_status` | `{ scanId }` | `DuplicateSummary` | reads app-owned status |
| `get_duplicate_groups` | `{ scanId, offset, limit }` | `DuplicateGroup[]` | pages app-owned NDJSON |
| `reveal_duplicate` | `{ scanId, groupId, copyId }` | none | resolves a persisted copy and reveals its parent |
| `create_duplicate_cleanup_plan` | `{ scanId, selections, action }` | immutable `DuplicateCleanupPlan` | resolves backend groups and enforces a keep copy; no mutation |
| `execute_duplicate_cleanup_plan` | `{ planId, confirmationToken }` | opaque operation ID | consumes the plan once, re-hashes, then starts Trash operations |
| `cancel_duplicate_cleanup` | `{ operationId }` | none | skips remaining planned copies |

## Destructive gate

`permanentDelete` is rejected unless the persisted setting is enabled. It uses the same `create_cleanup_plan` command and cannot introduce paths. Expert-risk permanent plans include a backend-issued confirmation phrase; `execute_cleanup_plan` rejects a missing or mismatched `typedConfirmation`. The frontend also uses the native Tauri confirm dialog, but backend plan validation remains authoritative.

Cleanup execution commands contain only `planId`, `confirmationToken`, and the optional backend-issued expert phrase. Duplicate and Custom scan roots are frontend paths because the user selects analysis scope, but they never authorize mutation. Duplicate cleanup accepts only persisted group/copy identifiers; the backend rejects the keep ID in a Trash set and rejects every selection that would remove all copies.

Diagnostics export accepts no path or content from the frontend. It serializes only version/platform metadata, redacted settings, aggregate disk and scan counts, cleanup outcome counts, and error codes. It omits configured root values, disk names and mount paths, scan IDs, finding paths, cleanup item records, hashes, and file contents.

## Event contract

Every scan or cleanup event includes its operation ID. Duplicate scans emit `duplicates://progress`, `group`, `completed`, and `failed`. Cleanup emits `started`, `progress`, `item-completed`, `completed`, and `failed`; per-item success is emitted only after the OS Trash operation succeeds.
